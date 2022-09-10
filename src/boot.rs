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

pub struct KexecData {
    pub kernel: String,
    pub cmdline: Option<String>,
    pub initrd: Option<String>,
    pub dt: Option<String>,
}

pub fn kexec(ctx: KexecData) {
    let mut cmd = Command::new("kexec");
    if let Some(cmdline) = ctx.cmdline {
        cmd.arg(format!("--command-line={}", cmdline));
    }
    if let Some(initrd) = ctx.initrd {
        cmd.arg(format!("--initrd={}", initrd));
    }
    if let Some(dt) = ctx.dt {
        todo!("kexec with devicetree");
    }
    cmd.arg("--kexec-syscall-auto");
    cmd.arg("--load");
    cmd.arg(ctx.kernel);

    let status = cmd.status().unwrap();
    if !status.success() {
        return;
    }

    // if we are the init it means we can just reboot() safely

    if getpid() == Pid::from_raw(1) {
        sys_reboot(RebootMode::RB_KEXEC).unwrap(); // infallible
        unreachable!("We should already have kexeced into new kernel");
    }

    if PathBuf::from("/usr/bin/systemctl").exists() {
        Command::new("systemctl").arg("kexec").status().unwrap(); // if fails we have to panic
        unreachable!("We should have already systemctl kexeced into new kernel");
    }
}

pub fn reboot() {
    // TODO: non-systemd distros
    if PathBuf::from("/usr/bin/systemctl").exists() {
        Command::new("systemctl").arg("reboot").status().unwrap();
        unreachable!("We should already have rebooted");
    }
    sys_reboot(RebootMode::RB_AUTOBOOT).unwrap();
}
