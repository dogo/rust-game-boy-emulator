#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareModel {
    Dmg0,   // Game Boy original (DMG-0)
    DmgAbc, // Game Boy original (DMG-A/B/C)
    Mgb,    // Game Boy Pocket (MGB)
    Sgb,    // Super Game Boy (SGB)
    Sgb2,   // Super Game Boy 2 (SGB2)
    Cgb,    // Game Boy Color (CGB-A/B/C/D/E)
    Cgb0,   // Game Boy Color first revision (CGB-0)
    Agb,    // Game Boy Advance running in Game Boy Color mode (AGB/AGS)
}

impl HardwareModel {
    pub const NAMES: &'static str = "dmg0, dmg, mgb, sgb, sgb2, cgb0, cgb, agb";

    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "dmg0" => Self::Dmg0,
            "dmg" => Self::DmgAbc,
            "mgb" => Self::Mgb,
            "sgb" => Self::Sgb,
            "sgb2" => Self::Sgb2,
            "cgb0" => Self::Cgb0,
            "cgb" => Self::Cgb,
            "agb" => Self::Agb,
            _ => return None,
        })
    }

    pub fn is_sgb(self) -> bool {
        matches!(self, Self::Sgb | Self::Sgb2)
    }

    pub fn is_cgb(self) -> bool {
        matches!(self, Self::Cgb | Self::Cgb0 | Self::Agb)
    }
}
