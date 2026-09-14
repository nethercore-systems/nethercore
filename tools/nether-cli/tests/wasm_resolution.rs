//! A shared Cargo target must never select an old or unrelated game's WASM.
use std::{fs, path::Path, process::Command};

fn wasm(tag: &str) -> Vec<u8> {
    let mut bytes = b"\0asm\x01\0\0\0".to_vec();
    assert!(tag.len() < 126);
    bytes.extend([0, (tag.len() + 1) as u8, tag.len() as u8]);
    bytes.extend(tag.as_bytes());
    bytes
}

fn build(project: &Path, target: &Path, output: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nether"))
        .args(["build", "--no-compile", "-p"])
        .arg(project)
        .arg("-o")
        .arg(output)
        .env("CARGO_TARGET_DIR", target)
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .output()
        .unwrap()
}

#[test]
fn shared_target_rejects_stale_wasm_and_preserves_explicit_override() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("game");
    let shared = temp.path().join("shared");
    let local = project.join("target/wasm32-unknown-unknown/release");
    let release = shared.join("wasm32-unknown-unknown/release");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::create_dir_all(&local).unwrap();
    fs::create_dir_all(&release).unwrap();
    fs::write(project.join("src/lib.rs"), "").unwrap();
    // Host helpers also report crate_types=["bin"], but are not game WASM targets.
    fs::write(project.join("build.rs"), "fn main() {}").unwrap();
    for dir in ["tests", "examples", "benches"] {
        fs::create_dir_all(project.join(dir)).unwrap();
        fs::write(project.join(dir).join("host_helper.rs"), "fn main() {}").unwrap();
    }
    fs::write(project.join("Cargo.toml"), "[package]\nname='cargo-package'\nversion='0.1.0'\nedition='2021'\n[lib]\nname='current_guest'\ncrate-type=['cdylib']\n[workspace]\n").unwrap();
    let manifest =
        "[game]\nid='different-game-id'\ntitle='Resolution test'\nauthor='QA'\nversion='0.1.0'\n";
    fs::write(project.join("nether.toml"), manifest).unwrap();
    fs::write(local.join("different_game_id.wasm"), wasm("STALE_LOCAL")).unwrap();
    fs::write(release.join("current_guest.wasm"), wasm("CURRENT_GUEST")).unwrap();
    fs::write(release.join("unrelated.wasm"), wasm("UNRELATED_GUEST")).unwrap();

    let rom = temp.path().join("current.nczx");
    let result = build(&project, &shared, &rom);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        String::from_utf8_lossy(&result.stdout).contains("current_guest.wasm"),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    assert!(fs::read(&rom)
        .unwrap()
        .windows(b"CURRENT_GUEST".len())
        .any(|w| w == b"CURRENT_GUEST"));

    fs::remove_file(release.join("current_guest.wasm")).unwrap();
    let missing = temp.path().join("missing.nczx");
    let result = build(&project, &shared, &missing);
    assert!(
        !result.status.success(),
        "must not select an unrelated or stale WASM"
    );
    assert!(
        !missing.exists(),
        "failed selection must not write a cartridge"
    );

    // Two actual game outputs are still ambiguous; helper exclusion must not guess.
    fs::write(release.join("current_guest.wasm"), wasm("CURRENT_GUEST")).unwrap();
    fs::write(project.join("src/main.rs"), "fn main() {}").unwrap();
    let ambiguous = temp.path().join("ambiguous.nczx");
    let result = build(&project, &shared, &ambiguous);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Expected one Cargo WASM target"));
    assert!(!ambiguous.exists());
    fs::remove_file(project.join("src/main.rs")).unwrap();

    fs::write(project.join("explicit.wasm"), wasm("EXPLICIT_GUEST")).unwrap();
    fs::write(
        project.join("nether.toml"),
        format!("{manifest}\n[build]\nwasm='explicit.wasm'\n"),
    )
    .unwrap();
    let explicit = temp.path().join("explicit.nczx");
    assert!(build(&project, &shared, &explicit).status.success());
    assert!(fs::read(explicit)
        .unwrap()
        .windows(b"EXPLICIT_GUEST".len())
        .any(|w| w == b"EXPLICIT_GUEST"));
}
