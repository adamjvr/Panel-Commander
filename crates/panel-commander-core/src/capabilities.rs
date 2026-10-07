use crate::VcpCode;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcpCapability {
    pub code: VcpCode,
    /// Discrete values advertised in the capabilities string. An empty list
    /// means the monitor advertised the feature but did not provide an enum.
    pub values: Vec<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    pub raw: String,
    pub model: Option<String>,
    pub mccs_version: Option<String>,
    pub vcp_codes: BTreeSet<VcpCode>,
    pub vcp_features: BTreeMap<VcpCode, VcpCapability>,
}

impl Capabilities {
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let model = field_value(&raw, "model");
        let mccs_version = field_value(&raw, "mccs_ver");
        let vcp_features = parse_vcp_features(&raw);
        let vcp_codes = vcp_features.keys().copied().collect();
        Self {
            raw,
            model,
            mccs_version,
            vcp_codes,
            vcp_features,
        }
    }

    pub fn supports(&self, code: VcpCode) -> bool {
        self.vcp_codes.contains(&code)
    }

    pub fn advertised_values(&self, code: VcpCode) -> Option<&[u16]> {
        self.vcp_features
            .get(&code)
            .map(|feature| feature.values.as_slice())
    }
}

fn field_value(raw: &str, name: &str) -> Option<String> {
    let needle = format!("{name}(");
    let start = raw.find(&needle)? + needle.len();
    let rest = &raw[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

fn parse_vcp_features(raw: &str) -> BTreeMap<VcpCode, VcpCapability> {
    let mut out = BTreeMap::new();
    let Some(vcp_start) = raw.find("vcp(") else {
        return out;
    };

    let chars: Vec<char> = raw[vcp_start + 4..].chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() || chars[i] == ')' {
            break;
        }

        let code_start = i;
        while i < chars.len() && chars[i].is_ascii_hexdigit() {
            i += 1;
        }
        let code_token: String = chars[code_start..i].iter().collect();
        if code_token.len() != 2 {
            if i < chars.len() {
                i += 1;
            }
            continue;
        }
        let Ok(code) = u8::from_str_radix(&code_token, 16) else {
            continue;
        };

        let mut values = Vec::new();
        if i < chars.len() && chars[i] == '(' {
            i += 1;
            let values_start = i;
            let mut depth = 1usize;
            while i < chars.len() && depth > 0 {
                match chars[i] {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            let values_end = i.saturating_sub(1);
            if values_end >= values_start {
                let body: String = chars[values_start..values_end].iter().collect();
                values = body
                    .split_whitespace()
                    .filter_map(|token| u16::from_str_radix(token, 16).ok())
                    .collect();
            }
        }

        let code = VcpCode(code);
        out.insert(code, VcpCapability { code, values });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_model_version_and_vcp() {
        let c = Capabilities::parse(
            "(prot(monitor)type(LCD)model(XG27WCMS)mccs_ver(2.2)vcp(10 12 14(01 02) E0 E6))",
        );
        assert_eq!(c.model.as_deref(), Some("XG27WCMS"));
        assert_eq!(c.mccs_version.as_deref(), Some("2.2"));
        assert!(c.supports(VcpCode(0x10)));
        assert!(c.supports(VcpCode(0xe6)));
        assert!(!c.supports(VcpCode(0x01)));
        assert!(!c.supports(VcpCode(0x02)));
        assert_eq!(c.advertised_values(VcpCode(0x14)), Some(&[0x01, 0x02][..]));
        assert_eq!(c.advertised_values(VcpCode(0x10)), Some(&[][..]));
    }

    #[test]
    fn parses_full_xg27wcms_hardware_capability_set() {
        let c = Capabilities::parse(
            "(prot(monitor) type(LCD)model(XG27WCMS) cmds(01 02 03 07 0C F3) vcp(02 04 05 08 10 12 14(05 06 08 0B) 16 18 1A 52 60(11 1A 0F) 62 72(50 64 78 8C A0) 86(01 02 0B 0D 0F) 8A 8D(01 02) AC AE B6 C6 C8 CC(01 02 03 04 05 06 07 08 09 0A 0C 0D 11 12 14 1A 1E 1F 23 24 30 31) D6(01 05) DF DC(01 02 03 04 05 06 07 08 09 0A) DD(00 01) E0 E1(00 01) E2(00 01 02 03) E3(00 02 07 09 0A 0B 0C) E4(00 01 02 03 04 05) E5(00 01 02 03 04) E6 E7(00 01) E8(01 02 03 04 05 06 07 08) EA(00 01 02) EB(00 01 02 03 04 05 06 07) EC(01 FF) EE(00 01 02 03 04 05) FC(004F) FD(0007))mccs_ver(2.2)asset_eep(32)mpu(01)mswhql(1))",
        );
        assert_eq!(c.vcp_codes.len(), 42);
        assert!(c.supports(VcpCode(0xeb)));
        assert!(c.supports(VcpCode(0xfd)));
        assert_eq!(c.advertised_values(VcpCode(0xfc)), Some(&[0x004f][..]));
        assert_eq!(c.advertised_values(VcpCode(0xfd)), Some(&[0x0007][..]));
    }

    #[test]
    fn parses_xg27wcms_hardware_capability_values() {
        let c = Capabilities::parse(
            "(prot(monitor) type(LCD)model(XG27WCMS) vcp(14(05 06 08 0B) 60(11 1A 0F) E4(00 01 02 03 04 05) FC(004F) FD(0007))mccs_ver(2.2))",
        );
        assert_eq!(c.advertised_values(VcpCode(0x14)), Some(&[5, 6, 8, 11][..]));
        assert_eq!(
            c.advertised_values(VcpCode(0x60)),
            Some(&[0x11, 0x1a, 0x0f][..])
        );
        assert_eq!(
            c.advertised_values(VcpCode(0xe4)),
            Some(&[0, 1, 2, 3, 4, 5][..])
        );
        assert_eq!(c.advertised_values(VcpCode(0xfc)), Some(&[0x004f][..]));
        assert_eq!(c.advertised_values(VcpCode(0xfd)), Some(&[0x0007][..]));
    }
}
