use panel_commander_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
use panel_commander_asus::{semantic_name, AsusModel};
#[cfg(target_os = "linux")]
use panel_commander_core::{Capabilities, DdcTransport};
#[cfg(target_os = "linux")]
use panel_commander_linux::{discover_connectors, open_ddc_for_connector};

const SNAPSHOT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentitySnapshot {
    pub manufacturer: String,
    pub product_code: u16,
    pub serial_number: u32,
    pub monitor_name: Option<String>,
    pub serial_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub code: u8,
    pub semantic: String,
    pub advertised_values: Vec<u16>,
    pub status: String,
    pub result_code: Option<u8>,
    pub vcp_type: Option<u8>,
    pub maximum: Option<u16>,
    pub current: Option<u16>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonitorSnapshot {
    pub format_version: u32,
    pub captured_unix_ms: u64,
    pub connector: String,
    pub identity: Option<IdentitySnapshot>,
    pub model_db: String,
    pub transport: String,
    pub transport_method: Option<String>,
    pub capabilities_raw: String,
    pub capabilities_model: Option<String>,
    pub mccs_version: Option<String>,
    pub entries: Vec<SnapshotEntry>,
}

#[cfg(target_os = "linux")]
pub fn cmd_snapshot(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        return Err(Error::InvalidArgument(
            "snapshot expects <connector> <output.json>".into(),
        ));
    }

    let connectors = discover_connectors()?;
    let connector = connectors
        .iter()
        .find(|connector| connector.name == args[0])
        .ok_or_else(|| Error::NotFound(format!("DRM connector {:?}", args[0])))?;

    let model = connector
        .identity
        .as_ref()
        .map(AsusModel::identify)
        .unwrap_or(AsusModel::Unknown);

    let mut ddc = open_ddc_for_connector(connector)?;
    let transport = ddc.path().display().to_string();
    let transport_method = ddc.active_method_name().map(str::to_string);
    let capabilities_raw = ddc.capabilities()?;
    let capabilities = Capabilities::parse(capabilities_raw.clone());

    let mut entries = Vec::with_capacity(capabilities.vcp_codes.len());
    for code in capabilities.vcp_codes.iter().copied() {
        let advertised_values = capabilities
            .advertised_values(code)
            .unwrap_or_default()
            .to_vec();
        let semantic = semantic_name(model, code).to_string();
        let entry = match ddc.get_vcp(code) {
            Ok(reply) if reply.supported() => SnapshotEntry {
                code: code.0,
                semantic,
                advertised_values,
                status: "supported".into(),
                result_code: Some(reply.result_code),
                vcp_type: Some(reply.vcp_type),
                maximum: Some(reply.maximum),
                current: Some(reply.current),
                error: None,
            },
            Ok(reply) => SnapshotEntry {
                code: code.0,
                semantic,
                advertised_values,
                status: "unsupported".into(),
                result_code: Some(reply.result_code),
                vcp_type: Some(reply.vcp_type),
                maximum: Some(reply.maximum),
                current: Some(reply.current),
                error: None,
            },
            Err(error) => SnapshotEntry {
                code: code.0,
                semantic,
                advertised_values,
                status: "error".into(),
                result_code: None,
                vcp_type: None,
                maximum: None,
                current: None,
                error: Some(error.to_string()),
            },
        };
        entries.push(entry);
    }

    let identity = connector
        .identity
        .as_ref()
        .map(|identity| IdentitySnapshot {
            manufacturer: identity.manufacturer.clone(),
            product_code: identity.product_code,
            serial_number: identity.serial_number,
            monitor_name: identity.monitor_name.clone(),
            serial_text: identity.serial_text.clone(),
        });

    let captured_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::Protocol(format!("system clock is before Unix epoch: {error}")))?
        .as_millis()
        .try_into()
        .map_err(|_| Error::Protocol("snapshot timestamp does not fit u64".into()))?;

    let snapshot = MonitorSnapshot {
        format_version: SNAPSHOT_FORMAT_VERSION,
        captured_unix_ms,
        connector: connector.name.clone(),
        identity,
        model_db: model.name().to_string(),
        transport,
        transport_method,
        capabilities_raw,
        capabilities_model: capabilities.model,
        mccs_version: capabilities.mccs_version,
        entries,
    };

    write_snapshot(Path::new(&args[1]), &snapshot)?;
    println!(
        "snapshot: wrote {} VCP entries for {} to {}",
        snapshot.entries.len(),
        snapshot.connector,
        args[1]
    );
    Ok(())
}

pub fn cmd_diff(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        return Err(Error::InvalidArgument(
            "diff expects <before.json> <after.json>".into(),
        ));
    }

    let before = read_snapshot(Path::new(&args[0]))?;
    let after = read_snapshot(Path::new(&args[1]))?;
    ensure_same_monitor(&before, &after)?;

    let changes = changed_codes(&before, &after);
    if changes.is_empty() {
        println!("No VCP state changes.");
        return Ok(());
    }

    let before_map = entries_by_code(&before);
    let after_map = entries_by_code(&after);
    println!("Changed VCPs:");
    for code in changes {
        let before_entry = before_map.get(&code).copied();
        let after_entry = after_map.get(&code).copied();
        let semantic = after_entry
            .or(before_entry)
            .map(|entry| entry.semantic.as_str())
            .unwrap_or("Unknown/monitor-defined VCP");
        println!();
        println!("0x{code:02X} {semantic}");
        println!("  before: {}", format_entry_state(before_entry));
        println!("  after:  {}", format_entry_state(after_entry));

        let advertised = after_entry
            .or(before_entry)
            .map(|entry| entry.advertised_values.as_slice())
            .unwrap_or_default();
        if !advertised.is_empty() {
            println!("  advertised: {}", format_values(advertised));
        }
    }
    Ok(())
}

fn write_snapshot(path: &Path, snapshot: &MonitorSnapshot) -> Result<()> {
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|error| Error::Protocol(format!("cannot encode snapshot JSON: {error}")))?;
    fs::write(path, format!("{json}\n"))?;
    Ok(())
}

fn read_snapshot(path: &Path) -> Result<MonitorSnapshot> {
    let raw = fs::read_to_string(path)?;
    let snapshot: MonitorSnapshot = serde_json::from_str(&raw).map_err(|error| {
        Error::InvalidArgument(format!("cannot parse snapshot {}: {error}", path.display()))
    })?;
    if snapshot.format_version != SNAPSHOT_FORMAT_VERSION {
        return Err(Error::InvalidArgument(format!(
            "snapshot {} uses format version {}, expected {}",
            path.display(),
            snapshot.format_version,
            SNAPSHOT_FORMAT_VERSION
        )));
    }
    Ok(snapshot)
}

fn ensure_same_monitor(before: &MonitorSnapshot, after: &MonitorSnapshot) -> Result<()> {
    if before.identity != after.identity {
        return Err(Error::InvalidArgument(
            "snapshot identities differ; refusing to correlate different monitors".into(),
        ));
    }
    Ok(())
}

fn entries_by_code(snapshot: &MonitorSnapshot) -> BTreeMap<u8, &SnapshotEntry> {
    snapshot
        .entries
        .iter()
        .map(|entry| (entry.code, entry))
        .collect()
}

fn changed_codes(before: &MonitorSnapshot, after: &MonitorSnapshot) -> Vec<u8> {
    let before_map = entries_by_code(before);
    let after_map = entries_by_code(after);
    let codes = before_map
        .keys()
        .chain(after_map.keys())
        .copied()
        .collect::<BTreeSet<_>>();

    codes
        .into_iter()
        .filter(|code| {
            state_fingerprint(before_map.get(code).copied())
                != state_fingerprint(after_map.get(code).copied())
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EntryStateFingerprint<'a> {
    status: &'a str,
    result_code: Option<u8>,
    vcp_type: Option<u8>,
    maximum: Option<u16>,
    current: Option<u16>,
    error: Option<&'a str>,
}

fn state_fingerprint(entry: Option<&SnapshotEntry>) -> Option<EntryStateFingerprint<'_>> {
    entry.map(|entry| EntryStateFingerprint {
        status: entry.status.as_str(),
        result_code: entry.result_code,
        vcp_type: entry.vcp_type,
        maximum: entry.maximum,
        current: entry.current,
        error: entry.error.as_deref(),
    })
}

fn format_entry_state(entry: Option<&SnapshotEntry>) -> String {
    let Some(entry) = entry else {
        return "absent".into();
    };
    match entry.status.as_str() {
        "supported" => format!(
            "supported result=0x{:02X} current={} max={} type=0x{:02X}",
            entry.result_code.unwrap_or_default(),
            entry.current.unwrap_or_default(),
            entry.maximum.unwrap_or_default(),
            entry.vcp_type.unwrap_or_default()
        ),
        "unsupported" => format!(
            "unsupported result=0x{:02X} current={} max={} type=0x{:02X}",
            entry.result_code.unwrap_or_default(),
            entry.current.unwrap_or_default(),
            entry.maximum.unwrap_or_default(),
            entry.vcp_type.unwrap_or_default()
        ),
        "error" => format!(
            "error: {}",
            entry.error.as_deref().unwrap_or("unknown error")
        ),
        other => format!("{other}: {:?}", entry.current),
    }
}

fn format_values(values: &[u16]) -> String {
    values
        .iter()
        .map(|value| {
            if *value <= 0xff {
                format!("{value:02X}")
            } else {
                format!("{value:04X}")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(current: u16) -> MonitorSnapshot {
        MonitorSnapshot {
            format_version: SNAPSHOT_FORMAT_VERSION,
            captured_unix_ms: 0,
            connector: "card1-DP-2".into(),
            identity: Some(IdentitySnapshot {
                manufacturer: "AUS".into(),
                product_code: 0x275e,
                serial_number: 0x01010101,
                monitor_name: Some("XG27WCMS".into()),
                serial_text: None,
            }),
            model_db: "XG27WCMS".into(),
            transport: "/dev/i2c-11".into(),
            transport_method: Some("I2C_RDWR ioctl".into()),
            capabilities_raw: "(vcp(E5(00 01 02 03 04)))".into(),
            capabilities_model: Some("XG27WCMS".into()),
            mccs_version: Some("2.2".into()),
            entries: vec![SnapshotEntry {
                code: 0xe5,
                semantic: "ASUS E5".into(),
                advertised_values: vec![0, 1, 2, 3, 4],
                status: "supported".into(),
                result_code: Some(0),
                vcp_type: Some(1),
                maximum: Some(255),
                current: Some(current),
                error: None,
            }],
        }
    }

    #[test]
    fn diff_finds_changed_current_value() {
        assert_eq!(changed_codes(&snapshot(0), &snapshot(3)), vec![0xe5]);
    }

    #[test]
    fn diff_ignores_capture_timestamp() {
        let mut after = snapshot(0);
        after.captured_unix_ms = 12345;
        assert!(changed_codes(&snapshot(0), &after).is_empty());
    }
}
