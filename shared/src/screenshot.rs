//! Screenshot origin verification for abuse prevention.
//!
//! This module provides HMAC-based signing and verification for screenshots
//! captured by the Nethercore player. Screenshots are signed at capture time
//! and verified on upload to prevent users from uploading arbitrary images.
//!
//! # Security Model
//!
//! This provides casual abuse prevention, not cryptographic security against
//! determined attackers. The shared secret is embedded in both the player and
//! backend binaries, which means it can be extracted through reverse engineering.
//! This is acceptable for the intended use case of preventing casual abuse.
//!
//! # Usage
//!
//! ## Signing (in the player)
//! ```ignore
//! use nethercore_shared::screenshot::{ScreenshotPayload, sign_screenshot};
//!
//! let payload = ScreenshotPayload::new(&pixel_hash, "zx", 960, 540);
//! let signed = sign_screenshot(&payload)?;
//! // Embed signed.to_json()? in PNG iTXt chunk
//! ```
//!
//! ## Verification (in the backend)
//! ```ignore
//! use nethercore_shared::screenshot::{SignedScreenshot, verify_screenshot};
//!
//! let signed = SignedScreenshot::from_json(json_str)?;
//! verify_screenshot(&signed)?; // Returns Ok(()) or Err
//! ```

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// PNG iTXt chunk keyword for screenshot verification.
/// Using "nethercore-sig" to be descriptive but compact.
pub const SCREENSHOT_SIGNATURE_KEYWORD: &str = "nethercore-sig";

/// Shared secret for HMAC-SHA256 signing.
///
/// This provides casual abuse prevention, not cryptographic security.
/// A determined attacker can extract this from the binary.
///
/// Generated with: openssl rand -hex 32
const SCREENSHOT_HMAC_SECRET: &[u8] =
    b"\x7a\x3b\x1c\x9d\x4e\x2f\x8a\x6b\x5c\x0d\x3e\x9f\x1a\x7b\x4c\x8d\
      \x2e\x5f\x0a\x6c\x3d\x9e\x1b\x7a\x4d\x8c\x2f\x5e\x0b\x6d\x3c\x9f";

/// Screenshot verification payload embedded in PNG metadata.
///
/// This is signed with HMAC-SHA256 and embedded in the PNG as an iTXt chunk.
/// On upload, the backend extracts this chunk and verifies the signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenshotPayload {
    /// Version for future-proofing (currently 1).
    pub version: u8,
    /// SHA-256 hash of the raw RGBA pixel data (hex-encoded, 64 chars).
    pub pixel_hash: String,
    /// Console type that captured this (e.g., "zx", "chroma").
    pub console_type: String,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
}

impl ScreenshotPayload {
    /// Create a new screenshot payload.
    ///
    /// # Arguments
    /// * `pixel_hash` - SHA-256 hash of RGBA pixel data, hex-encoded
    /// * `console_type` - Console identifier (e.g., "zx", "chroma")
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    pub fn new(pixel_hash: impl Into<String>, console_type: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            version: 1,
            pixel_hash: pixel_hash.into(),
            console_type: console_type.into(),
            width,
            height,
        }
    }
}

/// Signed screenshot data containing payload and HMAC signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedScreenshot {
    /// The verification payload.
    pub payload: ScreenshotPayload,
    /// HMAC-SHA256 signature of the payload JSON (hex-encoded, 64 chars).
    pub signature: String,
}

impl SignedScreenshot {
    /// Serialize to JSON string for embedding in PNG.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string extracted from PNG.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Error type for screenshot signing/verification.
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotSignError {
    #[error("Failed to serialize payload: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid HMAC key length")]
    InvalidKeyLength,

    #[error("Signature verification failed")]
    VerificationFailed,

    #[error("Invalid signature encoding")]
    InvalidSignatureEncoding,

    #[error("Unsupported payload version: {0}")]
    UnsupportedVersion(u8),
}

/// Sign a screenshot payload with HMAC-SHA256.
///
/// Returns a `SignedScreenshot` containing the payload and signature.
pub fn sign_screenshot(payload: &ScreenshotPayload) -> Result<SignedScreenshot, ScreenshotSignError> {
    let payload_json = serde_json::to_string(payload)?;

    let mut mac = HmacSha256::new_from_slice(SCREENSHOT_HMAC_SECRET)
        .map_err(|_| ScreenshotSignError::InvalidKeyLength)?;
    mac.update(payload_json.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    Ok(SignedScreenshot {
        payload: payload.clone(),
        signature,
    })
}

/// Verify a signed screenshot.
///
/// Returns `Ok(())` if the signature is valid, or an error describing the failure.
pub fn verify_screenshot(signed: &SignedScreenshot) -> Result<(), ScreenshotSignError> {
    // Check version
    if signed.payload.version != 1 {
        return Err(ScreenshotSignError::UnsupportedVersion(signed.payload.version));
    }

    // Recompute signature
    let payload_json = serde_json::to_string(&signed.payload)?;

    let mut mac = HmacSha256::new_from_slice(SCREENSHOT_HMAC_SECRET)
        .map_err(|_| ScreenshotSignError::InvalidKeyLength)?;
    mac.update(payload_json.as_bytes());

    // Decode provided signature
    let expected = hex::decode(&signed.signature)
        .map_err(|_| ScreenshotSignError::InvalidSignatureEncoding)?;

    // Verify
    mac.verify_slice(&expected)
        .map_err(|_| ScreenshotSignError::VerificationFailed)?;

    Ok(())
}

/// Compute SHA-256 hash of pixel data for the payload.
///
/// # Arguments
/// * `pixels` - Raw RGBA pixel data (4 bytes per pixel)
///
/// # Returns
/// Hex-encoded SHA-256 hash (64 characters)
pub fn compute_pixel_hash(pixels: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(pixels);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let payload = ScreenshotPayload::new(
            "abcd1234".repeat(8), // 64 char hex hash
            "zx",
            960,
            540,
        );

        let signed = sign_screenshot(&payload).unwrap();
        assert!(verify_screenshot(&signed).is_ok());
    }

    #[test]
    fn test_tampered_payload_fails() {
        let payload = ScreenshotPayload::new(
            "abcd1234".repeat(8),
            "zx",
            960,
            540,
        );

        let mut signed = sign_screenshot(&payload).unwrap();
        // Tamper with the payload
        signed.payload.console_type = "chroma".to_string();

        assert!(matches!(
            verify_screenshot(&signed),
            Err(ScreenshotSignError::VerificationFailed)
        ));
    }

    #[test]
    fn test_invalid_signature_fails() {
        let payload = ScreenshotPayload::new(
            "abcd1234".repeat(8),
            "zx",
            960,
            540,
        );

        let mut signed = sign_screenshot(&payload).unwrap();
        // Tamper with the signature
        signed.signature = "0".repeat(64);

        assert!(matches!(
            verify_screenshot(&signed),
            Err(ScreenshotSignError::VerificationFailed)
        ));
    }

    #[test]
    fn test_unsupported_version_fails() {
        let payload = ScreenshotPayload {
            version: 99,
            pixel_hash: "abcd1234".repeat(8),
            console_type: "zx".to_string(),
            width: 960,
            height: 540,
        };

        let signed = sign_screenshot(&payload).unwrap();
        assert!(matches!(
            verify_screenshot(&signed),
            Err(ScreenshotSignError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn test_json_roundtrip() {
        let payload = ScreenshotPayload::new(
            "abcd1234".repeat(8),
            "zx",
            960,
            540,
        );

        let signed = sign_screenshot(&payload).unwrap();
        let json = signed.to_json().unwrap();
        let parsed = SignedScreenshot::from_json(&json).unwrap();

        assert_eq!(signed.payload, parsed.payload);
        assert_eq!(signed.signature, parsed.signature);
        assert!(verify_screenshot(&parsed).is_ok());
    }

    #[test]
    fn test_compute_pixel_hash() {
        let pixels = vec![0u8; 960 * 540 * 4]; // RGBA
        let hash = compute_pixel_hash(&pixels);
        assert_eq!(hash.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }
}
