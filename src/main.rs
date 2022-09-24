mod entry;
mod bootloaderspec;
mod uefi;
mod boot;

use aski::Picker;

use ozone::{
    init, Config,
};

fn main() {
    let conf = Config::new()
        .mount_boot(true)
        .mount_sys(true);
    std::thread::sleep(std::time::Duration::from_secs(5));
    init(&conf).unwrap();

    let mut entries = entry::enumerate_all();
    let mut picker = Picker::new("Choose boot option".to_string());
    picker.add_options(entries).unwrap();

    let choice = picker.wait_choice().unwrap();
    println!("Chosen boot option: {}", choice.user_readable_name());
    choice.boot();
}
