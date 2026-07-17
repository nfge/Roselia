use std::process::Command;

fn main() {
    let hash = Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .unwrap();

    println!("cargo:rustc-env=GIT_COMMIT={}", String::from_utf8_lossy(&hash.stdout).trim());
}