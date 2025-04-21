use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("failed to execute git");
    let git_head = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HEAD={}", git_head);

    tauri_build::build()
}
