use std::path::Path;
use std::process::Command;

fn run(cmd: &mut Command, name: &str) {
    let status = cmd.status().expect("failed to spawn process");
    if !status.success() {
        panic!("{name} failed with status {status}");
    }
}

fn main() {
    let frontend_dir = Path::new("svelte");

    // Re-run build.rs if frontend files change
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/package-lock.json");
    println!("cargo:rerun-if-changed=frontend/src");

    // npm install
    run(
        Command::new("npm").arg("install").current_dir(frontend_dir),
        "npm install",
    );

    // npm run build
    run(
        Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(frontend_dir),
        "npm run build",
    );
}
