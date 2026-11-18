// Lets every program that depends on neo-rt find the user linker script
// (programs link with -Tuser.ld, see user/.cargo/config.toml).
fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={dir}");
    println!("cargo:rerun-if-changed=user.ld");
}
