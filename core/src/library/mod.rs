//! Game library management
//!
//! Console-agnostic game discovery, loading, and deletion.

mod data_dir;
mod game;
mod resolver;
pub mod rom;

pub use data_dir::DataDirProvider;
pub use game::{LocalGame, delete_game, get_local_games, get_local_games_with_loaders, is_cached};
pub use resolver::{GameResolutionError, resolve_game_id};
pub use rom::{RomLoader, RomLoaderRegistry, RomMetadata, install_rom};
