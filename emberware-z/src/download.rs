//! ROM download from platform API

use emberware_shared::{Game, RomUrlResponse};
use std::path::PathBuf;
use thiserror::Error;

const API_URL: &str = "https://api.emberware.io";

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Game not found")]
    NotFound,
}

pub async fn download_game<F>(
    game_id: &str,
    on_progress: F,
) -> Result<PathBuf, DownloadError>
where
    F: Fn(f32),
{
    let client = reqwest::Client::new();

    // Get game metadata
    let game: Game = client
        .get(format!("{}/api/games/{}", API_URL, game_id))
        .send()
        .await?
        .json()
        .await?;

    // Get presigned download URL
    let rom_url: RomUrlResponse = client
        .get(format!("{}/api/games/{}/rom-url", API_URL, game_id))
        .send()
        .await?
        .json()
        .await?;

    // Create game directory
    let game_dir = crate::config::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("games")
        .join(&game_id);
    std::fs::create_dir_all(&game_dir)?;

    // Download ROM
    let rom_path = game_dir.join("rom.wasm");
    let response = client.get(&rom_url.url).send().await?;
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let bytes = response.bytes().await?;
    downloaded = bytes.len() as u64;
    on_progress(downloaded as f32 / total_size.max(1) as f32);
    std::fs::write(&rom_path, bytes)?;

    // Save manifest
    let manifest = emberware_shared::LocalGameManifest {
        id: game.id,
        title: game.title,
        author: game.author.username,
        version: game.rom_version.unwrap_or_default(),
        downloaded_at: chrono::Utc::now().to_rfc3339(),
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(game_dir.join("manifest.json"), manifest_json)?;

    Ok(rom_path)
}
