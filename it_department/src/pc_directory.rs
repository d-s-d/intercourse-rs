use std::{cell::RefCell, rc::Rc};

use crate::{pc::{OperatingSystem, PcBuilder, PcHardware}, person::{Affiliation, ChfAmout, EmailAddr, Person, PersonBuilder}};
use thiserror::Error;

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
                if entry.owner.as_deref() != pcb.owner.as_ref() {
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

    /// Send an email to the person with address [`to`]. The email will be put
    /// into mailbox of the first PC that is turned on and belongs to the person
    /// with the given email address.
    pub fn send_email<E: TryInto<EmailAddr>, T: ToString>(
        &self,
        to: E,
        message: T,
    ) -> Result<(), PcDirectoryError> {
        let Ok(to) = to.try_into() else {
            return Err(PcDirectoryError::InvalidEMailAddress);
        };
        let mut owned_pc: Vec<_> = self
            .directory
            .iter()
            .filter(|pc| {
                pc.owner
                    .as_deref()
                    .map(|p| p.email == to)
                    .unwrap_or_default()
            })
            .collect();
        if owned_pc.is_empty() {
            return Err(PcDirectoryError::EmailNotFound { email: to });
        }
        if let Some(state) = owned_pc
            .iter_mut()
            .find(|pc| pc.state.borrow().maintenance.is_on())
            .map(|pc| pc.state.borrow_mut())
        {
            state.mailbox.borrow_mut().push(message.to_string());
            return Ok(());
        }
        Err(PcDirectoryError::Unavailable)
    }
}

impl<T> From<T> for PcDirectory
where
    T: IntoIterator<Item = PcBuilder>,
{
    fn from(iter: T) -> Self {
        let mut dir = PcDirectory::default();
        iter.into_iter().for_each(|pcb| dir.add_pc(pcb).unwrap());
        dir
    }
}

#[derive(Debug, Error)]
pub enum PcDirectoryError {
    #[error("A PC with a different owner, but same email address ({email:?}) already exists.")]
    DuplicateEmailAddress { email: EmailAddr },
    #[error("None of the available PCs has an owner with the given email address ({email:?}).")]
    EmailNotFound { email: EmailAddr },
    #[error("All affected PCs are either off or in maintenance mode.")]
    Unavailable,
    #[error("The requested PC is already in maintenance: {reason}")]
    InMaintenance { reason: String },
    #[error("The provided email address is invalid.")]
    InvalidEMailAddress,
}

pub struct PcDirectoryEntry {
    pub id: usize,
    pub hardware: PcHardware,
    pub owner: Option<Rc<Person>>,
    state: RefCell<PcState>,
}

impl PcDirectoryEntry {
    fn new(id: usize, builder: PcBuilder, owner: Option<Rc<Person>>) -> Self {
        Self {
            id,
            hardware: builder.hardware.unwrap(),
            state: RefCell::new(PcState {
                os: builder.os.unwrap(),
                mailbox: Default::default(),
                maintenance: OperationalState::On,
            }),
            owner,
        }
    }

    pub fn acquire_maintenance_lock<S: ToString>(
        &self,
        reason: S,
    ) -> Result<MaintenanceHandle<'_>, PcDirectoryError> {
        let mut state = self.state.borrow_mut();
        match &state.maintenance {
            OperationalState::BeingMaintained { reason } => Err(PcDirectoryError::InMaintenance {
                reason: reason.clone(),
            }),
            OperationalState::Off => Err(PcDirectoryError::Unavailable),
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


pub struct PcState {
    os: OperatingSystem,
    mailbox: RefCell<Vec<String>>,
    maintenance: OperationalState,
}

#[derive(Debug, Clone)]
pub enum OperationalState {
    On,
    Off,
    BeingMaintained { reason: String },
}

impl OperationalState {
    pub fn is_on(&self) -> bool {
        matches!(self, &OperationalState::On)
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

#[rustfmt::skip]
pub fn get_directory() -> PcDirectory {
    use OperatingSystem::*;
    let linux6 = Linux {
        major: 6,
        minor: 22,
    };
    let macos10 = MacOs {
        major: 10,
        minor: 14,
    };
    let super_income = Affiliation::Employee {
        annual_income: ChfAmout::new(10),
    };
    let mid_income = Affiliation::Employee {
        annual_income: ChfAmout::new(5),
    };
    let contractor = Affiliation::Contractor {
        company_name: "minisoft".into(),
    };

    [
        ("Maria", "Dingdong", "maria@dingong.com",   super_income.clone(), Windows11,      PcHardware::beefy_workstation()),
        ("Hans",  "Overkill", "hans@overkill.com",   super_income.clone(), linux6.clone(), PcHardware::nerd_workstation()),
        ("Sue",   "Sensible", "sue@whatever.com",    Affiliation::Intern,  macos10,        PcHardware::beefy_workstation()),
        ("Don",   "Drumpf",   "don@drumpf.com",      mid_income,           WindowsVista,   PcHardware::normal()),
        ("Lex",   "Long",     "lexlong@voll.com",    contractor,           WindowsVista,   PcHardware::normal()),
        ("Karl",  "Keule",    "karl@keule.com",      super_income,         linux6,         PcHardware::nerd_workstation()),
    ].into_iter().map(|item| {
        PcBuilder {
            owner: Some(
                PersonBuilder::new()
                .with_first_name(item.0)
                .with_last_name(item.1)
                .with_email_address(item.2)
                .with_affiliation(item.3)
                .build()
                .unwrap(),
            ),
            os: Some(item.4),
            hardware: Some(item.5)
        }
        }).into()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet, fs::{File, OpenOptions}, io::Write, path::PathBuf
    };

    use crate::person::{Affiliation, PersonBuilder};

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

        assert!(handles0.is_ok());
        assert!(handles1.is_err());
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
        assert!(handles.is_ok());
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

    #[test]
    fn test_email_does_not_exist() {
        let dir: PcDirectory = [john_does_pc(), maria_dingong_pc()].into();

        let given_email = EmailAddr::try_from("dings@bla.com").unwrap();
        assert!(matches!(
            dir.send_email(given_email.clone(), "my_message"),
            Err( PcDirectoryError::EmailNotFound { email } ) if email == given_email
        ));
    }

    #[test]
    fn test_cannot_send_email_to_don_while_upgrading_vist() {
        let dir = get_directory();

        // small helper
        fn vista_users(dir: &PcDirectory) -> impl Iterator<Item = &PcDirectoryEntry> {
            dir.iter_pcs()
                .filter(|pc| matches!(&pc.state.borrow().os, &OperatingSystem::WindowsVista))
        }

        // send all vista users an email
        vista_users(&dir)
            .filter_map(|pc| pc.owner.as_deref().map(|p| p.email.clone()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .try_for_each(|addr| dir.send_email(addr, "upgrade!"))
            .unwrap();

        // let's open up a maintenance window
        {
            let handles: Result<Vec<_>, _> = vista_users(&dir)
                .map(|pc| pc.acquire_maintenance_lock("Update from windows vista!"))
                .collect();

            let handles = handles.unwrap();
            for handle in handles.iter() {
                handle.state.borrow_mut().os = OperatingSystem::Windows11;
            }

            assert!(dir
                .send_email("don@drumpf.com", "You are now on Windows 11!")
                .is_err());
        }

        // but here, we can again!
        assert!(dir
            .send_email("don@drumpf.com", "You are now on Windows 11!")
            .is_ok());
    }

    fn john_does_pc() -> PcBuilder {
        PcBuilder {
            owner: Some(
                PersonBuilder::new()
                    .with_first_name("John")
                    .with_last_name("Doe")
                    .with_email_address("john@doe.com")
                    .with_affiliation(Affiliation::Intern)
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
                    .with_affiliation(Affiliation::Intern)
                    .build()
                    .unwrap(),
            ),
            os: Some(OperatingSystem::Windows7),
            ..Default::default()
        }
    }

    fn maria_dingong_pc() -> PcBuilder {
        PcBuilder {
            owner: Some(
                PersonBuilder::new()
                    .with_first_name("Maria")
                    .with_last_name("Dingdong")
                    .with_email_address("maria@dingdong.com")
                    .with_affiliation(Affiliation::Intern)
                    .build()
                    .unwrap(),
            ),
            os: Some(OperatingSystem::Windows11),
            ..Default::default()
        }
    }

    #[test]
    fn test_showcase_file_drop() {
        struct MySuperFile {
            // The file is closed when MySuperFile is dropped.
            f: File,
        }

        impl MySuperFile {
            fn new() -> Self {
                let tmp_dir = std::env::var("TMPDIR").unwrap();
                let path = PathBuf::from(tmp_dir).join("rust_testfile");
                Self {
                    f: OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(path)
                        .unwrap(),
                }
            }

            fn write_hello(&mut self) {
                let _ = self.f.write_all("hello".as_bytes());
            }
        }

        {
            let mut f = MySuperFile::new();
            f.write_hello();
            // file is closed here
        }

        // f.write_hello() // <-- this would fail
    }
}
