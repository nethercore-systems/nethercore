//! URL Protocol Registration
//!
//! Registers the `nethercore://` URL scheme with the operating system.
//! Called on first launch and is idempotent (safe to call multiple times).
//!
//! # Platform Support
//!
//! - **Windows**: Registry entry at `HKCU\Software\Classes\nethercore`
//! - **macOS**: Info.plist in Application Support + LaunchServices registration
//! - **Linux**: `.desktop` file in `~/.local/share/applications/`

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Check if the protocol is already registered
pub fn is_registered() -> bool {
    #[cfg(target_os = "windows")]
    {
        return windows::is_registered();
    }

    #[cfg(target_os = "macos")]
    {
        return macos::is_registered();
    }

    #[cfg(target_os = "linux")]
    {
        return linux::is_registered();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return false;
    }
}

/// Register the nethercore:// protocol handler
///
/// This is idempotent - calling it multiple times is safe.
/// Registration happens at the user level (no admin required).
pub fn register() -> Result<()> {
    if is_registered() {
        tracing::debug!("Protocol handler already registered");
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        windows::register()?;
    }

    #[cfg(target_os = "macos")]
    {
        macos::register()?;
    }

    #[cfg(target_os = "linux")]
    {
        linux::register()?;
    }

    tracing::info!("Registered nethercore:// protocol handler");
    Ok(())
}

/// Get the path to the current executable
fn get_exe_path() -> Result<PathBuf> {
    std::env::current_exe().context("Failed to get executable path")
}

// =============================================================================
// Windows Implementation
// =============================================================================

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use winreg::RegKey;
    use winreg::enums::*;

    pub fn is_registered() -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey("Software\\Classes\\nethercore").is_ok()
    }

    pub fn register() -> Result<()> {
        let exe = get_exe_path()?;
        let exe_str = exe.to_string_lossy();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // Create protocol key: HKCU\Software\Classes\nethercore
        let (key, _) = hkcu
            .create_subkey("Software\\Classes\\nethercore")
            .context("Failed to create registry key")?;

        // Set default value to describe the protocol
        key.set_value("", &"URL:Nethercore Protocol")
            .context("Failed to set protocol description")?;

        // Mark as URL protocol (required for Windows to recognize it)
        key.set_value("URL Protocol", &"")
            .context("Failed to set URL Protocol flag")?;

        // Create DefaultIcon subkey with application icon
        let (icon_key, _) = key
            .create_subkey("DefaultIcon")
            .context("Failed to create DefaultIcon key")?;
        icon_key
            .set_value("", &format!("{},0", exe_str))
            .context("Failed to set icon path")?;

        // Create shell\open\command subkey with launch command
        let (cmd_key, _) = key
            .create_subkey(r"shell\open\command")
            .context("Failed to create command key")?;
        cmd_key
            .set_value("", &format!("\"{}\" \"%1\"", exe_str))
            .context("Failed to set command")?;

        Ok(())
    }
}

// =============================================================================
// macOS Implementation
// =============================================================================

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn get_app_support_path() -> Result<PathBuf> {
        dirs::data_dir()
            .context("Failed to get data directory")
            .map(|d| d.join("nethercore"))
    }

    fn get_plist_path() -> Result<PathBuf> {
        get_app_support_path().map(|d| d.join("Info.plist"))
    }

    pub fn is_registered() -> bool {
        if let Ok(plist_path) = get_plist_path() {
            if let Ok(contents) = fs::read_to_string(&plist_path) {
                return contents.contains("nethercore");
            }
        }
        false
    }

    pub fn register() -> Result<()> {
        let exe = get_exe_path()?;
        let app_support = get_app_support_path()?;

        // Create application support directory
        fs::create_dir_all(&app_support).context("Failed to create app support directory")?;

        // Create minimal app bundle structure for URL handler
        let contents_dir = app_support.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        fs::create_dir_all(&macos_dir).context("Failed to create MacOS directory")?;

        // Create Info.plist for URL registration
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>io.nethercore.launcher</string>
    <key>CFBundleName</key>
    <string>Nethercore</string>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>Nethercore URL</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>nethercore</string>
            </array>
        </dict>
    </array>
    <key>CFBundleExecutable</key>
    <string>nethercore</string>
</dict>
</plist>"#
        );

        let plist_path = contents_dir.join("Info.plist");
        fs::write(&plist_path, plist_content).context("Failed to write Info.plist")?;

        // Create symlink to actual executable
        let target_exe = macos_dir.join("nethercore");
        if target_exe.exists() {
            fs::remove_file(&target_exe).ok();
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&exe, &target_exe).context("Failed to create symlink")?;

        // Register with LaunchServices
        let lsregister_path = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";
        if std::path::Path::new(lsregister_path).exists() {
            Command::new(lsregister_path)
                .args(["-R", "-f", app_support.to_str().unwrap_or("")])
                .output()
                .context("Failed to register with LaunchServices")?;
        }

        Ok(())
    }
}

// =============================================================================
// Linux Implementation
// =============================================================================

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn get_desktop_file_path() -> Result<PathBuf> {
        dirs::data_dir()
            .context("Failed to get data directory")
            .map(|d| d.join("applications/nethercore-handler.desktop"))
    }

    pub fn is_registered() -> bool {
        if let Ok(path) = get_desktop_file_path() {
            return path.exists();
        }
        false
    }

    pub fn register() -> Result<()> {
        let exe = get_exe_path()?;

        // Create applications directory
        let apps_dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("applications");
        fs::create_dir_all(&apps_dir).context("Failed to create applications directory")?;

        // Create .desktop file
        let desktop_content = format!(
            r#"[Desktop Entry]
Name=Nethercore
Comment=Nethercore Fantasy Console Launcher
Exec="{}" %u
Type=Application
Terminal=false
MimeType=x-scheme-handler/nethercore;
NoDisplay=true
Categories=Game;
"#,
            exe.display()
        );

        let desktop_path = apps_dir.join("nethercore-handler.desktop");
        fs::write(&desktop_path, desktop_content).context("Failed to write .desktop file")?;

        // Make it executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&desktop_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&desktop_path, perms)?;
        }

        // Update desktop database (non-fatal if fails)
        Command::new("update-desktop-database")
            .arg(&apps_dir)
            .output()
            .ok();

        // Register as default handler for the scheme
        Command::new("xdg-mime")
            .args([
                "default",
                "nethercore-handler.desktop",
                "x-scheme-handler/nethercore",
            ])
            .output()
            .ok();

        Ok(())
    }
}

// =============================================================================
// Stub implementations for unsupported platforms
// =============================================================================

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod unsupported {
    use super::*;

    pub fn is_registered() -> bool {
        false
    }

    pub fn register() -> Result<()> {
        tracing::warn!("Protocol registration not supported on this platform");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_exe_path() {
        // Should not panic
        let result = get_exe_path();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_registered_does_not_panic() {
        // Just verify it doesn't panic
        let _ = is_registered();
    }
}
