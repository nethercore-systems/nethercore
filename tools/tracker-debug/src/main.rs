//! Tracker Debug Tool
//!
//! A CLI tool for testing and debugging the tracker music pipeline with
//! real-time playback and interactive controls.

mod audio;
mod cli;
mod display;
mod player;
mod sound_loader;

use anyhow::{Result, bail};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use audio::AudioPlayer;
use cli::{Cli, Commands};
use display::Display;
use player::{DebugPlayer, PlayerCommand};

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

fn main_loop(player: &Arc<Mutex<DebugPlayer>>, display: &mut Display) -> Result<()> {
    loop {
        // Poll for keyboard input with timeout
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()? {
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
