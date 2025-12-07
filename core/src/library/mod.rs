//! Game library management
//!
//! Console-agnostic game discovery, loading, and deletion.

pub mod cart;
mod data_dir;
mod game;
mod resolver;

pub use cart::install_z_rom;
pub use data_dir::DataDirProvider;
pub use game::{delete_game, get_local_games, is_cached, LocalGame};
pub use resolver::{resolve_game_id, GameResolutionError};
