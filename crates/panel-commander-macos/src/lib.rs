#![forbid(unsafe_code)]

use panel_commander_core::{DdcTransport, Error, Result, VcpCode, VcpReply};

/// Transport boundary for the native macOS backend.
///
/// The recovered vendor implementation uses:
/// - Intel: IOFramebuffer/IOI2CInterface
/// - Apple Silicon: IOAVServiceReadI2C/IOAVServiceWriteI2C
///
/// The common DDC/CI packet implementation lives in panel-commander-core.  The
/// platform FFI will be isolated here and kept out of the GUI/application layers.
#[derive(Debug, Default)]
pub struct MacDdcTransport;

impl MacDdcTransport {
    pub fn open_default() -> Result<Self> {
        Err(Error::Unsupported(
            "macOS native transport is scaffolded but not wired in v0.1.0-dev".into(),
        ))
    }
}

impl DdcTransport for MacDdcTransport {
    fn get_vcp(&mut self, _code: VcpCode) -> Result<VcpReply> {
        Err(Error::Unsupported(
            "macOS DDC transport not wired yet".into(),
        ))
    }

    fn set_vcp(&mut self, _code: VcpCode, _value: u16) -> Result<()> {
        Err(Error::Unsupported(
            "macOS DDC transport not wired yet".into(),
        ))
    }
}
