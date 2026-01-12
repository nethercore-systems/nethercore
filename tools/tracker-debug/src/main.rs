//! Tracker Debug Tool
//!
//! A CLI tool for testing and debugging the tracker music pipeline with
//! real-time playback and interactive controls.

mod cli;
#[cfg(any(test, feature = "playback"))]
mod sound_loader;

#[cfg(all(not(test), feature = "playback"))]
mod audio;
#[cfg(all(not(test), feature = "playback"))]
mod display;
#[cfg(all(not(test), feature = "playback"))]
mod player;

#[cfg(all(not(test), feature = "playback"))]
use anyhow::{Result, bail};
#[cfg(all(not(test), feature = "playback"))]
use clap::Parser;
#[cfg(all(not(test), feature = "playback"))]
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
#[cfg(all(not(test), feature = "playback"))]
use std::path::Path;
#[cfg(all(not(test), feature = "playback"))]
use std::sync::{Arc, Mutex};
#[cfg(all(not(test), feature = "playback"))]
use std::time::Duration;

#[cfg(all(not(test), feature = "playback"))]
use audio::AudioPlayer;
#[cfg(all(not(test), feature = "playback"))]
use cli::{Cli, Commands};
#[cfg(all(not(test), feature = "playback"))]
use display::Display;
#[cfg(all(not(test), feature = "playback"))]
use player::{DebugPlayer, PlayerCommand};

#[cfg(all(not(test), feature = "playback"))]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Play {
            file,
            via_ncxm,
            via_ncit,
            verbose,
        } => {
            run_player(&file, via_ncxm, via_ncit, verbose)?;
        }
    }

    Ok(())
}

#[cfg(all(not(test), feature = "playback"))]
fn run_player(path: &Path, via_ncxm: bool, via_ncit: bool, verbose: bool) -> Result<()> {
    // Determine format from extension
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    // Load player based on format
    let mut player = match extension.as_str() {
        "xm" => DebugPlayer::load_xm(path, via_ncxm)?,
        "it" => DebugPlayer::load_it(path, via_ncit)?,
        _ => bail!("Unsupported format: .{}", extension),
    };

    player.set_verbose(verbose);

    // Wrap player for audio thread
    let player = Arc::new(Mutex::new(player));

    // Start audio
    let _audio = AudioPlayer::new(Arc::clone(&player))?;

    // Initialize terminal
    ratatui::init();
    let mut display = Display::new()?;

    // Main loop
    let result = main_loop(&player, &mut display);

    // Restore terminal
    ratatui::restore();

    result
}

#[cfg(all(not(test), feature = "playback"))]
fn main_loop(player: &Arc<Mutex<DebugPlayer>>, display: &mut Display) -> Result<()> {
    loop {
        // Poll for keyboard input with timeout
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            // Only handle key press events (not release)
            if key.kind != KeyEventKind::Press {
                continue;
            }

            let cmd = match key.code {
                KeyCode::Char(' ') => Some(PlayerCommand::TogglePause),
                KeyCode::Left => Some(PlayerCommand::SeekRow(-1)),
                KeyCode::Right => Some(PlayerCommand::SeekRow(1)),
                KeyCode::Up => Some(PlayerCommand::SeekPattern(-1)),
                KeyCode::Down => Some(PlayerCommand::SeekPattern(1)),
                KeyCode::Char('+') | KeyCode::Char('=') => Some(PlayerCommand::AdjustTempo(1)),
                KeyCode::Char('-') => Some(PlayerCommand::AdjustTempo(-1)),
                KeyCode::Char('v') | KeyCode::Char('V') => Some(PlayerCommand::ToggleVerbose),
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    return Ok(());
                }
                KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                    Some(PlayerCommand::ToggleMute(c.to_digit(10).unwrap() as u8 - 1))
                }
                _ => None,
            };

            if let Some(cmd) = cmd {
                player.lock().unwrap().handle_command(cmd);
            }
        }

        // Update display
        let state = player.lock().unwrap().get_display_state();
        display.render(&state)?;
    }
}

#[cfg(all(not(test), not(feature = "playback")))]
fn main() {
    eprintln!("tracker-debug built without `playback` feature; enable it to run the interactive player.");
    std::process::exit(2);
}
