//! Game update checking and downloading
//!
//! Checks the Nethercore API for game updates and prompts the user to download.

use anyhow::{Context, Result};
use nethercore_core::library::LocalGame;
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

/// API base URL for production
const API_BASE_URL: &str = "https://api.nethercore.systems";

/// Timeout for update check requests
const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(3);

/// Response from the version check endpoint
#[derive(Debug, Deserialize)]
struct VersionResponse {
    version: Option<String>,
    rom_size: i64,
}

/// Response from the ROM URL endpoint
#[derive(Debug, Deserialize)]
struct RomUrlResponse {
    url: String,
    #[allow(dead_code)]
    expires_at: String,
}

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub game_id: String,
    pub game_title: String,
    pub local_version: String,
    pub remote_version: String,
    pub download_size: i64,
}

/// Result of checking for an update
#[derive(Debug)]
pub enum UpdateCheckResult {
    /// An update is available
    UpdateAvailable(UpdateInfo),
    /// Already up to date
    UpToDate,
    /// Check failed (network error, timeout, etc.) - should not block game launch
    CheckFailed(String),
}

/// Check if a game has an available update
///
/// This function has a 3-second timeout to avoid blocking game launch.
/// If the check fails for any reason, it returns `CheckFailed` and the game
/// should launch anyway with the current version.
pub fn check_for_update(game: &LocalGame) -> UpdateCheckResult {
    // Create a runtime for this single async operation
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            return UpdateCheckResult::CheckFailed(format!("Failed to create runtime: {}", e));
        }
    };

    rt.block_on(async { check_for_update_async(game).await })
}

async fn check_for_update_async(game: &LocalGame) -> UpdateCheckResult {
    let url = format!("{}/api/games/{}/version", API_BASE_URL, game.id);

    // Create client with timeout
    let client = match reqwest::Client::builder()
        .timeout(UPDATE_CHECK_TIMEOUT)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return UpdateCheckResult::CheckFailed(format!("Failed to create HTTP client: {}", e));
        }
    };

    // Make the request with timeout
    let response = match tokio::time::timeout(UPDATE_CHECK_TIMEOUT, client.get(&url).send()).await {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => return UpdateCheckResult::CheckFailed(format!("Network error: {}", e)),
        Err(_) => return UpdateCheckResult::CheckFailed("Request timed out".to_string()),
    };

    // Check HTTP status
    if !response.status().is_success() {
        return UpdateCheckResult::CheckFailed(format!("HTTP {}", response.status()));
    }

    // Parse response
    let version_info: VersionResponse = match response.json().await {
        Ok(v) => v,
        Err(e) => {
            return UpdateCheckResult::CheckFailed(format!("Failed to parse response: {}", e));
        }
    };

    // Compare versions
    let remote_version = match version_info.version {
        Some(v) => v,
        None => return UpdateCheckResult::CheckFailed("No version in response".to_string()),
    };

    // Simple version comparison - if they differ, an update is available
    // Note: This doesn't handle semantic versioning properly, but it's sufficient
    // for our use case where versions are sequential
    if remote_version != game.version && !game.version.is_empty() {
        UpdateCheckResult::UpdateAvailable(UpdateInfo {
            game_id: game.id.clone(),
            game_title: game.title.clone(),
            local_version: game.version.clone(),
            remote_version,
            download_size: version_info.rom_size,
        })
    } else {
        UpdateCheckResult::UpToDate
    }
}

/// Prompt the user about an available update
///
/// Returns `true` if the user wants to update, `false` to play anyway.
pub fn prompt_for_update(update: &UpdateInfo) -> bool {
    let message = format!(
        "Update available for {}\n\nCurrent version: {}\nNew version: {}\nDownload size: {:.2} MB\n\nWould you like to update now?",
        update.game_title,
        update.local_version,
        update.remote_version,
        update.download_size as f64 / 1024.0 / 1024.0
    );

    // Use rfd for a native dialog
    rfd::MessageDialog::new()
        .set_title("Update Available")
        .set_description(&message)
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        == rfd::MessageDialogResult::Yes
}

/// Download and install a game update
///
/// This downloads the new ROM, extracts it, and updates the local manifest.
pub fn download_update(update: &UpdateInfo, data_dir: &Path) -> Result<()> {
    // Create a runtime for this async operation
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create runtime")?;

    rt.block_on(async { download_update_async(update, data_dir).await })
}

async fn download_update_async(update: &UpdateInfo, data_dir: &Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300)) // 5 minute timeout for downloads
        .build()
        .context("Failed to create HTTP client")?;

    // 1. Get presigned download URL
    let url_endpoint = format!("{}/api/games/{}/rom-url", API_BASE_URL, update.game_id);
    let url_response: RomUrlResponse = client
        .get(&url_endpoint)
        .send()
        .await
        .context("Failed to get download URL")?
        .json()
        .await
        .context("Failed to parse download URL response")?;

    // 2. Download the ROM file
    tracing::info!("Downloading ROM update from presigned URL...");
    let rom_response = client
        .get(&url_response.url)
        .send()
        .await
        .context("Failed to download ROM")?;

    if !rom_response.status().is_success() {
        anyhow::bail!("Download failed with HTTP {}", rom_response.status());
    }

    let rom_bytes = rom_response
        .bytes()
        .await
        .context("Failed to read ROM data")?;

    // 3. Write the ROM file to the game directory
    let game_dir = data_dir.join("games").join(&update.game_id);
    let rom_path = game_dir.join("rom.nczx"); // TODO: Support other console types

    std::fs::write(&rom_path, &rom_bytes).context("Failed to write ROM file")?;

    // 4. Update the manifest with the new version
    let manifest_path = game_dir.join("manifest.json");
    if manifest_path.exists() {
        let manifest_content =
            std::fs::read_to_string(&manifest_path).context("Failed to read manifest")?;

        // Parse, update version, and write back
        let mut manifest: serde_json::Value =
            serde_json::from_str(&manifest_content).context("Failed to parse manifest")?;

        manifest["version"] = serde_json::Value::String(update.remote_version.clone());
        manifest["updated_at"] = serde_json::Value::String(chrono::Utc::now().to_rfc3339());

        let updated_manifest =
            serde_json::to_string_pretty(&manifest).context("Failed to serialize manifest")?;

        std::fs::write(&manifest_path, updated_manifest).context("Failed to write manifest")?;
    }

    tracing::info!(
        "Update installed successfully: v{} -> v{}",
        update.local_version,
        update.remote_version
    );
    Ok(())
}

/// Check for update and prompt user if one is available
///
/// This is the main entry point for update checking. It:
/// 1. Checks for an update (with 3-second timeout)
/// 2. If available, prompts the user
/// 3. If user accepts, downloads and installs the update
///
/// Returns `true` if an update was installed (caller should reload game data).
pub fn check_and_prompt_for_update(game: &LocalGame, data_dir: &Path) -> bool {
    match check_for_update(game) {
        UpdateCheckResult::UpdateAvailable(update) => {
            tracing::info!(
                "Update available for {}: {} -> {}",
                update.game_title,
                update.local_version,
                update.remote_version
            );

            if prompt_for_update(&update) {
                // Show a progress dialog or just log
                tracing::info!("User accepted update, downloading...");

                match download_update(&update, data_dir) {
                    Ok(()) => {
                        // Show success message
                        rfd::MessageDialog::new()
                            .set_title("Update Complete")
                            .set_description(format!(
                                "{} has been updated to version {}",
                                update.game_title, update.remote_version
                            ))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show();
                        true
                    }
                    Err(e) => {
                        tracing::error!("Update download failed: {}", e);
                        rfd::MessageDialog::new()
                            .set_title("Update Failed")
                            .set_description(format!(
                                "Failed to download update: {}\n\nThe game will launch with the current version.",
                                e
                            ))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show();
                        false
                    }
                }
            } else {
                tracing::info!("User declined update, launching current version");
                false
            }
        }
        UpdateCheckResult::UpToDate => {
            tracing::debug!("Game {} is up to date", game.id);
            false
        }
        UpdateCheckResult::CheckFailed(reason) => {
            tracing::debug!("Update check failed for {}: {}", game.id, reason);
            // Silently continue with current version
            false
        }
    }
}
