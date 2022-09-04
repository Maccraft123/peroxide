use crate::bootloaderspec::FreedesktopBootEntry;
use crate::acetone::AcetoneEntry;
use crate::uefi::EfiEntry;

pub fn enumerate_all() -> Vec<Box<dyn BootEntry>> {
    let mut ret: Vec<Box<dyn BootEntry>> = Vec::new();

    let fd_entries: Vec<FreedesktopBootEntry> = FreedesktopBootEntry::enumerate();
    for entry in fd_entries {
        ret.push(Box::new(entry));
    };

    let acetone_entries: Vec<AcetoneEntry> = AcetoneEntry::enumerate();
    for entry in acetone_entries {
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

impl PartialEq for dyn BootEntry {
    fn eq(&self, other: &Self) -> bool {
        self.user_readable_name() == other.user_readable_name()
    }
    fn ne(&self, other: &Self) -> bool {
        self.user_readable_name() != other.user_readable_name()
    }
}
