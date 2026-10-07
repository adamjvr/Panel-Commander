use panel_commander_asus::{known_probe_codes, semantic_name, AsusModel};
use panel_commander_core::{Capabilities, DdcTransport, Error, Result, VcpCode};

#[cfg(target_os = "linux")]
use panel_commander_linux::{
    discover_connectors, open_ddc_for_connector, Connector, I2cDdcTransport,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("panelctl: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || matches!(args[0].as_str(), "-h" | "--help" | "help") {
        print_help();
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        return Err(Error::Unsupported(
            "v0.1.0-dev panelctl hardware commands are currently wired for Linux".into(),
        ));
    }

    #[cfg(target_os = "linux")]
    match args[0].as_str() {
        "list" => cmd_list(),
        "probe" => cmd_probe(&args[1..]),
        "get" => cmd_get(&args[1..]),
        "set" => cmd_set(&args[1..]),
        "scan" => cmd_scan(&args[1..]),
        "capabilities" | "caps" => cmd_capabilities(&args[1..]),
        other => Err(Error::InvalidArgument(format!(
            "unknown command {other:?}; run panelctl --help"
        ))),
    }
}

fn print_help() {
    println!(
        r#"Panel Commander control/probe utility

Usage:
  panelctl list
  panelctl probe <connector>
  panelctl probe --all
  panelctl get <connector> <hex-vcp>
  panelctl set <connector> <hex-vcp> <value> --write
  panelctl scan <connector> [--all-codes]
  panelctl capabilities <connector>

Examples:
  panelctl list
  panelctl probe card1-DP-1
  panelctl get card1-DP-1 0x10
  panelctl set card1-DP-1 0x10 50 --write

Writes are deliberately blocked unless --write is present.
"#
    );
}

#[cfg(target_os = "linux")]
fn cmd_list() -> Result<()> {
    for c in discover_connectors()? {
        let identity = c
            .identity
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "-".into());
        let buses = format_candidates(&c);
        println!("{:<18} {:<12} {:<24} {}", c.name, c.status, buses, identity);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn cmd_probe(args: &[String]) -> Result<()> {
    if args.len() != 1 {
        return Err(Error::InvalidArgument(
            "probe expects <connector> or --all".into(),
        ));
    }
    let connectors = discover_connectors()?;
    if args[0] == "--all" {
        let mut failures = 0usize;
        for c in connectors.into_iter().filter(Connector::connected) {
            println!("=== {} ===", c.name);
            if let Err(e) = probe_one(&c) {
                failures += 1;
                println!("ERROR: {e}");
            }
            println!();
        }
        if failures > 0 {
            eprintln!("{failures} connected connector(s) could not be fully probed");
        }
        return Ok(());
    }
    let c = find_connector(&connectors, &args[0])?;
    probe_one(c)
}

#[cfg(target_os = "linux")]
fn probe_one(c: &Connector) -> Result<()> {
    println!("connector: {}", c.name);
    println!("status:    {}", c.status);
    if let Some(id) = &c.identity {
        println!("identity:  {id}");
    }
    let model = c
        .identity
        .as_ref()
        .map(AsusModel::identify)
        .unwrap_or(AsusModel::Unknown);
    println!("model db:  {}", model.name());

    println!("candidates: {}", format_candidates(c));
    let mut ddc = open_ddc_for_connector(c)?;
    println!("transport:  {}", ddc.path().display());
    if let Some(method) = ddc.active_method_name() {
        println!("method:     {method}");
    }

    let capabilities = match ddc.capabilities() {
        Ok(raw) => {
            let caps = Capabilities::parse(raw);
            println!("caps model: {:?}", caps.model);
            println!("MCCS:       {:?}", caps.mccs_version);
            println!("caps VCPs:  {}", caps.vcp_codes.len());
            Some(caps)
        }
        Err(e) => {
            println!("capabilities: unavailable ({e})");
            None
        }
    };

    let probe_codes = capabilities
        .as_ref()
        .map(|caps| caps.vcp_codes.iter().copied().collect::<Vec<_>>())
        .unwrap_or_else(|| known_probe_codes(model));

    println!("VCP:");
    for code in probe_codes {
        match ddc.get_vcp(code) {
            Ok(r) if r.supported() => {
                let advertised = capabilities
                    .as_ref()
                    .and_then(|caps| caps.advertised_values(code))
                    .filter(|values| !values.is_empty())
                    .map(format_values)
                    .unwrap_or_default();
                println!(
                    "  {} {:<48} current={} max={} type=0x{:02X}{}",
                    code,
                    semantic_name(model, code),
                    r.current,
                    r.maximum,
                    r.vcp_type,
                    advertised
                );
            }
            Ok(r) => println!(
                "  {} {:<48} unsupported/result=0x{:02X}",
                code,
                semantic_name(model, code),
                r.result_code
            ),
            Err(e) => println!("  {} {:<48} error: {}", code, semantic_name(model, code), e),
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn cmd_get(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        return Err(Error::InvalidArgument(
            "get expects <connector> <hex-vcp>".into(),
        ));
    }
    let connectors = discover_connectors()?;
    let c = find_connector(&connectors, &args[0])?;
    let code = parse_code(&args[1])?;
    let model = c
        .identity
        .as_ref()
        .map(AsusModel::identify)
        .unwrap_or(AsusModel::Unknown);
    let mut ddc = open_connector(c)?;
    let r = ddc.get_vcp(code)?;
    println!(
        "{} {}: result=0x{:02X} current={} max={} type=0x{:02X}",
        code,
        semantic_name(model, code),
        r.result_code,
        r.current,
        r.maximum,
        r.vcp_type
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn cmd_set(args: &[String]) -> Result<()> {
    if args.len() != 4 || args[3] != "--write" {
        return Err(Error::InvalidArgument(
            "set requires: <connector> <hex-vcp> <value> --write".into(),
        ));
    }
    let connectors = discover_connectors()?;
    let c = find_connector(&connectors, &args[0])?;
    let code = parse_code(&args[1])?;
    let value = args[2]
        .parse::<u16>()
        .map_err(|_| Error::InvalidArgument(format!("invalid u16 value {:?}", args[2])))?;

    let model = c
        .identity
        .as_ref()
        .map(AsusModel::identify)
        .unwrap_or(AsusModel::Unknown);

    if code.0 >= 0xe0 {
        if model != AsusModel::Xg27wcms {
            return Err(Error::InvalidArgument(format!(
                "refusing private ASUS write {code}: connector is not identified as XG27WCMS"
            )));
        }
        let Some(feature) = model.private_feature(code) else {
            return Err(Error::InvalidArgument(format!(
                "refusing uncharacterized XG27WCMS private write {code}"
            )));
        };
        if !feature.writable {
            return Err(Error::InvalidArgument(format!(
                "refusing XG27WCMS private write {code}: value semantics are not hardware-verified yet"
            )));
        }
    }

    let mut ddc = open_connector(c)?;
    ddc.set_vcp(code, value)?;
    println!(
        "wrote {} ({}) = {} on {}",
        code,
        semantic_name(model, code),
        value,
        c.name
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn cmd_scan(args: &[String]) -> Result<()> {
    if args.is_empty() || args.len() > 2 {
        return Err(Error::InvalidArgument(
            "scan expects <connector> [--all-codes]".into(),
        ));
    }
    let all = args.get(1).map(String::as_str) == Some("--all-codes");
    let connectors = discover_connectors()?;
    let c = find_connector(&connectors, &args[0])?;
    let model = c
        .identity
        .as_ref()
        .map(AsusModel::identify)
        .unwrap_or(AsusModel::Unknown);
    let mut ddc = open_connector(c)?;

    let codes: Vec<VcpCode> = if all {
        (0u16..=255).map(|value| VcpCode(value as u8)).collect()
    } else {
        match ddc.capabilities() {
            Ok(raw) => Capabilities::parse(raw).vcp_codes.into_iter().collect(),
            Err(_) => known_probe_codes(model),
        }
    };

    for code in codes {
        if let Ok(r) = ddc.get_vcp(code) {
            if r.supported() {
                println!(
                    "{} {:<48} current={} max={} type=0x{:02X}",
                    code,
                    semantic_name(model, code),
                    r.current,
                    r.maximum,
                    r.vcp_type
                );
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn cmd_capabilities(args: &[String]) -> Result<()> {
    if args.len() != 1 {
        return Err(Error::InvalidArgument(
            "capabilities expects <connector>".into(),
        ));
    }
    let connectors = discover_connectors()?;
    let c = find_connector(&connectors, &args[0])?;
    let mut ddc = open_connector(c)?;
    println!("{}", ddc.capabilities()?);
    Ok(())
}

#[cfg(target_os = "linux")]
fn open_connector(c: &Connector) -> Result<I2cDdcTransport> {
    open_ddc_for_connector(c)
}

#[cfg(target_os = "linux")]
fn format_candidates(c: &Connector) -> String {
    if c.i2c_candidates.is_empty() {
        return "-".into();
    }
    c.i2c_candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(target_os = "linux")]
fn find_connector<'a>(connectors: &'a [Connector], name: &str) -> Result<&'a Connector> {
    connectors
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| Error::NotFound(format!("DRM connector {name:?}")))
}

fn format_values(values: &[u16]) -> String {
    let values = values
        .iter()
        .map(|value| {
            if *value <= 0xff {
                format!("{value:02X}")
            } else {
                format!("{value:04X}")
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(" advertised=[{values}]")
}

fn parse_code(text: &str) -> Result<VcpCode> {
    VcpCode::parse(text).ok_or_else(|| Error::InvalidArgument(format!("invalid VCP code {text:?}")))
}
