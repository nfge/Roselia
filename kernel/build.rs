use std::process::Command;

fn main() {
    let hash = Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .unwrap();
    println!("cargo:rustc-env=GIT_COMMIT={}", String::from_utf8_lossy(&hash.stdout).trim());

    // cc::Build::new()
    //     .file("./src/terminal/test.c")
    //     .compiler("clang")
    //     .flag("-ffreestanding")
    //     .flag("-fno-stack-protector")
    //     .target("x86_64-unknown-none")
    //     .archiver("llvm-ar")
    //     .compile("test");

}