use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=js/editor.js");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=vite.config.js");

    // Run npm build to bundle CodeMirror
    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("Failed to run npm build");

    if !status.success() {
        panic!("npm build failed");
    }
}
