use panel_commander_core::{Edid, Error, MonitorIdentity, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Connector {
    pub name: String,
    pub sysfs_path: PathBuf,
    pub status: String,
    /// Primary DRM-advertised DDC bus, retained for compatibility/diagnostics.
    pub i2c_device: Option<PathBuf>,
    /// Ordered candidate DDC transports. The DRM `ddc` bus is first, followed by
    /// any DisplayPort AUX-backed I2C adapters associated with this connector.
    pub i2c_candidates: Vec<PathBuf>,
    pub identity: Option<MonitorIdentity>,
    pub edid_bytes: Option<Vec<u8>>,
}

impl Connector {
    pub fn connected(&self) -> bool {
        self.status.trim() == "connected"
    }

    pub fn ddc_candidates(&self) -> &[PathBuf] {
        &self.i2c_candidates
    }
}

pub fn discover_connectors() -> Result<Vec<Connector>> {
    discover_connectors_at(Path::new("/sys/class/drm"))
}

pub fn discover_connectors_at(root: &Path) -> Result<Vec<Connector>> {
    let mut out = Vec::new();
    let entries = fs::read_dir(root).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            Error::Permission(format!("cannot read {}: {e}", root.display()))
        } else {
            Error::Io(e)
        }
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() && !path.is_symlink() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !looks_like_connector(name) || !path.join("status").exists() {
            continue;
        }
        let status = fs::read_to_string(path.join("status"))
            .unwrap_or_else(|_| "unknown".into())
            .trim()
            .to_string();

        let edid_bytes = fs::read(path.join("edid")).ok().filter(|b| !b.is_empty());
        let identity = edid_bytes
            .as_deref()
            .and_then(|b| Edid::parse(b).ok())
            .map(|e| e.identity().clone());

        let i2c_device = find_drm_ddc_device(&path);
        let mut i2c_candidates = Vec::new();
        if let Some(path) = &i2c_device {
            push_unique(&mut i2c_candidates, path.clone());
        }
        for candidate in find_dp_aux_i2c_devices(&path) {
            push_unique(&mut i2c_candidates, candidate);
        }

        out.push(Connector {
            name: name.to_string(),
            sysfs_path: path.clone(),
            status,
            i2c_device,
            i2c_candidates,
            identity,
            edid_bytes,
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn looks_like_connector(name: &str) -> bool {
    // DRM connectors are normally cardN-DP-N/cardN-HDMI-A-N/etc.
    name.starts_with("card") && name.contains('-')
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn find_drm_ddc_device(connector: &Path) -> Option<PathBuf> {
    // Common DRM layout:
    // /sys/class/drm/cardN-DP-M/ddc/i2c-dev/i2c-X
    let i2c_dev_dir = connector.join("ddc/i2c-dev");
    if let Ok(entries) = fs::read_dir(&i2c_dev_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if is_i2c_name(name) {
                    return Some(PathBuf::from("/dev").join(name));
                }
            }
        }
    }

    // Some sysfs layouts expose ddc as a symlink resolving into i2c-X.
    let ddc = connector.join("ddc");
    if let Ok(canonical) = fs::canonicalize(ddc) {
        for comp in canonical.components().rev() {
            let text = comp.as_os_str().to_string_lossy();
            if is_i2c_name(&text) {
                return Some(PathBuf::from("/dev").join(text.as_ref()));
            }
        }
    }
    None
}

fn find_dp_aux_i2c_devices(connector: &Path) -> Vec<PathBuf> {
    find_dp_aux_i2c_devices_at(
        connector,
        Path::new("/sys/class/drm_dp_aux_dev"),
        Path::new("/sys/bus/i2c/devices"),
        Path::new("/dev"),
    )
}

fn find_dp_aux_i2c_devices_at(
    connector: &Path,
    aux_root: &Path,
    i2c_root: &Path,
    dev_root: &Path,
) -> Vec<PathBuf> {
    let Ok(connector_canonical) = fs::canonicalize(connector) else {
        return Vec::new();
    };

    let mut aux_names = Vec::new();
    let Ok(aux_entries) = fs::read_dir(aux_root) else {
        return Vec::new();
    };

    for entry in aux_entries.flatten() {
        let aux_path = entry.path();
        let Ok(aux_canonical) = fs::canonicalize(&aux_path) else {
            continue;
        };
        if aux_canonical.parent() != Some(connector_canonical.as_path()) {
            continue;
        }
        let Ok(name) = fs::read_to_string(aux_path.join("name")) else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty() {
            aux_names.push(name.to_string());
        }
    }

    if aux_names.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let Ok(i2c_entries) = fs::read_dir(i2c_root) else {
        return out;
    };
    for entry in i2c_entries.flatten() {
        let Some(bus_name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if !is_i2c_name(&bus_name) {
            continue;
        }
        let Ok(adapter_name) = fs::read_to_string(entry.path().join("name")) else {
            continue;
        };
        if aux_names.iter().any(|name| name == adapter_name.trim()) {
            push_unique(&mut out, dev_root.join(bus_name));
        }
    }
    out.sort();
    out
}

fn is_i2c_name(name: &str) -> bool {
    name.strip_prefix("i2c-")
        .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_linux_i2c_device_names() {
        assert!(is_i2c_name("i2c-0"));
        assert!(is_i2c_name("i2c-11"));
        assert!(!is_i2c_name("i2c-"));
        assert!(!is_i2c_name("i2c-aux1"));
        assert!(!is_i2c_name("foo-i2c-1"));
    }
}
