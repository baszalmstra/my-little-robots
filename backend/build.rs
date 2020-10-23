use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src/*");
    println!("cargo:rerun-if-changed=frontend/Cargo.toml");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("web-client");
    let output = Command::new("wasm-pack")
        .args(&[
            "build",
            "--target",
            "web",
            "--out-name",
            "client",
            "--out-dir",
        ])
        .arg(&dest_path)
        .arg("./frontend/")
        .output()
        .expect("To build wasm files successfully");

    if !output.status.success() {
        panic!(
            "Error while compiling:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let js_file = dest_path.join("client.js");
    let wasm_file = dest_path.join("client_bg.wasm");

    for file in &[&js_file, &wasm_file] {
        let file = std::fs::metadata(file).expect("file to exist");
        assert!(file.is_file());
    }

    println!("cargo:rustc-env=MY_LITTLE_ROBOTS_JS={}", js_file.display());
    println!(
        "cargo:rustc-env=MY_LITTLE_ROBOTS_WASM={}",
        wasm_file.display()
    );
}
