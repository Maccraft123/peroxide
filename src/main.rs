mod entry;
mod acetone;
mod bootloaderspec;

use crate::entry::BootEntry;

fn main() {
    let entries = entry::enumerate_all();
    for entry in entries {
        println!("{}", entry.user_readable_name());
    }
}
