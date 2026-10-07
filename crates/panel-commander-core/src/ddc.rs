use crate::{Error, Result, VcpCode};

pub const DDC_CI_7BIT_ADDRESS: u16 = 0x37;
pub const DDC_CI_WRITE_ADDRESS: u8 = 0x6e;
pub const DDC_CI_VIRTUAL_HOST_ADDRESS: u8 = 0x50;
pub const DDC_CI_HOST_SOURCE: u8 = 0x51;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VcpReply {
    pub code: VcpCode,
    pub result_code: u8,
    pub vcp_type: u8,
    pub maximum: u16,
    pub current: u16,
}

impl VcpReply {
    pub fn supported(&self) -> bool {
        self.result_code == 0
    }
}

pub trait DdcTransport {
    fn get_vcp(&mut self, code: VcpCode) -> Result<VcpReply>;
    fn set_vcp(&mut self, code: VcpCode, value: u16) -> Result<()>;
    fn capabilities(&mut self) -> Result<String> {
        Err(Error::Unsupported(
            "capabilities request is not implemented by this transport".into(),
        ))
    }
}

pub fn xor_checksum(seed: u8, bytes: &[u8]) -> u8 {
    bytes.iter().fold(seed, |acc, b| acc ^ b)
}

/// Bytes written to I2C slave 0x37 for Get VCP Feature.
/// The outer write address 0x6e participates in the checksum but is not part of this buffer.
pub fn encode_get_vcp(code: VcpCode) -> [u8; 5] {
    let body = [DDC_CI_HOST_SOURCE, 0x82, 0x01, code.0];
    let checksum = xor_checksum(DDC_CI_WRITE_ADDRESS, &body);
    [body[0], body[1], body[2], body[3], checksum]
}

/// Bytes written to I2C slave 0x37 for Set VCP Feature.
pub fn encode_set_vcp(code: VcpCode, value: u16) -> [u8; 7] {
    let body = [
        DDC_CI_HOST_SOURCE,
        0x84,
        0x03,
        code.0,
        (value >> 8) as u8,
        value as u8,
    ];
    let checksum = xor_checksum(DDC_CI_WRITE_ADDRESS, &body);
    [
        body[0], body[1], body[2], body[3], body[4], body[5], checksum,
    ]
}

/// Capabilities Request at a byte offset.
pub fn encode_capabilities_request(offset: u16) -> [u8; 6] {
    let body = [
        DDC_CI_HOST_SOURCE,
        0x83,
        0xf3,
        (offset >> 8) as u8,
        offset as u8,
    ];
    let checksum = xor_checksum(DDC_CI_WRITE_ADDRESS, &body);
    [body[0], body[1], body[2], body[3], body[4], checksum]
}

/// Decode the 11-byte VCP Feature Reply beginning with source address 0x6e.
pub fn decode_vcp_reply(bytes: &[u8]) -> Result<VcpReply> {
    if bytes.len() < 11 {
        return Err(Error::Protocol(format!(
            "short VCP reply: got {}, need 11 bytes",
            bytes.len()
        )));
    }
    if bytes[0] == 0x6e && bytes[1] == 0x80 {
        return Err(Error::Protocol(
            "monitor returned DDC/CI null message".into(),
        ));
    }
    if bytes[0] != 0x6e {
        return Err(Error::Protocol(format!(
            "unexpected reply source 0x{:02x}",
            bytes[0]
        )));
    }
    if bytes[1] != 0x88 || bytes[2] != 0x02 {
        return Err(Error::Protocol(format!(
            "unexpected VCP reply header {:02x} {:02x}",
            bytes[1], bytes[2]
        )));
    }
    let checksum = xor_checksum(DDC_CI_VIRTUAL_HOST_ADDRESS, &bytes[..11]);
    if checksum != 0 {
        return Err(Error::Protocol(format!(
            "invalid reply checksum (xor=0x{checksum:02x})"
        )));
    }
    Ok(VcpReply {
        result_code: bytes[3],
        code: VcpCode(bytes[4]),
        vcp_type: bytes[5],
        maximum: u16::from_be_bytes([bytes[6], bytes[7]]),
        current: u16::from_be_bytes([bytes[8], bytes[9]]),
    })
}

/// Decode one Capabilities Reply packet. Returns (offset, data, finished).
pub fn decode_capabilities_reply(bytes: &[u8]) -> Result<(u16, Vec<u8>, bool)> {
    if bytes.len() < 6 {
        return Err(Error::Protocol("short capabilities reply".into()));
    }
    if bytes[0] == 0x6e && bytes[1] == 0x80 {
        return Err(Error::Protocol(
            "monitor returned DDC/CI null message".into(),
        ));
    }
    if bytes[0] != 0x6e {
        return Err(Error::Protocol(format!(
            "unexpected capabilities source 0x{:02x}",
            bytes[0]
        )));
    }
    let payload_len = (bytes[1] & 0x7f) as usize;
    // Payload is opcode E3 + two-byte offset + data.
    if payload_len < 3 {
        return Err(Error::Protocol(format!(
            "invalid capabilities payload length {payload_len}"
        )));
    }
    let total = 2 + payload_len + 1;
    if bytes.len() < total {
        return Err(Error::Protocol(format!(
            "short capabilities packet: advertised {total}, got {}",
            bytes.len()
        )));
    }
    if bytes[2] != 0xe3 {
        return Err(Error::Protocol(format!(
            "unexpected capabilities opcode 0x{:02x}",
            bytes[2]
        )));
    }
    if xor_checksum(DDC_CI_VIRTUAL_HOST_ADDRESS, &bytes[..total]) != 0 {
        return Err(Error::Protocol("invalid capabilities checksum".into()));
    }
    let offset = u16::from_be_bytes([bytes[3], bytes[4]]);
    let data = bytes[5..total - 1].to_vec();
    let finished = data.is_empty() || data.contains(&0);
    Ok((offset, data, finished))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_vcp_brightness_50_is_byte_exact() {
        let p = encode_set_vcp(VcpCode::BRIGHTNESS, 50);
        assert_eq!(&p[..6], &[0x51, 0x84, 0x03, 0x10, 0x00, 0x32]);
        assert_eq!(
            DDC_CI_WRITE_ADDRESS ^ p.iter().copied().fold(0, |a, b| a ^ b),
            0
        );
    }

    #[test]
    fn get_vcp_packet_checksum() {
        let p = encode_get_vcp(VcpCode::BRIGHTNESS);
        assert_eq!(&p[..4], &[0x51, 0x82, 0x01, 0x10]);
        assert_eq!(
            DDC_CI_WRITE_ADDRESS ^ p.iter().copied().fold(0, |a, b| a ^ b),
            0
        );
    }

    #[test]
    fn parse_known_reply() {
        // Brightness 50/100. Final checksum is computed using virtual host 0x50.
        let mut p = [
            0x6e, 0x88, 0x02, 0x00, 0x10, 0x00, 0x00, 0x64, 0x00, 0x32, 0,
        ];
        p[10] = xor_checksum(DDC_CI_VIRTUAL_HOST_ADDRESS, &p[..10]);
        let r = decode_vcp_reply(&p).unwrap();
        assert_eq!(r.code, VcpCode::BRIGHTNESS);
        assert_eq!(r.current, 50);
        assert_eq!(r.maximum, 100);
    }
}
