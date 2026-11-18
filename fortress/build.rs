// 1. Hands the right linker script to the linker for the target being built.
// 2. Builds the user programs (the separate `user/` workspace) and tells the
//    kernel where their ELF files are, so it can embed them.
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let script = dir.join(format!("src/arch/{arch}/linker.ld"));
    println!("cargo:rustc-link-arg-bins=-T{}", script.display());
    println!("cargo:rerun-if-changed={}", script.display());

    let user = dir.parent().unwrap().join("user");
    let triple = format!("{arch}-unknown-none");
    let target_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("user-target");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.current_dir(&user).args(["build", "--release", "--target", &triple, "--target-dir"]).arg(&target_dir);
    // Do not let the kernel build's settings leak into the user build.
    for (k, _) in std::env::vars() {
        if k.starts_with("CARGO_") && k != "CARGO_HOME" || k == "RUSTFLAGS" || k == "RUSTC_WRAPPER" {
            cmd.env_remove(k);
        }
    }
    let status = cmd.status().expect("could not run cargo for the user programs");
    assert!(status.success(), "building the user programs failed");

    let bin = target_dir.join(&triple).join("release");
    // The console service drives an x86 serial port through I/O ports.
    let progs: &[&str] = match arch.as_str() {
        "x86_64" => &["hello-rust", "crash", "ring-send", "console", "hello-console"],
        _ => &["hello-rust", "crash", "ring-send"],
    };
    for prog in progs {
        let env = prog.to_uppercase().replace('-', "_");
        println!("cargo:rustc-env=USER_ELF_{env}={}", bin.join(prog).display());
    }
    println!("cargo:rerun-if-changed={}", user.display());
}
