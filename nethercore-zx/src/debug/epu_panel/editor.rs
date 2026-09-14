//! Semantic EPU editor with metadata-driven UI.
//!
//! This module provides an editor UI that uses the generated metadata from
//! `epu_meta_gen` to display semantic labels, units, and appropriate controls
//! for each EPU opcode field.

use super::super::epu_capabilities;
use super::super::epu_meta_gen::{
    FieldSpec, MapKind, OPCODES, OpcodeKind, domain_count, domain_name, field_specs, opcode_kind,
    opcode_name, variant_count, variant_name,
};
use super::isolation::{
    LayerContribution, LayerIsolationState, render_color_swatch, render_contribution_preview,
    render_isolation_banner, render_layer_isolation_controls,
};
use super::visualization::DirectionGizmo;
use crate::graphics::epu::{
    EpuBlend, EpuConfig, EpuLayer, EpuOpcode, FlowPattern, REGION_ALL, REGION_FLOOR, REGION_SKY,
    REGION_WALLS, pack_meta5, pack_thresholds,
};
use nethercore_core::workbench::{
    EpuWorkbenchBlend, EpuWorkbenchConfig, EpuWorkbenchLayer, EpuWorkbenchLayerPatch,
};

/// State for editing a single EPU layer.
#[derive(Clone, Debug)]
pub struct LayerEditState {
    /// Selected opcode index (0..=31)
    pub opcode: u8,
    /// Selected variant index (0..7)
    pub variant_id: u8,
    /// Selected domain index (0..3)
    pub domain_id: u8,
    /// Region mask (3-bit: SKY=4, WALLS=2, FLOOR=1)
    pub region_mask: u8,
    /// Blend mode
    pub blend: EpuBlend,
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

impl Default for LayerEditState {
    fn default() -> Self {
        Self {
            opcode: 0,
            variant_id: 0,
            domain_id: 0,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            color_a: [255, 255, 255],
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: 128,
            param_a: 128,
            param_b: 128,
            param_c: 128,
            param_d: 128,
            direction: 0x8080, // Near +Z direction (quantized octahedral center)
        }
    }
}

impl LayerEditState {
    /// Create edit state from an EPU layer.
    pub fn from_layer(layer: &EpuLayer) -> Self {
        let meta5 = layer.meta5;
        let variant_id = meta5 & 0x07;
        let domain_id = (meta5 >> 3) & 0x03;

        Self {
            opcode: layer.opcode as u8,
            variant_id,
            domain_id,
            region_mask: layer.region_mask,
            blend: layer.blend,
            color_a: layer.color_a,
            color_b: layer.color_b,
            alpha_a: layer.alpha_a,
            alpha_b: layer.alpha_b,
            intensity: layer.intensity,
            param_a: layer.param_a,
            param_b: layer.param_b,
            param_c: layer.param_c,
            param_d: layer.param_d,
            direction: layer.direction,
        }
    }

    /// Convert edit state back to an EPU layer.
    pub fn to_layer(&self) -> EpuLayer {
        EpuLayer {
            opcode: epu_opcode_from_u8(self.opcode),
            region_mask: self.region_mask,
            blend: self.blend,
            meta5: pack_meta5(self.domain_id, self.variant_id),
            color_a: self.color_a,
            color_b: self.color_b,
            alpha_a: self.alpha_a,
            alpha_b: self.alpha_b,
            intensity: self.intensity,
            param_a: self.param_a,
            param_b: self.param_b,
            param_c: self.param_c,
            param_d: self.param_d,
            direction: self.direction,
        }
    }

    pub fn to_workbench_layer(&self) -> EpuWorkbenchLayer {
        EpuWorkbenchLayer {
            opcode: self.opcode,
            variant_id: self.variant_id,
            domain_id: self.domain_id,
            region_mask: self.region_mask,
            blend: self.blend.into(),
            color_a: self.color_a,
            color_b: self.color_b,
            alpha_a: self.alpha_a,
            alpha_b: self.alpha_b,
            intensity: self.intensity,
            param_a: self.param_a,
            param_b: self.param_b,
            param_c: self.param_c,
            param_d: self.param_d,
            direction: self.direction,
        }
    }

    pub fn apply_workbench_patch(&mut self, patch: &EpuWorkbenchLayerPatch) {
        if let Some(opcode) = patch.opcode {
            self.opcode = opcode;
        }
        if let Some(variant_id) = patch.variant_id {
            self.variant_id = variant_id;
        }
        if let Some(domain_id) = patch.domain_id {
            self.domain_id = domain_id;
        }
        if let Some(region_mask) = patch.region_mask {
            self.region_mask = region_mask;
        }
        if let Some(blend) = patch.blend {
            self.blend = blend.into();
        }
        if let Some(color_a) = patch.color_a {
            self.color_a = color_a;
        }
        if let Some(color_b) = patch.color_b {
            self.color_b = color_b;
        }
        if let Some(alpha_a) = patch.alpha_a {
            self.alpha_a = alpha_a.min(15);
        }
        if let Some(alpha_b) = patch.alpha_b {
            self.alpha_b = alpha_b.min(15);
        }
        if let Some(intensity) = patch.intensity {
            self.intensity = intensity;
        }
        if let Some(param_a) = patch.param_a {
            self.param_a = param_a;
        }
        if let Some(param_b) = patch.param_b {
            self.param_b = param_b;
        }
        if let Some(param_c) = patch.param_c {
            self.param_c = param_c;
        }
        if let Some(param_d) = patch.param_d {
            self.param_d = param_d;
        }
        if let Some(direction) = patch.direction {
            self.direction = direction;
        }
    }
}

impl From<&EpuWorkbenchLayer> for LayerEditState {
    fn from(layer: &EpuWorkbenchLayer) -> Self {
        Self {
            opcode: layer.opcode,
            variant_id: layer.variant_id,
            domain_id: layer.domain_id,
            region_mask: layer.region_mask,
            blend: layer.blend.into(),
            color_a: layer.color_a,
            color_b: layer.color_b,
            alpha_a: layer.alpha_a,
            alpha_b: layer.alpha_b,
            intensity: layer.intensity,
            param_a: layer.param_a,
            param_b: layer.param_b,
            param_c: layer.param_c,
            param_d: layer.param_d,
            direction: layer.direction,
        }
    }
}

impl From<EpuBlend> for EpuWorkbenchBlend {
    fn from(blend: EpuBlend) -> Self {
        match blend {
            EpuBlend::Add => EpuWorkbenchBlend::Add,
            EpuBlend::Multiply => EpuWorkbenchBlend::Multiply,
            EpuBlend::Max => EpuWorkbenchBlend::Max,
            EpuBlend::Lerp => EpuWorkbenchBlend::Lerp,
            EpuBlend::Screen => EpuWorkbenchBlend::Screen,
            EpuBlend::HsvMod => EpuWorkbenchBlend::HsvMod,
            EpuBlend::Min => EpuWorkbenchBlend::Min,
            EpuBlend::Overlay => EpuWorkbenchBlend::Overlay,
        }
    }
}

impl From<EpuWorkbenchBlend> for EpuBlend {
    fn from(blend: EpuWorkbenchBlend) -> Self {
        match blend {
            EpuWorkbenchBlend::Add => EpuBlend::Add,
            EpuWorkbenchBlend::Multiply => EpuBlend::Multiply,
            EpuWorkbenchBlend::Max => EpuBlend::Max,
            EpuWorkbenchBlend::Lerp => EpuBlend::Lerp,
            EpuWorkbenchBlend::Screen => EpuBlend::Screen,
            EpuWorkbenchBlend::HsvMod => EpuBlend::HsvMod,
            EpuWorkbenchBlend::Min => EpuBlend::Min,
            EpuWorkbenchBlend::Overlay => EpuBlend::Overlay,
        }
    }
}

/// Convert u8 to EpuOpcode (with bounds check).
fn epu_opcode_from_u8(code: u8) -> EpuOpcode {
    match code {
        0x00 => EpuOpcode::Nop,
        0x01 => EpuOpcode::Ramp,
        0x02 => EpuOpcode::Sector,
        0x03 => EpuOpcode::Silhouette,
        0x04 => EpuOpcode::Split,
        0x05 => EpuOpcode::Cell,
        0x06 => EpuOpcode::Patches,
        0x07 => EpuOpcode::Aperture,
        0x08 => EpuOpcode::Decal,
        0x09 => EpuOpcode::Grid,
        0x0A => EpuOpcode::Scatter,
        0x0B => EpuOpcode::Flow,
        0x0C => EpuOpcode::Trace,
        0x0D => EpuOpcode::Veil,
        0x0E => EpuOpcode::Atmosphere,
        0x0F => EpuOpcode::Plane,
        0x10 => EpuOpcode::Celestial,
        0x11 => EpuOpcode::Portal,
        0x12 => EpuOpcode::LobeRadiance,
        0x13 => EpuOpcode::BandRadiance,
        0x14 => EpuOpcode::Mottle,
        0x15 => EpuOpcode::Advect,
        0x16 => EpuOpcode::Surface,
        0x17 => EpuOpcode::Mass,
        0x18 => EpuOpcode::ScatterPhased,
        _ => EpuOpcode::Nop,
    }
}

/// EPU semantic editor state.
///
/// Provides metadata-driven UI for editing EPU configurations with semantic
/// labels, units, and appropriate controls for each field.
pub struct EpuEditor {
    /// Currently selected layer index (0..7)
    pub selected_layer: usize,
    /// Edit state for each of the 8 layers
    pub layers: [LayerEditState; 8],
    /// Whether the editor has unsaved changes
    pub dirty: bool,
    /// Layer isolation state (solo/mute)
    pub isolation: LayerIsolationState,
    /// Whether to show contribution previews in layer tabs
    pub show_contributions: bool,
}

impl Default for EpuEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl EpuEditor {
    /// Create a new EPU editor.
    pub fn new() -> Self {
        Self {
            selected_layer: 0,
            layers: Default::default(),
            dirty: false,
            isolation: LayerIsolationState::new(),
            show_contributions: true,
        }
    }

    /// Load an EpuConfig into the editor.
    pub fn load_config(&mut self, config: &EpuConfig) {
        for (i, packed) in config.layers.iter().enumerate() {
            self.layers[i] = LayerEditState::from_layer(&decode_packed_layer(*packed));
        }
        self.dirty = false;
    }

    /// Export the editor state to an EpuConfig.
    pub fn export_config(&self) -> EpuConfig {
        let mut config = EpuConfig::default();
        for (i, state) in self.layers.iter().enumerate() {
            config.layers[i] = state.to_layer().encode();
        }
        config
    }

    /// Export the current render config with solo/mute visibility applied.
    pub fn export_render_config(&self) -> EpuConfig {
        let mut config = self.export_config();
        let visibility_mask = self.layer_visibility_mask();
        for (index, layer) in config.layers.iter_mut().enumerate() {
            if visibility_mask & (1 << index) == 0 {
                *layer = [0, 0];
            }
        }
        config
    }

    pub fn export_workbench_config(&self) -> EpuWorkbenchConfig {
        EpuWorkbenchConfig {
            layers: [
                self.layers[0].to_workbench_layer(),
                self.layers[1].to_workbench_layer(),
                self.layers[2].to_workbench_layer(),
                self.layers[3].to_workbench_layer(),
                self.layers[4].to_workbench_layer(),
                self.layers[5].to_workbench_layer(),
                self.layers[6].to_workbench_layer(),
                self.layers[7].to_workbench_layer(),
            ],
        }
    }

    pub fn load_workbench_config(&mut self, config: &EpuWorkbenchConfig) {
        for (index, layer) in config.layers.iter().enumerate() {
            self.layers[index] = LayerEditState::from(layer);
        }
        self.dirty = true;
    }

    /// Render the full editor UI.
    ///
    /// Returns `true` if any value was changed.
    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // Isolation banner (if active)
        if render_isolation_banner(ui, &self.isolation) {
            self.isolation.show_all();
        }

        // Layer controls header
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_contributions, "Show Previews");

            ui.separator();

            // Visibility info
            let visible = self.isolation.visible_count();
            let total = 8;
            if visible < total {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("{}/{} layers visible", visible, total),
                );
            } else {
                ui.weak(format!("{} layers", total));
            }
        });

        ui.separator();

        // Layer selector tabs with isolation controls
        changed |= self.render_layer_tabs(ui);

        ui.separator();

        // Contribution preview for selected layer
        if self.show_contributions {
            let contribution = LayerContribution::from_layer(&self.layers[self.selected_layer]);
            ui.horizontal(|ui| {
                ui.label("Layer contribution:");
                render_contribution_preview(ui, &contribution, false);
            });
            ui.separator();
        }

        // Render the selected layer editor
        let layer = &mut self.layers[self.selected_layer];
        changed |= Self::render_layer_editor(ui, layer);

        if changed {
            self.dirty = true;
        }

        changed
    }

    /// Render layer selector tabs with isolation controls.
    fn render_layer_tabs(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // First row: layer tabs
        ui.horizontal(|ui| {
            for i in 0..8 {
                let opcode = self.layers[i].opcode;
                let is_selected = self.selected_layer == i;
                let is_visible = self.isolation.should_render_layer(i);

                // Build tab label
                let contribution = LayerContribution::from_layer(&self.layers[i]);
                let status = contribution.status_icon();
                let name = if opcode == 0 {
                    format!("{}: {}", i, status)
                } else {
                    format!("{}: {} {}", i, status, opcode_name(opcode))
                };

                // Style based on visibility
                let text_color = if !is_visible {
                    egui::Color32::DARK_GRAY
                } else if is_selected {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::LIGHT_GRAY
                };

                let label = egui::RichText::new(&name).color(text_color);

                if ui.selectable_label(is_selected, label).clicked() {
                    self.selected_layer = i;
                }
            }
        });

        // Second row: solo/mute controls
        ui.horizontal(|ui| {
            ui.label("S/M:");
            for i in 0..8 {
                let (solo_clicked, mute_clicked) =
                    render_layer_isolation_controls(ui, i, &self.isolation);

                if solo_clicked {
                    self.isolation.toggle_solo(i);
                    changed = true;
                }
                if mute_clicked {
                    self.isolation.toggle_mute(i);
                    changed = true;
                }

                // Add small spacing between layer groups
                if i < 7 {
                    ui.add_space(4.0);
                }
            }
        });

        // Third row: color swatches (if enabled)
        if self.show_contributions {
            ui.horizontal(|ui| {
                ui.label("    "); // Align with S/M label
                for i in 0..8 {
                    let layer = &self.layers[i];
                    let secondary = if layer.color_a != layer.color_b {
                        Some(layer.color_b)
                    } else {
                        None
                    };

                    // Dim swatch if layer not visible
                    let primary = if self.isolation.should_render_layer(i) {
                        layer.color_a
                    } else {
                        // Dim the color
                        [
                            layer.color_a[0] / 3,
                            layer.color_a[1] / 3,
                            layer.color_a[2] / 3,
                        ]
                    };

                    render_color_swatch(ui, primary, secondary, 20.0);

                    if i < 7 {
                        ui.add_space(24.0); // Match button spacing
                    }
                }
            });
        }

        changed
    }

    /// Get the layer visibility mask for GPU rendering.
    ///
    /// Returns a bitmask where bit N is set if layer N should be rendered.
    pub fn layer_visibility_mask(&self) -> u8 {
        self.isolation.visibility_mask()
    }

    /// Check if layer isolation is currently active.
    pub fn is_isolation_active(&self) -> bool {
        self.isolation.isolation_active
    }

    /// Get the currently isolated layer, if any.
    pub fn isolated_layer(&self) -> Option<usize> {
        self.isolation.isolated_layer
    }

    /// Render editor for a single layer.
    fn render_layer_editor(ui: &mut egui::Ui, layer: &mut LayerEditState) -> bool {
        let mut changed = false;

        // Opcode selector
        changed |= Self::render_opcode_selector(ui, layer);

        // Only show more controls if opcode is not NOP
        if layer.opcode == 0 {
            ui.label("Layer disabled (NOP)");
            return changed;
        }

        ui.separator();

        // Variant selector (if opcode has variants)
        changed |= Self::render_variant_selector(ui, layer);

        // Domain selector (if opcode has domains)
        changed |= Self::render_domain_selector(ui, layer);

        let capability_report =
            epu_capabilities::report_for(layer.opcode, layer.variant_id, layer.domain_id);
        if !capability_report.is_empty() {
            ui.group(|ui| {
                ui.label(egui::RichText::new("Capability guidance").strong());
                epu_capabilities::render_report(ui, &capability_report);
            });
            ui.separator();
        }

        // Common controls: region, blend, colors
        changed |= Self::render_common_controls(ui, layer);

        ui.separator();

        // Per-opcode field controls using metadata
        changed |= Self::render_field_controls(ui, layer);

        changed
    }

    /// Render the opcode dropdown selector.
    fn render_opcode_selector(ui: &mut egui::Ui, layer: &mut LayerEditState) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Opcode:");

            let current_name = if layer.opcode == 0 {
                "NOP"
            } else {
                opcode_name(layer.opcode)
            };

            let kind_label = match opcode_kind(layer.opcode) {
                Some(OpcodeKind::Bounds) => " [bounds]",
                Some(OpcodeKind::Radiance) => " [feature]",
                None => "",
            };

            egui::ComboBox::from_id_salt("opcode_selector")
                .selected_text(format!("{}{}", current_name, kind_label))
                .show_ui(ui, |ui| {
                    // NOP option
                    if ui.selectable_value(&mut layer.opcode, 0, "NOP").clicked() {
                        changed = true;
                    }

                    ui.separator();
                    ui.label("Bounds:");

                    // Bounds opcodes (0x01..=0x07)
                    for code in 1u8..=7 {
                        if let Some(info) = &OPCODES[code as usize]
                            && ui
                                .selectable_value(&mut layer.opcode, info.code, info.name)
                                .clicked()
                        {
                            changed = true;
                            // Reset variant/domain when changing opcode
                            layer.variant_id = 0;
                            layer.domain_id = 0;
                        }
                    }

                    ui.separator();
                    ui.label("Features:");

                    // Feature opcodes (0x08..=0x1F)
                    for code in 8u8..=0x1F {
                        if let Some(info) = &OPCODES[code as usize]
                            && ui
                                .selectable_value(&mut layer.opcode, info.code, info.name)
                                .clicked()
                        {
                            changed = true;
                            layer.variant_id = 0;
                            layer.domain_id = 0;
                        }
                    }
                });
        });

        changed
    }

    /// Render variant selector dropdown (if opcode has variants).
    fn render_variant_selector(ui: &mut egui::Ui, layer: &mut LayerEditState) -> bool {
        let count = variant_count(layer.opcode);
        if count == 0 {
            return false;
        }

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Variant:");

            let current = variant_name(layer.opcode, layer.variant_id);
            let current_text = if current.is_empty() {
                format!("{}", layer.variant_id)
            } else {
                current.to_string()
            };

            egui::ComboBox::from_id_salt("variant_selector")
                .selected_text(current_text)
                .show_ui(ui, |ui| {
                    for i in 0..count as u8 {
                        let name = variant_name(layer.opcode, i);
                        let label = if name.is_empty() {
                            format!("{}", i)
                        } else {
                            name.to_string()
                        };
                        if ui
                            .selectable_value(&mut layer.variant_id, i, label)
                            .clicked()
                        {
                            changed = true;
                        }
                    }
                });
        });

        changed
    }

    /// Render domain selector dropdown (if opcode has domains).
    fn render_domain_selector(ui: &mut egui::Ui, layer: &mut LayerEditState) -> bool {
        let count = domain_count(layer.opcode);
        if count == 0 {
            return false;
        }

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Domain:");

            let current = domain_name(layer.opcode, layer.domain_id);
            let current_text = if current.is_empty() {
                format!("{}", layer.domain_id)
            } else {
                current.to_string()
            };

            egui::ComboBox::from_id_salt("domain_selector")
                .selected_text(current_text)
                .show_ui(ui, |ui| {
                    for i in 0..count as u8 {
                        let name = domain_name(layer.opcode, i);
                        let label = if name.is_empty() {
                            format!("{}", i)
                        } else {
                            name.to_string()
                        };
                        if ui
                            .selectable_value(&mut layer.domain_id, i, label)
                            .clicked()
                        {
                            changed = true;
                        }
                    }
                });
        });

        changed
    }

    /// Render common controls (region, blend, colors, alphas).
    fn render_common_controls(ui: &mut egui::Ui, layer: &mut LayerEditState) -> bool {
        let mut changed = false;

        // Only feature dispatch consumes the stored region mask.
        ui.add_enabled_ui(layer.opcode >= EpuOpcode::Decal as u8, |ui| {
            ui.horizontal(|ui| {
                ui.label("Region:");

                let mut sky = (layer.region_mask & REGION_SKY) != 0;
                let mut walls = (layer.region_mask & REGION_WALLS) != 0;
                let mut floor = (layer.region_mask & REGION_FLOOR) != 0;

                if ui.checkbox(&mut sky, "Sky").changed() {
                    changed = true;
                    if sky {
                        layer.region_mask |= REGION_SKY;
                    } else {
                        layer.region_mask &= !REGION_SKY;
                    }
                }
                if ui.checkbox(&mut walls, "Walls").changed() {
                    changed = true;
                    if walls {
                        layer.region_mask |= REGION_WALLS;
                    } else {
                        layer.region_mask &= !REGION_WALLS;
                    }
                }
                if ui.checkbox(&mut floor, "Floor").changed() {
                    changed = true;
                    if floor {
                        layer.region_mask |= REGION_FLOOR;
                    } else {
                        layer.region_mask &= !REGION_FLOOR;
                    }
                }
            });
        }).response.on_hover_text(
            "Features use this region mask. Bounds opcodes write their own regions and paint; these stored bits are inactive.",
        );

        // Blend mode
        ui.horizontal(|ui| {
            ui.label("Blend:");

            let blend_name = match layer.blend {
                EpuBlend::Add => "Add",
                EpuBlend::Multiply => "Multiply",
                EpuBlend::Max => "Max",
                EpuBlend::Lerp => "Lerp",
                EpuBlend::Screen => "Screen",
                EpuBlend::HsvMod => "RGB Offset",
                EpuBlend::Min => "Min",
                EpuBlend::Overlay => "Overlay",
            };

            egui::ComboBox::from_id_salt("blend_selector")
                .selected_text(blend_name)
                .show_ui(ui, |ui| {
                    for (mode, name) in [
                        (EpuBlend::Add, "Add"),
                        (EpuBlend::Multiply, "Multiply"),
                        (EpuBlend::Max, "Max"),
                        (EpuBlend::Lerp, "Lerp"),
                        (EpuBlend::Screen, "Screen"),
                        (EpuBlend::HsvMod, "RGB Offset"),
                        (EpuBlend::Min, "Min"),
                        (EpuBlend::Overlay, "Overlay"),
                    ] {
                        if ui.selectable_value(&mut layer.blend, mode, name).clicked() {
                            changed = true;
                        }
                    }
                });
        });

        if layer.blend == EpuBlend::HsvMod {
            ui.label("RGB Offset: signed RGB adjustment, not HSV modulation.")
                .on_hover_text("Legacy HSV_MOD identifier (value 5; serialized as hsv_mod). Per component: clamp(dst + (src - 0.5) * clamp(alpha, 0, 1) * 2, 0, 1). Source 0.5 is neutral; lower values subtract and higher values add. Encoding and rendering behavior are unchanged.");
        }

        // Color A
        ui.add_enabled_ui(
            layer.opcode != EpuOpcode::Atmosphere as u8 || layer.variant_id <= 4,
            |ui| {
                ui.horizontal(|ui| {
                    ui.label("Color A:");
                    let mut color = egui::Color32::from_rgb(
                        layer.color_a[0],
                        layer.color_a[1],
                        layer.color_a[2],
                    );
                    if egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut color,
                        egui::color_picker::Alpha::Opaque,
                    )
                    .changed()
                    {
                        layer.color_a = [color.r(), color.g(), color.b()];
                        changed = true;
                    }

                    ui.label("Alpha:");
                    let mut alpha = layer.alpha_a as i32;
                    if ui
                        .add(egui::DragValue::new(&mut alpha).range(0..=15))
                        .changed()
                    {
                        layer.alpha_a = alpha.clamp(0, 15) as u8;
                        changed = true;
                    }
                });
            },
        );

        // Color B
        ui.add_enabled_ui(
            layer.opcode != EpuOpcode::Grid as u8
                && (layer.opcode != EpuOpcode::Atmosphere as u8 || matches!(layer.variant_id, 0 | 1 | 3 | 4))
                && (layer.opcode != EpuOpcode::Plane as u8 || matches!(layer.variant_id, 0 | 1 | 2 | 5 | 7))
                && (layer.opcode != EpuOpcode::Veil as u8 || layer.variant_id != 1), |ui| {
        ui.horizontal(|ui| {
            ui.label("Color B:");
            let mut color =
                egui::Color32::from_rgb(layer.color_b[0], layer.color_b[1], layer.color_b[2]);
            if egui::color_picker::color_edit_button_srgba(
                ui,
                &mut color,
                egui::color_picker::Alpha::Opaque,
            )
            .changed()
            {
                layer.color_b = [color.r(), color.g(), color.b()];
                changed = true;
            }

            let phased = layer.opcode == EpuOpcode::ScatterPhased as u8;
            let band = layer.opcode == EpuOpcode::BandRadiance as u8;
            ui.label(if phased || band { "Mod depth:" } else { "Alpha:" })
                .on_hover_text(if phased { "0 = steady; 15 = fully off/on. Color A alpha remains layer opacity." } else if band { "0 = plain ring; 15 = four-wave modulation (40..100%). Color A alpha remains layer opacity." } else { "Color B alpha / opcode-specific control" });
            let mut alpha = layer.alpha_b as i32;
            if ui
                .add_enabled(
                    ![
                        EpuOpcode::Ramp as u8, EpuOpcode::Grid as u8,
                        EpuOpcode::Sector as u8, EpuOpcode::Silhouette as u8,
                        EpuOpcode::Split as u8, EpuOpcode::Aperture as u8,
                        EpuOpcode::Scatter as u8, EpuOpcode::Flow as u8,
                        EpuOpcode::Atmosphere as u8, EpuOpcode::Plane as u8,
                        EpuOpcode::LobeRadiance as u8, EpuOpcode::Mottle as u8,
                        EpuOpcode::Advect as u8, EpuOpcode::Surface as u8,
                        EpuOpcode::Mass as u8,
                    ].contains(&layer.opcode),
                    egui::DragValue::new(&mut alpha).range(0..=15),
                )
                .on_disabled_hover_text("This opcode ignores alpha_b; its stored value is preserved. Color A alpha controls layer contribution.")
                .changed()
            {
                layer.alpha_b = alpha.clamp(0, 15) as u8;
                changed = true;
            }
        });
        });
        if layer.opcode == EpuOpcode::Grid as u8 {
            ui.weak("GRID uses Color A and a fixed Y-up cylindrical chart. Color B and direction are stored but inactive.");
        } else if layer.opcode == EpuOpcode::Ramp as u8 {
            ui.weak("RAMP ignores Color B alpha; stored value preserved.");
        } else if layer.opcode == EpuOpcode::LobeRadiance as u8 {
            ui.weak("Color B alpha is unused; stored nibble retained.");
        } else if layer.opcode == EpuOpcode::Plane as u8 {
            ui.weak("PLANE ignores Color B alpha; stored nibble retained.");
            match layer.variant_id {
                1 => {
                    ui.weak("HEX: Color A fills regular cells; gap exposes Color B.");
                }
                5 => {
                    ui.weak("GRATING: gap 0 = full bars; gap 0.2 = all Color B.");
                }
                _ => {}
            }
            if matches!(layer.variant_id, 3 | 4 | 6) {
                ui.weak("SAND, WATER and GRASS use Color A only; Color B is stored but inactive.");
            }
        } else if layer.opcode == EpuOpcode::Atmosphere as u8 {
            ui.weak("ATMOSPHERE ignores Color B alpha; stored nibble retained.");
            if layer.variant_id == 2 {
                ui.weak("MIE uses Color A only; Color B is stored but inactive.");
            }
            if !matches!(layer.variant_id, 2 | 3) {
                ui.weak("Sun direction is inactive for this variant; stored direction retained.");
            }
        }

        if layer.opcode == EpuOpcode::Veil as u8 && layer.variant_id == 1 {
            ui.weak("PILLARS has no glow: Color B and its alpha are stored but inactive.");
        }

        // Direction with visual gizmo
        ui.add_enabled_ui(
            layer.opcode != EpuOpcode::Grid as u8
                && (layer.opcode != EpuOpcode::Flow as u8 || layer.param_c & 0x0f <= 2)
                && (layer.opcode != EpuOpcode::Atmosphere as u8
                    || matches!(layer.variant_id, 2 | 3)),
            |ui| {
                ui.collapsing("Direction", |ui| {
                    // Raw hex value editor
                    ui.horizontal(|ui| {
                        ui.label("Raw (oct u16):");
                        let mut dir = layer.direction as i32;
                        if ui
                            .add(
                                egui::DragValue::new(&mut dir)
                                    .range(0..=65535)
                                    .hexadecimal(4, false, true),
                            )
                            .changed()
                        {
                            layer.direction = dir.clamp(0, 65535) as u16;
                            changed = true;
                        }
                    });

                    ui.separator();

                    // Visual direction gizmo
                    changed |= DirectionGizmo::new()
                        .with_size(120.0)
                        .with_axes(true)
                        .show(ui, &mut layer.direction);
                });
            },
        );

        changed
    }

    /// Render per-opcode field controls using metadata from FIELD_SPECS.
    fn render_field_controls(ui: &mut egui::Ui, layer: &mut LayerEditState) -> bool {
        let specs = field_specs(layer.opcode);
        if specs.is_empty() {
            ui.label("No field metadata for this opcode");
            return false;
        }

        let mut changed = false;

        ui.label("Parameters:");

        if layer.opcode == EpuOpcode::Portal as u8 {
            let extent = 0.05 + 0.75 * f64::from(layer.param_a) / 255.0;
            ui.weak(format!(
                "Base half extent: {:.2} degrees; shape proportions use this scale.",
                extent.atan().to_degrees()
            ));
        }

        if layer.opcode == EpuOpcode::Silhouette as u8 {
            ui.weak("Static silhouette: wall_depth is depth below the roofline, not animation speed.")
                .on_hover_text("Native SilhouetteParams retains historical drift_speed for param_d and drift_amount_q for reserved param_c low bits. Depth is measured in up-axis height space, not world distance. Reserved values survive load/export and idle; the advanced packed control edits the whole byte explicitly.");
        }

        if layer.opcode == EpuOpcode::Veil as u8 {
            let count = 2 + u32::from(layer.param_a) * 30 / 255;
            let max_width = (0.5 / count as f32).max(0.05);
            let width = 0.002 + (max_width - 0.002) * f32::from(layer.param_b) / 255.0;
            ui.weak(format!("Base thickness: {width:.6} chart units"));
            ui.weak(
                "Shape controls curtain sway, rain wind or shard tilt; only RAIN_WALL reads phase.",
            );
        }

        for (i, spec) in specs.iter().enumerate() {
            if layer.opcode == EpuOpcode::Ramp as u8 && spec.name == "param_d" {
                changed |= Self::render_ramp_thresholds_widget(ui, &mut layer.param_d);
            } else if layer.opcode == EpuOpcode::Scatter as u8
                || layer.opcode == EpuOpcode::ScatterPhased as u8
            {
                changed |= Self::render_scatter_field_widget(ui, spec, layer);
            } else if layer.opcode == EpuOpcode::Veil as u8 && spec.name == "param_a" {
                ui.horizontal(|ui| {
                    ui.label("Count");
                    let mut count = 2 + u32::from(layer.param_a) * 30 / 255;
                    if ui.add(egui::Slider::new(&mut count, 2..=32).show_value(false))
                        .on_hover_text("Integer base count = 2 + floor(raw * 30 / 255). Editing selects the lowest byte for that count; idle preserves the original byte.")
                        .changed() {
                        layer.param_a = ((count - 2) * 255).div_ceil(30) as u8;
                        changed = true;
                    }
                    ui.weak(format!("[{count} ribbons; raw {}]; RAIN_WALL doubles this count", layer.param_a));
                });
            } else if layer.opcode == EpuOpcode::Atmosphere as u8
                || layer.opcode == EpuOpcode::Plane as u8
                || layer.opcode == EpuOpcode::Veil as u8
            {
                let active = if layer.opcode == EpuOpcode::Plane as u8 {
                    match spec.name {
                        "intensity" | "param_a" => true,
                        "param_b" => matches!(layer.variant_id, 0 | 1 | 2 | 5 | 7),
                        "param_c" => matches!(layer.variant_id, 2 | 3 | 6 | 7),
                        "param_d" => layer.variant_id == 4,
                        _ => false,
                    }
                } else if layer.opcode == EpuOpcode::Veil as u8 {
                    match spec.name {
                        "param_c" => !matches!(layer.variant_id, 1 | 2),
                        "param_d" => layer.variant_id == 3,
                        _ => true,
                    }
                } else {
                    match spec.name {
                        "intensity" => layer.variant_id <= 4,
                        "param_a" | "param_b" => matches!(layer.variant_id, 0 | 1 | 3 | 4),
                        "param_c" | "param_d" => matches!(layer.variant_id, 2 | 3),
                        _ => false,
                    }
                };
                let raw = match spec.name {
                    "intensity" => &mut layer.intensity,
                    "param_a" => &mut layer.param_a,
                    "param_b" => &mut layer.param_b,
                    "param_c" => &mut layer.param_c,
                    "param_d" => &mut layer.param_d,
                    _ => continue,
                };
                ui.add_enabled_ui(active, |ui| {
                    changed |= Self::render_field_widget(ui, i, spec, raw);
                });
                if !active {
                    ui.weak("Inactive for this variant; stored byte retained.");
                }
            } else if layer.opcode == EpuOpcode::Celestial as u8
                && layer.variant_id == 4
                && matches!(spec.name, "param_c" | "param_d")
            {
                if spec.name == "param_c" {
                    ui.weak("Phase is unused by RINGED; stored byte retained.");
                } else {
                    let tilt = FieldSpec {
                        label: "Ring tilt",
                        unit: Some("deg"),
                        map: MapKind::U8Lerp,
                        min: 0.0,
                        max: 90.0,
                        ..*spec
                    };
                    changed |= Self::render_field_widget(ui, i, &tilt, &mut layer.param_d);
                    ui.weak("0 = edge-on; 90 = face-on. Inclination, not cyclic phase.");
                }
            } else if layer.opcode == EpuOpcode::Portal as u8
                && matches!(spec.name, "param_c" | "param_d")
            {
                if spec.name == "param_c" {
                    let active = matches!(layer.variant_id, 2..=5);
                    ui.add_enabled_ui(active, |ui| {
                        changed |= Self::render_field_widget(ui, i, spec, &mut layer.param_c);
                    });
                    if !active {
                        ui.weak("This shape ignores roughness; stored byte retained.");
                    }
                } else {
                    ui.add_enabled_ui(layer.variant_id == 3 && layer.param_c != 0, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Phase byte");
                            changed |= ui
                                .add(egui::Slider::new(&mut layer.param_d, 0..=255))
                                .changed();
                        });
                    });
                    ui.weak(format!(
                        "{}/256 = {:.6} cycles",
                        layer.param_d,
                        layer.param_d as f64 / 256.0
                    ));
                    if layer.variant_id != 3 {
                        ui.weak("VORTEX only: phase is inactive; stored byte retained.");
                    } else if layer.param_c == 0 {
                        ui.weak("Zero roughness: circular contour; phase is stored but inactive.");
                    }
                }
            } else if layer.opcode == EpuOpcode::LobeRadiance as u8
                && matches!(spec.name, "param_c" | "param_d")
            {
                if spec.name == "param_c" {
                    let label = match layer.param_c {
                        0 => "Steady (no modulation)",
                        1 => "Sine",
                        2 => "Triangle",
                        3 => "Strobe (4 pulses)",
                        _ => "Reserved (sine fallback)",
                    };
                    ui.horizontal(|ui| {
                        ui.label("Waveform");
                        egui::ComboBox::from_id_salt("lobe_waveform")
                            .selected_text(label)
                            .show_ui(ui, |ui| {
                                for (value, label) in [
                                    (0, "Steady (no modulation)"),
                                    (1, "Sine"),
                                    (2, "Triangle"),
                                    (3, "Strobe (4 pulses)"),
                                ] {
                                    changed |= ui
                                        .selectable_value(&mut layer.param_c, value, label)
                                        .changed();
                                }
                            });
                    });
                } else {
                    ui.add_enabled_ui(layer.param_c != 0, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Phase byte");
                            changed |= ui
                                .add(egui::Slider::new(&mut layer.param_d, 0..=255))
                                .changed();
                        });
                    });
                    ui.weak(format!(
                        "{}/256 = {:.6} cycles",
                        layer.param_d,
                        layer.param_d as f64 / 256.0
                    ));
                    if layer.param_c == 0 {
                        ui.weak("Steady: phase is inactive; stored byte retained.");
                    }
                }
            } else if layer.opcode == EpuOpcode::BandRadiance as u8 && spec.name == "param_d" {
                ui.add_enabled_ui(layer.alpha_b != 0, |ui| {
                    changed |= ui
                        .add(egui::Slider::new(&mut layer.param_d, 0..=255).text("Phase byte"))
                        .changed();
                });
                ui.weak(format!(
                    "{}/256 = {:.6} loops",
                    layer.param_d,
                    layer.param_d as f32 / 256.0
                ));
                if layer.alpha_b == 0 {
                    ui.weak("Zero depth: plain ring; phase is stored but inactive.");
                }
            } else if layer.opcode == EpuOpcode::Grid as u8 {
                changed |= Self::render_grid_field_widget(ui, spec, layer);
            } else if layer.opcode == EpuOpcode::Flow as u8 {
                changed |= Self::render_flow_field_widget(ui, spec, layer);
            } else {
                // Get the raw value reference based on field name
                let raw_value = match spec.name {
                    "intensity" => &mut layer.intensity,
                    "param_a" => &mut layer.param_a,
                    "param_b" => &mut layer.param_b,
                    "param_c" => &mut layer.param_c,
                    "param_d" => &mut layer.param_d,
                    _ => continue,
                };

                changed |= Self::render_field_widget(ui, i, spec, raw_value);
            }
        }

        changed
    }

    fn render_grid_field_widget(
        ui: &mut egui::Ui,
        spec: &FieldSpec,
        layer: &mut LayerEditState,
    ) -> bool {
        let mut changed = false;
        match spec.name {
            "param_a" => {
                let pattern = layer.param_c >> 4;
                let step = if pattern == 2 { 2 } else { 1 };
                let mut count = grid_repeats(layer.param_a, pattern);
                if ui.add(egui::Slider::new(&mut count, step..=64).step_by(f64::from(step)).text("Repeats"))
                    .on_hover_text("Whole cylindrical repeat count. Checker uses even counts so its colors close at the chart cut. Editing selects the closest stored byte; idle never normalizes it.")
                    .changed() {
                    layer.param_a = (0..=255).min_by_key(|&raw| (grid_repeats(raw, pattern).abs_diff(count), raw.abs_diff(layer.param_a))).unwrap();
                    count = grid_repeats(layer.param_a, pattern);
                    changed = true;
                }
                ui.weak(format!(
                    "{count} whole repeats; stored raw {}",
                    layer.param_a
                ));
            }
            "param_c" => {
                ui.horizontal(|ui| {
                    ui.label("Pattern");
                    let mut pattern = layer.param_c >> 4;
                    for (value, label) in [(0, "Stripes"), (1, "Grid"), (2, "Checker")] {
                        if ui.selectable_value(&mut pattern, value, label).changed() {
                            layer.param_c = (pattern << 4) | (layer.param_c & 15);
                            changed = true;
                        }
                    }
                });
                if layer.param_c >> 4 > 2 {
                    ui.weak(format!(
                        "Raw pattern {}: Stripes fallback; preserved until edited.",
                        layer.param_c >> 4
                    ));
                }
                let mut cycles = layer.param_c & 15;
                if ui.add(egui::Slider::new(&mut cycles, 0..=15).text("Cycles / loop"))
                    .on_hover_text("Whole pattern cycles per 256 guest phase steps. Checker has two cells per cycle. Zero is stationary; no host clock.")
                    .changed() {
                    layer.param_c = (layer.param_c & 0xf0) | cycles;
                    changed = true;
                }
            }
            "param_d" => {
                ui.add_enabled_ui(layer.param_c & 15 != 0, |ui| {
                    changed |= ui
                        .add(egui::Slider::new(&mut layer.param_d, 0..=255).text("Phase byte"))
                        .changed();
                });
                ui.weak(format!(
                    "{}/256 = {:.6} loops",
                    layer.param_d,
                    layer.param_d as f32 / 256.0
                ));
                if layer.param_c & 15 == 0 {
                    ui.weak("Zero cycles: phase is stored but does not move the pattern.");
                }
            }
            _ => {
                let enabled = spec.name != "param_b" || layer.param_c >> 4 != 2;
                let raw = match spec.name {
                    "intensity" => &mut layer.intensity,
                    "param_b" => &mut layer.param_b,
                    _ => return false,
                };
                ui.add_enabled_ui(enabled, |ui| {
                    changed |= Self::render_field_widget(ui, 0, spec, raw);
                });
                if !enabled {
                    ui.weak("Checker ignores thickness; stored value preserved.");
                }
            }
        }
        changed
    }

    fn render_flow_field_widget(
        ui: &mut egui::Ui,
        spec: &FieldSpec,
        layer: &mut LayerEditState,
    ) -> bool {
        let mut changed = false;
        match spec.name {
            "param_a" => {
                ui.horizontal(|ui| {
                    ui.label("Frequency");
                    let mut frequency = 1 + (layer.param_a as u32 * 15) / 255;
                    if ui.add(egui::Slider::new(&mut frequency, 1..=16).show_value(false))
                        .on_hover_text("Integer frequency = 1 + floor(raw * 15 / 255). Editing selects the lowest byte for that frequency; otherwise the original byte is preserved.")
                        .changed()
                    {
                        layer.param_a = ((frequency - 1) * 17) as u8;
                        changed = true;
                    }
                    ui.label(frequency.to_string());
                    ui.weak(format!("[1..16]; stored byte {}", layer.param_a));
                });
            }
            "param_c" => {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Pattern:");
                    let mut pattern = layer.param_c & 15;
                    for (value, name) in [
                        (FlowPattern::Noise, "Noise"),
                        (FlowPattern::Streaks, "Streaks"),
                        (FlowPattern::Caustic, "Caustic"),
                    ] {
                        if ui
                            .selectable_value(&mut pattern, value as u8, name)
                            .changed()
                        {
                            layer.param_c = (layer.param_c & 0xF0) | pattern;
                            changed = true;
                        }
                    }
                });
                let pattern = layer.param_c & 15;
                if pattern > 2 {
                    ui.weak(format!("Raw pattern {pattern}: static single-noise fallback. Preserved until a named pattern is selected."));
                }
                ui.add_enabled_ui(pattern <= 1, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Octaves");
                        let mut octaves = (layer.param_c >> 4).min(4);
                        if ui.add(egui::Slider::new(&mut octaves, 0..=4).show_value(false))
                            .on_hover_text("Noise octave count; Streaks uses this as a lane-variation seed. Zero is valid. The shader clamps high-nibble values 5..15 to 4; editing preserves the pattern nibble.")
                            .changed()
                        {
                            layer.param_c = (octaves << 4) | (layer.param_c & 15);
                            changed = true;
                        }
                        ui.label(octaves.to_string());
                        ui.weak(format!("[0..4]; stored high nibble {}", layer.param_c >> 4));
                    });
                });
                if pattern > 1 {
                    ui.weak("Octaves inactive for Caustic and raw fallback patterns; stored nibble preserved.");
                } else if pattern == 0 && layer.param_c >> 4 == 0 {
                    ui.weak("Noise with zero octaves has a constant pattern value of 0.5.");
                }
                ui.weak("Pattern is param_c low nibble, not meta5. FLOW ignores meta5 and Color B alpha.");
            }
            "param_d" => {
                ui.add_enabled_ui(layer.param_c & 15 <= 2, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Cyclic phase");
                        if ui.add(egui::Slider::new(&mut layer.param_d, 0..=255).show_value(false))
                            .on_hover_text("256 cyclic steps: raw / 256 turns, not raw / 255. The last step precedes wrap to zero; 1.0 is not a stored endpoint.")
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label(format!("{}/256 = {:.6} turns", layer.param_d, layer.param_d as f32 / 256.0));
                    });
                });
                if layer.param_c & 15 > 2 {
                    ui.weak("Phase and direction inactive for raw fallback patterns; stored values preserved.");
                }
            }
            _ => {
                let raw = match spec.name {
                    "intensity" => &mut layer.intensity,
                    "param_b" => &mut layer.param_b,
                    _ => return false,
                };
                changed |= Self::render_field_widget(ui, 0, spec, raw);
            }
        }
        changed
    }

    fn render_ramp_thresholds_widget(ui: &mut egui::Ui, raw_value: &mut u8) -> bool {
        let mut changed = false;
        for (label, ceiling) in [("Floor threshold", false), ("Ceiling threshold", true)] {
            ui.horizontal(|ui| {
                ui.label(label);
                let mut step = if ceiling { *raw_value >> 4 } else { *raw_value & 15 };
                if ui
                    .add(egui::Slider::new(&mut step, 0..=15).show_value(false))
                    .on_hover_text("Signed height along the authored up vector, in 16 steps. Exact zero is not representable. Editing preserves the other threshold.")
                    .changed()
                {
                    *raw_value = if ceiling {
                        pack_thresholds(step, *raw_value & 15)
                    } else {
                        pack_thresholds(*raw_value >> 4, step)
                    };
                    changed = true;
                }
                ui.label(format!("{:.3}", step as f32 / 15.0 * 2.0 - 1.0));
                ui.weak("[-1.000..1.000]");
            });
        }
        let floor = (*raw_value & 15) as f32 / 15.0 * 2.0 - 1.0;
        let ceiling = (*raw_value >> 4) as f32 / 15.0 * 2.0 - 1.0;
        if floor > ceiling {
            ui.weak(format!(
                "Renderer sorts reversed endpoints: effective floor {ceiling:.3}, ceiling {floor:.3}. Stored values unchanged."
            ));
        }
        changed
    }

    fn render_scatter_field_widget(
        ui: &mut egui::Ui,
        spec: &FieldSpec,
        layer: &mut LayerEditState,
    ) -> bool {
        match spec.name {
            "param_b" => Self::render_scatter_size_widget(
                ui,
                &mut layer.param_b,
                layer.param_a,
                layer.variant_id,
            ),
            "param_c" if layer.opcode == EpuOpcode::ScatterPhased as u8 => {
                Self::render_field_widget(ui, 0, spec, &mut layer.param_c)
            }
            "param_c" => {
                ui.add_enabled_ui(layer.variant_id != 2, |ui| {
                    Self::render_scatter_twinkle_widget(ui, &mut layer.param_c)
                })
                .inner
            } // WINDOWS does not use brightness variation.
            "param_d" => Self::render_scatter_seed_widget(ui, &mut layer.param_d),
            _ => {
                let raw_value = match spec.name {
                    "intensity" => &mut layer.intensity,
                    "param_a" => &mut layer.param_a,
                    _ => return false,
                };
                Self::render_field_widget(ui, 0, spec, raw_value)
            }
        }
    }

    fn render_scatter_size_widget(
        ui: &mut egui::Ui,
        raw_value: &mut u8,
        density_raw: u8,
        variant_id: u8,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("base radius (rad)");
            let mut value = *raw_value as i32;
            if ui
                .add(egui::Slider::new(&mut value, 0..=255).show_value(false))
                .changed()
            {
                *raw_value = value.clamp(0, 255) as u8;
                changed = true;
            }

            let display = scatter_size_display(density_raw, *raw_value, variant_id);
            ui.label(format!("{:.4}", display.base_radius));
            ui.weak(format!(
                "[{:.4}..{:.4}]",
                display.min_radius, display.max_radius
            ));
            if let Some((variant, multiplier)) = display.extended_falloff {
                ui.weak(format!(
                    "{} extended falloff: {:.4} rad ({:.0}x base)",
                    variant,
                    display.base_radius * multiplier,
                    multiplier
                ));
            }
        });
        changed
    }

    fn render_scatter_twinkle_widget(ui: &mut egui::Ui, raw_value: &mut u8) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("brightness variation (static)").on_hover_text(
                "Per-point variation, not an animation phase. Keep the seed fixed; \
                 a guest can animate the two color endpoints at different phases.",
            );
            let mut step = (*raw_value >> 4) as i32;
            if ui
                .add(egui::Slider::new(&mut step, 0..=15).show_value(false))
                .changed()
            {
                *raw_value = scatter_twinkle_raw_with_step(*raw_value, step as u8);
                changed = true;
            }
            ui.label(format!("{:.3}", scatter_twinkle_value(*raw_value)));
            ui.weak("[0.000..1.000]; low 4 bits reserved");
        });
        changed
    }

    fn render_scatter_seed_widget(ui: &mut egui::Ui, raw_value: &mut u8) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("seed (byte)");
            let mut value = *raw_value as i32;
            if ui
                .add(egui::Slider::new(&mut value, 0..=255).show_value(false))
                .changed()
            {
                *raw_value = value.clamp(0, 255) as u8;
                changed = true;
            }
            ui.label((*raw_value).to_string());
        });
        changed
    }

    /// Render a single field widget based on its FieldSpec.
    fn render_field_widget(
        ui: &mut egui::Ui,
        _field_index: usize,
        spec: &FieldSpec,
        raw_value: &mut u8,
    ) -> bool {
        let mut changed = false;

        // Skip fields with "-" label (unused)
        if spec.label == "-" {
            return false;
        }

        // Build the label with optional unit
        let label = if let Some(unit) = spec.unit {
            format!("{} ({})", spec.label, unit)
        } else {
            spec.label.to_string()
        };

        ui.horizontal(|ui| {
            ui.label(&label);

            // Calculate semantic value from raw u8 based on mapping type
            let semantic_value = map_u8_to_semantic(*raw_value, spec);

            match spec.map {
                MapKind::U8_01 | MapKind::U4_01 => {
                    // Normalized 0..1 value - show as percentage or raw
                    let mut val = *raw_value as i32;
                    if ui
                        .add(egui::Slider::new(&mut val, 0..=255).show_value(false))
                        .changed()
                    {
                        *raw_value = val.clamp(0, 255) as u8;
                        changed = true;
                    }
                    ui.label(format!("{:.2}", semantic_value));
                }
                MapKind::U8Lerp => {
                    // Linearly interpolated value with min/max
                    let mut val = *raw_value as i32;
                    if ui
                        .add(egui::Slider::new(&mut val, 0..=255).show_value(false))
                        .changed()
                    {
                        *raw_value = val.clamp(0, 255) as u8;
                        changed = true;
                    }

                    // Show semantic value with appropriate precision
                    let formatted = format_semantic_value(semantic_value, spec);
                    ui.label(formatted);
                }
                MapKind::Dir16Oct => {
                    // Direction encoding - just show raw for now
                    let mut val = *raw_value as i32;
                    if ui
                        .add(egui::DragValue::new(&mut val).range(0..=255))
                        .changed()
                    {
                        *raw_value = val.clamp(0, 255) as u8;
                        changed = true;
                    }
                }
            }

            // Show range hint
            if spec.map == MapKind::U8Lerp {
                let hint = if spec.unit == Some("turns") {
                    format!("[{:.3}..{:.3}]", spec.min, spec.max)
                } else {
                    format!("[{:.2}..{:.2}]", spec.min, spec.max)
                };
                ui.weak(hint);
            }
        });

        changed
    }
}

fn grid_repeats(raw: u8, pattern: u8) -> u8 {
    let numerator = 63 * u32::from(raw);
    if pattern == 2 {
        (2 * ((510 + numerator) / 510)) as u8
    } else {
        ((382 + numerator) / 255) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScatterSizeDisplay {
    base_radius: f32,
    min_radius: f32,
    max_radius: f32,
    extended_falloff: Option<(&'static str, f32)>,
}

fn scatter_size_display(density_raw: u8, size_raw: u8, variant_id: u8) -> ScatterSizeDisplay {
    let density = 1.0 + density_raw as f32;
    let multiplier = match variant_id {
        1 => 2.0, // DUST
        2 => 1.5, // WINDOWS
        3 => 2.5, // BUBBLES
        4 => 1.2, // EMBERS
        5 => 0.3, // RAIN
        6 => 1.8, // SNOW
        _ => 1.0, // STARS
    };
    let max_base_radius = (0.5 / density).max(0.05);
    let t = size_raw as f32 / 255.0;
    let base_min = 0.001 * multiplier;
    let base_max = max_base_radius * multiplier;
    let base_radius = (0.001 + t * (max_base_radius - 0.001)) * multiplier;
    let extended_falloff = match variant_id {
        4 => Some(("EMBERS", 2.0)), // EMBERS glow reaches size * 2.
        5 => Some(("RAIN", 3.0)),   // RAIN streak reaches size * 3.
        _ => None,
    };

    ScatterSizeDisplay {
        base_radius,
        min_radius: base_min,
        max_radius: base_max,
        extended_falloff,
    }
}

fn scatter_twinkle_value(raw: u8) -> f32 {
    (raw >> 4) as f32 / 15.0
}

fn scatter_twinkle_raw_with_step(raw: u8, step: u8) -> u8 {
    (raw & 0x0F) | ((step.min(15) & 0x0F) << 4)
}

/// Map a u8 raw value to its semantic float value based on the FieldSpec.
fn map_u8_to_semantic(raw: u8, spec: &FieldSpec) -> f32 {
    match spec.map {
        MapKind::U8_01 => raw as f32 / 255.0,
        MapKind::U4_01 => (raw & 0x0F) as f32 / 15.0,
        MapKind::U8Lerp => {
            let t = raw as f32 / 255.0;
            spec.min + t * (spec.max - spec.min)
        }
        MapKind::Dir16Oct => raw as f32, // No semantic mapping for direction
    }
}

/// Map a semantic float value back to u8 raw value based on the FieldSpec.
#[allow(dead_code)]
fn map_semantic_to_u8(semantic: f32, spec: &FieldSpec) -> u8 {
    match spec.map {
        MapKind::U8_01 => (semantic.clamp(0.0, 1.0) * 255.0).round() as u8,
        MapKind::U4_01 => (semantic.clamp(0.0, 1.0) * 15.0).round() as u8,
        MapKind::U8Lerp => {
            if (spec.max - spec.min).abs() < 0.0001 {
                0
            } else {
                let t = (semantic - spec.min) / (spec.max - spec.min);
                (t.clamp(0.0, 1.0) * 255.0).round() as u8
            }
        }
        MapKind::Dir16Oct => semantic.clamp(0.0, 255.0).round() as u8,
    }
}

/// Format a semantic value for display with appropriate precision.
fn format_semantic_value(value: f32, spec: &FieldSpec) -> String {
    let range = spec.max - spec.min;

    // Choose precision based on range
    if range < 1.0 {
        format!("{:.3}", value)
    } else if range < 10.0 {
        format!("{:.2}", value)
    } else if range < 100.0 {
        format!("{:.1}", value)
    } else {
        format!("{:.0}", value)
    }
}

/// Decode a packed [u64; 2] layer back to EpuLayer.
fn decode_packed_layer(packed: [u64; 2]) -> EpuLayer {
    let [hi, lo] = packed;

    // Extract from hi word
    let opcode_raw = ((hi >> 59) & 0x1F) as u8;
    let region_mask = ((hi >> 56) & 0x7) as u8;
    let blend_raw = ((hi >> 53) & 0x7) as u8;
    let meta_hi = ((hi >> 49) & 0xF) as u8;
    let meta_lo = ((hi >> 48) & 0x1) as u8;
    let meta5 = (meta_hi << 1) | meta_lo;

    let color_a_packed = (hi >> 24) & 0xFF_FFFF;
    let color_b_packed = hi & 0xFF_FFFF;

    let color_a = [
        ((color_a_packed >> 16) & 0xFF) as u8,
        ((color_a_packed >> 8) & 0xFF) as u8,
        (color_a_packed & 0xFF) as u8,
    ];
    let color_b = [
        ((color_b_packed >> 16) & 0xFF) as u8,
        ((color_b_packed >> 8) & 0xFF) as u8,
        (color_b_packed & 0xFF) as u8,
    ];

    // Extract from lo word
    let intensity = ((lo >> 56) & 0xFF) as u8;
    let param_a = ((lo >> 48) & 0xFF) as u8;
    let param_b = ((lo >> 40) & 0xFF) as u8;
    let param_c = ((lo >> 32) & 0xFF) as u8;
    let param_d = ((lo >> 24) & 0xFF) as u8;
    let direction = ((lo >> 8) & 0xFFFF) as u16;
    let alpha_a = ((lo >> 4) & 0xF) as u8;
    let alpha_b = (lo & 0xF) as u8;

    let blend = match blend_raw {
        0 => EpuBlend::Add,
        1 => EpuBlend::Multiply,
        2 => EpuBlend::Max,
        3 => EpuBlend::Lerp,
        4 => EpuBlend::Screen,
        5 => EpuBlend::HsvMod,
        6 => EpuBlend::Min,
        7 => EpuBlend::Overlay,
        _ => EpuBlend::Add,
    };

    EpuLayer {
        opcode: epu_opcode_from_u8(opcode_raw),
        region_mask,
        blend,
        meta5,
        color_a,
        color_b,
        alpha_a,
        alpha_b,
        intensity,
        param_a,
        param_b,
        param_c,
        param_d,
        direction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_authoring_nominal_width_display_and_roundtrip() {
        let specs = field_specs(EpuOpcode::Split as u8);
        let width = specs.iter().find(|s| s.name == "param_a").unwrap();
        // Nominal width, not the evaluator's 0.001 AA floor.
        for raw in [0, 1, 127, 255] {
            let nominal = raw as f32 / 255.0 * 0.2;
            assert!(
                (map_u8_to_semantic(raw, width) - nominal).abs() < 1e-7,
                "raw={raw}"
            );
        }
        for (raw, display) in [(0, "0.000"), (1, "0.001"), (127, "0.100"), (255, "0.200")] {
            let nominal = raw as f32 / 255.0 * 0.2;
            assert!((map_u8_to_semantic(raw, width) - nominal).abs() < 1e-7);
            assert_eq!(format_semantic_value(nominal, width), display);
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: EpuOpcode::Split as u8,
                param_a: raw,
                ..Default::default()
            };
            let before = layer.to_layer().encode();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            assert_eq!(layer.to_layer().encode(), before);
            flow_label_rect(&output, display);
            flow_label_rect(&output, "[0.00..0.20]");
        }
        for raw in 0..=255u8 {
            let nominal = map_u8_to_semantic(raw, width);
            assert!((nominal - raw as f32 / 255.0 * 0.2).abs() < 1e-7);
            assert_eq!(map_semantic_to_u8(nominal, width), raw);
        }
    }

    #[test]
    fn split_authoring_preserves_packing_and_exports() {
        use crate::graphics::epu::{SplitParams, epu_begin, epu_finish};
        let mut builder = epu_begin();
        builder.split_bounds(SplitParams {
            sky_color: [0x12, 0x34, 0x56],
            wall_color: [0xA1, 0xB2, 0xC3],
            blend_width: 0xD4,
            wedge_angle: 0xE5,
            count: 0xF6,
            offset: 0x28,
            variant_id: 6,
            ..Default::default()
        });
        let original = epu_finish(builder);
        assert_eq!(original.layers[0], [0x2706123456A1B2C3, 0x00D4E5F628FF80FF]);
        let mut editor = EpuEditor::new();
        editor.load_config(&original);
        assert_eq!(editor.export_config(), original);
        let workbench = editor.export_workbench_config();
        editor.load_workbench_config(&workbench);
        assert_eq!(editor.export_config(), original);
        let rust = super::super::format_rust_layers("SPLIT", &editor.export_config());
        assert!(rust.contains("0x2706123456A1B2C3"));
        assert!(rust.contains("0x00D4E5F628FF80FF"));
        assert!(flow_input(&mut editor.layers[0], "blend_width", Some(1.0)));
        let mut expected = original;
        expected.layers[0][1] = 0x00FFE5F628FF80FF;
        assert_eq!(editor.export_config(), expected);
    }

    #[test]
    fn silhouette_authoring_metadata_describes_packed_octaves() {
        let specs = field_specs(EpuOpcode::Silhouette as u8);
        let octaves = specs.iter().find(|s| s.name == "param_c").unwrap();
        assert_eq!(
            (octaves.map, octaves.min, octaves.max),
            (MapKind::U8Lerp, 0.0, 255.0)
        );
        assert!(octaves.label.contains("reserved"));
        assert!(octaves.label.contains("MOUNTAINS"));
        let depth = specs.iter().find(|s| s.name == "param_d").unwrap();
        assert_eq!(
            (depth.label, depth.min, depth.max),
            ("wall_depth", 0.05, 1.2)
        );
    }

    #[test]
    fn silhouette_authoring_preserves_bytes_and_wall_depth_only_edit() {
        use crate::graphics::epu::{SilhouetteParams, epu_begin, epu_finish};
        let mut params = SilhouetteParams {
            silhouette_color: [0x12, 0x34, 0x56],
            background_color: [0xA1, 0xB2, 0xC3],
            edge_softness: 0xD4,
            horizon_bias: 0xE5,
            roughness: 0xF6,
            octaves_q: 0xB,
            drift_amount_q: 0xD,
            drift_speed: 0x28,
            strength: 9,
            variant_id: 6,
            ..Default::default()
        };
        let mut builder = epu_begin();
        builder.silhouette_bounds(params);
        let original = epu_finish(builder);
        assert_eq!(original.layers[0], [0x1F06123456A1B2C3, 0xD4E5F6BD28FF8090]);
        let mut editor = EpuEditor::new();
        editor.load_config(&original);
        assert_eq!(editor.export_config(), original);
        let workbench = editor.export_workbench_config();
        editor.load_workbench_config(&workbench);
        assert_eq!(editor.export_config(), original);
        assert!(flow_input(&mut editor.layers[0], "wall_depth", Some(1.0)));
        let mut expected = original;
        expected.layers[0][1] = 0xD4E5F6BDFFFF8090;
        assert_eq!(editor.export_config(), expected);
        params.drift_speed = 255; // Historical field name: wall depth, not animation.
        let mut builder = epu_begin();
        builder.silhouette_bounds(params);
        assert_eq!(epu_finish(builder), expected);
    }

    fn flow_label_rect(output: &egui::FullOutput, label: &str) -> egui::Rect {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == label => {
                    Some(egui::Rect::from_min_size(text.pos, text.galley.size()))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing native FLOW control/display: {label}"))
    }

    fn flow_input(layer: &mut LayerEditState, label: &str, slider_fraction: Option<f32>) -> bool {
        let ctx = egui::Context::default();
        let before = layer.to_layer().encode();
        ramp_frame(&ctx, layer, vec![]);
        let (output, changed) = ramp_frame(&ctx, layer, vec![]);
        assert!(!changed);
        assert_eq!(
            layer.to_layer().encode(),
            before,
            "idle must preserve raw encoding"
        );
        let rect = flow_label_rect(&output, label);
        let pos = match slider_fraction {
            Some(t) => {
                // Use the painted rail/handle, not the text label's ink bounds:
                // short labels and dense integer sliders expose that offset.
                let rail = output
                    .shapes
                    .iter()
                    .find_map(|s| match &s.shape {
                        egui::epaint::Shape::Rect(r)
                            if r.rect.left() >= rect.right()
                                && (r.rect.center().y - rect.center().y).abs() < 2.0
                                && (r.rect.width() - ctx.style().spacing.slider_width).abs()
                                    < 0.5 =>
                        {
                            Some(r.rect)
                        }
                        _ => None,
                    })
                    .expect("painted slider rail");
                let radius = output
                    .shapes
                    .iter()
                    .find_map(|s| match &s.shape {
                        egui::epaint::Shape::Circle(c)
                            if rail.x_range().contains(c.center.x)
                                && (c.center.y - rail.center().y).abs() < 0.5 =>
                        {
                            Some(c.radius)
                        }
                        egui::epaint::Shape::Rect(r)
                            if rail.x_range().contains(r.rect.center().x)
                                && (r.rect.center().y - rail.center().y).abs() < 0.5
                                && r.rect.width() < rail.width() * 0.5
                                && r.rect.height() > rail.height() =>
                        {
                            Some(r.rect.width() * 0.5)
                        }
                        _ => None,
                    })
                    .expect("painted slider handle");
                egui::pos2(
                    rail.left()
                        + if t == 0.0 {
                            1.0
                        } else if t == 1.0 {
                            rail.width() - 1.0
                        } else {
                            radius + (rail.width() - 2.0 * radius) * t
                        },
                    rail.center().y,
                )
            }
            None => rect.center(),
        };
        let mut edited = false;
        for pressed in [true, false] {
            edited |= ramp_frame(
                &ctx,
                layer,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            )
            .1;
        }
        edited
    }

    #[test]
    fn bounds_region_controls_preserve_bits_features_remain_editable() {
        let mut cases = 0;
        for opcode in 1..=EpuOpcode::ScatterPhased as u8 {
            for (label, bit) in [
                ("Sky", REGION_SKY),
                ("Walls", REGION_WALLS),
                ("Floor", REGION_FLOOR),
            ] {
                let mut layer = LayerEditState {
                    opcode,
                    region_mask: 5,
                    ..Default::default()
                };
                let mut expected = layer.to_layer();
                let changed = flow_input(&mut layer, label, None);
                let active = opcode >= EpuOpcode::Decal as u8;
                assert_eq!(
                    changed, active,
                    "region checkbox opcode={opcode} label={label}"
                );
                if active {
                    expected.region_mask ^= bit;
                }
                assert_eq!(layer.to_layer().encode(), expected.encode());
                cases += 1;
            }
        }
        println!("REGION_CONTROL_STORAGE cases={cases}");
    }

    #[test]
    fn flow_frequency_is_discrete_and_preserves_raw_until_edited() {
        for raw in 0..=255u32 {
            // Match u8_to_01 followed by FLOW's f32 multiply and truncation.
            assert_eq!((raw as f32 / 255.0 * 15.0) as u32, raw * 15 / 255);
        }
        for frequency in 1..=16u8 {
            let mut layer = LayerEditState {
                opcode: EpuOpcode::Flow as u8,
                param_a: if frequency == 1 { 254 } else { 1 },
                param_c: 0xF7,
                variant_id: 7,
                domain_id: 3,
                ..Default::default()
            };
            let mut expected = layer.to_layer();
            assert!(flow_input(
                &mut layer,
                "Frequency",
                Some((frequency - 1) as f32 / 15.0)
            ));
            expected.param_a = (frequency - 1) * 17;
            assert_eq!(layer.to_layer().encode(), expected.encode());
            assert_eq!(1 + (layer.param_a as u32 * 15) / 255, frequency as u32);
        }
    }

    #[test]
    fn flow_named_pattern_preserves_octave_nibble_and_meta5() {
        for high in [0, 4, 5, 15] {
            for (pattern, name) in [(0, "Noise"), (1, "Streaks"), (2, "Caustic")] {
                let mut layer = LayerEditState {
                    opcode: EpuOpcode::Flow as u8,
                    param_c: (high << 4) | 13,
                    variant_id: 7,
                    domain_id: 3,
                    ..Default::default()
                };
                let mut expected = layer.to_layer();
                assert!(flow_input(&mut layer, name, None));
                expected.param_c = (high << 4) | pattern;
                assert_eq!(layer.to_layer().encode(), expected.encode());
            }
        }
    }

    #[test]
    fn flow_octaves_preserve_pattern_and_match_runtime_clamp_and_inactivity() {
        for pattern in [0, 1, 2, 7, 15] {
            for octave in 0..=4u8 {
                let initial_high = if pattern <= 1 && octave == 4 { 0 } else { 15 };
                let mut layer = LayerEditState {
                    opcode: EpuOpcode::Flow as u8,
                    param_c: (initial_high << 4) | pattern,
                    variant_id: 6,
                    domain_id: 2,
                    ..Default::default()
                };
                let mut expected = layer.to_layer();
                let edited = flow_input(&mut layer, "Octaves", Some(octave as f32 / 4.0));
                if pattern <= 1 {
                    assert!(edited);
                    expected.param_c = (octave << 4) | pattern;
                } else {
                    assert!(!edited, "inactive value must retain raw 15");
                }
                assert_eq!(layer.to_layer().encode(), expected.encode());
            }
        }
        let mut clamped = LayerEditState {
            opcode: EpuOpcode::Flow as u8,
            param_c: 0xF1,
            ..Default::default()
        };
        let before = clamped.to_layer().encode();
        assert!(!flow_input(&mut clamped, "Octaves", Some(1.0)));
        assert_eq!(
            clamped.to_layer().encode(),
            before,
            "unchanged effective 4 retains raw 15"
        );
    }

    #[test]
    fn flow_phase_display_matches_shader_endpoints_and_fallback_is_inactive() {
        for raw in [0, 128, 255] {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: EpuOpcode::Flow as u8,
                param_c: 0x40,
                param_d: raw,
                ..Default::default()
            };
            let before = layer.to_layer().encode();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            assert_eq!(layer.to_layer().encode(), before);
            flow_label_rect(
                &output,
                &format!("{raw}/256 = {:.6} turns", raw as f32 / 256.0),
            );
        }
        for pattern in [0, 1, 2, 3, 15] {
            let mut layer = LayerEditState {
                opcode: EpuOpcode::Flow as u8,
                param_c: 0x40 | pattern,
                param_d: 128,
                ..Default::default()
            };
            let mut expected = layer.to_layer();
            assert_eq!(
                flow_input(&mut layer, "Cyclic phase", Some(1.0)),
                pattern <= 2
            );
            if pattern <= 2 {
                expected.param_d = 255;
            }
            assert_eq!(layer.to_layer().encode(), expected.encode());
        }
    }

    #[test]
    fn scatter_metadata_has_no_fixed_radius_or_normalized_packed_bytes() {
        for spec in field_specs(EpuOpcode::Scatter as u8)
            .iter()
            .filter(|s| matches!(s.name, "param_b" | "param_c" | "param_d"))
        {
            assert_eq!(
                (spec.map, spec.min, spec.max, spec.unit),
                (MapKind::U8Lerp, 0.0, 255.0, None),
                "{}",
                spec.name
            );
        }
    }

    #[test]
    fn flow_scatter_metadata_is_honest_advanced_raw_fallback() {
        for (opcode, name, words) in [
            (
                EpuOpcode::Flow,
                "param_a",
                vec!["advanced raw", "1+floor(raw*15/255)"],
            ),
            (
                EpuOpcode::Flow,
                "param_c",
                vec!["advanced packed", "octaves high", "pattern low"],
            ),
            (
                EpuOpcode::Flow,
                "param_d",
                vec!["advanced raw", "raw/256", "cyclic"],
            ),
            (
                EpuOpcode::Scatter,
                "param_b",
                vec!["advanced raw", "density", "variant"],
            ),
            (
                EpuOpcode::Scatter,
                "param_c",
                vec!["advanced packed", "high nibble", "WINDOWS"],
            ),
            (EpuOpcode::Scatter, "param_d", vec!["seed", "raw byte"]),
        ] {
            let spec = field_specs(opcode as u8)
                .iter()
                .find(|s| s.name == name)
                .unwrap();
            assert_eq!(spec.map, MapKind::U8Lerp, "{opcode:?} {name}");
            assert_eq!((spec.min, spec.max), (0.0, 255.0), "{opcode:?} {name}");
            assert_eq!(spec.unit, None);
            for word in words {
                assert!(spec.label.contains(word), "{} missing {word}", spec.label);
            }
            for raw in 0..=255 {
                assert!((map_u8_to_semantic(raw, spec) - raw as f32).abs() < 0.0001);
            }
        }
        assert_eq!(
            variant_count(EpuOpcode::Flow as u8),
            0,
            "meta5 is not param_c pattern"
        );
    }

    // Headless egui input only: exercises the real field routing, not a desktop/player.
    fn ramp_frame(
        ctx: &egui::Context,
        layer: &mut LayerEditState,
        events: Vec<egui::Event>,
    ) -> (egui::FullOutput, bool) {
        let mut changed = false;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200.0, 900.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    changed |= EpuEditor::render_common_controls(ui, layer);
                    changed |= EpuEditor::render_field_controls(ui, layer);
                });
            },
        );
        (output, changed)
    }

    #[test]
    fn grid_unused_color_and_direction_controls_preserve_storage() {
        for (opcode, pattern) in [
            (EpuOpcode::Grid, 0),
            (EpuOpcode::Flow, 0),
            (EpuOpcode::Flow, 3),
            (EpuOpcode::Flow, 15),
        ] {
            for (label, active) in [
                ("Color A:", true),
                ("Color B:", opcode != EpuOpcode::Grid),
                ("Direction", opcode != EpuOpcode::Grid && pattern <= 2),
            ] {
                let ctx = egui::Context::default();
                ctx.style_mut(|style| style.animation_time = 0.0);
                let mut layer = LayerEditState {
                    opcode: opcode as u8,
                    param_c: pattern,
                    direction: 0x4321,
                    ..Default::default()
                };
                let before = layer.to_layer().encode();
                ramp_frame(&ctx, &mut layer, vec![]);
                let (out, _) = ramp_frame(&ctx, &mut layer, vec![]);
                let rect = flow_label_rect(&out, label);
                let pos = if label == "Direction" {
                    rect.center()
                } else {
                    egui::pos2(
                        rect.right()
                            + ctx.style().spacing.item_spacing.x
                            + ctx.style().spacing.interact_size.x / 2.0,
                        rect.center().y,
                    )
                };
                for pressed in [true, false] {
                    ramp_frame(
                        &ctx,
                        &mut layer,
                        vec![
                            egui::Event::PointerMoved(pos),
                            egui::Event::PointerButton {
                                pos,
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                    );
                }
                let (out, _) = ramp_frame(&ctx, &mut layer, vec![]);
                let opened = if label == "Direction" {
                    out.shapes.iter().any(|shape| {
                        matches!(&shape.shape,
                        egui::epaint::Shape::Text(text) if text.galley.text() == "Raw (oct u16):")
                    })
                } else {
                    egui::Popup::is_any_open(&ctx)
                };
                assert_eq!(
                    opened, active,
                    "opcode={opcode:?}, pattern={pattern}, {label}"
                );
                assert_eq!(layer.to_layer().encode(), before);
            }
        }
    }

    #[test]
    fn atmosphere_color_pickers_are_disabled_when_unused() {
        for variant in 0..8 {
            for (label, active) in [
                ("Color A:", variant <= 4),
                ("Color B:", matches!(variant, 0 | 1 | 3 | 4)),
            ] {
                let ctx = egui::Context::default();
                let mut layer = LayerEditState {
                    opcode: EpuOpcode::Atmosphere as u8,
                    variant_id: variant,
                    ..Default::default()
                };
                let before = layer.to_layer().encode();
                ramp_frame(&ctx, &mut layer, vec![]);
                let (out, _) = ramp_frame(&ctx, &mut layer, vec![]);
                let rect = flow_label_rect(&out, label);
                let pos = egui::pos2(
                    rect.right()
                        + ctx.style().spacing.item_spacing.x
                        + ctx.style().spacing.interact_size.x / 2.0,
                    rect.center().y,
                );
                for pressed in [true, false] {
                    ramp_frame(
                        &ctx,
                        &mut layer,
                        vec![
                            egui::Event::PointerMoved(pos),
                            egui::Event::PointerButton {
                                pos,
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                    );
                }
                assert_eq!(
                    egui::Popup::is_any_open(&ctx),
                    active,
                    "variant={variant}, {label}"
                );
                assert_eq!(layer.to_layer().encode(), before);
            }
        }
    }

    #[test]
    fn plane_alpha_b_is_disabled_without_changing_storage() {
        for (opcode, variant) in (0..8)
            .map(|v| (EpuOpcode::Plane, v))
            .chain([(EpuOpcode::Veil, 1)])
        {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: opcode as u8,
                variant_id: variant,
                alpha_b: 9,
                ..Default::default()
            };
            let expected = layer.to_layer();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            let text = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text) if text.galley.text() == "9" => Some(text),
                    _ => None,
                })
                .expect("stored alpha_b must remain visible");
            let start = text.pos + text.galley.size() / 2.0;
            let end = start + egui::vec2(50.0, 0.0);
            let mut edited = false;
            for (pos, pressed) in [(start, Some(true)), (end, None), (end, Some(false))] {
                let mut events = vec![egui::Event::PointerMoved(pos)];
                if let Some(pressed) = pressed {
                    events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                edited |= ramp_frame(&ctx, &mut layer, events).1;
            }
            assert!(
                !edited,
                "variant={variant}: opcode {opcode:?} ignores Color B alpha"
            );
            assert_eq!(layer.to_layer().encode(), expected.encode());
        }
    }

    #[test]
    fn cyclic_phase_fields_use_256_steps_without_duplicate_endpoint() {
        let mut failures = Vec::new();
        let mut samples = 0;
        for opcode in [
            EpuOpcode::Decal,
            EpuOpcode::Veil,
            EpuOpcode::Mottle,
            EpuOpcode::Advect,
            EpuOpcode::Surface,
            EpuOpcode::Mass,
        ] {
            let spec = field_specs(opcode as u8)
                .iter()
                .find(|s| s.name == "param_d")
                .unwrap();
            let mut wrong = 0;
            for raw in 0..=255u8 {
                let turns = map_u8_to_semantic(raw, spec);
                wrong += usize::from((turns - f32::from(raw) / 256.0).abs() >= 1e-7);
                assert_eq!(map_semantic_to_u8(turns, spec), raw);
                samples += 1;
            }
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: opcode as u8,
                param_d: 255,
                ..Default::default()
            };
            let before = layer.to_layer().encode();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            let mut missing = 0;
            for expected in ["phase (turns)", "0.996", "[0.000..0.996]"] {
                missing += usize::from(!output.shapes.iter().any(|s| {
                    matches!(
                        &s.shape, egui::epaint::Shape::Text(t) if t.galley.text() == expected
                    )
                }));
            }
            assert_eq!(layer.to_layer().encode(), before);
            println!(
                "CYCLIC_FIELD opcode={} wrong_values={} missing_readouts={}",
                opcode as u8, wrong, missing
            );
            if wrong != 0 || missing != 0 {
                failures.push((opcode as u8, wrong, missing));
            }
        }
        println!(
            "CYCLIC_PHASE_METADATA samples={samples} fields=6 failures={} tolerance=1e-7",
            failures.len()
        );
        assert!(failures.is_empty(), "{failures:?}");
    }

    #[test]
    fn plane_water_phase_uses_256_steps_without_duplicate_endpoint() {
        let spec = field_specs(EpuOpcode::Plane as u8)
            .iter()
            .find(|spec| spec.name == "param_d")
            .unwrap();
        for raw in 0..=255u8 {
            let turns = map_u8_to_semantic(raw, spec);
            assert!(
                (turns - f32::from(raw) / 256.0).abs() < 1e-7,
                "WATER phase raw={raw} has wrong semantic value {turns}"
            );
            assert_eq!(map_semantic_to_u8(turns, spec), raw);
        }
        let ctx = egui::Context::default();
        let mut layer = LayerEditState {
            opcode: EpuOpcode::Plane as u8,
            variant_id: 4,
            param_d: 255,
            ..Default::default()
        };
        let before = layer.to_layer().encode();
        ramp_frame(&ctx, &mut layer, vec![]);
        let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
        assert!(!changed);
        for expected in ["phase (turns)", "0.996", "[0.000..0.996]"] {
            assert!(
                output.shapes.iter().any(|shape| matches!(
                    &shape.shape, egui::epaint::Shape::Text(text)
                        if text.galley.text() == expected
                )),
                "missing WATER phase readout: {expected}"
            );
        }
        assert_eq!(layer.to_layer().encode(), before);
    }

    #[test]
    fn veil_authoring_fields_match_evaluator_activity() {
        let mut cases = 0;
        for variant in 0..8 {
            for spec in field_specs(EpuOpcode::Veil as u8) {
                if spec.name == "param_a" {
                    continue;
                } // Discrete count has its own pointer check.
                let mut layer = LayerEditState {
                    opcode: EpuOpcode::Veil as u8,
                    variant_id: variant,
                    intensity: 127,
                    param_a: 127,
                    param_b: 127,
                    param_c: 127,
                    param_d: 127,
                    alpha_a: 7,
                    alpha_b: 13,
                    ..Default::default()
                };
                let mut expected = layer.to_layer();
                let active = match spec.name {
                    "param_c" => !matches!(variant, 1 | 2),
                    "param_d" => variant == 3,
                    _ => true,
                };
                let label = spec.unit.map_or_else(
                    || spec.label.to_string(),
                    |u| format!("{} ({u})", spec.label),
                );
                let changed = flow_input(&mut layer, &label, Some(1.0));
                assert_eq!(changed, active, "variant={variant} field={}", spec.name);
                if active {
                    match spec.name {
                        "intensity" => expected.intensity = 255,
                        "param_b" => expected.param_b = 255,
                        "param_c" => expected.param_c = 255,
                        "param_d" => expected.param_d = 255,
                        _ => unreachable!(),
                    }
                }
                assert_eq!(
                    layer.to_layer().encode(),
                    expected.encode(),
                    "sibling storage changed"
                );
                cases += 1;
            }
        }
        println!("VEIL_FIELDS records={cases} stored and sibling bytes preserved");
    }

    #[test]
    fn veil_authoring_discrete_count_and_width_readout() {
        for raw in 0..=255u8 {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: EpuOpcode::Veil as u8,
                param_a: raw,
                param_b: 255,
                ..Default::default()
            };
            let before = layer.to_layer().encode();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (out, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            assert_eq!(layer.to_layer().encode(), before);
            let count = 2 + u32::from(raw) * 30 / 255;
            let width = (0.5 / count as f32).max(0.05);
            flow_label_rect(&out, &format!("Base thickness: {width:.6} chart units"));
            flow_label_rect(
                &out,
                &format!("[{count} ribbons; raw {raw}]; RAIN_WALL doubles this count"),
            );
        }
        for count in 2..=32u32 {
            let mut layer = LayerEditState {
                opcode: EpuOpcode::Veil as u8,
                param_a: if count == 2 { 254 } else { 0 },
                ..Default::default()
            };
            let mut expected = layer.to_layer();
            assert!(flow_input(
                &mut layer,
                "Count",
                Some((count - 2) as f32 / 30.0)
            ));
            expected.param_a = ((count - 2) * 255).div_ceil(30) as u8;
            assert_eq!(layer.to_layer().encode(), expected.encode());
            assert_eq!(2 + u32::from(layer.param_a) * 30 / 255, count);
        }
        println!("VEIL_COUNT records=256 idle mappings, 31 pointer edits; exact packing preserved");
    }

    #[test]
    fn plane_authoring_inactive_fields_preserve_storage() {
        for variant in 0..8 {
            for spec in field_specs(EpuOpcode::Plane as u8) {
                let mut layer = LayerEditState {
                    opcode: EpuOpcode::Plane as u8,
                    variant_id: variant,
                    intensity: 127,
                    param_a: 127,
                    param_b: 127,
                    param_c: 127,
                    param_d: 127,
                    alpha_a: 7,
                    alpha_b: 13,
                    ..Default::default()
                };
                let before = layer.to_layer().encode();
                let active = match spec.name {
                    "intensity" | "param_a" => true,
                    "param_b" => matches!(variant, 0 | 1 | 2 | 5 | 7),
                    "param_c" => matches!(variant, 2 | 3 | 6 | 7),
                    "param_d" => variant == 4,
                    _ => unreachable!(),
                };
                let label = spec.unit.map_or_else(
                    || spec.label.to_string(),
                    |unit| format!("{} ({})", spec.label, unit),
                );
                let changed = flow_input(&mut layer, &label, Some(1.0));
                assert_eq!(changed, active, "variant={variant} field={}", spec.name);
                let mut expected = before;
                if active {
                    let shift = match spec.name {
                        "intensity" => 56,
                        "param_a" => 48,
                        "param_b" => 40,
                        "param_c" => 32,
                        "param_d" => 24,
                        _ => unreachable!(),
                    };
                    expected[1] = (expected[1] & !(255u64 << shift)) | (255u64 << shift);
                }
                assert_eq!(
                    layer.to_layer().encode(),
                    expected,
                    "sibling storage must survive"
                );
            }
        }
    }

    #[test]
    fn plane_and_veil_color_pickers_are_disabled_when_unused() {
        for opcode in [EpuOpcode::Plane, EpuOpcode::Veil] {
            for variant in 0..8 {
                for (label, active) in [
                    ("Color A:", true),
                    (
                        "Color B:",
                        if opcode == EpuOpcode::Veil {
                            variant != 1
                        } else {
                            matches!(variant, 0 | 1 | 2 | 5 | 7)
                        },
                    ),
                ] {
                    let ctx = egui::Context::default();
                    let mut layer = LayerEditState {
                        opcode: opcode as u8,
                        variant_id: variant,
                        ..Default::default()
                    };
                    let before = layer.to_layer().encode();
                    ramp_frame(&ctx, &mut layer, vec![]);
                    let (out, _) = ramp_frame(&ctx, &mut layer, vec![]);
                    let rect = flow_label_rect(&out, label);
                    let pos = egui::pos2(
                        rect.right()
                            + ctx.style().spacing.item_spacing.x
                            + ctx.style().spacing.interact_size.x / 2.0,
                        rect.center().y,
                    );
                    for pressed in [true, false] {
                        ramp_frame(
                            &ctx,
                            &mut layer,
                            vec![
                                egui::Event::PointerMoved(pos),
                                egui::Event::PointerButton {
                                    pos,
                                    button: egui::PointerButton::Primary,
                                    pressed,
                                    modifiers: egui::Modifiers::NONE,
                                },
                            ],
                        );
                    }
                    assert_eq!(
                        egui::Popup::is_any_open(&ctx),
                        active,
                        "variant={variant}, {label}"
                    );
                    assert_eq!(layer.to_layer().encode(), before);
                }
            }
        }
    }

    #[test]
    fn atmosphere_authoring_inactive_fields_preserve_storage() {
        for variant in 0..8 {
            for spec in field_specs(EpuOpcode::Atmosphere as u8) {
                let mut layer = LayerEditState {
                    opcode: EpuOpcode::Atmosphere as u8,
                    variant_id: variant,
                    intensity: 127,
                    param_a: 127,
                    param_b: 127,
                    param_c: 127,
                    param_d: 127,
                    alpha_a: 7,
                    alpha_b: 13,
                    ..Default::default()
                };
                let before = layer.to_layer().encode();
                let active = match spec.name {
                    "intensity" => variant <= 4,
                    "param_a" | "param_b" => matches!(variant, 0 | 1 | 3 | 4),
                    "param_c" | "param_d" => matches!(variant, 2 | 3),
                    _ => unreachable!(),
                };
                let changed = flow_input(&mut layer, spec.label, Some(1.0));
                assert_eq!(changed, active, "variant={variant} field={}", spec.name);
                let mut expected = before;
                if active {
                    let shift = match spec.name {
                        "intensity" => 56,
                        "param_a" => 48,
                        "param_b" => 40,
                        "param_c" => 32,
                        "param_d" => 24,
                        _ => unreachable!(),
                    };
                    expected[1] = (expected[1] & !(255u64 << shift)) | (255u64 << shift);
                }
                assert_eq!(
                    layer.to_layer().encode(),
                    expected,
                    "sibling storage must survive"
                );
            }
        }
    }

    #[test]
    fn portal_authoring_active_fields_phase_and_storage_are_truthful() {
        let phase = field_specs(EpuOpcode::Portal as u8)
            .iter()
            .find(|s| s.name == "param_d")
            .unwrap();
        assert_eq!(phase.map, MapKind::U8Lerp);
        assert_eq!((phase.min, phase.max), (0.0, 255.0));
        assert_eq!(phase.label, "phase byte");
        let size = field_specs(EpuOpcode::Portal as u8)
            .iter()
            .find(|s| s.name == "param_a")
            .unwrap();
        assert_eq!(size.unit, Some("tan"));
        for variant in 0..8 {
            for rough in [0, 1, 255] {
                for raw in [0, 1, 127, 128, 254, 255] {
                    let ctx = egui::Context::default();
                    let mut layer = LayerEditState {
                        opcode: EpuOpcode::Portal as u8,
                        variant_id: variant,
                        param_c: rough,
                        param_d: raw,
                        alpha_a: 7,
                        alpha_b: 13,
                        ..Default::default()
                    };
                    let before = layer.to_layer().encode();
                    ramp_frame(&ctx, &mut layer, vec![]);
                    let (out, changed) = ramp_frame(&ctx, &mut layer, vec![]);
                    assert!(!changed);
                    assert_eq!(layer.to_layer().encode(), before);
                    let texts: Vec<_> = out
                        .shapes
                        .iter()
                        .filter_map(|s| match &s.shape {
                            egui::epaint::Shape::Text(t) => Some(t.galley.text()),
                            _ => None,
                        })
                        .collect();
                    let cycles = format!("{raw}/256 = {:.6} cycles", raw as f64 / 256.0);
                    assert!(texts.contains(&cycles.as_str()));
                    if variant != 3 {
                        assert!(
                            texts
                                .contains(&"VORTEX only: phase is inactive; stored byte retained.")
                        );
                    } else if rough == 0 {
                        assert!(texts.contains(
                            &"Zero roughness: circular contour; phase is stored but inactive."
                        ));
                    }
                    if !matches!(variant, 2..=5) {
                        assert!(
                            texts.contains(&"This shape ignores roughness; stored byte retained.")
                        );
                    }
                }
            }
        }
        for (variant, rough) in [(3, 192), (3, 0), (0, 192)] {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: EpuOpcode::Portal as u8,
                variant_id: variant,
                param_c: rough,
                param_d: 127,
                alpha_a: 7,
                alpha_b: 13,
                ..Default::default()
            };
            ramp_frame(&ctx, &mut layer, vec![]);
            let (out, _) = ramp_frame(&ctx, &mut layer, vec![]);
            let pos = out
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::epaint::Shape::Text(t) if t.galley.text() == "Phase byte" => {
                        Some(t.pos + egui::vec2(t.galley.size().x + 30.0, t.galley.size().y / 2.0))
                    }
                    _ => None,
                })
                .unwrap();
            let before = layer.to_layer().encode();
            let mut changed = false;
            for pressed in [true, false] {
                changed |= ramp_frame(
                    &ctx,
                    &mut layer,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                )
                .1;
            }
            let mut expected = before;
            if variant == 3 && rough != 0 {
                assert!(changed);
                assert_ne!(layer.param_d, 127);
                expected[1] = (expected[1] & !(255u64 << 24)) | (u64::from(layer.param_d) << 24);
            } else {
                assert!(!changed);
            }
            assert_eq!(layer.to_layer().encode(), expected);
        }
    }

    #[test]
    fn lobe_authoring_waveform_and_phase_preserve_storage() {
        for raw in 0u8..=255 {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: EpuOpcode::LobeRadiance as u8,
                param_c: raw,
                param_d: 255,
                alpha_b: 13,
                ..Default::default()
            };
            let before = layer.to_layer().encode();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            assert_eq!(layer.to_layer().encode(), before);
            let texts: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::epaint::Shape::Text(t) => Some(t.galley.text()),
                    _ => None,
                })
                .collect();
            let label = match raw {
                0 => "Steady (no modulation)",
                1 => "Sine",
                2 => "Triangle",
                3 => "Strobe (4 pulses)",
                _ => "Reserved (sine fallback)",
            };
            assert!(texts.contains(&label), "LOBE waveform {raw}: {texts:?}");
            assert!(texts.contains(&"255/256 = 0.996094 cycles"));
            assert!(texts.contains(&"Color B alpha is unused; stored nibble retained."));
            if raw == 0 {
                assert!(texts.contains(&"Steady: phase is inactive; stored byte retained."));
            }
        }
        let ctx = egui::Context::default();
        let mut layer = LayerEditState {
            opcode: EpuOpcode::LobeRadiance as u8,
            param_c: 1,
            param_d: 127,
            alpha_b: 13,
            ..Default::default()
        };
        ramp_frame(&ctx, &mut layer, vec![]);
        let (out, _) = ramp_frame(&ctx, &mut layer, vec![]);
        let pos = out
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::epaint::Shape::Text(t) if t.galley.text() == "Phase byte" => {
                    Some(t.pos + egui::vec2(t.galley.size().x + 30., t.galley.size().y / 2.))
                }
                _ => None,
            })
            .unwrap();
        let before = layer.to_layer().encode();
        let mut changed = false;
        for pressed in [true, false] {
            changed |= ramp_frame(
                &ctx,
                &mut layer,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            )
            .1;
        }
        assert!(changed);
        assert_ne!(layer.param_d, 127);
        let mut expected = before;
        expected[1] = (expected[1] & !(255u64 << 24)) | (u64::from(layer.param_d) << 24);
        assert_eq!(layer.to_layer().encode(), expected);
    }

    #[test]
    fn celestial_ring_authoring_shows_tilt_and_preserves_siblings() {
        for raw in 0u8..=255 {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: EpuOpcode::Celestial as u8,
                variant_id: 4,
                param_d: raw,
                param_c: 173,
                ..Default::default()
            };
            let before = layer.to_layer().encode();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            assert_eq!(layer.to_layer().encode(), before);
            let texts: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::epaint::Shape::Text(t) => Some(t.galley.text()),
                    _ => None,
                })
                .collect();
            assert!(texts.contains(&"Ring tilt (deg)"));
            assert!(texts.contains(&"[0.00..90.00]"));
            assert!(texts.contains(&"Phase is unused by RINGED; stored byte retained."));
            if raw == 128 {
                let pos = output
                    .shapes
                    .iter()
                    .find_map(|s| match &s.shape {
                        egui::epaint::Shape::Text(t) if t.galley.text() == "Ring tilt (deg)" => {
                            Some(
                                t.pos + egui::vec2(t.galley.size().x + 30., t.galley.size().y / 2.),
                            )
                        }
                        _ => None,
                    })
                    .unwrap();
                let mut changed = false;
                for pressed in [true, false] {
                    changed |= ramp_frame(
                        &ctx,
                        &mut layer,
                        vec![
                            egui::Event::PointerMoved(pos),
                            egui::Event::PointerButton {
                                pos,
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                    )
                    .1;
                }
                assert!(changed);
                assert_ne!(layer.param_d, raw);
                let mut expected = before;
                expected[1] = (expected[1] & !(255u64 << 24)) | (u64::from(layer.param_d) << 24);
                assert_eq!(layer.to_layer().encode(), expected);
            }
        }
    }

    #[test]
    fn grid_authoring_shows_effective_counts_cycles_and_phase_without_normalizing_storage() {
        for pattern in [0u8, 1, 2, 15] {
            for raw in 0u8..=255 {
                let ctx = egui::Context::default();
                let mut layer = LayerEditState {
                    opcode: EpuOpcode::Grid as u8,
                    param_a: raw,
                    param_c: (pattern << 4) | 7,
                    param_d: 255,
                    ..Default::default()
                };
                let before = layer.to_layer().encode();
                ramp_frame(&ctx, &mut layer, vec![]);
                let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
                assert!(!changed);
                assert_eq!(layer.to_layer().encode(), before);
                let target = 1.0 + 63.0 * raw as f64 / 255.0;
                let count = if pattern == 2 {
                    (target / 2.0).round().max(1.0) * 2.0
                } else {
                    target.round()
                } as u8;
                let texts: Vec<_> = output
                    .shapes
                    .iter()
                    .filter_map(|s| match &s.shape {
                        egui::epaint::Shape::Text(t) => Some(t.galley.text()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    texts.contains(&format!("{count} whole repeats; stored raw {raw}").as_str()),
                    "GRID must display actual repeats, raw={raw} pattern={pattern}"
                );
                assert!(texts.contains(&"Cycles / loop"));
                assert!(texts.contains(&"255/256 = 0.996094 loops"));
            }
        }
    }

    #[test]
    fn grid_real_controls_preserve_sibling_fields_and_nibbles() {
        let ctx = egui::Context::default();
        let mut layer = LayerEditState {
            opcode: EpuOpcode::Grid as u8,
            param_a: 127,
            param_c: 0x17,
            param_d: 255,
            ..Default::default()
        };
        ramp_frame(&ctx, &mut layer, vec![]);
        let position = |output: &egui::FullOutput, label: &str| {
            output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::epaint::Shape::Text(t) if t.galley.text() == label => {
                        Some(t.pos + t.galley.size() / 2.0)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing real GRID control {label}"))
        };
        let mut expected = layer.to_layer();
        let (output, _) = ramp_frame(&ctx, &mut layer, vec![]);
        let pos = position(&output, "Checker");
        for pressed in [true, false] {
            ramp_frame(
                &ctx,
                &mut layer,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert_eq!(layer.param_c, 0x27);
        expected.param_c = 0x27;
        assert_eq!(layer.to_layer().encode(), expected.encode());
        for (label, field, dx) in [
            ("7", "cycles", 60.0),
            ("32", "count", 60.0),
            ("255", "phase", -60.0),
        ] {
            let (output, _) = ramp_frame(&ctx, &mut layer, vec![]);
            // The phase value is disambiguated from brightness by choosing the last exact label.
            let start = if field == "phase" {
                output
                    .shapes
                    .iter()
                    .rev()
                    .find_map(|s| match &s.shape {
                        egui::epaint::Shape::Text(t) if t.galley.text() == label => {
                            Some(t.pos + t.galley.size() / 2.0)
                        }
                        _ => None,
                    })
                    .unwrap()
            } else {
                position(&output, label)
            };
            let end = start + egui::vec2(dx, 0.0);
            let before = layer.to_layer();
            let mut edited = false;
            for (pos, pressed) in [(start, Some(true)), (end, None), (end, Some(false))] {
                let mut events = vec![egui::Event::PointerMoved(pos)];
                if let Some(pressed) = pressed {
                    events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                edited |= ramp_frame(&ctx, &mut layer, events).1;
            }
            assert!(edited, "{field} must actually receive input");
            let mut expected = before;
            match field {
                "cycles" => {
                    assert_eq!(layer.param_c >> 4, 2);
                    assert_ne!(layer.param_c & 15, 7);
                    expected.param_c = layer.param_c;
                }
                "count" => {
                    assert_ne!(grid_repeats(layer.param_a, 2), 32);
                    assert_eq!(grid_repeats(layer.param_a, 2) % 2, 0);
                    expected.param_a = layer.param_a;
                }
                _ => {
                    assert_ne!(layer.param_d, 255);
                    expected.param_d = layer.param_d;
                }
            }
            assert_eq!(
                layer.to_layer().encode(),
                expected.encode(),
                "{field} must preserve all sibling fields"
            );
        }
    }

    #[test]
    fn band_depth_and_phase_controls_preserve_sibling_bits() {
        for field in ["depth", "phase", "inactive_phase"] {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: EpuOpcode::BandRadiance as u8,
                alpha_b: if field == "inactive_phase" { 0 } else { 7 },
                param_d: 255,
                ..Default::default()
            };
            let initial = layer.to_layer().encode();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            assert_eq!(layer.to_layer().encode(), initial);
            let texts: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::epaint::Shape::Text(t) => Some(t.galley.text()),
                    _ => None,
                })
                .collect();
            assert!(texts.contains(&"Mod depth:"));
            assert!(texts.contains(&"255/256 = 0.996094 loops"));
            if field == "inactive_phase" {
                assert!(texts.contains(&"Zero depth: plain ring; phase is stored but inactive."));
            }
            let label = if field == "depth" { "7" } else { "255" };
            let start = output
                .shapes
                .iter()
                .rev()
                .find_map(|s| match &s.shape {
                    egui::epaint::Shape::Text(t) if t.galley.text() == label => {
                        Some(t.pos + t.galley.size() / 2.0)
                    }
                    _ => None,
                })
                .expect("real BAND control");
            let end = start + egui::vec2(if field == "depth" { 60.0 } else { -60.0 }, 0.0);
            let mut edited = false;
            for (pos, pressed) in [(start, Some(true)), (end, None), (end, Some(false))] {
                let mut events = vec![egui::Event::PointerMoved(pos)];
                if let Some(pressed) = pressed {
                    events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                edited |= ramp_frame(&ctx, &mut layer, events).1;
            }
            let mut expected = LayerEditState {
                opcode: EpuOpcode::BandRadiance as u8,
                alpha_b: if field == "inactive_phase" { 0 } else { 7 },
                param_d: 255,
                ..Default::default()
            }
            .to_layer();
            match field {
                "depth" => {
                    assert!(edited);
                    assert_ne!(layer.alpha_b, 7);
                    expected.alpha_b = layer.alpha_b;
                }
                "phase" => {
                    assert!(edited);
                    assert_ne!(layer.param_d, 255);
                    expected.param_d = layer.param_d;
                }
                _ => assert!(!edited),
            }
            assert_eq!(
                layer.to_layer().encode(),
                expected.encode(),
                "{field} sibling preservation"
            );
        }
    }

    fn check_ramp_threshold_control(label: &str, ceiling: bool) {
        let ctx = egui::Context::default();
        let mut layer = LayerEditState {
            opcode: EpuOpcode::Ramp as u8,
            param_d: 0xA5,
            alpha_b: 9,
            ..Default::default()
        };
        let mut expected = layer.to_layer();
        // Warm up layout, then locate the real label (missing before the fix).
        ramp_frame(&ctx, &mut layer, vec![]);
        let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
        assert!(!changed);
        assert_eq!(layer.to_layer().encode(), expected.encode());
        let text = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == label => Some(text),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing native RAMP control: {label}"));
        // Drive the adjacent default egui slider at each representable position.
        // Its handle center travels inside a radius of thickness / 2.5 at either end.
        let spacing = &ctx.style().spacing;
        let radius = text.galley.size().y.max(spacing.interact_size.y) / 2.5;
        let start = text.pos
            + egui::vec2(
                text.galley.size().x + spacing.item_spacing.x + radius,
                text.galley.size().y / 2.0,
            );
        for step in 0..=15u8 {
            let pos = start
                + egui::vec2(
                    (spacing.slider_width - 2.0 * radius) * step as f32 / 15.0,
                    0.0,
                );
            let mut edited = false;
            for pressed in [true, false] {
                edited |= ramp_frame(
                    &ctx,
                    &mut layer,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                )
                .1;
            }
            assert!(edited, "{label}: step {step} must report the edit");
            expected.param_d = if ceiling {
                (step << 4) | 5
            } else {
                0xA0 | step
            };
            // Covers every representable nibble, the neighbor, alpha_b, and all other fields.
            assert_eq!(
                layer.to_layer().encode(),
                expected.encode(),
                "{label}: step {step}"
            );
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            let signed = format!("{:.3}", step as f32 / 15.0 * 2.0 - 1.0);
            assert!(
                output.shapes.iter().any(|shape| matches!(
                    &shape.shape, egui::epaint::Shape::Text(text) if text.galley.text() == signed
                )),
                "missing signed value {signed}"
            );
            // Reversed authored endpoints must remain reversed in storage, not be repaired by UI.
            let ceiling_y = (layer.param_d >> 4) as f32 / 15.0 * 2.0 - 1.0;
            let floor_y = (layer.param_d & 15) as f32 / 15.0 * 2.0 - 1.0;
            if floor_y > ceiling_y {
                let effective = format!(
                    "Renderer sorts reversed endpoints: effective floor {:.3}, ceiling {:.3}. Stored values unchanged.",
                    floor_y.min(ceiling_y),
                    floor_y.max(ceiling_y)
                );
                assert!(
                    output.shapes.iter().any(|shape| matches!(
                        &shape.shape, egui::epaint::Shape::Text(text)
                            if text.galley.text() == effective
                    )),
                    "reversed endpoints need a truthful renderer-order explanation"
                );
            }
        }
    }

    #[test]
    fn ramp_floor_control_preserves_ceiling_and_other_fields() {
        check_ramp_threshold_control("Floor threshold", false);
    }

    #[test]
    fn ramp_ceiling_control_preserves_floor_and_other_fields() {
        check_ramp_threshold_control("Ceiling threshold", true);
    }

    #[test]
    fn unused_feature_alpha_b_preserves_bytes_and_active_controls() {
        // These bounds and features never consume B alpha in the shared shader path.
        const INACTIVE: [u8; 11] = [
            0x02, 0x03, 0x04, 0x07, 0x09, 0x0A, 0x0B, 0x14, 0x15, 0x16, 0x17,
        ];
        // Every remaining alpha_b consumer retains its editable control.
        for opcode in INACTIVE
            .into_iter()
            .chain([0x05, 0x06, 0x08, 0x0C, 0x0D, 0x10, 0x11, 0x13, 0x18])
        {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode,
                alpha_b: 9,
                ..Default::default()
            };
            let mut expected = layer.to_layer();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            let text = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text) if text.galley.text() == "9" => Some(text),
                    _ => None,
                })
                .expect("stored alpha_b must remain visible");
            let start = text.pos + text.galley.size() / 2.0;
            let end = start + egui::vec2(50.0, 0.0);
            let mut edited = false;
            for (pos, pressed) in [(start, Some(true)), (end, None), (end, Some(false))] {
                let mut events = vec![egui::Event::PointerMoved(pos)];
                if let Some(pressed) = pressed {
                    events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                edited |= ramp_frame(&ctx, &mut layer, events).1;
            }
            if INACTIVE.contains(&opcode) {
                assert!(!edited, "unused alpha_b edited for opcode {opcode:#04x}");
                assert_eq!(layer.alpha_b, 9);
            } else {
                assert!(
                    edited,
                    "active alpha_b must remain editable for opcode {opcode:#04x}"
                );
                assert_ne!(layer.alpha_b, 9);
                expected.alpha_b = layer.alpha_b;
            }
            assert_eq!(layer.to_layer().encode(), expected.encode());
        }
    }

    #[test]
    fn ramp_alpha_b_is_disabled_without_changing_storage_or_other_opcodes() {
        for opcode in [EpuOpcode::Ramp, EpuOpcode::Cell] {
            let ctx = egui::Context::default();
            let mut layer = LayerEditState {
                opcode: opcode as u8,
                alpha_b: 9,
                ..Default::default()
            };
            let mut expected = layer.to_layer();
            ramp_frame(&ctx, &mut layer, vec![]);
            let (output, changed) = ramp_frame(&ctx, &mut layer, vec![]);
            assert!(!changed);
            let text = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text) if text.galley.text() == "9" => Some(text),
                    _ => None,
                })
                .expect("stored alpha_b must remain visible");
            let start = text.pos + text.galley.size() / 2.0;
            let end = start + egui::vec2(50.0, 0.0);
            let mut edited = false;
            for (pos, pressed) in [(start, Some(true)), (end, None), (end, Some(false))] {
                let mut events = vec![egui::Event::PointerMoved(pos)];
                if let Some(pressed) = pressed {
                    events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                edited |= ramp_frame(&ctx, &mut layer, events).1;
            }
            if opcode == EpuOpcode::Ramp {
                assert!(!edited);
                assert!(output.shapes.iter().any(|shape| matches!(
                    &shape.shape, egui::epaint::Shape::Text(text)
                        if text.galley.text() == "RAMP ignores Color B alpha; stored value preserved."
                )));
            } else {
                assert!(edited, "non-RAMP alpha_b must remain editable");
                assert_ne!(layer.alpha_b, 9);
                expected.alpha_b = layer.alpha_b;
            }
            assert_eq!(layer.to_layer().encode(), expected.encode());
        }
    }

    #[test]
    fn ramp_metadata_is_explicitly_advanced_packed_not_normalized() {
        let spec = field_specs(EpuOpcode::Ramp as u8)
            .iter()
            .find(|spec| spec.name == "param_d")
            .unwrap();
        assert_eq!(spec.map, MapKind::U8Lerp);
        assert_eq!((spec.min, spec.max), (0.0, 255.0));
        assert!(spec.label.contains("advanced packed"));
        for raw in 0..=255 {
            assert!((map_u8_to_semantic(raw, spec) - raw as f32).abs() < 0.0001);
        }
    }

    #[test]
    fn test_layer_roundtrip() {
        let original = EpuLayer {
            opcode: EpuOpcode::Decal,
            region_mask: REGION_SKY | REGION_WALLS,
            blend: EpuBlend::Lerp,
            meta5: pack_meta5(1, 2),
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

        let packed = original.encode();
        let decoded = decode_packed_layer(packed);

        assert_eq!(decoded.opcode, original.opcode);
        assert_eq!(decoded.region_mask, original.region_mask);
        assert_eq!(decoded.blend, original.blend);
        assert_eq!(decoded.meta5, original.meta5);
        assert_eq!(decoded.color_a, original.color_a);
        assert_eq!(decoded.color_b, original.color_b);
        assert_eq!(decoded.alpha_a, original.alpha_a);
        assert_eq!(decoded.alpha_b, original.alpha_b);
        assert_eq!(decoded.intensity, original.intensity);
        assert_eq!(decoded.param_a, original.param_a);
        assert_eq!(decoded.param_b, original.param_b);
        assert_eq!(decoded.param_c, original.param_c);
        assert_eq!(decoded.param_d, original.param_d);
        assert_eq!(decoded.direction, original.direction);
    }

    #[test]
    fn test_edit_state_roundtrip() {
        let layer = EpuLayer {
            opcode: EpuOpcode::Scatter,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(2, 3),
            color_a: [200, 100, 50],
            color_b: [25, 12, 6],
            alpha_a: 15,
            alpha_b: 10,
            intensity: 180,
            param_a: 90,
            param_b: 120,
            param_c: 60,
            param_d: 30,
            direction: 0x8080,
        };

        let state = LayerEditState::from_layer(&layer);
        let restored = state.to_layer();

        assert_eq!(restored.opcode, layer.opcode);
        assert_eq!(restored.region_mask, layer.region_mask);
        assert_eq!(restored.blend, layer.blend);
        assert_eq!(restored.meta5, layer.meta5);
        assert_eq!(restored.color_a, layer.color_a);
        assert_eq!(restored.color_b, layer.color_b);
    }

    #[test]
    fn phased_scatter_authoring_roundtrip() {
        let spec = field_specs(0x18)
            .iter()
            .find(|s| s.name == "param_c")
            .unwrap();
        assert_eq!(opcode_name(0x18), "SCATTER_PHASED");
        for phase in 0..=255u8 {
            assert!((map_u8_to_semantic(phase, spec) - f32::from(phase) / 256.0).abs() < 0.000001);
            for opcode in [EpuOpcode::Scatter, EpuOpcode::ScatterPhased] {
                for depth in 0..=15 {
                    let state = LayerEditState {
                        opcode: opcode as u8,
                        param_c: phase,
                        alpha_b: depth,
                        domain_id: phase & 3,
                        variant_id: phase % 8,
                        ..LayerEditState::default()
                    };
                    let packed = state.to_layer().encode();
                    assert_eq!((packed[0] >> 59) & 31, opcode as u64);
                    assert_eq!((packed[1] >> 32) & 255, phase as u64);
                    assert_eq!(packed[1] & 15, depth as u64);
                    let decoded = decode_packed_layer(packed);
                    assert_eq!(
                        LayerEditState::from_layer(&decoded).to_layer().encode(),
                        packed
                    );
                    let json = serde_json::to_string(&state.to_workbench_layer()).unwrap();
                    let patch: EpuWorkbenchLayerPatch = serde_json::from_str(&json).unwrap();
                    let mut restored = LayerEditState::default();
                    restored.apply_workbench_patch(&patch);
                    assert_eq!(restored.to_layer().encode(), packed);
                }
            }
        }
    }

    #[test]
    fn test_decode_mass_layer() {
        let layer = EpuLayer {
            opcode: EpuOpcode::Mass,
            region_mask: REGION_WALLS,
            blend: EpuBlend::Lerp,
            meta5: pack_meta5(0, 0),
            color_a: [135, 152, 164],
            color_b: [8, 13, 18],
            alpha_a: 15,
            alpha_b: 0,
            intensity: 248,
            param_a: 104,
            param_b: 212,
            param_c: 82,
            param_d: 74,
            direction: 0x8000,
        };

        let decoded = decode_packed_layer(layer.encode());

        assert_eq!(decoded.opcode, EpuOpcode::Mass);
        assert_eq!(decoded.region_mask, REGION_WALLS);
        assert_eq!(decoded.blend, EpuBlend::Lerp);
    }

    #[test]
    fn test_semantic_mapping() {
        let spec_01 = FieldSpec {
            name: "test",
            label: "test",
            unit: None,
            map: MapKind::U8_01,
            min: 0.0,
            max: 1.0,
        };

        assert!((map_u8_to_semantic(0, &spec_01) - 0.0).abs() < 0.01);
        assert!((map_u8_to_semantic(255, &spec_01) - 1.0).abs() < 0.01);
        assert!((map_u8_to_semantic(128, &spec_01) - 0.5).abs() < 0.01);

        let spec_lerp = FieldSpec {
            name: "test",
            label: "test",
            unit: Some("x"),
            map: MapKind::U8Lerp,
            min: 1.0,
            max: 16.0,
        };

        assert!((map_u8_to_semantic(0, &spec_lerp) - 1.0).abs() < 0.01);
        assert!((map_u8_to_semantic(255, &spec_lerp) - 16.0).abs() < 0.1);
    }

    #[test]
    fn scatter_size_uses_density_and_variant_in_ui_mapping() {
        let density_raw = 200;
        let size_raw = 128;
        let density = 1.0 + density_raw as f32;
        let expected =
            (0.001 + ((0.5 / density).max(0.05) - 0.001) * (size_raw as f32 / 255.0)) * 0.3;
        let display = scatter_size_display(density_raw, size_raw, 5);

        assert!((display.base_radius - expected).abs() < 0.0001);
        assert_eq!(display.extended_falloff, Some(("RAIN", 3.0)));
        assert!((display.max_radius - 0.015).abs() < 0.0001);
        assert_eq!(
            scatter_size_display(density_raw, size_raw, 4).extended_falloff,
            Some(("EMBERS", 2.0))
        );
    }

    #[test]
    fn scatter_twinkle_uses_high_nibble_and_preserves_reserved_bits() {
        let raw = 0xA5;
        assert_eq!(scatter_twinkle_raw_with_step(raw, raw >> 4), raw);
        let edited = scatter_twinkle_raw_with_step(raw, 3);

        assert_eq!(edited, 0x35);
        assert_eq!(edited & 0x0F, raw & 0x0F);
        assert!((scatter_twinkle_value(edited) - 3.0 / 15.0).abs() < 0.0001);
    }
}
