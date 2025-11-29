//! Emberware Z - Fantasy console runtime

use std::env;

mod app;
mod config;
mod console;
mod deep_link;
mod download;
mod ffi;
mod graphics;
mod library;
mod runtime;
mod ui;

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    let mode = if let Some(uri) = deep_link::parse(&args) {
        tracing::info!("Launched via deep link: {:?}", uri);
        app::AppMode::Playing { game_id: uri.game_id }
    } else {
        tracing::info!("Launched directly, showing library");
        app::AppMode::Library
    };

    if let Err(e) = app::run(mode) {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }
}
