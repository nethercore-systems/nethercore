//! EPU Debug Panel
//!
//! Provides an egui panel for inspecting and debugging the Environment Processing Unit (EPU).
//! The EPU handles environmental rendering effects like atmosphere, terrain bounds, and radiance.
//!
//! Toggle with ` (backtick/grave) when running the player.
//!
//! # Features
//!
//! - Opcode browser with categorization (Bounds vs Radiance)
//! - Semantic field editor with metadata-driven controls
//! - Layer-by-layer editing with per-opcode UI
//! - Variant/Domain selectors when applicable
//! - Color pickers for color_a and color_b
//! - Region mask checkboxes (Sky, Walls, Floor)
//! - Direction gizmo visualization for octahedral-encoded directions
//! - Preset save/load system for persisting configurations
//! - Layer isolation (solo/mute) for debugging individual layers
//! - Contribution preview showing what each layer adds

mod editor;
pub mod isolation;
pub mod presets;
pub mod visualization;

pub use editor::{EpuEditor, LayerEditState};
pub use isolation::{LayerCategory, LayerContribution, LayerIsolationState};
pub use presets::{EpuPreset, PresetManager, PresetUiState};

use crate::debug::epu_meta_gen::{self, FieldSpec, MapKind, OpcodeKind, OPCODES, OPCODE_COUNT};
use crate::graphics::epu::EpuConfig;

/// View mode for the debug panel
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelView {
    /// Browse opcodes and view metadata
    #[default]
    Browser,
    /// Edit a specific environment config
    Editor,
}

/// EPU Debug Panel state
///
/// Note: Clone creates a fresh panel (doesn't preserve state) because the panel
/// is typically cloned when creating a new console instance for a new game.
pub struct EpuDebugPanel {
    /// Whether the panel is visible
    pub visible: bool,
    /// Currently selected opcode for detailed view (None = overview)
    selected_opcode: Option<u8>,
    /// Current view mode
    view_mode: PanelView,
    /// Semantic editor for editing configs
    pub editor: EpuEditor,
    /// Environment ID being edited (when in Editor mode)
    editing_env_id: Option<u32>,
    /// Preset manager for save/load operations
    preset_manager: PresetManager,
    /// Preset UI state
    preset_ui_state: PresetUiState,
}

impl Default for EpuDebugPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EpuDebugPanel {
    fn clone(&self) -> Self {
        // Create a fresh panel - don't preserve state across clones
        // (cloning typically happens when creating a new console for a new game)
        Self::new()
    }
}

impl EpuDebugPanel {
    /// Create a new EPU debug panel
    pub fn new() -> Self {
        Self {
            visible: false,
            selected_opcode: None,
            view_mode: PanelView::Browser,
            editor: EpuEditor::new(),
            editing_env_id: None,
            preset_manager: PresetManager::default(),
            preset_ui_state: PresetUiState::default(),
        }
    }

    /// Toggle panel visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Set panel visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if panel is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Render the EPU debug panel
    ///
    /// Returns true if any value was changed (for future integration with live editing)
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        epu_configs: &hashbrown::HashMap<u32, EpuConfig>,
    ) -> bool {
        if !self.visible {
            return false;
        }

        let mut changed = false;

        egui::Window::new("EPU Debug Panel")
            .id(egui::Id::new("epu_debug_panel"))
            .default_pos([10.0, 300.0])
            .default_size([500.0, 600.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // Header with view mode tabs
                ui.horizontal(|ui| {
                    ui.heading("Environment Processing Unit");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .selectable_label(self.view_mode == PanelView::Editor, "Editor")
                            .clicked()
                        {
                            self.view_mode = PanelView::Editor;
                        }
                        if ui
                            .selectable_label(self.view_mode == PanelView::Browser, "Browser")
                            .clicked()
                        {
                            self.view_mode = PanelView::Browser;
                        }
                    });
                });

                ui.separator();

                // Summary stats
                ui.horizontal(|ui| {
                    ui.label(format!("Defined opcodes: {}", OPCODE_COUNT));
                    ui.separator();
                    ui.label(format!("Active configs: {}", epu_configs.len()));
                    if self.editor.dirty {
                        ui.separator();
                        ui.colored_label(egui::Color32::YELLOW, "* Modified");
                    }
                });

                ui.separator();

                match self.view_mode {
                    PanelView::Browser => {
                        // Two-column layout: opcode list on left, details on right
                        ui.columns(2, |columns| {
                            // Left column: Opcode list
                            columns[0].heading("Opcodes");
                            egui::ScrollArea::vertical()
                                .id_salt("epu_opcode_list")
                                .show(&mut columns[0], |ui| {
                                    self.render_opcode_list(ui);
                                });

                            // Right column: Selected opcode details or active configs
                            if let Some(opcode) = self.selected_opcode {
                                columns[1].heading("Opcode Details");
                                self.render_opcode_details(&mut columns[1], opcode);
                            } else {
                                columns[1].heading("Active Configs");
                                self.render_active_configs_with_edit(
                                    &mut columns[1],
                                    epu_configs,
                                );
                            }
                        });
                    }
                    PanelView::Editor => {
                        // Semantic editor view
                        changed |= self.render_editor_view(ui, epu_configs);
                    }
                }
            });

        changed
    }

    /// Render the semantic editor view
    fn render_editor_view(
        &mut self,
        ui: &mut egui::Ui,
        epu_configs: &hashbrown::HashMap<u32, EpuConfig>,
    ) -> bool {
        let mut changed = false;

        // Environment selector
        ui.horizontal(|ui| {
            ui.label("Environment:");

            let current_text = match self.editing_env_id {
                Some(id) => format!("Env {}", id),
                None => "New Config".to_string(),
            };

            egui::ComboBox::from_id_salt("env_selector")
                .selected_text(current_text)
                .show_ui(ui, |ui| {
                    // Option for new config
                    if ui
                        .selectable_value(&mut self.editing_env_id, None, "New Config")
                        .clicked()
                    {
                        self.editor = EpuEditor::new();
                    }

                    ui.separator();

                    // Existing configs
                    for &env_id in epu_configs.keys() {
                        if ui
                            .selectable_value(
                                &mut self.editing_env_id,
                                Some(env_id),
                                format!("Env {}", env_id),
                            )
                            .clicked()
                        {
                            if let Some(config) = epu_configs.get(&env_id) {
                                self.editor.load_config(config);
                            }
                        }
                    }
                });

            // Load button
            if let Some(env_id) = self.editing_env_id {
                if ui.button("Reload").clicked() {
                    if let Some(config) = epu_configs.get(&env_id) {
                        self.editor.load_config(config);
                    }
                }
            }

            // Reset button
            if ui.button("Reset").clicked() {
                self.editor = EpuEditor::new();
                self.editing_env_id = None;
            }
        });

        ui.separator();

        // Preset management section
        if let Some(loaded_layers) = presets::render_preset_ui(
            ui,
            &mut self.preset_manager,
            &mut self.preset_ui_state,
            &self.editor.layers,
        ) {
            self.editor.layers = loaded_layers;
            self.editor.dirty = true;
            changed = true;
        }

        ui.separator();

        // Render the semantic editor
        egui::ScrollArea::vertical()
            .id_salt("epu_editor_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                changed |= self.editor.render(ui);
            });

        changed
    }

    /// Render active configs with edit buttons
    fn render_active_configs_with_edit(
        &mut self,
        ui: &mut egui::Ui,
        configs: &hashbrown::HashMap<u32, EpuConfig>,
    ) {
        if configs.is_empty() {
            ui.label("No active EPU configs this frame");
            ui.label("");
            ui.label("EPU configs are created via epu_set() FFI calls.");
            ui.label("Use environment_index() to select which env_id to configure.");

            ui.separator();
            if ui.button("Create New in Editor").clicked() {
                self.view_mode = PanelView::Editor;
                self.editing_env_id = None;
                self.editor = EpuEditor::new();
            }
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("epu_active_configs")
            .show(ui, |ui| {
                for (env_id, config) in configs.iter() {
                    ui.horizontal(|ui| {
                        ui.collapsing(format!("Environment {}", env_id), |ui| {
                            self.render_config_summary(ui, config);
                        });

                        if ui.button("Edit").clicked() {
                            self.view_mode = PanelView::Editor;
                            self.editing_env_id = Some(*env_id);
                            self.editor.load_config(config);
                        }
                    });
                }
            });
    }

    /// Render the list of all defined opcodes
    fn render_opcode_list(&mut self, ui: &mut egui::Ui) {
        // Bounds opcodes
        ui.collapsing("Bounds", |ui| {
            for (code, info_opt) in OPCODES.iter().enumerate() {
                if let Some(info) = info_opt {
                    if info.kind == OpcodeKind::Bounds {
                        let selected = self.selected_opcode == Some(code as u8);
                        let label = format!("0x{:02X} {}", code, info.name);
                        if ui.selectable_label(selected, label).clicked() {
                            self.selected_opcode = if selected { None } else { Some(code as u8) };
                        }
                    }
                }
            }
        });

        // Radiance opcodes
        ui.collapsing("Radiance", |ui| {
            for (code, info_opt) in OPCODES.iter().enumerate() {
                if let Some(info) = info_opt {
                    if info.kind == OpcodeKind::Radiance {
                        let selected = self.selected_opcode == Some(code as u8);
                        let label = format!("0x{:02X} {}", code, info.name);
                        if ui.selectable_label(selected, label).clicked() {
                            self.selected_opcode = if selected { None } else { Some(code as u8) };
                        }
                    }
                }
            }
        });

        // Clear selection button
        if self.selected_opcode.is_some() {
            ui.separator();
            if ui.button("Clear Selection").clicked() {
                self.selected_opcode = None;
            }
        }
    }

    /// Render details for a selected opcode
    fn render_opcode_details(&self, ui: &mut egui::Ui, opcode: u8) {
        let info = match &OPCODES[opcode as usize] {
            Some(i) => i,
            None => {
                ui.label("Unknown opcode");
                return;
            }
        };

        // Opcode header
        ui.horizontal(|ui| {
            ui.strong(format!("0x{:02X}", opcode));
            ui.label(info.name);
            ui.label(format!(
                "({})",
                match info.kind {
                    OpcodeKind::Bounds => "Bounds",
                    OpcodeKind::Radiance => "Radiance",
                }
            ));
        });

        ui.separator();

        // Variants
        let variants = epu_meta_gen::VARIANTS[opcode as usize];
        if !variants.is_empty() {
            ui.collapsing(format!("Variants ({})", variants.len()), |ui| {
                for (i, name) in variants.iter().enumerate() {
                    ui.label(format!("  {}: {}", i, name));
                }
            });
        }

        // Domains
        let domains = epu_meta_gen::DOMAINS[opcode as usize];
        if !domains.is_empty() {
            ui.collapsing(format!("Domains ({})", domains.len()), |ui| {
                for (i, name) in domains.iter().enumerate() {
                    ui.label(format!("  {}: {}", i, name));
                }
            });
        }

        // Field specifications
        let fields = epu_meta_gen::field_specs(opcode);
        if !fields.is_empty() {
            ui.collapsing(format!("Fields ({})", fields.len()), |ui| {
                for field in fields {
                    self.render_field_spec(ui, field);
                }
            });
        }
    }

    /// Render a single field specification
    fn render_field_spec(&self, ui: &mut egui::Ui, field: &FieldSpec) {
        ui.horizontal(|ui| {
            ui.strong(field.label);
            if let Some(unit) = field.unit {
                ui.label(format!("({})", unit));
            }
        });

        ui.indent(field.name, |ui| {
            ui.label(format!("Raw: {}", field.name));
            let map_str: String = match field.map {
                MapKind::U8_01 => "u8 -> 0.0..1.0".to_string(),
                MapKind::U8Lerp => format!("u8 -> {:.2}..{:.2}", field.min, field.max),
                MapKind::U4_01 => "u4 -> 0.0..1.0".to_string(),
                MapKind::Dir16Oct => "16-bit octahedral direction".to_string(),
            };
            ui.label(format!("Mapping: {}", map_str));
        });
    }

    /// Render a summary of an EPU config
    fn render_config_summary(&self, ui: &mut egui::Ui, config: &EpuConfig) {
        // Count active layers (non-NOP)
        let mut bounds_count = 0;
        let mut radiance_count = 0;

        for (slot, layer) in config.layers.iter().enumerate() {
            let hi = layer[0];
            let opcode = ((hi >> 59) & 0x1F) as u8;

            if opcode != 0 {
                // NOP opcode is 0
                if opcode <= 0x07 {
                    bounds_count += 1;
                } else {
                    radiance_count += 1;
                }

                // Show layer info
                let opcode_name = epu_meta_gen::opcode_name(opcode);
                let region = ((hi >> 56) & 0x7) as u8;
                let blend = ((hi >> 53) & 0x7) as u8;

                ui.horizontal(|ui| {
                    ui.label(format!("Slot {}: ", slot));
                    ui.strong(format!("0x{:02X} {}", opcode, opcode_name));
                });

                ui.indent(format!("slot_{}_details", slot), |ui| {
                    ui.label(format!("Region: {:03b}", region));
                    ui.label(format!("Blend: {}", blend));
                });
            }
        }

        ui.separator();
        ui.label(format!(
            "Active layers: {} bounds, {} radiance",
            bounds_count, radiance_count
        ));

        // Note about full instruction dump (future feature)
        ui.separator();
        ui.small("Full instruction editing coming in future update");
    }
}
