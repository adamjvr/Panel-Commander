#![forbid(unsafe_code)]

use panel_commander_core::{MonitorIdentity, VcpCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsusSemantic {
    Overdrive,
    PowerSaving,
    ScreenSaver,
    Crosshair,
    GamePlusTimer,
    InputRange,
    DynamicDimming,
    ShadowBoost,
    BlueLightFilter,
    DisplayAlignment,
    GamePlusPosition,
    FpsCounter,
    ResetModeDefault,
    ProximitySensor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrivateFeature {
    pub code: VcpCode,
    pub semantic: AsusSemantic,
    pub model_gated: bool,
    /// Writes remain disabled until the physical XG27WCMS value semantics have
    /// been verified. Static reverse-engineering evidence alone is not enough.
    pub writable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsusModel {
    Xg27wcms,
    UnsupportedAsus,
    NonAsus,
    Unknown,
}

impl AsusModel {
    pub fn identify(identity: &MonitorIdentity) -> Self {
        if identity.manufacturer == "AUS" && identity.product_code == 0x275e {
            Self::Xg27wcms
        } else if matches!(identity.manufacturer.as_str(), "AUS" | "ACI") {
            Self::UnsupportedAsus
        } else {
            Self::NonAsus
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Xg27wcms => "XG27WCMS",
            Self::UnsupportedAsus => "unsupported ASUS model",
            Self::NonAsus => "non-ASUS display",
            Self::Unknown => "unknown display",
        }
    }

    pub fn private_features(self) -> &'static [PrivateFeature] {
        match self {
            Self::Xg27wcms => XG27WCMS_PRIVATE_FEATURES,
            Self::UnsupportedAsus | Self::NonAsus | Self::Unknown => &[],
        }
    }

    pub fn private_feature(self, code: VcpCode) -> Option<&'static PrivateFeature> {
        self.private_features()
            .iter()
            .find(|feature| feature.code == code)
    }
}

/// Hardware-visible/private ASUS commands recovered from the ASUS software and
/// now cross-checked against the physical XG27WCMS capability string.
///
/// No private feature is write-enabled yet. Read-only characterization must
/// establish the exact value semantics before a write can be unlocked.
pub const XG27WCMS_PRIVATE_FEATURES: &[PrivateFeature] = &[
    PrivateFeature {
        code: VcpCode(0xe0),
        semantic: AsusSemantic::Overdrive,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xe1),
        semantic: AsusSemantic::PowerSaving,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xe2),
        semantic: AsusSemantic::ScreenSaver,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xe3),
        semantic: AsusSemantic::Crosshair,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xe6),
        semantic: AsusSemantic::BlueLightFilter,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xe7),
        semantic: AsusSemantic::DisplayAlignment,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xe8),
        semantic: AsusSemantic::GamePlusPosition,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xea),
        semantic: AsusSemantic::FpsCounter,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xec),
        semantic: AsusSemantic::ResetModeDefault,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xed),
        semantic: AsusSemantic::ProximitySensor,
        model_gated: true,
        writable: false,
    },
    PrivateFeature {
        code: VcpCode(0xf9),
        semantic: AsusSemantic::ScreenSaver,
        model_gated: true,
        writable: false,
    },
];

pub fn known_probe_codes(model: AsusModel) -> Vec<VcpCode> {
    let mut out = vec![
        VcpCode::BRIGHTNESS,
        VcpCode::CONTRAST,
        VcpCode::COLOR_PRESET,
        VcpCode::RED_GAIN,
        VcpCode::GREEN_GAIN,
        VcpCode::BLUE_GAIN,
        VcpCode::INPUT_SOURCE,
        VcpCode::AUDIO_VOLUME,
        VcpCode::SATURATION,
        VcpCode::HUE,
        VcpCode::POWER_MODE,
        VcpCode::MCCS_VERSION,
    ];
    out.extend(model.private_features().iter().map(|feature| feature.code));
    if model == AsusModel::Xg27wcms {
        // Ambiguous private aliases remain read-only.
        out.push(VcpCode(0xe4));
        out.push(VcpCode(0xe5));
    }
    out.sort();
    out.dedup();
    out
}

pub fn semantic_name(model: AsusModel, code: VcpCode) -> &'static str {
    match code.0 {
        0x10 => "Brightness",
        0x12 => "Contrast",
        0x14 => "Color preset",
        0x16 => "Red gain",
        0x18 => "Green gain",
        0x1a => "Blue gain",
        0x60 => "Input source",
        0x62 => "Audio volume",
        0x6c => "Red offset",
        0x6e => "Green offset",
        0x70 => "Blue offset",
        0x72 => "Gamma",
        0x8a => "Saturation",
        0x90 => "Hue",
        0xd6 => "Power mode",
        0xdf => "MCCS version",
        0xe0 if model == AsusModel::Xg27wcms => "ASUS Overdrive",
        0xe1 if model == AsusModel::Xg27wcms => "ASUS Power saving",
        0xe2 if model == AsusModel::Xg27wcms => "ASUS Screen saver (E2)",
        0xe3 if model == AsusModel::Xg27wcms => "ASUS Crosshair",
        0xe4 if model == AsusModel::Xg27wcms => "ASUS E4 (multiplexed: GamePlus timer/Input range)",
        0xe5 if model == AsusModel::Xg27wcms => {
            "ASUS E5 (multiplexed: Dynamic dimming/Shadow boost)"
        }
        0xe6 if model == AsusModel::Xg27wcms => "ASUS Blue-light filter",
        0xe7 if model == AsusModel::Xg27wcms => "ASUS Display alignment",
        0xe8 if model == AsusModel::Xg27wcms => "ASUS GamePlus position",
        0xea if model == AsusModel::Xg27wcms => "ASUS FPS counter",
        0xec if model == AsusModel::Xg27wcms => "ASUS Reset mode default",
        0xed if model == AsusModel::Xg27wcms => "ASUS Proximity sensor",
        0xf9 if model == AsusModel::Xg27wcms => "ASUS Screen saver (F9)",
        _ => "Unknown/monitor-defined VCP",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(manufacturer: &str, product_code: u16) -> MonitorIdentity {
        MonitorIdentity {
            manufacturer: manufacturer.to_string(),
            product_code,
            serial_number: 0,
            monitor_name: None,
            serial_text: None,
        }
    }

    #[test]
    fn distinguishes_known_unknown_and_non_asus_displays() {
        assert_eq!(
            AsusModel::identify(&identity("AUS", 0x275e)),
            AsusModel::Xg27wcms
        );
        assert_eq!(
            AsusModel::identify(&identity("ACI", 0x249a)),
            AsusModel::UnsupportedAsus
        );
        assert_eq!(
            AsusModel::identify(&identity("SAM", 0x0e33)),
            AsusModel::NonAsus
        );
    }

    #[test]
    fn private_writes_are_not_enabled_by_static_evidence() {
        assert!(XG27WCMS_PRIVATE_FEATURES
            .iter()
            .all(|feature| !feature.writable));
    }
}
