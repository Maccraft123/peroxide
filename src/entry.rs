use crate::bootloaderspec::FreedesktopBootEntry;
use crate::acetone::AcetoneEntry;

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

    ret
}

pub trait BootEntry {
    fn user_readable_name(&self) -> &str;
    fn enumerate() -> Vec<Self> where Self: Sized;
    fn boot(&self);
    fn hide(&self) -> bool;
}
