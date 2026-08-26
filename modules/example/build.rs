use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let linker_script = manifest_dir.join("../module.ld");

    println!(
        "cargo:rustc-link-arg=-T{}",
        linker_script.display()
    );

    println!("cargo:rerun-if-changed={}", linker_script.display());
    println!("cargo:rustc-link-arg=--unresolved-symbols=ignore-all");
    println!("cargo:rustc-link-arg=-shared");
    println!("cargo:rustc-link-arg=-no-pie");
}