use crate::entry::BootEntry;
use std::fs::{
    self,
    File,
};
use std::io::{
    BufRead,
    BufReader
};

#[derive(Debug)]
pub struct AcetoneEntry {
    title: Option<String>,
    version: Option<String>,
    machine_id: Option<String>,
    linux: String,
    initrd: Option<String>,
    options: Option<String>,
    devicetree: Option<String>,
}

impl AcetoneEntry {
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

impl BootEntry for AcetoneEntry {
    fn user_readable_name(&self) -> &str {
        if let Some(title) = &self.title {
            return title;
        } else {
            return &self.linux;
        }
    }
    fn enumerate() -> Vec<AcetoneEntry> {
        let mut ret = Vec::new();
        ret
    }
    fn boot(&self) {
        println!("Booting {:?}({})", self.title, self.linux);
    }
    fn hide(&self) -> bool {
        false
    }
}
