//! Real CLI failure contracts. The game/script/report live only in a temporary directory.
use std::{
    fs,
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn replay_cli_reports_failures_without_stale_success_or_source_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let rom = dir.path().join("game.wasm");
    let script = dir.path().join("test.ncrs");
    let report = dir.path().join("report.json");
    let idle = "console = \"zx\"\n[[frames]]\nf = 0\n";
    let cases = [
        (
            "init timeout",
            "(func (export \"init\") (loop $l (br $l))) (func (export \"update\"))",
            idle,
        ),
        (
            "update timeout",
            "(func (export \"update\") (loop $l (br $l)))",
            idle,
        ),
        (
            "update trap",
            "(func (export \"update\") unreachable)",
            idle,
        ),
        ("missing update", "", idle),
        (
            "bad directive",
            "(func (export \"update\"))",
            "console = \"zx\"\n[[frames]]\nf = 0\nparams = {}\n",
        ),
        (
            "empty script",
            "(func (export \"update\"))",
            "console = \"zx\"\nframes = []\n",
        ),
        (
            "headless screenshot",
            "(func (export \"update\"))",
            "console = \"zx\"\n[[frames]]\nf = 0\nscreenshot = true\n",
        ),
    ];
    for (name, body, input) in cases {
        fs::write(
            &rom,
            wat::parse_str(format!("(module (memory (export \"memory\") 1) {body})")).unwrap(),
        )
        .unwrap();
        fs::write(&script, input).unwrap();
        fs::write(&report, r#"{"summary":{"status":"PASSED"}}"#).unwrap();
        let started = Instant::now();
        let output = Command::new(env!("CARGO_BIN_EXE_nethercore-zx"))
            .arg(&rom)
            .args(["--headless", "--replay"])
            .arg(&script)
            .arg("--report")
            .arg(&report)
            .args(["--timeout", "1"])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{name}");
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "{name} exceeded deadline"
        );
        let data: serde_json::Value = serde_json::from_slice(&fs::read(&report).unwrap()).unwrap();
        assert_eq!(data["summary"]["status"], "ERROR", "{name}: {data}");
        assert!(data["error"]["message"].as_str().unwrap().len() > 3);
    }
    fs::write(&script, idle).unwrap();
    let before = fs::read(&rom).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nethercore-zx"))
        .arg(&rom)
        .args(["--headless", "--replay"])
        .arg(&script)
        .arg("--report")
        .arg(&rom)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(&rom).unwrap(), before);
    fs::remove_file(&report).unwrap();
    fs::hard_link(&rom, &report).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nethercore-zx"))
        .arg(&rom)
        .args(["--headless", "--replay"])
        .arg(&script)
        .arg("--report")
        .arg(&report)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "hardlink-safe replacement failed: {output:?}"
    );
    assert_eq!(fs::read(&rom).unwrap(), before);
    let output = Command::new(env!("CARGO_BIN_EXE_nethercore-zx"))
        .arg(&rom)
        .args(["--headless", "--replay"])
        .arg(&script)
        .arg("--report")
        .arg(&report)
        .args(["--timeout", "0"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let data: serde_json::Value = serde_json::from_slice(&fs::read(&report).unwrap()).unwrap();
    assert_eq!(data["summary"]["status"], "ERROR");
}
