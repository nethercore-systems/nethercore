//! Build-time WASM analysis module
//!
//! Provides stub state and types for analyzing WASM games at build time.
//! The actual analysis execution happens in ember-cli, but the types
//! are shared here for use by both the CLI and any future analysis tools.
//!
//! # Purpose
//!
//! Build-time analysis executes a game's `init()` function with stub FFI
//! to capture configuration calls like `render_mode()`, `set_resolution()`,
//! and texture loading requests. This information determines how assets
//! should be compressed (e.g., BC7 for modes 1-3, RGBA8 for mode 0).
//!
//! # Example
//!
//! ```ignore
//! // In ember-cli:
//! let state = StubFFIState::default();
//! // ... execute init() with stub FFI ...
//! let result = AnalysisResult {
//!     render_mode: state.render_mode.unwrap_or(0),
//!     resolution: state.resolution,
//!     texture_ids: state.extract_texture_ids(),
//! };
//! ```

/// Result of build-time WASM analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    /// Detected render mode (0-3, defaults to 0 if not set)
    pub render_mode: u8,

    /// Resolution if set (width, height)
    pub resolution: Option<(u32, u32)>,

    /// Tick rate if set
    pub tick_rate: Option<u32>,

    /// ROM texture IDs requested during init
    pub texture_ids: Vec<String>,

    /// ROM mesh IDs requested during init
    pub mesh_ids: Vec<String>,
}

impl AnalysisResult {
    /// Check if render mode uses BC7 compression
    pub fn uses_bc7(&self) -> bool {
        self.render_mode >= 1 && self.render_mode <= 3
    }

    /// Get the texture format for a given slot
    ///
    /// Returns the appropriate format based on render mode and slot:
    /// - Mode 0: Always RGBA8
    /// - Mode 1: BC7 for all slots (matcap)
    /// - Mode 2: BC7 sRGB for slots 0,3; BC7 Linear for slot 1 (material)
    /// - Mode 3: BC7 sRGB for slots 0,2,3; BC7 Linear for slot 1 (material)
    pub fn texture_format_for_slot(&self, slot: u8) -> TextureFormatHint {
        match self.render_mode {
            0 => TextureFormatHint::Rgba8,
            1 => TextureFormatHint::Bc7Srgb, // Matcap - all sRGB
            2 | 3 => {
                if slot == 1 {
                    TextureFormatHint::Bc7Linear // Material map
                } else {
                    TextureFormatHint::Bc7Srgb // Albedo, specular, env
                }
            }
            _ => TextureFormatHint::Rgba8, // Invalid mode defaults to RGBA8
        }
    }
}

/// Hint for texture format based on analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormatHint {
    /// Uncompressed RGBA8
    Rgba8,
    /// BC7 compressed, sRGB color space
    Bc7Srgb,
    /// BC7 compressed, linear color space (for material maps)
    Bc7Linear,
}

/// Texture request captured during analysis
#[derive(Debug, Clone)]
pub struct TextureRequest {
    /// Allocated handle (for ordering/debugging)
    pub handle: u32,

    /// Texture dimensions (0,0 if loaded from ROM - dimensions come from asset file)
    pub width: u32,
    pub height: u32,

    /// Source of the texture
    pub source: TextureSource,
}

/// Source of a texture request
#[derive(Debug, Clone)]
pub enum TextureSource {
    /// Loaded from WASM memory via load_texture()
    WasmMemory {
        /// Pointer in WASM linear memory
        ptr: u32,
    },

    /// Loaded from ROM data pack via rom_texture()
    RomPack {
        /// Asset ID string
        id: String,
    },
}

/// Stub FFI state for build-time analysis
///
/// Captures init-time configuration calls without actual GPU/audio resources.
/// This is a minimal state that only records what the game requested.
#[derive(Debug, Clone, Default)]
pub struct StubFFIState {
    /// Render mode set by render_mode() call
    pub render_mode: Option<u8>,

    /// Resolution set by set_resolution() call (width, height)
    pub resolution: Option<(u32, u32)>,

    /// Tick rate set by set_tick_rate() call
    pub tick_rate: Option<u32>,

    /// Clear color set by set_clear_color() call
    pub clear_color: Option<u32>,

    /// Texture requests captured during init
    pub texture_requests: Vec<TextureRequest>,

    /// Mesh IDs requested from ROM during init
    pub mesh_requests: Vec<String>,

    /// Skeleton IDs requested from ROM during init
    pub skeleton_requests: Vec<String>,

    /// Sound IDs requested from ROM during init
    pub sound_requests: Vec<String>,

    /// Next texture handle to allocate
    next_texture_handle: u32,

    /// Next mesh handle to allocate
    next_mesh_handle: u32,
}

impl StubFFIState {
    /// Create new stub state
    pub fn new() -> Self {
        Self {
            next_texture_handle: 1, // 0 is reserved for invalid
            next_mesh_handle: 1,
            ..Default::default()
        }
    }

    /// Allocate a texture handle and record the request
    pub fn alloc_texture(&mut self, width: u32, height: u32, source: TextureSource) -> u32 {
        let handle = self.next_texture_handle;
        self.next_texture_handle += 1;

        self.texture_requests.push(TextureRequest {
            handle,
            width,
            height,
            source,
        });

        handle
    }

    /// Allocate a mesh handle and record ROM request
    pub fn alloc_mesh(&mut self, id: String) -> u32 {
        let handle = self.next_mesh_handle;
        self.next_mesh_handle += 1;
        self.mesh_requests.push(id);
        handle
    }

    /// Extract texture IDs from ROM requests
    pub fn extract_texture_ids(&self) -> Vec<String> {
        self.texture_requests
            .iter()
            .filter_map(|req| match &req.source {
                TextureSource::RomPack { id } => Some(id.clone()),
                TextureSource::WasmMemory { .. } => None,
            })
            .collect()
    }

    /// Convert to AnalysisResult
    pub fn to_result(&self) -> AnalysisResult {
        AnalysisResult {
            render_mode: self.render_mode.unwrap_or(0),
            resolution: self.resolution,
            tick_rate: self.tick_rate,
            texture_ids: self.extract_texture_ids(),
            mesh_ids: self.mesh_requests.clone(),
        }
    }
}

/// Validation error for analysis results
#[derive(Debug, Clone, thiserror::Error)]
pub enum AnalysisError {
    /// Invalid render mode (must be 0-3)
    #[error("invalid render mode {0} (must be 0-3)")]
    InvalidRenderMode(u8),

    /// WASM execution error
    #[error("WASM execution failed: {0}")]
    WasmError(String),

    /// init() function not found
    #[error("init() function not exported by WASM module")]
    InitNotFound,
}

/// Validate analysis result
pub fn validate_result(result: &AnalysisResult) -> Result<(), AnalysisError> {
    if result.render_mode > 3 {
        return Err(AnalysisError::InvalidRenderMode(result.render_mode));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_render_mode() {
        let state = StubFFIState::new();
        let result = state.to_result();
        assert_eq!(result.render_mode, 0);
    }

    #[test]
    fn test_capture_render_mode() {
        let mut state = StubFFIState::new();
        state.render_mode = Some(2);

        let result = state.to_result();
        assert_eq!(result.render_mode, 2);
    }

    #[test]
    fn test_uses_bc7() {
        assert!(!AnalysisResult { render_mode: 0, ..Default::default() }.uses_bc7());
        assert!(AnalysisResult { render_mode: 1, ..Default::default() }.uses_bc7());
        assert!(AnalysisResult { render_mode: 2, ..Default::default() }.uses_bc7());
        assert!(AnalysisResult { render_mode: 3, ..Default::default() }.uses_bc7());
    }

    #[test]
    fn test_texture_format_for_slot_mode0() {
        let result = AnalysisResult { render_mode: 0, ..Default::default() };
        assert_eq!(result.texture_format_for_slot(0), TextureFormatHint::Rgba8);
        assert_eq!(result.texture_format_for_slot(1), TextureFormatHint::Rgba8);
    }

    #[test]
    fn test_texture_format_for_slot_mode1() {
        let result = AnalysisResult { render_mode: 1, ..Default::default() };
        assert_eq!(result.texture_format_for_slot(0), TextureFormatHint::Bc7Srgb);
        assert_eq!(result.texture_format_for_slot(1), TextureFormatHint::Bc7Srgb);
        assert_eq!(result.texture_format_for_slot(2), TextureFormatHint::Bc7Srgb);
        assert_eq!(result.texture_format_for_slot(3), TextureFormatHint::Bc7Srgb);
    }

    #[test]
    fn test_texture_format_for_slot_mode2() {
        let result = AnalysisResult { render_mode: 2, ..Default::default() };
        assert_eq!(result.texture_format_for_slot(0), TextureFormatHint::Bc7Srgb);  // Albedo
        assert_eq!(result.texture_format_for_slot(1), TextureFormatHint::Bc7Linear); // Material
        assert_eq!(result.texture_format_for_slot(3), TextureFormatHint::Bc7Srgb);  // Env
    }

    #[test]
    fn test_texture_format_for_slot_mode3() {
        let result = AnalysisResult { render_mode: 3, ..Default::default() };
        assert_eq!(result.texture_format_for_slot(0), TextureFormatHint::Bc7Srgb);  // Albedo
        assert_eq!(result.texture_format_for_slot(1), TextureFormatHint::Bc7Linear); // Material
        assert_eq!(result.texture_format_for_slot(2), TextureFormatHint::Bc7Srgb);  // Specular
        assert_eq!(result.texture_format_for_slot(3), TextureFormatHint::Bc7Srgb);  // Env
    }

    #[test]
    fn test_alloc_texture() {
        let mut state = StubFFIState::new();

        let h1 = state.alloc_texture(64, 64, TextureSource::RomPack { id: "player".to_string() });
        let h2 = state.alloc_texture(32, 32, TextureSource::WasmMemory { ptr: 0x1000 });

        assert_eq!(h1, 1);
        assert_eq!(h2, 2);
        assert_eq!(state.texture_requests.len(), 2);
    }

    #[test]
    fn test_extract_texture_ids() {
        let mut state = StubFFIState::new();
        state.alloc_texture(64, 64, TextureSource::RomPack { id: "player".to_string() });
        state.alloc_texture(32, 32, TextureSource::WasmMemory { ptr: 0x1000 });
        state.alloc_texture(128, 128, TextureSource::RomPack { id: "enemy".to_string() });

        let ids = state.extract_texture_ids();
        assert_eq!(ids, vec!["player", "enemy"]);
    }

    #[test]
    fn test_validation_valid_modes() {
        for mode in 0..=3 {
            let result = AnalysisResult { render_mode: mode, ..Default::default() };
            assert!(validate_result(&result).is_ok());
        }
    }

    #[test]
    fn test_validation_invalid_mode() {
        let result = AnalysisResult { render_mode: 4, ..Default::default() };
        assert!(matches!(
            validate_result(&result),
            Err(AnalysisError::InvalidRenderMode(4))
        ));
    }
}
