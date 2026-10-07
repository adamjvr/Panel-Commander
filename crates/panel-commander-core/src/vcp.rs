use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VcpCode(pub u8);

impl VcpCode {
    pub const BRIGHTNESS: Self = Self(0x10);
    pub const CONTRAST: Self = Self(0x12);
    pub const COLOR_PRESET: Self = Self(0x14);
    pub const RED_GAIN: Self = Self(0x16);
    pub const GREEN_GAIN: Self = Self(0x18);
    pub const BLUE_GAIN: Self = Self(0x1a);
    pub const INPUT_SOURCE: Self = Self(0x60);
    pub const AUDIO_VOLUME: Self = Self(0x62);
    pub const RED_OFFSET: Self = Self(0x6c);
    pub const GREEN_OFFSET: Self = Self(0x6e);
    pub const BLUE_OFFSET: Self = Self(0x70);
    pub const GAMMA: Self = Self(0x72);
    pub const SATURATION: Self = Self(0x8a);
    pub const HUE: Self = Self(0x90);
    pub const POWER_MODE: Self = Self(0xd6);
    pub const MCCS_VERSION: Self = Self(0xdf);

    pub const ASUS_OVERDRIVE: Self = Self(0xe0);
    pub const ASUS_POWER_SAVING: Self = Self(0xe1);
    pub const ASUS_SCREEN_SAVER_E2: Self = Self(0xe2);
    pub const ASUS_CROSSHAIR: Self = Self(0xe3);
    pub const ASUS_PRIVATE_E4: Self = Self(0xe4);
    pub const ASUS_PRIVATE_E5: Self = Self(0xe5);
    pub const ASUS_BLUE_LIGHT_FILTER: Self = Self(0xe6);
    pub const ASUS_DISPLAY_ALIGNMENT: Self = Self(0xe7);
    pub const ASUS_GAMEPLUS_POSITION: Self = Self(0xe8);
    pub const ASUS_FPS_COUNTER: Self = Self(0xea);
    pub const ASUS_RESET_MODE_DEFAULT: Self = Self(0xec);
    pub const ASUS_PROXIMITY_SENSOR: Self = Self(0xed);
    pub const ASUS_SCREEN_SAVER_F9: Self = Self(0xf9);

    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub fn parse(text: &str) -> Option<Self> {
        let t = text.trim();
        let t = t
            .strip_prefix("0x")
            .or_else(|| t.strip_prefix("0X"))
            .unwrap_or(t);
        u8::from_str_radix(t, 16).ok().map(Self)
    }
}

impl Display for VcpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02X}", self.0)
    }
}
