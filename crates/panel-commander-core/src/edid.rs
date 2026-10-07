use crate::{Error, Result};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorIdentity {
    pub manufacturer: String,
    pub product_code: u16,
    pub serial_number: u32,
    pub monitor_name: Option<String>,
    pub serial_text: Option<String>,
}

impl Display for MonitorIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:04X} serial={:08X}",
            self.manufacturer, self.product_code, self.serial_number
        )?;
        if let Some(name) = &self.monitor_name {
            write!(f, " ({name})")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Edid {
    bytes: Vec<u8>,
    identity: MonitorIdentity,
}

impl Edid {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 128 {
            return Err(Error::InvalidEdid(format!(
                "EDID is {} bytes, expected at least 128",
                bytes.len()
            )));
        }
        if bytes[..8] != [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00] {
            return Err(Error::InvalidEdid("bad EDID header".into()));
        }
        if bytes[..128].iter().fold(0u8, |a, b| a.wrapping_add(*b)) != 0 {
            return Err(Error::InvalidEdid(
                "base block checksum does not sum to zero".into(),
            ));
        }

        let raw_mfg = u16::from_be_bytes([bytes[8], bytes[9]]);
        let manufacturer = decode_manufacturer(raw_mfg)?;
        let product_code = u16::from_le_bytes([bytes[10], bytes[11]]);
        let serial_number = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        let mut monitor_name = None;
        let mut serial_text = None;
        for start in [54usize, 72, 90, 108] {
            let d = &bytes[start..start + 18];
            if d[0..3] == [0, 0, 0] {
                match d[3] {
                    0xfc => monitor_name = descriptor_text(&d[5..18]),
                    0xff => serial_text = descriptor_text(&d[5..18]),
                    _ => {}
                }
            }
        }

        Ok(Self {
            bytes: bytes.to_vec(),
            identity: MonitorIdentity {
                manufacturer,
                product_code,
                serial_number,
                monitor_name,
                serial_text,
            },
        })
    }

    pub fn identity(&self) -> &MonitorIdentity {
        &self.identity
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

fn decode_manufacturer(raw: u16) -> Result<String> {
    let chars = [
        ((raw >> 10) & 0x1f) as u8,
        ((raw >> 5) & 0x1f) as u8,
        (raw & 0x1f) as u8,
    ];
    if chars.iter().any(|v| !(1..=26).contains(v)) {
        return Err(Error::InvalidEdid(format!(
            "invalid manufacturer code 0x{raw:04x}"
        )));
    }
    Ok(chars
        .iter()
        .map(|v| (b'A' + v - 1) as char)
        .collect::<String>())
}

fn descriptor_text(data: &[u8]) -> Option<String> {
    let end = data
        .iter()
        .position(|b| *b == 0x0a || *b == 0x00)
        .unwrap_or(data.len());
    let s = String::from_utf8_lossy(&data[..end]).trim().to_string();
    (!s.is_empty()).then_some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manufacturer_decoding() {
        // AUS = 1,21,19
        let raw = (1u16 << 10) | (21u16 << 5) | 19u16;
        assert_eq!(decode_manufacturer(raw).unwrap(), "AUS");
    }
}
