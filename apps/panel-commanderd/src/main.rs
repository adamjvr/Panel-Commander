use std::io::{BufRead, BufReader, Write};

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

#[cfg(target_os = "linux")]
use panel_commander_asus::{semantic_name, AsusModel};
#[cfg(target_os = "linux")]
use panel_commander_core::{DdcTransport, VcpCode};
#[cfg(target_os = "linux")]
use panel_commander_linux::{discover_connectors, open_ddc_for_connector};

fn main() {
    #[cfg(unix)]
    {
        if let Err(e) = unix_main() {
            eprintln!("panel-commanderd: {e}");
            std::process::exit(1);
        }
    }

    #[cfg(not(unix))]
    {
        eprintln!("panel-commanderd currently requires a Unix-like platform");
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn unix_main() -> std::io::Result<()> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let socket = runtime.join("panel-commanderd.sock");
    if socket.exists() {
        std::fs::remove_file(&socket)?;
    }
    let listener = UnixListener::bind(&socket)?;
    println!("Panel Commander daemon listening on {}", socket.display());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_client(stream) {
                    eprintln!("client error: {e}");
                }
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn handle_client(mut stream: UnixStream) -> std::io::Result<()> {
    let mut line = String::new();
    BufReader::new(stream.try_clone()?).read_line(&mut line)?;
    let reply = dispatch(line.trim());
    stream.write_all(reply.as_bytes())
}

fn dispatch(command: &str) -> String {
    match command {
        "PING" => "OK PONG\n".into(),
        "VERSION" => format!("OK {}\n", env!("CARGO_PKG_VERSION")),
        _ => dispatch_platform(command),
    }
}

#[cfg(target_os = "linux")]
fn dispatch_platform(command: &str) -> String {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        ["LIST"] => match discover_connectors() {
            Ok(connectors) => {
                let mut s = String::from("OK\n");
                for c in connectors {
                    let bus = if c.i2c_candidates.is_empty() {
                        "-".into()
                    } else {
                        c.i2c_candidates
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    };
                    let identity = c
                        .identity
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "-".into());
                    s.push_str(&format!(
                        "{}\t{}\t{}\t{}\n",
                        c.name, c.status, bus, identity
                    ));
                }
                s.push_str(".\n");
                s
            }
            Err(e) => format!("ERR {e}\n"),
        },
        ["GET", connector, code_text] => {
            let Some(code) = VcpCode::parse(code_text) else {
                return "ERR invalid VCP code\n".into();
            };
            match discover_connectors() {
                Ok(connectors) => {
                    let Some(c) = connectors.iter().find(|c| c.name == *connector) else {
                        return "ERR connector not found\n".into();
                    };
                    let model = c
                        .identity
                        .as_ref()
                        .map(AsusModel::identify)
                        .unwrap_or(AsusModel::Unknown);
                    match open_ddc_for_connector(c).and_then(|mut d| d.get_vcp(code)) {
                        Ok(r) => format!(
                            "OK code={} name={:?} result={} current={} max={} type={}\n",
                            code,
                            semantic_name(model, code),
                            r.result_code,
                            r.current,
                            r.maximum,
                            r.vcp_type
                        ),
                        Err(e) => format!("ERR {e}\n"),
                    }
                }
                Err(e) => format!("ERR {e}\n"),
            }
        }
        ["CAPS", connector] => match discover_connectors() {
            Ok(connectors) => {
                let Some(c) = connectors.iter().find(|c| c.name == *connector) else {
                    return "ERR connector not found\n".into();
                };
                match open_ddc_for_connector(c).and_then(|mut d| d.capabilities()) {
                    Ok(caps) => format!("OK {caps}\n"),
                    Err(e) => format!("ERR {e}\n"),
                }
            }
            Err(e) => format!("ERR {e}\n"),
        },
        _ => "ERR commands: PING VERSION LIST GET <connector> <code> CAPS <connector>\n".into(),
    }
}

#[cfg(not(target_os = "linux"))]
fn dispatch_platform(_command: &str) -> String {
    "ERR hardware daemon commands are not wired on this platform yet\n".into()
}
