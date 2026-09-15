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
    pub platform_game_id: String,
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

fn published_game_id(game: &LocalGame) -> Result<String> {
    let id = if game.rom_path.extension().is_some_and(|ext| ext == "nczx") {
        let bytes = nethercore_shared::read_file_with_limit(
            &game.rom_path,
            nethercore_shared::MAX_ROM_BYTES,
        )?;
        zx_common::ZXRom::from_bytes(&bytes)?
            .metadata
            .platform_game_id
            .unwrap_or_else(|| game.id.clone())
    } else {
        game.id.clone()
    };
    anyhow::ensure!(
        nethercore_shared::is_safe_game_id(&id),
        "Invalid published game identity"
    );
    Ok(id)
}

async fn check_for_update_async(game: &LocalGame) -> UpdateCheckResult {
    let platform_game_id = match published_game_id(game) {
        Ok(id) => id,
        Err(error) => return UpdateCheckResult::CheckFailed(error.to_string()),
    };
    let url = format!("{}/api/games/{}/version", API_BASE_URL, platform_game_id);

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
            platform_game_id,
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

    rt.block_on(async { download_update_async(update, data_dir, API_BASE_URL).await })
}

async fn download_update_async(update: &UpdateInfo, data_dir: &Path, api_base: &str) -> Result<()> {
    use nethercore_shared::{MAX_ROM_BYTES, is_safe_game_id};
    anyhow::ensure!(
        is_safe_game_id(&update.game_id) && is_safe_game_id(&update.platform_game_id),
        "Invalid update identity"
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    let endpoint = format!("{}/api/games/{}/rom-url", api_base, update.platform_game_id);
    let url: RomUrlResponse = client
        .get(endpoint)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .context("Invalid download URL response")?;
    let mut response = client
        .get(&url.url)
        .send()
        .await?
        .error_for_status()
        .context("ROM download failed")?;
    // reqwest decodes supported Brotli responses and removes Content-Encoding.
    anyhow::ensure!(
        response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .is_none_or(|encoding| encoding == "identity"),
        "Unsupported download Content-Encoding"
    );
    anyhow::ensure!(
        response
            .content_length()
            .is_none_or(|len| len <= MAX_ROM_BYTES),
        "ROM download exceeds size limit"
    );
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.context("Incomplete ROM download")? {
        anyhow::ensure!(
            bytes.len() as u64 + chunk.len() as u64 <= MAX_ROM_BYTES,
            "Decoded ROM exceeds size limit"
        );
        bytes.extend_from_slice(&chunk);
    }
    let mut rom = zx_common::ZXRom::from_bytes(&bytes).context("Invalid update cartridge")?;
    anyhow::ensure!(
        rom.metadata.id == update.game_id,
        "Update belongs to a different game"
    );
    anyhow::ensure!(
        rom.metadata.version == update.remote_version,
        "Update version does not match its announcement"
    );
    anyhow::ensure!(
        rom.metadata
            .platform_game_id
            .as_ref()
            .is_none_or(|id| id == &update.platform_game_id),
        "Update belongs to a different published game"
    );
    rom.metadata.platform_game_id = Some(update.platform_game_id.clone());
    // The shared installer validates WASM and atomically replaces the complete cart.
    // Nothing in the current installation is changed until that validation succeeds.
    zx_common::ZXRomLoader.install_bytes(&rom.to_bytes()?, data_dir)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use zx_common::{PackedData, ZXDataPack, ZXMetadata, ZXRom, ZXRomLoader};

    fn cart(id: &str, version: &str) -> ZXRom {
        ZXRom {
            version: nethercore_shared::ZX_ROM_FORMAT.version,
            metadata: ZXMetadata {
                id: id.into(),
                title: "Update fixture".into(),
                author: "Test".into(),
                version: version.into(),
                description: String::new(),
                tags: vec![],
                platform_game_id: Some("published-game".into()),
                platform_author_id: None,
                created_at: String::new(),
                tool_version: String::new(),
                render_mode: Some(0),
                default_resolution: None,
                target_fps: Some(60),
                netplay: nethercore_shared::netplay::NetplayMetadata::new(
                    nethercore_shared::ConsoleType::ZX,
                    nethercore_shared::TickRate::Fixed60,
                    2,
                    0,
                ),
            },
            code: b"\0asm\x01\0\0\0".to_vec(),
            data_pack: Some({
                let mut pack = ZXDataPack::default();
                pack.data
                    .push(PackedData::new("level", version.as_bytes().to_vec()));
                pack
            }),
            thumbnail: None,
            screenshots: vec![],
        }
    }

    // Two real local HTTP requests: URL lookup, then a download that may break mid-body.
    fn server(
        body: Vec<u8>,
        encoding: &'static str,
        extra_length: usize,
    ) -> (String, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let url_json =
            serde_json::json!({"url": format!("{base}/cart"), "expires_at": ""}).to_string();
        listener.set_nonblocking(true).unwrap();
        let handle = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            let mut served = 0;
            while served < 2 && std::time::Instant::now() < deadline {
                let (mut stream, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::park_timeout(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("{error}"),
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut request = String::new();
                let mut reader = BufReader::new(&mut stream);
                reader.read_line(&mut request).unwrap();
                loop {
                    let mut header = String::new();
                    if reader.read_line(&mut header).unwrap() == 0 || header == "\r\n" {
                        break;
                    }
                }
                let (bytes, headers, extra) = if request.contains("/rom-url") {
                    assert!(request.contains("/api/games/published-game/rom-url"));
                    (
                        url_json.as_bytes(),
                        "Content-Type: application/json\r\n".to_string(),
                        0,
                    )
                } else {
                    (
                        body.as_slice(),
                        if encoding.is_empty() {
                            String::new()
                        } else {
                            format!("Content-Encoding: {encoding}\r\n")
                        },
                        extra_length,
                    )
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nConnection: close\r\n{headers}Content-Length: {}\r\n\r\n",
                    bytes.len() + extra
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(bytes);
                served += 1;
            }
            assert_eq!(
                served, 2,
                "native updater did not execute both HTTP requests"
            );
        });
        (base, handle)
    }

    #[test]
    fn shipping_native_update_validates_before_replacing() {
        let valid = cart("update-game", "2.0").to_bytes().unwrap();
        let mut compressed = vec![];
        {
            let mut encoder = brotli::CompressorWriter::new(&mut compressed, 4096, 5, 22);
            encoder.write_all(&valid).unwrap();
        }
        let mut invalid_wasm = cart("update-game", "2.0");
        invalid_wasm.code.extend_from_slice(b"not wasm sections");
        let cases = vec![
            (cart("wrong-game", "2.0").to_bytes().unwrap(), "", 0, false),
            (
                cart("update-game", "wrong-version").to_bytes().unwrap(),
                "",
                0,
                false,
            ),
            (invalid_wasm.to_bytes().unwrap(), "", 0, false),
            (b"not a cartridge".to_vec(), "", 0, false),
            (valid[..valid.len() - 1].to_vec(), "", 0, false),
            (valid.clone(), "", 40, false),
            (valid.clone(), "unknown-codec", 0, false),
            (compressed, "br", 0, true),
            (valid, "", 0, true),
        ];
        for (index, (bytes, encoding, extra, succeeds)) in cases.into_iter().enumerate() {
            let dir = tempfile::tempdir().unwrap();
            let original = cart("update-game", "1.0").to_bytes().unwrap();
            let game = ZXRomLoader.install_bytes(&original, dir.path()).unwrap();
            assert_eq!(published_game_id(&game).unwrap(), "published-game");
            let manifest_path = game.rom_path.parent().unwrap().join("manifest.json");
            let old_manifest = std::fs::read(&manifest_path).unwrap();
            let update = UpdateInfo {
                game_id: game.id.clone(),
                game_title: game.title.clone(),
                local_version: "1.0".into(),
                remote_version: "2.0".into(),
                download_size: bytes.len() as i64,
                platform_game_id: "published-game".into(),
            };
            let (api_base, server) = server(bytes, encoding, extra);
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(download_update_async(&update, dir.path(), &api_base));
            server.join().unwrap();
            assert_eq!(result.is_ok(), succeeds, "update case {index}: {result:?}");
            if succeeds {
                let installed = ZXRom::from_bytes(&std::fs::read(&game.rom_path).unwrap()).unwrap();
                assert_eq!(installed.metadata.id, game.id);
                assert_eq!(installed.metadata.version, "2.0");
                assert_eq!(
                    installed
                        .data_pack
                        .unwrap()
                        .find_data("level")
                        .unwrap()
                        .data,
                    b"2.0"
                );
            } else {
                assert_eq!(
                    std::fs::read(&game.rom_path).unwrap(),
                    original,
                    "case {index} replaced cart"
                );
                assert_eq!(
                    std::fs::read(manifest_path).unwrap(),
                    old_manifest,
                    "case {index} replaced manifest"
                );
            }
        }
    }
}
