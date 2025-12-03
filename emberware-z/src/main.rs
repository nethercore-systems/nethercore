//! Emberware Z - Fantasy console runtime

use std::env;
use tracing_subscriber::EnvFilter;

mod app;
mod audio;
mod config;
mod console;
mod deep_link;
mod download;
mod ffi;
mod font;
mod graphics;
mod input;
mod library;
mod settings_ui;
mod shader_gen;
mod state;
mod ui;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    tracing::debug!("DEBUG TEST");

    let args: Vec<String> = env::args().collect();

    let mode = if let Some(uri) = deep_link::parse(&args) {
        tracing::info!("Launched via deep link: {:?}", uri);
        app::AppMode::Playing {
            game_id: uri.game_id,
        }
    } else {
        tracing::info!("Launched directly, showing library");
        app::AppMode::Library
    };

    if let Err(e) = app::run(mode) {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }
}
