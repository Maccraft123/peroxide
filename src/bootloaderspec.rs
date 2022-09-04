use crate::entry::BootEntry;
use crate::boot::kexec;
use crate::boot::KexecData;

use std::fs::{
    self,
    File,
};
use std::io::{
    BufRead,
    BufReader
};

#[derive(Debug)]
pub struct FreedesktopBootEntry {
    title: Option<String>,
    version: Option<String>,
    machine_id: Option<String>,
    linux: String,
    initrd: Option<String>,
    options: Option<String>,
    devicetree: Option<String>,
}

impl FreedesktopBootEntry {
    fn from_file(input: File) -> Option<Self> {
        let mut title = None;
        let mut version = None;
        let mut machine_id = None;
        let mut linux = None;
        let mut initrd = None;
        let mut options = None;
        let mut devicetree = None;

        let reader = BufReader::new(input);
        for maybe_line in reader.lines() {
            if let Ok(line) = maybe_line {
                if line.starts_with('#') {
                    continue;
                }

                if let Some(new_title) = line.strip_prefix("title ") {
                    title = Some(new_title.to_string());
                }
                if let Some(new_version) = line.strip_prefix("version ") {
                    version = Some(new_version.to_string());
                }
                if let Some(new_machine_id) = line.strip_prefix("machine-id ") {
                    machine_id = Some(new_machine_id.to_string());
                }
                if let Some(new_linux) = line.strip_prefix("linux ") {
                    linux = Some(new_linux.to_string());
                }
                if let Some(new_initrd) = line.strip_prefix("initrd ") {
                    initrd = Some(new_initrd.to_string());
                }
                if let Some(new_options) = line.strip_prefix("options ") {
                    options = Some(new_options.to_string());
                }
                if let Some(new_devicetree) = line.strip_prefix("devicetree ") {
                    devicetree  = Some(new_devicetree.to_string());
                }
            }
        }

        if linux.is_none() {
            None
        } else {
            Some(Self {
                linux: linux.unwrap().to_string(),
                title, version, machine_id,
                initrd, options, devicetree,
            })
        }
    }
}

impl BootEntry for FreedesktopBootEntry {
    fn user_readable_name(&self) -> &str {
        if let Some(title) = &self.title {
            return title;
        } else {
            return &self.linux;
        }
    }
    fn enumerate() -> Vec<FreedesktopBootEntry> {
        let mut ret = Vec::new();
        if let Ok(dir) = fs::read_dir("/boot/loader/entries/") {
            for maybe_entry in dir {
                if let Ok(entry) = maybe_entry {
                    if let Ok(file) = File::open(entry.path()) {
                        if let Some(boot_entry) = FreedesktopBootEntry::from_file(file) {
                            ret.push(boot_entry);
                        }
                    }
                }
            }
        }
        ret
    }
    fn boot(&self) {
        println!("Booting {:?}({})", self.title, self.linux);
        let abs_initrd_path = if let Some(rel_initrd_path) = self.initrd.as_ref() {
            Some(format!("/boot/{}", rel_initrd_path))
        } else {
            None
        };
        let data = KexecData {
            kernel: format!("/boot/{}", self.linux.clone()),
            cmdline: self.options.clone(),
            initrd: abs_initrd_path,
            dt: self.devicetree.clone(),
        };
        kexec(data);
    }
    fn hide(&self) -> bool {
        false
    }
}
