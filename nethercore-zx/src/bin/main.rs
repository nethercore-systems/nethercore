//! Nethercore ZX - Standalone Player
//!
//! A minimal player for running Nethercore ZX ROM files without the library UI.
//!
//! # Usage
//!
//! ```bash
//! nethercore-zx path/to/game.nczx
//! nethercore-zx game.nczx --fullscreen
//! nethercore-zx game.nczx --debug
//! nethercore-zx game.nczx --preview
//! nethercore-zx game.nczx --preview --asset textures/player
//! ```
//!
//! # Keyboard Shortcuts
//!
//! - ESC: Quit
//! - F3: Toggle debug overlay
//! - F5: Pause/Resume
//! - F6: Frame step (when paused)
//! - F11: Toggle fullscreen

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use nethercore_core::replay::{ExecutionReport, HeadlessConfig};
use nethercore_core::rollback::ConnectionMode;
use nethercore_zx::player::{PlayerConfig, run};
use nethercore_zx::preview::{PreviewConfig, run as run_preview};

#[derive(Parser)]
#[command(name = "nethercore-zx")]
#[command(
    author,
    version,
    about = "Nethercore ZX - PS1/N64 aesthetic fantasy console"
)]
struct Args {
    /// ROM file to play (.nczx or .wasm)
    rom: PathBuf,

    /// Start in fullscreen mode (borderless window, scales to fit)
    #[arg(long, short = 'f')]
    fullscreen: bool,

    /// Integer scaling factor (default: 2, only affects windowed mode)
    #[arg(long, short = 's', default_value = "2")]
    scale: u32,

    /// Enable debug overlay on startup
    #[arg(long, short = 'd')]
    debug: bool,

    // === Multiplayer Options ===
    /// Number of players (1-4)
    #[arg(long, short = 'p', default_value = "1")]
    players: usize,

    /// Input delay in frames (0-10, higher = smoother online play)
    #[arg(long, default_value = "0")]
    input_delay: usize,

    /// Run in sync-test mode to verify game determinism
    #[arg(long)]
    sync_test: bool,

    /// Sync-test check distance (frames between state checksums)
    #[arg(long, default_value = "2")]
    check_distance: usize,

    // === P2P Testing (Local Development) ===
    /// Enable P2P mode for local testing
    #[arg(long)]
    p2p: bool,

    /// Local port to bind for P2P/Host mode
    #[arg(long, default_value = "7777")]
    bind: u16,

    /// Peer port to connect to in P2P mode
    #[arg(long, default_value = "7778")]
    peer: u16,

    /// Which player this instance controls (0 or 1) in P2P mode
    #[arg(long, default_value = "0")]
    local_player: usize,

    // === Network Play ===
    /// Host a multiplayer game on this port
    #[arg(long)]
    host: Option<u16>,

    /// Join a multiplayer game at this address (ip:port)
    #[arg(long)]
    join: Option<String>,

    /// Session config file from library lobby (NCHS pre-negotiated session)
    #[arg(long, value_name = "FILE")]
    session: Option<PathBuf>,

    // === Replay Mode ===
    /// Run a replay script (.ncrs) for automated playback and screenshots
    #[arg(long, value_name = "FILE")]
    replay: Option<PathBuf>,

    /// Execute the replay without creating graphics or audio devices
    #[arg(long)]
    headless: bool,

    /// Write the replay execution report as JSON
    #[arg(long, value_name = "JSON")]
    report: Option<PathBuf>,

    /// Stop after the first failed assertion
    #[arg(long)]
    fail_fast: bool,

    /// Whole replay execution timeout in seconds
    #[arg(long, default_value = "300")]
    timeout: u64,

    /// Exit after this many advanced input frames, useful for automated sync-test gates
    #[arg(long)]
    exit_after_frames: Option<u32>,

    /// Enable the local EPU workbench HTTP service on 127.0.0.1:<port>
    #[arg(long)]
    epu_workbench_port: Option<u16>,

    /// Durable directory for EPU workbench captures/exports
    #[arg(long, value_name = "DIR")]
    epu_workbench_dir: Option<PathBuf>,

    // === Preview Mode ===
    /// Run in preview mode to inspect ROM assets
    #[arg(long)]
    preview: bool,

    /// Specific asset to focus on in preview mode (e.g., "textures/player")
    #[arg(long)]
    asset: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(path) = &args.report {
        for input in std::iter::once(&args.rom).chain(args.replay.iter()) {
            if let (Ok(output), Ok(input)) = (path.canonicalize(), input.canonicalize()) {
                anyhow::ensure!(output != input, "report must not overwrite ROM or script");
            }
        }
        let message = validate_args(&args)
            .err()
            .map(|e| format!("{e:#}"))
            .unwrap_or_else(|| "execution not completed".into());
        ExecutionReport::startup_error(
            args.replay.as_ref().map(|p| p.display().to_string()),
            "startup",
            message,
        )
        .write_to_file(path)?;
    }
    validate_args(&args)?;

    if args.headless {
        return run_headless(&args);
    }

    // Validate ROM path exists
    if !args.rom.exists() {
        anyhow::bail!("ROM file not found: {}", args.rom.display());
    }

    // Handle preview mode
    if args.preview {
        let config = PreviewConfig {
            rom_path: args.rom,
            asset_path: args.asset,
            scale: args.scale,
        };
        return run_preview(config);
    }

    // Validate player count
    if args.players == 0 || args.players > 4 {
        anyhow::bail!("Player count must be between 1 and 4");
    }

    // Validate input delay
    if args.input_delay > 10 {
        anyhow::bail!("Input delay must be between 0 and 10");
    }

    // Determine connection mode from arguments
    // Priority: session > join > host > p2p > sync_test > local
    let connection_mode = if let Some(session_file) = args.session {
        ConnectionMode::Session { session_file }
    } else if let Some(ref address) = args.join {
        ConnectionMode::Join {
            address: address.clone(),
        }
    } else if let Some(port) = args.host {
        ConnectionMode::Host { port }
    } else if args.p2p {
        ConnectionMode::P2P {
            bind_port: args.bind,
            peer_port: args.peer,
            local_player: args.local_player,
        }
    } else if args.sync_test {
        ConnectionMode::SyncTest {
            check_distance: args.check_distance,
        }
    } else {
        ConnectionMode::Local
    };

    let config = PlayerConfig {
        rom_path: args.rom,
        fullscreen: args.fullscreen,
        scale: args.scale,
        debug: args.debug,
        num_players: args.players,
        input_delay: args.input_delay,
        connection_mode,
        replay_script: args.replay,
        exit_after_frames: args.exit_after_frames,
        epu_workbench_port: args.epu_workbench_port,
        epu_workbench_dir: args.epu_workbench_dir,
    };

    run(config)
}

fn validate_args(args: &Args) -> Result<()> {
    if args.headless {
        anyhow::ensure!(args.replay.is_some(), "--headless requires --replay SCRIPT");
        anyhow::ensure!(!args.preview, "--headless conflicts with --preview");
        anyhow::ensure!(!args.sync_test, "--headless conflicts with --sync-test");
        anyhow::ensure!(!args.p2p, "--headless conflicts with --p2p");
        anyhow::ensure!(args.host.is_none(), "--headless conflicts with --host");
        anyhow::ensure!(args.join.is_none(), "--headless conflicts with --join");
        anyhow::ensure!(
            args.session.is_none(),
            "--headless conflicts with --session"
        );
        anyhow::ensure!(
            args.exit_after_frames.is_none(),
            "--headless conflicts with --exit-after-frames"
        );
    } else {
        anyhow::ensure!(args.report.is_none(), "--report requires --headless");
        anyhow::ensure!(!args.fail_fast, "--fail-fast requires --headless");
    }
    anyhow::ensure!(args.timeout > 0, "--timeout must be greater than zero");
    Ok(())
}

fn run_headless(args: &Args) -> Result<()> {
    let script = args.replay.as_ref().expect("validated replay argument");
    let config = HeadlessConfig {
        fail_fast: args.fail_fast,
        timeout_secs: args.timeout,
        script_path: Some(script.display().to_string()),
    };
    // Bound loading/compilation as well as guest execution. This CLI owns the process.
    let (cancel, cancelled) = std::sync::mpsc::channel::<()>();
    let report_path = args.report.clone();
    let script_name = script.display().to_string();
    let timeout = args.timeout;
    let deadline = std::thread::spawn(move || {
        if cancelled
            .recv_timeout(std::time::Duration::from_secs(timeout))
            .is_err()
        {
            let error = ExecutionReport::startup_error(
                Some(script_name),
                "timeout",
                format!("headless invocation exceeded {timeout}s"),
            );
            if let Some(path) = report_path {
                let _ = error.write_to_file(&path);
            }
            eprintln!("headless invocation exceeded {timeout}s");
            std::process::exit(1);
        }
    });
    let result = nethercore_zx::replay::run_headless(&args.rom, script, config);
    let _ = cancel.send(());
    let _ = deadline.join();
    let report = match result {
        Ok(report) => report,
        Err(error) => ExecutionReport::startup_error(
            Some(script.display().to_string()),
            "startup",
            format!("{error:#}"),
        ),
    };

    if let Some(path) = &args.report {
        report
            .write_to_file(path)
            .with_context(|| format!("Failed to write report: {}", path.display()))?;
    }
    println!(
        "Replay {}: {}/{} frames, {} assertions failed",
        report.summary.status,
        report.frames_executed,
        report.total_frames,
        report.summary.assertions_failed
    );
    if !report.succeeded() {
        let detail = report
            .error
            .as_ref()
            .map(|error| error.message.as_str())
            .unwrap_or("one or more assertions failed");
        anyhow::bail!("replay {}: {detail}", report.summary.status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_requires_a_replay_script() {
        let args = Args::try_parse_from(["nethercore-zx", "game.wasm", "--headless"]).unwrap();
        assert!(
            validate_args(&args)
                .unwrap_err()
                .to_string()
                .contains("--replay")
        );
    }

    #[test]
    fn report_is_rejected_without_headless_mode() {
        let args = Args::try_parse_from(["nethercore-zx", "game.wasm", "--report", "report.json"])
            .unwrap();
        assert!(
            validate_args(&args)
                .unwrap_err()
                .to_string()
                .contains("--headless")
        );
    }
}
