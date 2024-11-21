use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=resources/");
    println!("cargo:rerun-if-changed=Makefile");

    let status = Command::new("make")
        .arg("esbuild")
        .status()
        .expect("Failed to execute make esbuild");

    if !status.success() {
        panic!("make esbuild failed");
    }
}
