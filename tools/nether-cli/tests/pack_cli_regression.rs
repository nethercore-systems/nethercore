//! Exercise default manifest discovery through the real CLI and Cargo metadata.
use std::{fs, process::Command};

#[test]
fn bare_manifest_uses_current_directory_for_cargo_output() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "").unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='pack-path-probe'\nversion='0.1.0'\nedition='2021'\n\
         [lib]\ncrate-type=['cdylib']\n",
    )
    .unwrap();
    fs::write(
        root.join("nether.toml"),
        "[game]\nid='pack-path-probe'\ntitle='Path Probe'\nauthor='Test'\nversion='0.1.0'\n\
         max_players=1\ntick_rate=60\nrender_mode=0\ncompress_textures=false\n\
         [netplay]\nenabled=false\n",
    )
    .unwrap();
    let target = root.join("target/wasm32-unknown-unknown/release");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("pack_path_probe.wasm"), b"\0asm\x01\0\0\0").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_nether"))
        .arg("pack")
        .current_dir(root)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("pack-path-probe.nczx").is_file());
}
