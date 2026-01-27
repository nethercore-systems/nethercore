//! PlayerLauncher builder and launch types.

use std::path::{Path, PathBuf};

use anyhow::Result;

use nethercore_core::library::LocalGame;
use nethercore_shared::ConsoleType;

use super::helpers::{
    console_type_from_extension, console_type_from_str, supported_console_types,
    supported_extension_list,
};
use super::player::{launch_player_with_options, run_player_with_options};

/// Multiplayer connection mode
#[derive(Debug, Clone)]
pub enum ConnectionMode {
    /// Host a game and wait for connections
    Host { port: u16 },
    /// Join an existing game
    Join { host_ip: String, port: u16 },
    /// Pre-established session from library lobby (session config in file)
    Session { file: PathBuf },
}

/// Options to pass to the player process
#[derive(Debug, Clone, Default)]
pub struct PlayerOptions {
    /// Start in fullscreen mode
    pub fullscreen: bool,
    /// Enable debug overlay
    pub debug: bool,
    /// Number of players (1-4)
    pub players: Option<usize>,
    /// Multiplayer connection mode
    pub connection: Option<ConnectionMode>,
    /// Enable preview mode (asset browser without running game code)
    pub preview: bool,
    /// Initial asset to focus in preview mode
    pub preview_asset: Option<String>,
    /// Replay script path (.ncrs file)
    pub replay_script: Option<PathBuf>,
}

/// Target for the player launcher (ROM path or game reference)
#[derive(Debug, Clone)]
pub enum LaunchTarget {
    /// ROM file path with console type
    Rom {
        path: PathBuf,
        console_type: ConsoleType,
    },
    /// Local game reference
    Game(LocalGame),
}

/// Builder for launching game players with a fluent API.
///
/// Consolidates all launch/run functions into a single builder pattern.
///
/// # Examples
///
/// ```ignore
/// // Launch a ROM with fullscreen and debug mode
/// PlayerLauncher::with_rom("game.nczx", ConsoleType::ZX)
///     .fullscreen(true)
///     .debug(true)
///     .launch()?;
///
/// // Run a game and wait for it to finish
/// PlayerLauncher::with_game(&local_game)?
///     .run()?;
///
/// // Host a multiplayer game
/// PlayerLauncher::from_path("game.nczx")?
///     .players(2)
///     .host(7777)
///     .launch()?;
/// ```
#[derive(Debug, Clone)]
pub struct PlayerLauncher {
    target: LaunchTarget,
    options: PlayerOptions,
}

impl PlayerLauncher {
    /// Create a launcher for a ROM file with explicit console type.
    pub fn with_rom(path: impl AsRef<Path>, console_type: ConsoleType) -> Self {
        Self {
            target: LaunchTarget::Rom {
                path: path.as_ref().to_path_buf(),
                console_type,
            },
            options: PlayerOptions::default(),
        }
    }

    /// Create a launcher for a local game.
    pub fn with_game(game: &LocalGame) -> Result<Self> {
        let _console_type = console_type_from_str(&game.console_type).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown console type: '{}'. Supported consoles: {}",
                game.console_type,
                supported_console_types()
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

        Ok(Self {
            target: LaunchTarget::Game(game.clone()),
            options: PlayerOptions::default(),
        })
    }

    /// Create a launcher by detecting console type from file extension.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let console_type = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(console_type_from_extension)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown ROM file type: {}. Supported extensions: {}",
                    path.display(),
                    supported_extension_list()
                )
            })?;

        Ok(Self {
            target: LaunchTarget::Rom {
                path: path.to_path_buf(),
                console_type,
            },
            options: PlayerOptions::default(),
        })
    }

    /// Set fullscreen mode.
    pub fn fullscreen(mut self, enabled: bool) -> Self {
        self.options.fullscreen = enabled;
        self
    }

    /// Enable debug overlay.
    pub fn debug(mut self, enabled: bool) -> Self {
        self.options.debug = enabled;
        self
    }

    /// Set number of players (1-4).
    pub fn players(mut self, count: usize) -> Self {
        self.options.players = Some(count);
        self
    }

    /// Host a multiplayer game on the specified port.
    pub fn host(mut self, port: u16) -> Self {
        self.options.connection = Some(ConnectionMode::Host { port });
        self
    }

    /// Join a multiplayer game at the specified address.
    pub fn join(mut self, host_ip: impl Into<String>, port: u16) -> Self {
        self.options.connection = Some(ConnectionMode::Join {
            host_ip: host_ip.into(),
            port,
        });
        self
    }

    /// Enable preview mode (asset browser).
    pub fn preview(mut self, enabled: bool) -> Self {
        self.options.preview = enabled;
        self
    }

    /// Set initial asset to focus in preview mode.
    pub fn preview_asset(mut self, asset: impl Into<String>) -> Self {
        self.options.preview_asset = Some(asset.into());
        self
    }

    /// Set all options from a PlayerOptions struct.
    pub fn options(mut self, options: PlayerOptions) -> Self {
        self.options = options;
        self
    }

    /// Get the ROM path and console type from the target.
    fn resolve_target(&self) -> Result<(&Path, ConsoleType)> {
        match &self.target {
            LaunchTarget::Rom { path, console_type } => Ok((path.as_path(), *console_type)),
            LaunchTarget::Game(game) => {
                let console_type = console_type_from_str(&game.console_type).ok_or_else(|| {
                    anyhow::anyhow!("Unknown console type: '{}'", game.console_type)
                })?;
                Ok((game.rom_path.as_path(), console_type))
            }
        }
    }

    /// Launch the player (spawns and returns immediately).
    pub fn launch(self) -> Result<()> {
        let (rom_path, console_type) = self.resolve_target()?;
        launch_player_with_options(rom_path, console_type, &self.options)
    }

    /// Run the player and wait for it to finish.
    pub fn run(self) -> Result<()> {
        let (rom_path, console_type) = self.resolve_target()?;
        run_player_with_options(rom_path, console_type, &self.options)
    }
}
