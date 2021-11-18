use std::process::Command;

use clap::crate_version;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!(
        "cargo:rustc-env=VERSION_WITH_GIT_HASH={}",
        format!("v{} ({})", crate_version!(), &git_hash[..10])
    );
}
