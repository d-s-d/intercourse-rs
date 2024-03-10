use std::{borrow::BorrowMut, cell::RefCell, collections::HashSet, ops::Deref, rc::Rc};

use phantom_newtype::Amount;
use thiserror::Error;

use crate::person::{EmailAddr, Person};

#[derive(Default)]
pub struct PcDirectory {
    directory: Vec<PcDirectoryEntry>,
}

impl PcDirectory {
    pub fn iter_pcs(&self) -> impl Iterator<Item = &PcDirectoryEntry> {
        self.directory.iter()
    }

    /// Add a new PC to the directory.
    ///
    /// # Returns
    pub fn add_pc(&mut self, mut pcb: PcBuilder) -> Result<(), PcDirectoryError> {
        pcb.fill_defaults();

        let new_entry = if let Some(pivot_email) = pcb.owner.as_ref().map(|p| &p.email) {
            // In a real world scenario, we would of course store email addresses in
            // some lookup-table to quickly find entries containing that email
            // address.
            if let Some(entry) = self.iter_pcs().find(|e| {
                e.owner
                    .as_ref()
                    .map(|p| &p.email == pivot_email)
                    .unwrap_or_default()
            }) {
                // if the owner is not the same, return an error
                if entry.owner.as_ref().map(Rc::deref) != pcb.owner.as_ref() {
                    return Err(PcDirectoryError::DuplicateEmailAddress {
                        email: pivot_email.clone(),
                    });
                }
                PcDirectoryEntry::new(self.directory.len(), pcb, entry.owner.clone())
            } else {
                let owner = pcb.owner.take().map(Rc::new);
                PcDirectoryEntry::new(self.directory.len(), pcb, owner)
            }
        } else {
            PcDirectoryEntry::new(self.directory.len(), pcb, None)
        };
        self.directory.push(new_entry);
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum PcDirectoryError {
    #[error("A PC with a different owner, but same email address ({email:?}) already exists.")]
    DuplicateEmailAddress { email: EmailAddr },
}

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
    fn fill_defaults(&mut self) {
        if self.hardware.is_none() {
            self.hardware = Some(PcHardware::beefy_workstation());
        }
        if self.os.is_none() {
            self.os = Some(OperatingSystem::Linux { major: 5, minor: 5 });
        }
    }
}

pub struct PcDirectoryEntry {
    id: usize,
    hardware: PcHardware,
    state: RefCell<PcState>,
    owner: Option<Rc<Person>>,
}

impl PcDirectoryEntry {
    fn new(id: usize, builder: PcBuilder, owner: Option<Rc<Person>>) -> Self {
        Self {
            id,
            hardware: builder.hardware.unwrap(),
            state: RefCell::new(PcState {
                os: builder.os.unwrap(),
                mailbox: vec![],
                maintenance: OperationalState::On,
            }),
            owner,
        }
    }

    pub fn acquire_maintenance_lock<S: ToString>(
        &self,
        reason: S,
    ) -> Result<MaintenanceHandle<'_>, ()> {
        // fn get_maintenance_lock<'a, S: ToString>(&'a self, reason: S) -> Result<MaintenanceHandle<'a>, ()> {
        let mut state = self.state.borrow_mut();
        match state.maintenance {
            OperationalState::BeingMaintained { reason: _ } => Err(()),
            OperationalState::Off => Err(()),
            OperationalState::On => {
                state.maintenance = OperationalState::BeingMaintained {
                    reason: reason.to_string(),
                };
                Ok(MaintenanceHandle { state: &self.state })
            }
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct PcHardware {
    flags: HashSet<CpuFlag>,
    ram: NumBytes,
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

// This is a marker type that can and should not be instantiated.
pub enum Bytes {}
type NumBytes = Amount<Bytes, u64>;

pub const MEBIBYTE: NumBytes = NumBytes::new(1u64 << 20);
pub const GIBIBYTE: NumBytes = NumBytes::new(1u64 << 30);

pub struct PcState {
    os: OperatingSystem,
    mailbox: Vec<()>,
    maintenance: OperationalState,
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

#[derive(Debug, Clone)]
pub enum OperationalState {
    On,
    Off,
    BeingMaintained { reason: String },
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
        match self {
            Self::WindowsXp | Self::WindowsVista | Self::Windows7 | Self::Windows11 => true,
            _ => false,
        }
    }
}

pub struct MaintenanceHandle<'a> {
    state: &'a RefCell<PcState>,
}

impl<'a> MaintenanceHandle<'a> {
    pub fn update_os(&self, new: OperatingSystem) {
        self.state.borrow_mut().os = new;
    }
}

impl Drop for MaintenanceHandle<'_> {
    fn drop(&mut self) {
        self.state.borrow_mut().maintenance = OperationalState::On;
    }
}

#[cfg(test)]
mod tests {
    use crate::person::PersonBuilder;

    use super::*;

    #[test]
    fn test_maintenance() {
        let mut dir = PcDirectory::default();
        dir.add_pc(john_does_pc()).unwrap();
        let _handles = dir
            .iter_pcs()
            .map(|pc| pc.acquire_maintenance_lock("test"))
            .collect::<Vec<_>>();
    }

    #[test]
    fn test_maintenance_twice_fails() {
        let mut dir = PcDirectory::default();
        dir.add_pc(john_does_pc()).unwrap();
        let handles0: Result<Vec<_>, _> = dir
            .iter_pcs()
            .map(|pc| pc.acquire_maintenance_lock("test"))
            .collect();
        let handles1: Result<Vec<_>, _> = dir
            .iter_pcs()
            .map(|pc| pc.acquire_maintenance_lock("test"))
            .collect();

        assert!(matches!(handles0, Ok(_)));
        assert!(matches!(handles1, Err(_)));
    }

    #[test]
    fn test_release_maintenance() {
        let mut dir = PcDirectory::default();
        dir.add_pc(john_does_pc()).unwrap();

        {
            let handles = dir
                .iter_pcs()
                .map(|pc| pc.acquire_maintenance_lock("test"))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            handles[0].update_os(OperatingSystem::Linux { major: 5, minor: 5 });

            // The variable holding the locks on the maintenance state are release here.
        }
        let handles: Result<Vec<_>, _> = dir
            .iter_pcs()
            .map(|pc| pc.acquire_maintenance_lock("test"))
            .collect();
        assert!(matches!(handles, Ok(_)));
    }

    #[test]
    fn test_same_email_but_different_name_fails() {
        let mut dir = PcDirectory::default();
        dir.add_pc(john_does_pc()).unwrap();
        assert!(matches!(
            dir.add_pc(john2_does_pc()),
            Err(PcDirectoryError::DuplicateEmailAddress { email: _ })
        ));
    }

    fn john_does_pc() -> PcBuilder {
        PcBuilder {
            owner: Some(
                PersonBuilder::new()
                    .with_first_name("John")
                    .with_last_name("Doe")
                    .with_email_address("john@doe.com")
                    .build()
                    .unwrap(),
            ),
            os: Some(OperatingSystem::Windows7),
            ..Default::default()
        }
    }

    fn john2_does_pc() -> PcBuilder {
        PcBuilder {
            owner: Some(
                PersonBuilder::new()
                    .with_first_name("John2")
                    .with_last_name("Doe")
                    .with_email_address("john@doe.com")
                    .build()
                    .unwrap(),
            ),
            os: Some(OperatingSystem::Windows7),
            ..Default::default()
        }
    }
}
