use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

fn first_existing(candidates: &[&str]) -> Option<PathBuf> {
    candidates.iter().map(Path::new).find(|p| p.exists()).map(PathBuf::from)
}

fn system_ovmf_paths() -> (PathBuf, PathBuf) {
    // Prefer the 4M firmware pair; fall back to non-4M if needed.
    let code = first_existing(&[
        "/usr/share/OVMF/OVMF_CODE_4M.fd",
        "/usr/share/OVMF/OVMF_CODE.fd",
    ])
    .expect("OVMF CODE not found (looked for OVMF_CODE_4M.fd and OVMF_CODE.fd in /usr/share/OVMF)");

    let vars_tmpl = first_existing(&[
        "/usr/share/OVMF/OVMF_VARS_4M.fd",
        "/usr/share/OVMF/OVMF_VARS.fd",
    ])
    .expect("OVMF VARS not found (looked for OVMF_VARS_4M.fd and OVMF_VARS.fd in /usr/share/OVMF)");

    (code, vars_tmpl)
}

fn target_ovmf_dir() -> PathBuf {
    let target = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let dir = Path::new(&target).join("ovmf");
    fs::create_dir_all(&dir).expect("failed to create target/ovmf directory");
    dir
}

fn writable_vars_path(ovmf_dir: &Path, vars_template: &Path) -> PathBuf {
    let fname = if vars_template.file_name().and_then(|n| n.to_str()).unwrap_or("").contains("_4M") {
        "OVMF_VARS_4M.writable.fd"
    } else {
        "OVMF_VARS.writable.fd"
    };
    let dst = ovmf_dir.join(fname);
    // Always refresh the writable copy (cheap, avoids stale perms/contents).
    fs::copy(vars_template, &dst).unwrap_or_else(|e| {
        panic!(
            "failed to copy VARS template from {} to {}: {e}",
            vars_template.display(),
            dst.display()
        )
    });
    dst
}

fn main() -> ExitCode {
    // Resolve firmware files from the system
    let (code_path, vars_template) = system_ovmf_paths();

    // Prepare cargo-cleanable outputs
    let ovmf_dir = target_ovmf_dir();
    let vars_rw = writable_vars_path(&ovmf_dir, &vars_template);
    let ovmf_log = ovmf_dir.join("ovmf.log");

    // UEFI ESP image produced by your build script
    let uefi_img = env!("UEFI_IMAGE");
    if !Path::new(uefi_img).exists() {
        eprintln!("UEFI_IMAGE does not exist: {uefi_img}");
        return ExitCode::from(2);
    }

    // Launch QEMU (reliable, debug-friendly defaults)
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args([
        "-serial", "stdio",

        // Firmware: unit=0 = CODE (readonly), unit=1 = VARS (writable copy)
        "-drive", &format!("if=pflash,format=raw,unit=0,readonly=on,file={}", code_path.display()),
        "-drive", &format!("if=pflash,format=raw,unit=1,file={}", vars_rw.display()),
        
        // UEFI ESP image (must contain /EFI/BOOT/BOOTX64.EFI)
        "-drive", &format!("format=raw,file={}", uefi_img),
        
        // Firmware debug console → target/ovmf/ovmf.log
        "-global", "isa-debugcon.iobase=0x402",
        "-debugcon", &format!("file:{}", ovmf_log.display()),
    ]);

    match qemu.status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!("failed to start qemu-system-x86_64: {e}");
            ExitCode::from(1)
        }
    }
}
