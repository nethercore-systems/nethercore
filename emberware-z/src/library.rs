//! Local game library management

use emberware_shared::LocalGameManifest;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LocalGame {
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: String,
    pub rom_path: PathBuf,
}

pub fn get_local_games() -> Vec<LocalGame> {
    let games_dir = match crate::config::data_dir() {
        Some(dir) => dir.join("games"),
        None => return vec![],
    };

    let Ok(entries) = std::fs::read_dir(games_dir) else {
        return vec![];
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }

            let manifest_path = path.join("manifest.json");
            let manifest_content = std::fs::read_to_string(manifest_path).ok()?;
            let manifest: LocalGameManifest = serde_json::from_str(&manifest_content).ok()?;

            Some(LocalGame {
                id: manifest.id,
                title: manifest.title,
                author: manifest.author,
                version: manifest.version,
                rom_path: path.join("rom.wasm"),
            })
        })
        .collect()
}

pub fn is_cached(game_id: &str) -> bool {
    crate::config::data_dir()
        .map(|dir| dir.join("games").join(game_id).join("rom.wasm").exists())
        .unwrap_or(false)
}

pub fn delete_game(game_id: &str) -> std::io::Result<()> {
    if let Some(dir) = crate::config::data_dir() {
        let game_dir = dir.join("games").join(game_id);
        if game_dir.exists() {
            std::fs::remove_dir_all(game_dir)?;
        }
    }
    Ok(())
}
