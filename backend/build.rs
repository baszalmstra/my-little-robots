use std::env;
use std::path::{Path, PathBuf};
use wasm_pack::command::build::{Build, BuildOptions, Target};

fn main() {
    println!("cargo:rerun-if-changed=frontend/src/*");
    println!("cargo:rerun-if-changed=frontend/Cargo.toml");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("web-client");
    let is_release = env::var("PROFILE").unwrap() == "release";

    Build::try_from_opts(BuildOptions {
        path: Some(manifest_dir.join("frontend")),
        disable_dts: true,
        target: Target::Web,
        debug: false,
        dev: !is_release,
        release: is_release,
        profiling: false,
        out_dir: dest_path.display().to_string(),
        out_name: Some("client".into()),
        ..Default::default()
    })
    .and_then(|mut b| b.run())
    .expect("could not build wasm");

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
