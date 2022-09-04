use crate::entry::BootEntry;
use regex::Regex;

fn char16_to_string(buf: &[u8]) -> String {
    let mut iter = buf.iter();
    let mut out: Vec<u16> = Vec::new();

    loop {
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

    std::char::decode_utf16(out)
        .map(|r| r.unwrap_or(' '))
        .map(|r| if r.is_ascii() {r} else {' '})
        .collect::<String>()
}

#[derive(Debug)]
pub struct EfiEntry {
    id: u16,
    id_str: String,
    description: String,
}

impl BootEntry for EfiEntry {
    fn user_readable_name(&self) -> &str {
        &self.description
    }
    fn enumerate() -> Vec<EfiEntry> {
        let mut ret = Vec::new();

        let mut buf: [u8; 4096] = [0u8; 4096];
        let boot_xxxx = Regex::new(r"^Boot\d\d\d\d$").unwrap();
        let manager = efivar::system();
        if let Ok(var_iter) = manager.get_var_names() {
            for var in var_iter {
                if boot_xxxx.is_match(var.variable()) {
                    let res = manager.read(&var, &mut buf);
                    if res.is_ok() {
                        let description = char16_to_string(&buf[(32+16)/8..]);
                        let boot_id = var.short_name().split_off(4);
                        if boot_id.len() > 8 {
                            continue;
                        }
                        let id = if let Ok(tmp) = u16::from_str_radix(&boot_id, 16) {
                            tmp
                        } else {
                            0
                        };

                        ret.push(EfiEntry {
                            id,
                            id_str: boot_id,
                            description,
                        });
                    }
                }
            }
        }
        
        ret
    }
    fn boot(&self) {
        println!("Rebooting into Boot{}", &self.id_str);
    }
    fn hide(&self) -> bool {
        true
    }
}
