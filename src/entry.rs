use crate::bootloaderspec::FreedesktopBootEntry;
use crate::uefi::EfiEntry;
use std::fmt;

pub fn enumerate_all() -> Vec<Box<dyn BootEntry + Sync + Send>> {
    let mut ret: Vec<Box<dyn BootEntry + Sync + Send>> = Vec::new();

    let fd_entries: Vec<FreedesktopBootEntry> = FreedesktopBootEntry::enumerate();
    for entry in fd_entries {
        ret.push(Box::new(entry));
    };

    let uefi_entries: Vec<EfiEntry> = EfiEntry::enumerate();
    for entry in uefi_entries {
        ret.push(Box::new(entry));
    };

    ret
}

pub trait BootEntry {
    fn user_readable_name(&self) -> &str;
    fn enumerate() -> Vec<Self> where Self: Sized;
    fn boot(&self);
    fn hide(&self) -> bool;
}

impl fmt::Display for dyn BootEntry + Send + Sync {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.user_readable_name())
    }
}

impl PartialEq for dyn BootEntry {
    fn eq(&self, other: &Self) -> bool {
        self.user_readable_name() == other.user_readable_name()
    }
    fn ne(&self, other: &Self) -> bool {
        self.user_readable_name() != other.user_readable_name()
    }
}
