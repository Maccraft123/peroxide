use crate::boot::reboot;
use crate::entry::BootEntry;
use efivar::efi::VariableFlags;
use efivar::efi::VariableName;
use regex::Regex;
use uefi::proto::device_path::DevicePath;
use uefi::proto::device_path::DeviceType;
use uuid::Uuid;
use std::collections::HashSet;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use nix::dir::Dir;
use nix::sys::stat::Mode;
use nix::fcntl::OFlag;
use nix::dir::Type;
use nix::fcntl::readlink;
use anyhow::Context;
use gpt::GptConfig;

fn char16_to_string(buf: &[u8]) -> (String, usize) {
    let mut iter = buf.iter();
    let mut out: Vec<u16> = Vec::new();
    let mut i = 0;

    loop {
        i += 2;
        if let (Some(lower), Some(upper)) = (iter.next(), iter.next()) {
            let tmp = (*upper as u16) << 8 | *lower as u16;
            if tmp == '\0' as u16 {
                break;
            } else {
                out.push(tmp);
            }
        } else {
            break;
        }
    }

    (std::char::decode_utf16(out)
        .map(|r| r.unwrap_or(' '))
        .map(|r| if r.is_ascii() {r} else {' '})
        .collect::<String>(), i)
}

fn sys_block_data(device: &str, data: &str) -> String {
    let mut buf = Vec::new();
    let path = format!("/sys/class/block/{}/device/{}", device, data);
    if let Ok(mut f) = File::open(&path) {
        if let Err(e) = f.read_to_end(&mut buf).with_context(|| format!("Failed to read data from {}", path)) {
            eprintln!("{}", e);
        }
    }
    buf.dedup(); // remove excess

    let mut out_string = "".to_string();
    for byte in buf.iter() {
        let c = char::from(*byte);
        if c.is_ascii() && c != '\n' {
            out_string.push(c);
        }
    }
    out_string
}

#[derive(Debug)]
pub struct EfiEntry {
    id: u16,
    description: String,
    partuuid: Option<Uuid>,
    is_default: bool,
}

impl BootEntry for EfiEntry {
    fn user_readable_name(&self) -> &str {
        &self.description
    }
    fn enumerate() -> Vec<EfiEntry> {
        let mut ret = Vec::new();

        // avoid panicking on efi-less systems
        if !PathBuf::from("/sys/firmware/efi").exists() {
            return ret;
        }

        let mut buf: [u8; 4096] = [0u8; 4096];
        let boot_xxxx = Regex::new(r"^Boot\d\d\d\d$").unwrap();
        let manager = efivar::system();
        
        if let Ok(var_iter) = manager.get_var_names() {
            for var in var_iter {
                if boot_xxxx.is_match(var.variable()) {
                    let res = manager.read(&var, &mut buf);
                    if res.is_ok() {
                        let (description, end) = char16_to_string(&buf[(32+16)/8..]);
                        let desc_end_offset = (32+16)/8 + end; // defined somewhere in efi spec,
                                                                // forgot where

                        let boot_id = var.short_name().split_off(4);
                        if boot_id.len() > 8 {
                            continue;
                        }

                        let id = if let Ok(tmp) = u16::from_str_radix(&boot_id, 16) {
                            tmp
                        } else {
                            0
                        };
                        
                        let device_path: &DevicePath = unsafe {
                            std::mem::transmute(&buf[desc_end_offset..])
                        };

                        let mut default_entry = false;
                        let mut partuuid = None;

                        for node in device_path.node_iter() {
                            if let Some(file) = node.as_file_path_media_device_path() {
                                let path = file.path_name().to_cstring16().unwrap();
                                let lowercase = path.to_string().to_lowercase();

                                if lowercase.contains(r"\efi\boot\bootx64.efi") ||
                                        lowercase.contains(r"\efi\boot\bootia.efi") ||
                                        lowercase.contains(r"\efi\boot\bootaa64.efi") {
                                    default_entry = true;
                                }
                            } else {
                                if node.device_type() == DeviceType::MEDIA {
                                    let media = &buf[desc_end_offset..];
                                    let signature = &media[24..][..16];
                                    let signature_type = &media[41];
                                    
                                    if signature_type != &0x02 {
                                        continue; // not a gpt partition
                                    }

                                    if let Ok(uuid) = Uuid::from_slice_le(signature) {
                                        partuuid = Some(uuid);
                                    }
                                }
                            }
                        }

                        ret.push(EfiEntry {
                            id,
                            description,
                            partuuid,
                            is_default: default_entry,
                        });
                    }
                }
            }
        }

        let mut guid_name_map = HashMap::new();
        if let Ok(mut dev) = Dir::open("/dev/", OFlag::O_DIRECTORY | OFlag::O_RDONLY, Mode::empty()) {
            let iter = dev.iter();
            let xda_regex = Regex::new(r"^(v|h|s)da$").unwrap();
            let mmcblk_regex = Regex::new(r"^mmcblk\d$").unwrap();
            let nvme_regex = Regex::new(r"^nvme\dn\d$").unwrap();

            let is_disk = |input: &str| -> bool {
                xda_regex.is_match(input) || mmcblk_regex.is_match(input) || nvme_regex.is_match(input)
            };

            for maybe_file in iter {
                if let Ok(file) = maybe_file {
                    if file.file_type().unwrap_or(Type::File) != Type::BlockDevice {
                        continue;
                    }

                    if !is_disk(file.file_name().to_str().unwrap_or("")) {
                        continue;
                    }

                    let disk: PathBuf = file.file_name().to_str().unwrap().into();
                    let dev_disk = PathBuf::from("/dev/").join(&disk);
                    let disk_string = disk.to_str().unwrap().to_string();

                    let mut name = sys_block_data(&disk_string, "vendor") + &sys_block_data(&disk_string, "model");
                    // if last character is a space, remove it
                    if let Some(last) = name.chars().last() {
                        if last == ' ' {
                            name.pop();
                        }
                    }

                    let gpt_cfg = GptConfig::new().writable(false);
                    if let Ok(gpt_disk) = gpt_cfg.open(dev_disk) {
                        for (_, part) in gpt_disk.partitions() {
                            guid_name_map.insert(part.part_guid.as_u128(), name.clone());
                        }
                    }
                }
            }
        }

        let mut seen_parts = HashSet::new();
        for entry in &mut ret {
            if let Some(uuid) = entry.partuuid.clone() {
                if let Some(name) = guid_name_map.get(&uuid.as_u128()) {
                    entry.description += &format!(" on {}", name);
                }
                if !entry.is_default {
                    seen_parts.insert(uuid);
                }
            }
        }

        ret.into_iter().filter(|e| !(e.is_default && seen_parts.contains(&e.partuuid.unwrap_or_default())) || e.partuuid.is_none()).collect()
    }
    fn boot(&self) {
        let mut manager = efivar::system();
        let next = VariableName::new("BootNext");
        let attr = VariableFlags::NON_VOLATILE | VariableFlags::BOOTSERVICE_ACCESS | VariableFlags::RUNTIME_ACCESS;
        let val: [u8; 2] = self.id.to_le_bytes();
        manager.write(&next, attr, &val).expect("Failed to write BootNext");

        reboot();
    }
    fn hide(&self) -> bool {
        !self.description.starts_with("Windows Boot Manager")
    }
}
