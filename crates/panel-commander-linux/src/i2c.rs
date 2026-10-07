use panel_commander_core::ddc::{
    decode_capabilities_reply, decode_vcp_reply, encode_capabilities_request, encode_get_vcp,
    encode_set_vcp, DdcTransport, VcpReply, DDC_CI_7BIT_ADDRESS,
};
use panel_commander_core::{Error, Result, VcpCode};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::raw::{c_int, c_ulong};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

const I2C_SLAVE: c_ulong = 0x0703;
const I2C_FUNCS: c_ulong = 0x0705;
const I2C_RDWR: c_ulong = 0x0707;
const I2C_M_RD: u16 = 0x0001;
const I2C_FUNC_I2C: c_ulong = 0x0000_0001;
const DEFAULT_REPLY_DELAY: Duration = Duration::from_millis(50);
const DEFAULT_INTER_MESSAGE_DELAY: Duration = Duration::from_millis(50);
const MAX_CAPABILITY_BYTES: usize = 64 * 1024;

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

#[repr(C)]
struct I2cMsg {
    addr: u16,
    flags: u16,
    len: u16,
    buf: *mut u8,
}

#[repr(C)]
struct I2cRdwrIoctlData {
    msgs: *mut I2cMsg,
    nmsgs: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cTransferMethod {
    Auto,
    Rdwr,
    File,
}

impl I2cTransferMethod {
    fn from_env() -> Result<Self> {
        match env::var("PANEL_COMMANDER_I2C_MODE") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "auto" => Ok(Self::Auto),
                "rdwr" | "ioctl" | "i2c-rdwr" => Ok(Self::Rdwr),
                "file" | "file-io" => Ok(Self::File),
                other => Err(Error::InvalidArgument(format!(
                    "invalid PANEL_COMMANDER_I2C_MODE={other:?}; expected auto, rdwr, or file"
                ))),
            },
            Err(env::VarError::NotPresent) => Ok(Self::Auto),
            Err(e) => Err(Error::InvalidArgument(format!(
                "cannot read PANEL_COMMANDER_I2C_MODE: {e}"
            ))),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Rdwr => "I2C_RDWR ioctl",
            Self::File => "file I/O",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectiveMethod {
    Rdwr,
    File,
}

impl EffectiveMethod {
    fn name(self) -> &'static str {
        match self {
            Self::Rdwr => "I2C_RDWR ioctl",
            Self::File => "file I/O",
        }
    }
}

#[derive(Debug)]
pub struct I2cDdcTransport {
    path: PathBuf,
    file: File,
    configured_method: I2cTransferMethod,
    active_method: Option<EffectiveMethod>,
    file_io_ready: bool,
    file_io_setup_error: Option<String>,
    adapter_functionality: Option<c_ulong>,
    reply_delay: Duration,
    inter_message_delay: Duration,
}

impl I2cDdcTransport {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let configured_method = I2cTransferMethod::from_env()?;
        Self::open_with_method(path, configured_method)
    }

    pub fn open_with_method(
        path: impl AsRef<Path>,
        configured_method: I2cTransferMethod,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::Permission(format!(
                        "cannot open {} read/write; configure i2c permissions: {e}",
                        path.display()
                    ))
                } else {
                    contextual_io(&path, "open", e)
                }
            })?;

        let adapter_functionality = query_functionality(&file).ok();

        // I2C_SLAVE is required by the read()/write() interface, but I2C_RDWR carries
        // the slave address in each message. Do not reject an adapter merely because
        // I2C_SLAVE setup failed if ioctl transport is still available.
        let (file_io_ready, file_io_setup_error) = match set_slave_address(&file) {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };

        if configured_method == I2cTransferMethod::File && !file_io_ready {
            return Err(Error::Protocol(format!(
                "file I/O transport unavailable on {}: {}",
                path.display(),
                file_io_setup_error.as_deref().unwrap_or("I2C_SLAVE failed")
            )));
        }

        if configured_method == I2cTransferMethod::Rdwr {
            if let Some(funcs) = adapter_functionality {
                if funcs & I2C_FUNC_I2C == 0 {
                    return Err(Error::Unsupported(format!(
                        "{} does not advertise I2C_FUNC_I2C; cannot force I2C_RDWR transport",
                        path.display()
                    )));
                }
            }
        }

        Ok(Self {
            path,
            file,
            configured_method,
            active_method: None,
            file_io_ready,
            file_io_setup_error,
            adapter_functionality,
            reply_delay: DEFAULT_REPLY_DELAY,
            inter_message_delay: DEFAULT_INTER_MESSAGE_DELAY,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn configured_method(&self) -> I2cTransferMethod {
        self.configured_method
    }

    pub fn active_method_name(&self) -> Option<&'static str> {
        self.active_method.map(EffectiveMethod::name)
    }

    pub fn adapter_functionality(&self) -> Option<c_ulong> {
        self.adapter_functionality
    }

    pub fn adapter_supports_i2c(&self) -> Option<bool> {
        self.adapter_functionality
            .map(|funcs| funcs & I2C_FUNC_I2C != 0)
    }

    pub fn file_io_setup_error(&self) -> Option<&str> {
        self.file_io_setup_error.as_deref()
    }

    pub fn set_delays(&mut self, reply: Duration, inter_message: Duration) {
        self.reply_delay = reply;
        self.inter_message_delay = inter_message;
    }

    fn transact_fixed<const N: usize>(&mut self, request: &[u8]) -> Result<[u8; N]> {
        match self.configured_method {
            I2cTransferMethod::Rdwr => self.transact_rdwr::<N>(request),
            I2cTransferMethod::File => self.transact_file::<N>(request),
            I2cTransferMethod::Auto => self.transact_auto::<N>(request),
        }
    }

    fn transact_auto<const N: usize>(&mut self, request: &[u8]) -> Result<[u8; N]> {
        let first = match self.active_method {
            Some(EffectiveMethod::File) => EffectiveMethod::File,
            _ => EffectiveMethod::Rdwr,
        };
        let second = match first {
            EffectiveMethod::Rdwr => EffectiveMethod::File,
            EffectiveMethod::File => EffectiveMethod::Rdwr,
        };

        match self.transact_by::<N>(first, request) {
            Ok(reply) => {
                self.active_method = Some(first);
                Ok(reply)
            }
            Err(first_error) => match self.transact_by::<N>(second, request) {
                Ok(reply) => {
                    self.active_method = Some(second);
                    Ok(reply)
                }
                Err(second_error) => Err(Error::Protocol(format!(
                    "DDC transaction failed on {} using both Linux transports; {}: {}; {}: {}",
                    self.path.display(),
                    first.name(),
                    first_error,
                    second.name(),
                    second_error
                ))),
            },
        }
    }

    fn transact_by<const N: usize>(
        &mut self,
        method: EffectiveMethod,
        request: &[u8],
    ) -> Result<[u8; N]> {
        match method {
            EffectiveMethod::Rdwr => self.transact_rdwr::<N>(request),
            EffectiveMethod::File => self.transact_file::<N>(request),
        }
    }

    fn transact_rdwr<const N: usize>(&mut self, request: &[u8]) -> Result<[u8; N]> {
        self.rdwr_write(request)?;
        sleep(self.reply_delay);

        let mut reply = [0u8; N];
        self.rdwr_read(&mut reply)?;
        sleep(self.inter_message_delay);
        Ok(reply)
    }

    fn transact_file<const N: usize>(&mut self, request: &[u8]) -> Result<[u8; N]> {
        if !self.file_io_ready {
            return Err(Error::Protocol(format!(
                "file I/O unavailable: {}",
                self.file_io_setup_error
                    .as_deref()
                    .unwrap_or("I2C_SLAVE was not configured")
            )));
        }

        self.file
            .write_all(request)
            .map_err(|e| contextual_io(&self.path, "file-I/O DDC write", e))?;
        self.file
            .flush()
            .map_err(|e| contextual_io(&self.path, "file-I/O flush", e))?;
        sleep(self.reply_delay);

        let mut reply = [0u8; N];
        self.file
            .read_exact(&mut reply)
            .map_err(|e| contextual_io(&self.path, "file-I/O DDC read", e))?;
        sleep(self.inter_message_delay);
        Ok(reply)
    }

    fn rdwr_write(&mut self, request: &[u8]) -> Result<()> {
        let mut bytes = request.to_vec();
        let mut msg = I2cMsg {
            addr: DDC_CI_7BIT_ADDRESS,
            flags: 0,
            len: u16::try_from(bytes.len())
                .map_err(|_| Error::InvalidArgument("I2C message exceeds u16 length".into()))?,
            buf: bytes.as_mut_ptr(),
        };
        self.rdwr_ioctl(&mut msg, "I2C_RDWR DDC write")
    }

    fn rdwr_read(&mut self, reply: &mut [u8]) -> Result<()> {
        let mut msg = I2cMsg {
            addr: DDC_CI_7BIT_ADDRESS,
            flags: I2C_M_RD,
            len: u16::try_from(reply.len())
                .map_err(|_| Error::InvalidArgument("I2C message exceeds u16 length".into()))?,
            buf: reply.as_mut_ptr(),
        };
        self.rdwr_ioctl(&mut msg, "I2C_RDWR DDC read")
    }

    fn rdwr_ioctl(&self, msg: &mut I2cMsg, operation: &str) -> Result<()> {
        let mut data = I2cRdwrIoctlData {
            msgs: msg as *mut I2cMsg,
            nmsgs: 1,
        };

        // SAFETY: data points to one valid I2cMsg whose buffer lives for the duration
        // of ioctl(). The file descriptor remains open, and the address/flags/length
        // layout matches linux/i2c.h and linux/i2c-dev.h.
        let rc = unsafe {
            ioctl(
                self.file.as_raw_fd(),
                I2C_RDWR,
                &mut data as *mut I2cRdwrIoctlData,
            )
        };
        if rc < 0 {
            return Err(contextual_io(
                &self.path,
                operation,
                std::io::Error::last_os_error(),
            ));
        }
        if rc != 1 {
            return Err(Error::Protocol(format!(
                "{operation} on {} completed {rc} messages, expected 1",
                self.path.display()
            )));
        }
        Ok(())
    }

    fn write_only(&mut self, request: &[u8]) -> Result<()> {
        match self.configured_method {
            I2cTransferMethod::Rdwr => {
                self.rdwr_write(request)?;
                self.active_method = Some(EffectiveMethod::Rdwr);
            }
            I2cTransferMethod::File => {
                self.file_write_only(request)?;
                self.active_method = Some(EffectiveMethod::File);
            }
            I2cTransferMethod::Auto => {
                let first = match self.active_method {
                    Some(EffectiveMethod::File) => EffectiveMethod::File,
                    _ => EffectiveMethod::Rdwr,
                };
                let second = match first {
                    EffectiveMethod::Rdwr => EffectiveMethod::File,
                    EffectiveMethod::File => EffectiveMethod::Rdwr,
                };
                if let Err(first_error) = self.write_only_by(first, request) {
                    if let Err(second_error) = self.write_only_by(second, request) {
                        let message = format!(
                            "DDC write failed on {} using both Linux transports; {}: {}; {}: {}",
                            self.path.display(),
                            first.name(),
                            first_error,
                            second.name(),
                            second_error
                        );
                        return Err(Error::Protocol(message));
                    }
                    self.active_method = Some(second);
                } else {
                    self.active_method = Some(first);
                }
            }
        }
        sleep(self.inter_message_delay);
        Ok(())
    }

    fn write_only_by(&mut self, method: EffectiveMethod, request: &[u8]) -> Result<()> {
        match method {
            EffectiveMethod::Rdwr => self.rdwr_write(request),
            EffectiveMethod::File => self.file_write_only(request),
        }
    }

    fn file_write_only(&mut self, request: &[u8]) -> Result<()> {
        if !self.file_io_ready {
            return Err(Error::Protocol(format!(
                "file I/O unavailable: {}",
                self.file_io_setup_error
                    .as_deref()
                    .unwrap_or("I2C_SLAVE was not configured")
            )));
        }
        self.file
            .write_all(request)
            .map_err(|e| contextual_io(&self.path, "file-I/O DDC write", e))?;
        self.file
            .flush()
            .map_err(|e| contextual_io(&self.path, "file-I/O flush", e))?;
        Ok(())
    }

    fn read_capabilities_packet(&mut self, offset: u16) -> Result<(u16, Vec<u8>, bool)> {
        let request = encode_capabilities_request(offset);
        // DDC/CI capability reply allows up to 32 bytes of data.
        // Source + length + opcode + offset(2) + data(32) + checksum = 38.
        let reply = self.transact_fixed::<38>(&request)?;
        decode_capabilities_reply(&reply)
    }
}

impl DdcTransport for I2cDdcTransport {
    fn get_vcp(&mut self, code: VcpCode) -> Result<VcpReply> {
        let request = encode_get_vcp(code);
        let reply = self.transact_fixed::<11>(&request)?;
        let parsed = decode_vcp_reply(&reply)?;
        if parsed.code != code {
            return Err(Error::Protocol(format!(
                "asked for {code}, monitor replied for {}",
                parsed.code
            )));
        }
        Ok(parsed)
    }

    fn set_vcp(&mut self, code: VcpCode, value: u16) -> Result<()> {
        let request = encode_set_vcp(code, value);
        self.write_only(&request)
    }

    fn capabilities(&mut self) -> Result<String> {
        let mut offset = 0u16;
        let mut all = Vec::new();

        loop {
            let (returned_offset, mut data, finished) = self.read_capabilities_packet(offset)?;
            if returned_offset != offset {
                return Err(Error::Protocol(format!(
                    "capabilities offset mismatch: requested {offset}, got {returned_offset}"
                )));
            }
            if let Some(nul) = data.iter().position(|b| *b == 0) {
                data.truncate(nul);
            }
            if data.is_empty() {
                break;
            }
            all.extend_from_slice(&data);
            if finished {
                break;
            }
            if all.len() > MAX_CAPABILITY_BYTES {
                return Err(Error::Protocol(
                    "capabilities string exceeded 64 KiB safety limit".into(),
                ));
            }
            offset = u16::try_from(all.len())
                .map_err(|_| Error::Protocol("capabilities offset overflow".into()))?;
        }

        String::from_utf8(all)
            .map_err(|e| Error::Protocol(format!("capabilities reply is not UTF-8/ASCII: {e}")))
    }
}

fn set_slave_address(file: &File) -> std::io::Result<()> {
    // SAFETY: ioctl is called with the documented Linux i2c-dev I2C_SLAVE request
    // and a scalar 7-bit address argument. file remains open for the call.
    let rc = unsafe { ioctl(file.as_raw_fd(), I2C_SLAVE, DDC_CI_7BIT_ADDRESS as c_ulong) };
    if rc < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn query_functionality(file: &File) -> std::io::Result<c_ulong> {
    let mut funcs: c_ulong = 0;
    // SAFETY: I2C_FUNCS expects a pointer to an unsigned long writable by the kernel.
    let rc = unsafe { ioctl(file.as_raw_fd(), I2C_FUNCS, &mut funcs as *mut c_ulong) };
    if rc < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(funcs)
    }
}

fn contextual_io(path: &Path, operation: &str, error: std::io::Error) -> Error {
    Error::Io(std::io::Error::new(
        error.kind(),
        format!("{operation} on {}: {error}", path.display()),
    ))
}
