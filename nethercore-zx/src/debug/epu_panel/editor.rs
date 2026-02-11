//! Semantic EPU editor with metadata-driven UI.
//!
//! This module provides an editor UI that uses the generated metadata from
//! `epu_meta_gen` to display semantic labels, units, and appropriate controls
//! for each EPU opcode field.

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
    EpuBlend, EpuConfig, EpuLayer, EpuOpcode, REGION_ALL, REGION_FLOOR, REGION_SKY, REGION_WALLS,
    pack_meta5,
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
            direction: 0x8080, // +Y direction (octahedral center)
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

        ui.separator();

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
                Some(OpcodeKind::Radiance) => " [radiance]",
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
                        if let Some(info) = &OPCODES[code as usize] {
                            if ui
                                .selectable_value(&mut layer.opcode, info.code, info.name)
                                .clicked()
                            {
                                changed = true;
                                // Reset variant/domain when changing opcode
                                layer.variant_id = 0;
                                layer.domain_id = 0;
                            }
                        }
                    }

                    ui.separator();
                    ui.label("Radiance:");

                    // Radiance opcodes (0x08..=0x1F)
                    for code in 8u8..=0x1F {
                        if let Some(info) = &OPCODES[code as usize] {
                            if ui
                                .selectable_value(&mut layer.opcode, info.code, info.name)
                                .clicked()
                            {
                                changed = true;
                                layer.variant_id = 0;
                                layer.domain_id = 0;
                            }
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

        // Region mask checkboxes
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

        // Blend mode
        ui.horizontal(|ui| {
            ui.label("Blend:");

            let blend_name = match layer.blend {
                EpuBlend::Add => "Add",
                EpuBlend::Multiply => "Multiply",
                EpuBlend::Max => "Max",
                EpuBlend::Lerp => "Lerp",
                EpuBlend::Screen => "Screen",
                EpuBlend::HsvMod => "HSV Mod",
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
                        (EpuBlend::HsvMod, "HSV Mod"),
                        (EpuBlend::Min, "Min"),
                        (EpuBlend::Overlay, "Overlay"),
                    ] {
                        if ui.selectable_value(&mut layer.blend, mode, name).clicked() {
                            changed = true;
                        }
                    }
                });
        });

        // Color A
        ui.horizontal(|ui| {
            ui.label("Color A:");
            let mut color =
                egui::Color32::from_rgb(layer.color_a[0], layer.color_a[1], layer.color_a[2]);
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

        // Color B
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

            ui.label("Alpha:");
            let mut alpha = layer.alpha_b as i32;
            if ui
                .add(egui::DragValue::new(&mut alpha).range(0..=15))
                .changed()
            {
                layer.alpha_b = alpha.clamp(0, 15) as u8;
                changed = true;
            }
        });

        // Direction with visual gizmo
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

        for (i, spec) in specs.iter().enumerate() {
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
                ui.weak(format!("[{:.2}..{:.2}]", spec.min, spec.max));
            }
        });

        changed
    }
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
}
