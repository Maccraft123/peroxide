use std::path::PathBuf;
use std::process::Command;

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

    if PathBuf::from("/usr/bin/systemctl").exists() {
        Command::new("systemctl").arg("kexec").status().unwrap();
    }
}

pub fn reboot() {
    todo!("reboot");
}
