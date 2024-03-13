use std::collections::HashSet;

use phantom_newtype::Amount;

use crate::person::Person;


#[derive(Default, Debug, Clone)]
pub struct PcBuilder {
    pub hardware: Option<PcHardware>,
    pub os: Option<OperatingSystem>,
    pub owner: Option<Person>,
}

impl PcBuilder {
    // Let's assume we don't want to set assume global defaults for the
    // individual types, we set the defaults for the fields here in the
    // builder.
    pub fn fill_defaults(&mut self) {
        if self.hardware.is_none() {
            self.hardware = Some(PcHardware::beefy_workstation());
        }
        if self.os.is_none() {
            self.os = Some(OperatingSystem::Linux { major: 5, minor: 5 });
        }
    }
}

#[derive(Debug, Clone)]
pub struct PcHardware {
    pub flags: HashSet<CpuFlag>,
    pub ram: NumBytes,
}

impl PcHardware {
    pub fn nerd_workstation() -> Self {
        use CpuFlag::*;
        Self {
            flags: [MMX, SSE, SEV, AVX].into_iter().collect(),
            ram: NumBytes::new(GIBIBYTE.get() * 64),
        }
    }

    pub fn beefy_workstation() -> Self {
        use CpuFlag::*;
        Self {
            flags: [MMX, SSE, SEV].into_iter().collect(),
            ram: NumBytes::new(GIBIBYTE.get() * 32),
        }
    }

    pub fn normal() -> Self {
        use CpuFlag::*;
        Self {
            flags: [MMX, SSE].into_iter().collect(),
            ram: NumBytes::new(GIBIBYTE.get() * 16),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CpuFlag {
    MMX,
    SSE,
    SEV,
    AVX,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperatingSystem {
    WindowsXp,
    WindowsVista,
    Windows7,
    Windows11,
    MacOs { major: u16, minor: u16 },
    Linux { major: u16, minor: u16 },
}


impl OperatingSystem {
    // ;-)
    pub fn is_crappy(&self) -> bool {
        match self {
            Self::WindowsXp | Self::WindowsVista => true,
            Self::Linux { major, minor: _ } if *major < 5 => true,
            _ => false,
        }
    }

    pub fn is_windows(&self) -> bool {
        matches!(
            self,
            Self::WindowsXp | Self::WindowsVista | Self::Windows7 | Self::Windows11
        )
        /* ... as a tip from clippy revealed, the above is the same as:
        match self {
            Self::WindowsXp | Self::WindowsVista | Self::Windows7 | Self::Windows11 => true,
            _ => false,
        }
        */
    }
}

// This is a marker type that can and should not be instantiated.
pub enum Bytes {}
type NumBytes = Amount<Bytes, u64>;

pub const MEBIBYTE: NumBytes = NumBytes::new(1u64 << 20);
pub const GIBIBYTE: NumBytes = NumBytes::new(1u64 << 30);