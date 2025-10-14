use std::{
    env, process::{self, Command}
};

use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

fn main() {
    let prebuilt =
        Prebuilt::fetch(Source::LATEST, "target/ovmf").unwrap();
    let ovmf_code = prebuilt.get_file(Arch::X64, FileType::Code);
    let ovmf_vars = prebuilt.get_file(Arch::X64, FileType::Vars);
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args([
        "-serial", "stdio",

        // Debugging
        "-d", "guest_errors", "-D", "qemu.log",
        "-global", "isa-debugcon.iobase=0x402",
        "-debugcon", "file:ovmf.log",

        // OVMF firmware (code readonly, vars writable)
        "-drive", &format!("if=pflash,format=raw,unit=0,readonly=on,file={}", ovmf_code.to_str().unwrap()),
        "-drive", &format!("if=pflash,format=raw,unit=1,file={}", ovmf_vars.to_str().unwrap()),
        
        // UEFI disk image: MUST contain /EFI/BOOT/BOOTX64.EFI on a FAT ESP
        "-drive", &format!("format=raw,file={}", env!("UEFI_IMAGE")),
    ]);
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}