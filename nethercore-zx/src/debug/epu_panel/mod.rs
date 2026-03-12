//! EPU Debug Panel
//!
//! Provides an egui panel for inspecting and debugging the Environment Processing Unit (EPU).
//! The EPU handles environmental rendering effects like atmosphere, terrain bounds, and feature layers.
//!
//! Toggle with ` (backtick/grave) when running the player.
//!
//! # Features
//!
//! - Opcode browser with categorization (Bounds vs Features)
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

use crate::debug::epu_capabilities;
use crate::debug::epu_meta_gen::{self, FieldSpec, MapKind, OPCODE_COUNT, OPCODES, OpcodeKind};
use crate::graphics::epu::EpuConfig;
use nethercore_core::workbench::{
    EpuWorkbenchConfig, EpuWorkbenchExportOptions, EpuWorkbenchExportResult, EpuWorkbenchMetadata,
    EpuWorkbenchViewState,
};

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
    /// Snapshot of game configs (for display when locked)
    pub snapshot_configs: hashbrown::HashMap<u32, EpuConfig>,
    /// Whether lock mode is active (debugger config replaces ALL game configs)
    pub locked: bool,
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
            snapshot_configs: hashbrown::HashMap::new(),
            locked: false,
        }
    }

    /// Update snapshot with current game configs
    pub fn update_snapshot(&mut self, game_configs: &hashbrown::HashMap<u32, EpuConfig>) {
        self.snapshot_configs.clone_from(game_configs);
    }

    /// Check if lock mode is active
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Get the debugger's override config (from editor state)
    pub fn get_override_config(&self) -> EpuConfig {
        self.editor.export_render_config()
    }

    pub fn export_workbench_config(&self) -> EpuWorkbenchConfig {
        self.editor.export_workbench_config()
    }

    pub fn load_snapshot_env(&mut self, env_id: u32) -> bool {
        if let Some(config) = self.snapshot_configs.get(&env_id) {
            self.editor.load_config(config);
            self.editing_env_id = Some(env_id);
            self.view_mode = PanelView::Editor;
            true
        } else {
            false
        }
    }

    pub fn load_workbench_config(&mut self, config: &EpuWorkbenchConfig) {
        self.editor.load_workbench_config(config);
        self.view_mode = PanelView::Editor;
        self.editing_env_id = Some(0);
    }

    pub fn workbench_view(&self) -> EpuWorkbenchViewState {
        EpuWorkbenchViewState {
            selected_layer: Some(self.editor.selected_layer),
            isolated_layer: self.editor.isolated_layer(),
            clear_layer_isolation: Some(false),
            locked: Some(self.locked),
            show_benchmarks: None,
            scene_index: None,
            show_ui: None,
            show_probe: None,
            show_background: None,
        }
    }

    pub fn set_workbench_view(&mut self, view: &EpuWorkbenchViewState) {
        if let Some(selected_layer) = view.selected_layer {
            self.editor.selected_layer = selected_layer.min(7);
        }
        if view.clear_layer_isolation.unwrap_or(false) {
            self.editor.isolation.show_all();
        } else if let Some(isolated_layer) = view.isolated_layer {
            self.editor.isolation.show_all();
            self.editor.isolation.toggle_solo(isolated_layer.min(7));
        }
        if let Some(locked) = view.locked {
            self.locked = locked;
        }
    }

    pub fn export_workbench(
        &self,
        options: &EpuWorkbenchExportOptions,
    ) -> EpuWorkbenchExportResult {
        let include_json = options.include_json_text || options.label.is_some();
        let include_rust = options.include_rust_text || options.rust_const_name.is_some();

        let json_text = include_json.then(|| {
            let mut preset = presets::EpuPreset::new(
                options
                    .label
                    .clone()
                    .unwrap_or_else(|| "epu-workbench".to_string()),
                &self.editor.layers,
            );
            if let Some(env_id) = self.editing_env_id {
                preset.description = Some(format!("Loaded from env {}", env_id));
            }
            preset.to_json().unwrap_or_else(|_| "{}".to_string())
        });

        let rust_text = include_rust.then(|| {
            let const_name = options
                .rust_const_name
                .clone()
                .unwrap_or_else(|| "EPU_LAYERS".to_string());
            format_rust_layers(&const_name, &self.editor.export_config())
        });

        EpuWorkbenchExportResult {
            json_path: None,
            rust_path: None,
            json_text,
            rust_text,
        }
    }

    pub fn workbench_metadata(&self) -> EpuWorkbenchMetadata {
        let opcode_names = (0..32u8)
            .map(|opcode| epu_meta_gen::opcode_name(opcode).to_string())
            .collect();
        EpuWorkbenchMetadata { opcode_names }
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
    pub fn render(&mut self, ctx: &egui::Context) -> bool {
        if !self.visible {
            return false;
        }

        let mut changed = false;

        // Capture config count before closure to avoid borrow conflicts
        let config_count = self.snapshot_configs.len();

        egui::Window::new("EPU Debug Panel")
            .id(egui::Id::new("epu_debug_panel"))
            .default_pos([10.0, 300.0])
            .default_size([500.0, 600.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // Lock mode banner (prominent when active)
                if self.locked {
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        let banner = egui::RichText::new(
                            "LOCKED - Game EPU disabled, using debugger config",
                        )
                        .color(egui::Color32::WHITE)
                        .strong();
                        ui.colored_label(egui::Color32::from_rgb(200, 80, 40), banner);
                    });
                    ui.add_space(2.0);
                }

                // Header with view mode tabs and lock toggle
                ui.horizontal(|ui| {
                    ui.heading("Environment Processing Unit");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Lock toggle (prominent)
                        let lock_text = if self.locked {
                            egui::RichText::new("LOCK")
                                .color(egui::Color32::from_rgb(255, 120, 60))
                                .strong()
                        } else {
                            egui::RichText::new("LOCK").color(egui::Color32::GRAY)
                        };
                        if ui.checkbox(&mut self.locked, lock_text).changed() {
                            changed = true;
                        }

                        ui.separator();

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
                    let config_label = if self.locked {
                        "Game configs (snapshot): ".to_string()
                    } else {
                        "Active configs: ".to_string()
                    };
                    ui.label(format!("{}{}", config_label, config_count));
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
                                self.render_active_configs_with_edit(&mut columns[1]);
                            }
                        });
                    }
                    PanelView::Editor => {
                        // Semantic editor view
                        changed |= self.render_editor_view(ui);
                    }
                }
            });

        changed
    }

    /// Render the semantic editor view
    fn render_editor_view(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // Environment selector (uses snapshot configs)
        ui.horizontal(|ui| {
            ui.label("Environment:");

            let current_text = match self.editing_env_id {
                Some(id) => format!("Env {}", id),
                None => "New Config".to_string(),
            };

            let env_ids: Vec<u32> = self.snapshot_configs.keys().copied().collect();
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

                    // Existing configs from snapshot
                    for env_id in env_ids {
                        if ui
                            .selectable_value(
                                &mut self.editing_env_id,
                                Some(env_id),
                                format!("Env {}", env_id),
                            )
                            .clicked()
                        {
                            if let Some(config) = self.snapshot_configs.get(&env_id) {
                                self.editor.load_config(config);
                            }
                        }
                    }
                });

            // Load button
            if let Some(env_id) = self.editing_env_id {
                if ui.button("Reload").clicked() {
                    if let Some(config) = self.snapshot_configs.get(&env_id) {
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

    /// Render active configs with edit buttons (uses snapshot_configs)
    fn render_active_configs_with_edit(&mut self, ui: &mut egui::Ui) {
        if self.snapshot_configs.is_empty() {
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

        // Collect config data to avoid borrow conflicts
        let config_entries: Vec<(u32, EpuConfig)> = self
            .snapshot_configs
            .iter()
            .map(|(&id, cfg)| (id, cfg.clone()))
            .collect();

        egui::ScrollArea::vertical()
            .id_salt("epu_active_configs")
            .show(ui, |ui| {
                for (env_id, config) in config_entries.iter() {
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

        // Feature opcodes
        ui.collapsing("Features", |ui| {
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
                    OpcodeKind::Radiance => "Feature",
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

        let base_report = epu_capabilities::base_report(opcode);
        let variants = epu_meta_gen::VARIANTS[opcode as usize];
        let domains = epu_meta_gen::DOMAINS[opcode as usize];
        let has_variant_notes = variants.iter().enumerate().any(|(variant_id, _)| {
            !epu_capabilities::variant_report(opcode, variant_id as u8).is_empty()
        });
        let has_domain_notes = domains.iter().enumerate().any(|(domain_id, _)| {
            !epu_capabilities::domain_report(opcode, domain_id as u8).is_empty()
        });

        if !base_report.is_empty() || has_variant_notes || has_domain_notes {
            ui.collapsing("Capability Guidance", |ui| {
                if !base_report.is_empty() {
                    epu_capabilities::render_report(ui, &base_report);
                }

                if has_variant_notes {
                    if !base_report.is_empty() {
                        ui.separator();
                    }
                    ui.small("Variant highlights");
                    for (variant_id, name) in variants.iter().enumerate() {
                        let report = epu_capabilities::variant_report(opcode, variant_id as u8);
                        if report.is_empty() {
                            continue;
                        }

                        ui.label(egui::RichText::new(*name).strong());
                        ui.indent(format!("variant_hint_{}_{}", opcode, variant_id), |ui| {
                            epu_capabilities::render_report(ui, &report);
                        });
                    }
                }

                if has_domain_notes {
                    if !base_report.is_empty() || has_variant_notes {
                        ui.separator();
                    }
                    ui.small("Domain highlights");
                    for (domain_id, name) in domains.iter().enumerate() {
                        let report = epu_capabilities::domain_report(opcode, domain_id as u8);
                        if report.is_empty() {
                            continue;
                        }

                        ui.label(egui::RichText::new(*name).strong());
                        ui.indent(format!("domain_hint_{}_{}", opcode, domain_id), |ui| {
                            epu_capabilities::render_report(ui, &report);
                        });
                    }
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
        let mut feature_count = 0;

        for (slot, layer) in config.layers.iter().enumerate() {
            let hi = layer[0];
            let opcode = ((hi >> 59) & 0x1F) as u8;

            if opcode != 0 {
                // NOP opcode is 0
                if opcode <= 0x07 {
                    bounds_count += 1;
                } else {
                    feature_count += 1;
                }

                // Show layer info
                let opcode_name = epu_meta_gen::opcode_name(opcode);
                let region = ((hi >> 56) & 0x7) as u8;
                let blend = ((hi >> 53) & 0x7) as u8;
                let meta_hi = ((hi >> 49) & 0xF) as u8;
                let meta_lo = ((hi >> 48) & 0x1) as u8;
                let meta5 = (meta_hi << 1) | meta_lo;
                let variant_id = meta5 & 0x07;
                let domain_id = (meta5 >> 3) & 0x03;
                let variant_name = epu_meta_gen::variant_name(opcode, variant_id);
                let domain_name = epu_meta_gen::domain_name(opcode, domain_id);
                let report = epu_capabilities::report_for(opcode, variant_id, domain_id);
                let warning_count = report.warning_count();

                ui.horizontal(|ui| {
                    ui.label(format!("Slot {}: ", slot));
                    ui.strong(format!("0x{:02X} {}", opcode, opcode_name));
                    if warning_count > 0 {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} warning{}",
                                warning_count,
                                if warning_count == 1 { "" } else { "s" }
                            ))
                            .small()
                            .color(egui::Color32::from_rgb(255, 140, 140)),
                        );
                    }
                });

                ui.indent(format!("slot_{}_details", slot), |ui| {
                    ui.label(format!("Region: {:03b}", region));
                    ui.label(format!("Blend: {}", blend));
                    if !variant_name.is_empty() {
                        ui.label(format!("Variant: {}", variant_name));
                    }
                    if !domain_name.is_empty() {
                        ui.label(format!("Domain: {}", domain_name));
                    }
                    if !report.is_empty() {
                        epu_capabilities::render_compact_report(ui, &report, 1, 2);
                    }
                });
            }
        }

        ui.separator();
        ui.label(format!(
            "Active layers: {} bounds, {} features",
            bounds_count, feature_count
        ));

        // Note about full instruction dump (future feature)
        ui.separator();
        ui.small("Full instruction editing coming in future update");
    }
}

fn format_rust_layers(const_name: &str, config: &EpuConfig) -> String {
    let mut output = String::new();
    output.push_str(&format!("const {}: [[u64; 2]; 8] = [\n", const_name));
    for layer in config.layers {
        output.push_str(&format!(
            "    [0x{:016X}, 0x{:016X}],\n",
            layer[0], layer[1]
        ));
    }
    output.push_str("];\n");
    output
}
