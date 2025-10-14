use std::{
    env,
    process::{self, Command},
};

fn main() {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args([
        "-serial", "stdio",
        "-drive", &format!("format=raw,file={}", env!("BIOS_IMAGE")),
        // "-s", "-S", // for gdb
    ]);
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}