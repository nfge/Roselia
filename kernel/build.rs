use vergen_gitcl::{Emitter,Gitcl};

fn main() {
    let git = Gitcl::all_git();
    Emitter::default().add_instructions(&git).expect("Vergen on git").emit().expect("vergen");
    // let head = Command::new("git").args(["rev-parse", "--git-dir"]).output().unwrap();

    // println!("cargo:rerun-if-changed={}/HEAD", String::from_utf8_lossy(&head.stdout).trim());
    // println!("cargo:rerun-if-changed={}/refs/heads", String::from_utf8_lossy(&head.stdout).trim());
    // println!("cargo:rerun-if-changed={}/packed-refs", String::from_utf8_lossy(&head.stdout).trim());

    // let hash = Command::new("git")
    // .args(["rev-parse", "--short", "HEAD"])
    // .output()
    // .unwrap();
    // println!("cargo:rustc-env=GIT_COMMIT={}", String::from_utf8_lossy(&hash.stdout).trim());

    // cc::Build::new()
    //     .file("./src/terminal/test.c")
    //     .compiler("clang")
    //     .flag("-ffreestanding")
    //     .flag("-fno-stack-protector")
    //     .target("x86_64-unknown-none")
    //     .archiver("llvm-ar")
    //     .compile("test");

}