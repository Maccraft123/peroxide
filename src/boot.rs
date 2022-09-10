use std::path::PathBuf;
use std::process::Command;

use nix::{
    unistd::{
        Pid,
        getpid,
    },
    sys::{
        reboot::{
            reboot as sys_reboot,
            RebootMode,
        },
    },
};

use std::ffi::CString;
use std::os::raw::c_int;
use std::os::unix::io::{AsRawFd, RawFd};
use std::fs::File;
use syscalls::{
    Sysno,
    syscall,
};

static KEXEC_FILE_NO_INITRAMFS: c_int = 0x4;

fn kexec_file_load(kernel: File, initrd: Option<File>, cmdline: String) {
    let sys_kernel_fd: c_int = kernel.as_raw_fd();

    let sys_initrd_fd: c_int = if let Some(i) = &initrd {
        i.as_raw_fd()
    } else {
        -1
    };
    
    let cmdline = unsafe { CString::from_vec_unchecked(cmdline.as_bytes().to_vec()) };
    let cmdline_len = cmdline.as_bytes_with_nul().len();
    let flags = if initrd.is_none() { KEXEC_FILE_NO_INITRAMFS } else { 0 };

    unsafe { syscall!(Sysno::kexec_file_load, sys_kernel_fd, sys_initrd_fd, cmdline_len, cmdline.as_ptr(), flags) }.unwrap();
}

pub struct KexecData {
    pub kernel: String,
    pub cmdline: Option<String>,
    pub initrd: Option<String>,
    pub dt: Option<String>,
}

pub fn kexec(ctx: KexecData) {
    let kernel_file = File::open(ctx.kernel).unwrap();
    let initrd_file = if let Some(initrd) = &ctx.initrd {
        Some(File::open(initrd).unwrap())
    } else {
        None
    };

    kexec_file_load(kernel_file, initrd_file, ctx.cmdline.unwrap_or("".to_string()));

    // if we are the init it means we can just reboot() safely
    if getpid() == Pid::from_raw(1) {
        sys_reboot(RebootMode::RB_KEXEC).unwrap(); // infallible
        unreachable!("We should already have kexeced into new kernel");
    }

    if PathBuf::from("/usr/bin/systemctl").exists() {
        Command::new("systemctl").arg("kexec").status().unwrap(); // if fails we have to panic
        unreachable!("We should have already systemctl kexeced into new kernel");
    }

    unreachable!("Couldn't find a method to reboot() into a kexec image");
}

pub fn reboot() {
    // TODO: non-systemd distros
    if PathBuf::from("/usr/bin/systemctl").exists() {
        Command::new("systemctl").arg("reboot").status().unwrap();
        unreachable!("We should already have rebooted");
    }
    sys_reboot(RebootMode::RB_AUTOBOOT).unwrap();
}
