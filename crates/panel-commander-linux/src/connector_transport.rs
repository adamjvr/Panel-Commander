use crate::{Connector, I2cDdcTransport};
use panel_commander_core::{DdcTransport, Error, Result, VcpCode};
use std::path::PathBuf;

/// Opens a working DDC/CI transport for a DRM connector.
///
/// Linux GPU drivers can expose more than one I2C adapter for a physical
/// DisplayPort connector. In particular, AMDGPU may expose both a raw DM I2C
/// hardware bus and a DP-AUX-backed I2C adapter. The DRM `ddc` symlink is not
/// always the adapter that actually carries DDC/CI transactions.
///
/// Candidate transports are therefore validated with a read-only Get VCP
/// request before one is selected. Unsupported brightness is still a valid
/// DDC/CI reply; only transport/protocol failure rejects the candidate.
pub fn open_ddc_for_connector(connector: &Connector) -> Result<I2cDdcTransport> {
    if connector.i2c_candidates.is_empty() {
        return Err(Error::NotFound(format!(
            "{} has no DDC i2c-dev candidates",
            connector.name
        )));
    }

    let mut failures = Vec::new();
    for path in &connector.i2c_candidates {
        match validate_candidate(path) {
            Ok(transport) => return Ok(transport),
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    Err(Error::Protocol(format!(
        "no usable DDC/CI transport for {}; tried {}",
        connector.name,
        failures.join("; ")
    )))
}

fn validate_candidate(path: &PathBuf) -> Result<I2cDdcTransport> {
    let mut transport = I2cDdcTransport::open(path)?;
    // Brightness (0x10) is a safe, read-only liveness request. A monitor may
    // return an unsupported result code; decode_vcp_reply still proves the
    // candidate carries valid DDC/CI traffic.
    transport.get_vcp(VcpCode::BRIGHTNESS)?;
    Ok(transport)
}
