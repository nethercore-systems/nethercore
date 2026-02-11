//! EPU Preset system for saving, loading, and exporting EPU configurations.
//!
//! Presets are stored in `~/.nethercore/epu_presets/` as JSON files.
//! Each preset contains the full state of all 8 EPU layers.

use super::editor::LayerEditState;
use crate::graphics::epu::EpuBlend;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

// =============================================================================
// Serializable Types
// =============================================================================

/// Serializable representation of blend mode (since EpuBlend doesn't derive Serialize)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    Add,
    Multiply,
    Max,
    Lerp,
    Screen,
    HsvMod,
    Min,
    Overlay,
}

impl From<EpuBlend> for BlendMode {
    fn from(blend: EpuBlend) -> Self {
        match blend {
            EpuBlend::Add => BlendMode::Add,
            EpuBlend::Multiply => BlendMode::Multiply,
            EpuBlend::Max => BlendMode::Max,
            EpuBlend::Lerp => BlendMode::Lerp,
            EpuBlend::Screen => BlendMode::Screen,
            EpuBlend::HsvMod => BlendMode::HsvMod,
            EpuBlend::Min => BlendMode::Min,
            EpuBlend::Overlay => BlendMode::Overlay,
        }
    }
}

impl From<BlendMode> for EpuBlend {
    fn from(mode: BlendMode) -> Self {
        match mode {
            BlendMode::Add => EpuBlend::Add,
            BlendMode::Multiply => EpuBlend::Multiply,
            BlendMode::Max => EpuBlend::Max,
            BlendMode::Lerp => EpuBlend::Lerp,
            BlendMode::Screen => EpuBlend::Screen,
            BlendMode::HsvMod => EpuBlend::HsvMod,
            BlendMode::Min => EpuBlend::Min,
            BlendMode::Overlay => EpuBlend::Overlay,
        }
    }
}

/// Serializable state for a single EPU layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerPresetState {
    /// Selected opcode index (0..=31)
    pub opcode: u8,
    /// Selected variant index (0..7)
    pub variant_id: u8,
    /// Selected domain index (0..3)
    pub domain_id: u8,
    /// Region mask (3-bit: SKY=4, WALLS=2, FLOOR=1)
    pub region_mask: u8,
    /// Blend mode
    pub blend: BlendMode,
    /// Primary RGB color
    pub color_a: [u8; 3],
    /// Secondary RGB color
    pub color_b: [u8; 3],
    /// Primary alpha (0-15)
    pub alpha_a: u8,
    /// Secondary alpha (0-15)
    pub alpha_b: u8,
    /// Intensity field (opcode-specific)
    pub intensity: u8,
    /// Parameter A
    pub param_a: u8,
    /// Parameter B
    pub param_b: u8,
    /// Parameter C
    pub param_c: u8,
    /// Parameter D
    pub param_d: u8,
    /// Direction (octahedral encoded u16)
    pub direction: u16,
}

impl From<&LayerEditState> for LayerPresetState {
    fn from(state: &LayerEditState) -> Self {
        Self {
            opcode: state.opcode,
            variant_id: state.variant_id,
            domain_id: state.domain_id,
            region_mask: state.region_mask,
            blend: state.blend.into(),
            color_a: state.color_a,
            color_b: state.color_b,
            alpha_a: state.alpha_a,
            alpha_b: state.alpha_b,
            intensity: state.intensity,
            param_a: state.param_a,
            param_b: state.param_b,
            param_c: state.param_c,
            param_d: state.param_d,
            direction: state.direction,
        }
    }
}

impl From<&LayerPresetState> for LayerEditState {
    fn from(preset: &LayerPresetState) -> Self {
        Self {
            opcode: preset.opcode,
            variant_id: preset.variant_id,
            domain_id: preset.domain_id,
            region_mask: preset.region_mask,
            blend: preset.blend.into(),
            color_a: preset.color_a,
            color_b: preset.color_b,
            alpha_a: preset.alpha_a,
            alpha_b: preset.alpha_b,
            intensity: preset.intensity,
            param_a: preset.param_a,
            param_b: preset.param_b,
            param_c: preset.param_c,
            param_d: preset.param_d,
            direction: preset.direction,
        }
    }
}

impl Default for LayerPresetState {
    fn default() -> Self {
        LayerEditState::default().into()
    }
}

impl From<LayerEditState> for LayerPresetState {
    fn from(state: LayerEditState) -> Self {
        (&state).into()
    }
}

impl From<LayerPresetState> for LayerEditState {
    fn from(preset: LayerPresetState) -> Self {
        (&preset).into()
    }
}

// =============================================================================
// Preset Types
// =============================================================================

/// An EPU preset containing saved layer configurations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpuPreset {
    /// Preset name (used as filename)
    pub name: String,
    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Saved layer states (all 8 layers)
    pub layers: [LayerPresetState; 8],
    /// Creation timestamp (seconds since UNIX epoch)
    #[serde(default = "default_timestamp")]
    pub created_at: u64,
    /// Format version for future compatibility
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn default_version() -> u32 {
    1
}

impl EpuPreset {
    /// Create a new preset from layer edit states.
    pub fn new(name: impl Into<String>, layers: &[LayerEditState; 8]) -> Self {
        Self {
            name: name.into(),
            description: None,
            layers: [
                (&layers[0]).into(),
                (&layers[1]).into(),
                (&layers[2]).into(),
                (&layers[3]).into(),
                (&layers[4]).into(),
                (&layers[5]).into(),
                (&layers[6]).into(),
                (&layers[7]).into(),
            ],
            created_at: default_timestamp(),
            version: 1,
        }
    }

    /// Create a preset with a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Convert preset layers back to edit states.
    pub fn to_layer_states(&self) -> [LayerEditState; 8] {
        [
            (&self.layers[0]).into(),
            (&self.layers[1]).into(),
            (&self.layers[2]).into(),
            (&self.layers[3]).into(),
            (&self.layers[4]).into(),
            (&self.layers[5]).into(),
            (&self.layers[6]).into(),
            (&self.layers[7]).into(),
        ]
    }

    /// Serialize the preset to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a preset from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// =============================================================================
// Preset Manager
// =============================================================================

/// Error type for preset operations.
#[derive(Debug, thiserror::Error)]
pub enum PresetError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Preset not found: {0}")]
    NotFound(String),
    #[error("Invalid preset name: {0}")]
    InvalidName(String),
    #[error("Failed to determine preset directory")]
    NoPresetDirectory,
}

/// Manages EPU presets on disk.
pub struct PresetManager {
    /// Directory where presets are stored
    presets_dir: PathBuf,
    /// Cached list of preset names (for quick listing)
    cached_presets: Vec<PresetInfo>,
    /// Whether the cache needs refreshing
    cache_dirty: bool,
}

/// Brief info about a preset (for listing without loading full data).
#[derive(Clone, Debug)]
pub struct PresetInfo {
    /// Preset name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
}

impl PresetManager {
    /// Create a new preset manager, initializing the presets directory if needed.
    pub fn new() -> Result<Self, PresetError> {
        let presets_dir = Self::get_presets_dir()?;

        // Create the directory if it doesn't exist
        if !presets_dir.exists() {
            fs::create_dir_all(&presets_dir)?;
        }

        let mut manager = Self {
            presets_dir,
            cached_presets: Vec::new(),
            cache_dirty: true,
        };

        // Load initial preset list
        manager.refresh_cache()?;

        Ok(manager)
    }

    /// Get the presets directory path.
    fn get_presets_dir() -> Result<PathBuf, PresetError> {
        // Try to get the project directories
        if let Some(proj_dirs) = ProjectDirs::from("systems", "nethercore", "nethercore") {
            let data_dir = proj_dirs.data_dir();
            return Ok(data_dir.join("epu_presets"));
        }

        // Fallback: use home directory
        if let Some(home) = dirs_fallback() {
            return Ok(home.join(".nethercore").join("epu_presets"));
        }

        Err(PresetError::NoPresetDirectory)
    }

    /// Get the file path for a preset by name.
    fn preset_path(&self, name: &str) -> PathBuf {
        let sanitized = sanitize_filename(name);
        self.presets_dir.join(format!("{}.json", sanitized))
    }

    /// Refresh the cached preset list from disk.
    pub fn refresh_cache(&mut self) -> Result<(), PresetError> {
        self.cached_presets.clear();

        if !self.presets_dir.exists() {
            self.cache_dirty = false;
            return Ok(());
        }

        let entries = fs::read_dir(&self.presets_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                // Try to load just the metadata
                match self.load_preset_info(&path) {
                    Ok(info) => self.cached_presets.push(info),
                    Err(e) => {
                        tracing::warn!("Failed to load preset {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by creation time (newest first)
        self.cached_presets
            .sort_by(|a, b| b.created_at.cmp(&a.created_at));

        self.cache_dirty = false;
        Ok(())
    }

    /// Load just the info from a preset file (faster than loading full preset).
    fn load_preset_info(&self, path: &PathBuf) -> Result<PresetInfo, PresetError> {
        let contents = fs::read_to_string(path)?;
        let preset: EpuPreset = serde_json::from_str(&contents)?;
        Ok(PresetInfo {
            name: preset.name,
            description: preset.description,
            created_at: preset.created_at,
        })
    }

    /// Get the list of available presets.
    pub fn list_presets(&mut self) -> Result<&[PresetInfo], PresetError> {
        if self.cache_dirty {
            self.refresh_cache()?;
        }
        Ok(&self.cached_presets)
    }

    /// Save a preset to disk.
    pub fn save_preset(&mut self, preset: &EpuPreset) -> Result<(), PresetError> {
        // Validate name
        if preset.name.is_empty() {
            return Err(PresetError::InvalidName(
                "Preset name cannot be empty".to_string(),
            ));
        }

        let path = self.preset_path(&preset.name);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Serialize and write
        let json = preset.to_json()?;
        fs::write(&path, json)?;

        // Mark cache as dirty
        self.cache_dirty = true;

        tracing::info!("Saved EPU preset: {}", preset.name);
        Ok(())
    }

    /// Load a preset from disk by name.
    pub fn load_preset(&self, name: &str) -> Result<EpuPreset, PresetError> {
        let path = self.preset_path(name);

        if !path.exists() {
            return Err(PresetError::NotFound(name.to_string()));
        }

        let contents = fs::read_to_string(&path)?;
        let preset = EpuPreset::from_json(&contents)?;

        tracing::info!("Loaded EPU preset: {}", name);
        Ok(preset)
    }

    /// Delete a preset from disk.
    pub fn delete_preset(&mut self, name: &str) -> Result<(), PresetError> {
        let path = self.preset_path(name);

        if !path.exists() {
            return Err(PresetError::NotFound(name.to_string()));
        }

        fs::remove_file(&path)?;
        self.cache_dirty = true;

        tracing::info!("Deleted EPU preset: {}", name);
        Ok(())
    }

    /// Check if a preset exists.
    pub fn preset_exists(&self, name: &str) -> bool {
        self.preset_path(name).exists()
    }

    /// Get the presets directory path (for display purposes).
    pub fn presets_directory(&self) -> &PathBuf {
        &self.presets_dir
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!("Failed to initialize PresetManager: {}", e);
            // Return a manager with a fallback path that may not work
            Self {
                presets_dir: PathBuf::from(".nethercore/epu_presets"),
                cached_presets: Vec::new(),
                cache_dirty: true,
            }
        })
    }
}

// =============================================================================
// UI State for Preset Management
// =============================================================================

/// UI state for the preset management interface.
#[derive(Default)]
pub struct PresetUiState {
    /// Name input for saving new presets
    pub save_name: String,
    /// Description input for saving new presets
    pub save_description: String,
    /// Currently selected preset name for loading
    pub selected_preset: Option<String>,
    /// Error message to display
    pub error_message: Option<String>,
    /// Success message to display
    pub success_message: Option<String>,
    /// Whether the save dialog is open
    pub show_save_dialog: bool,
    /// Whether the import dialog is open
    pub show_import_dialog: bool,
    /// Import text buffer (for clipboard import)
    pub import_buffer: String,
    /// Whether to show delete confirmation
    pub confirm_delete: Option<String>,
}

impl PresetUiState {
    /// Clear any displayed messages.
    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    /// Set an error message.
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
        self.success_message = None;
    }

    /// Set a success message.
    pub fn set_success(&mut self, msg: impl Into<String>) {
        self.success_message = Some(msg.into());
        self.error_message = None;
    }
}

// =============================================================================
// UI Rendering
// =============================================================================

/// Render the preset management UI section.
///
/// Returns `Some(layers)` if a preset was loaded that should update the editor.
pub fn render_preset_ui(
    ui: &mut egui::Ui,
    manager: &mut PresetManager,
    ui_state: &mut PresetUiState,
    current_layers: &[LayerEditState; 8],
) -> Option<[LayerEditState; 8]> {
    let mut loaded_layers = None;

    ui.collapsing("Presets", |ui| {
        // Display any messages
        if let Some(err) = &ui_state.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }
        if let Some(success) = &ui_state.success_message {
            ui.colored_label(egui::Color32::GREEN, success);
        }

        ui.horizontal(|ui| {
            // Save button
            if ui.button("Save Preset...").clicked() {
                ui_state.show_save_dialog = true;
                ui_state.clear_messages();
            }

            // Export to clipboard
            if ui.button("Export to Clipboard").clicked() {
                let preset = EpuPreset::new("exported", current_layers);
                match preset.to_json() {
                    Ok(json) => {
                        ui.ctx().copy_text(json);
                        ui_state.set_success("Preset copied to clipboard");
                    }
                    Err(e) => {
                        ui_state.set_error(format!("Export failed: {}", e));
                    }
                }
            }

            // Import from clipboard
            if ui.button("Import...").clicked() {
                ui_state.show_import_dialog = true;
                ui_state.import_buffer.clear();
                ui_state.clear_messages();
            }

            // Refresh list
            if ui.button("Refresh").clicked() {
                if let Err(e) = manager.refresh_cache() {
                    ui_state.set_error(format!("Refresh failed: {}", e));
                }
            }
        });

        ui.separator();

        // Preset list
        ui.label("Available Presets:");

        // Collect presets to avoid borrow issues when loading
        let presets_result: Result<Vec<_>, _> = manager.list_presets().map(|p| p.to_vec());

        match presets_result {
            Ok(presets) => {
                if presets.is_empty() {
                    ui.weak("No presets saved yet");
                } else {
                    // Track which preset to load (if any)
                    let mut preset_to_load: Option<String> = None;

                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .id_salt("preset_list")
                        .show(ui, |ui| {
                            for info in presets.iter() {
                                ui.horizontal(|ui| {
                                    let is_selected =
                                        ui_state.selected_preset.as_ref() == Some(&info.name);

                                    // Preset name/selection
                                    if ui.selectable_label(is_selected, &info.name).clicked() {
                                        ui_state.selected_preset = Some(info.name.clone());
                                        ui_state.clear_messages();
                                    }

                                    // Load button
                                    if ui.small_button("Load").clicked() {
                                        preset_to_load = Some(info.name.clone());
                                    }

                                    // Delete button
                                    if ui.small_button("X").clicked() {
                                        ui_state.confirm_delete = Some(info.name.clone());
                                    }
                                });

                                // Show description if present
                                if let Some(desc) = &info.description {
                                    ui.indent(format!("desc_{}", info.name), |ui| {
                                        ui.weak(desc);
                                    });
                                }
                            }
                        });

                    // Load the preset after the scroll area (outside the closure)
                    if let Some(name) = preset_to_load {
                        match manager.load_preset(&name) {
                            Ok(preset) => {
                                loaded_layers = Some(preset.to_layer_states());
                                ui_state.set_success(format!("Loaded: {}", name));
                            }
                            Err(e) => {
                                ui_state.set_error(format!("Load failed: {}", e));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                ui.colored_label(egui::Color32::RED, format!("Error listing presets: {}", e));
            }
        }
    });

    // Save dialog window
    if ui_state.show_save_dialog {
        let mut close_dialog = false;

        egui::Window::new("Save Preset")
            .id(egui::Id::new("save_preset_dialog"))
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut ui_state.save_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Description (optional):");
                });
                ui.text_edit_multiline(&mut ui_state.save_description);

                // Check for overwrite
                let will_overwrite =
                    !ui_state.save_name.is_empty() && manager.preset_exists(&ui_state.save_name);

                if will_overwrite {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!(
                            "Warning: '{}' already exists and will be overwritten",
                            ui_state.save_name
                        ),
                    );
                }

                ui.horizontal(|ui| {
                    let save_enabled = !ui_state.save_name.is_empty();

                    if ui
                        .add_enabled(save_enabled, egui::Button::new("Save"))
                        .clicked()
                    {
                        let mut preset = EpuPreset::new(&ui_state.save_name, current_layers);
                        if !ui_state.save_description.is_empty() {
                            preset = preset.with_description(&ui_state.save_description);
                        }

                        match manager.save_preset(&preset) {
                            Ok(()) => {
                                ui_state.set_success(format!("Saved: {}", ui_state.save_name));
                                ui_state.save_name.clear();
                                ui_state.save_description.clear();
                                close_dialog = true;
                            }
                            Err(e) => {
                                ui_state.set_error(format!("Save failed: {}", e));
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        close_dialog = true;
                    }
                });
            });

        if close_dialog {
            ui_state.show_save_dialog = false;
        }
    }

    // Import dialog window
    if ui_state.show_import_dialog {
        let mut close_dialog = false;

        egui::Window::new("Import Preset")
            .id(egui::Id::new("import_preset_dialog"))
            .collapsible(false)
            .resizable(true)
            .default_size([400.0, 300.0])
            .show(ui.ctx(), |ui| {
                ui.label("Paste preset JSON below:");

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut ui_state.import_buffer)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace),
                        );
                    });

                ui.horizontal(|ui| {
                    let import_enabled = !ui_state.import_buffer.is_empty();

                    if ui
                        .add_enabled(import_enabled, egui::Button::new("Import"))
                        .clicked()
                    {
                        match EpuPreset::from_json(&ui_state.import_buffer) {
                            Ok(preset) => {
                                loaded_layers = Some(preset.to_layer_states());
                                ui_state.set_success(format!("Imported: {}", preset.name));
                                close_dialog = true;
                            }
                            Err(e) => {
                                ui_state.set_error(format!("Import failed: {}", e));
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        close_dialog = true;
                    }
                });
            });

        if close_dialog {
            ui_state.show_import_dialog = false;
            ui_state.import_buffer.clear();
        }
    }

    // Delete confirmation dialog
    if let Some(name_to_delete) = ui_state.confirm_delete.clone() {
        let mut close_dialog = false;

        egui::Window::new("Confirm Delete")
            .id(egui::Id::new("delete_preset_dialog"))
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!("Delete preset '{}'?", name_to_delete));
                ui.label("This cannot be undone.");

                ui.horizontal(|ui| {
                    if ui
                        .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                        .clicked()
                    {
                        match manager.delete_preset(&name_to_delete) {
                            Ok(()) => {
                                ui_state.set_success(format!("Deleted: {}", name_to_delete));
                                if ui_state.selected_preset.as_ref() == Some(&name_to_delete) {
                                    ui_state.selected_preset = None;
                                }
                            }
                            Err(e) => {
                                ui_state.set_error(format!("Delete failed: {}", e));
                            }
                        }
                        close_dialog = true;
                    }

                    if ui.button("Cancel").clicked() {
                        close_dialog = true;
                    }
                });
            });

        if close_dialog {
            ui_state.confirm_delete = None;
        }
    }

    loaded_layers
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Sanitize a filename to be safe for the filesystem.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Fallback for getting home directory when ProjectDirs fails.
fn dirs_fallback() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_mode_conversion() {
        for blend in [
            EpuBlend::Add,
            EpuBlend::Multiply,
            EpuBlend::Max,
            EpuBlend::Lerp,
            EpuBlend::Screen,
            EpuBlend::HsvMod,
            EpuBlend::Min,
            EpuBlend::Overlay,
        ] {
            let mode: BlendMode = blend.into();
            let back: EpuBlend = mode.into();
            assert_eq!(blend, back);
        }
    }

    #[test]
    fn test_layer_state_conversion() {
        let original = LayerEditState {
            opcode: 8,
            variant_id: 2,
            domain_id: 1,
            region_mask: 0x07,
            blend: EpuBlend::Lerp,
            color_a: [255, 128, 64],
            color_b: [32, 16, 8],
            alpha_a: 12,
            alpha_b: 8,
            intensity: 200,
            param_a: 100,
            param_b: 150,
            param_c: 50,
            param_d: 75,
            direction: 0x4080,
        };

        let preset_state: LayerPresetState = (&original).into();
        let restored: LayerEditState = (&preset_state).into();

        assert_eq!(original.opcode, restored.opcode);
        assert_eq!(original.variant_id, restored.variant_id);
        assert_eq!(original.domain_id, restored.domain_id);
        assert_eq!(original.region_mask, restored.region_mask);
        assert_eq!(original.blend, restored.blend);
        assert_eq!(original.color_a, restored.color_a);
        assert_eq!(original.color_b, restored.color_b);
        assert_eq!(original.alpha_a, restored.alpha_a);
        assert_eq!(original.alpha_b, restored.alpha_b);
        assert_eq!(original.intensity, restored.intensity);
        assert_eq!(original.param_a, restored.param_a);
        assert_eq!(original.param_b, restored.param_b);
        assert_eq!(original.param_c, restored.param_c);
        assert_eq!(original.param_d, restored.param_d);
        assert_eq!(original.direction, restored.direction);
    }

    #[test]
    fn test_preset_json_roundtrip() {
        let layers: [LayerEditState; 8] = Default::default();
        let preset = EpuPreset::new("test_preset", &layers)
            .with_description("A test preset for unit testing");

        let json = preset.to_json().expect("serialization should succeed");
        let restored = EpuPreset::from_json(&json).expect("deserialization should succeed");

        assert_eq!(preset.name, restored.name);
        assert_eq!(preset.description, restored.description);
        assert_eq!(preset.version, restored.version);

        // Check layer data
        for i in 0..8 {
            assert_eq!(preset.layers[i].opcode, restored.layers[i].opcode);
            assert_eq!(preset.layers[i].blend, restored.layers[i].blend);
        }
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
        assert_eq!(sanitize_filename("with/slash"), "with_slash");
        assert_eq!(sanitize_filename("with:colon"), "with_colon");
        assert_eq!(sanitize_filename("with*star"), "with_star");
        assert_eq!(sanitize_filename("  trimmed  "), "trimmed");
    }
}
