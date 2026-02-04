//! Layer isolation and contribution preview for EPU debugging.
//!
//! This module provides:
//! - Solo/mute toggles per layer (like an audio mixer)
//! - Layer isolation mode to view individual layer contributions
//! - Text descriptions of what each layer contributes based on opcode/params
//! - Visual indicators for active vs inactive layers

use super::editor::LayerEditState;
use crate::debug::epu_meta_gen::{opcode_kind, opcode_name, variant_name, OpcodeKind};
use crate::graphics::epu::{REGION_ALL, REGION_FLOOR, REGION_SKY, REGION_WALLS};

// =============================================================================
// Layer Isolation State
// =============================================================================

/// State tracking layer isolation/solo/mute for debugging.
#[derive(Clone, Debug, Default)]
pub struct LayerIsolationState {
    /// Which layer is currently isolated (soloed), if any (0..7)
    pub isolated_layer: Option<usize>,
    /// Mute mask: bit i set = layer i is muted
    pub mute_mask: u8,
    /// Whether isolation mode is active
    pub isolation_active: bool,
}

impl LayerIsolationState {
    /// Create a new isolation state with no layers isolated or muted.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a specific layer is currently muted.
    #[inline]
    pub fn is_muted(&self, layer: usize) -> bool {
        layer < 8 && (self.mute_mask & (1 << layer)) != 0
    }

    /// Toggle mute state for a layer.
    pub fn toggle_mute(&mut self, layer: usize) {
        if layer < 8 {
            self.mute_mask ^= 1 << layer;
        }
    }

    /// Set mute state for a layer.
    pub fn set_muted(&mut self, layer: usize, muted: bool) {
        if layer < 8 {
            if muted {
                self.mute_mask |= 1 << layer;
            } else {
                self.mute_mask &= !(1 << layer);
            }
        }
    }

    /// Check if a specific layer is currently soloed (isolated).
    #[inline]
    pub fn is_soloed(&self, layer: usize) -> bool {
        self.isolated_layer == Some(layer)
    }

    /// Toggle solo/isolation for a layer.
    ///
    /// If the layer is already soloed, exits isolation mode.
    /// Otherwise, enters isolation mode with this layer.
    pub fn toggle_solo(&mut self, layer: usize) {
        if layer >= 8 {
            return;
        }

        if self.isolated_layer == Some(layer) {
            // Un-solo: exit isolation mode
            self.isolated_layer = None;
            self.isolation_active = false;
        } else {
            // Solo this layer
            self.isolated_layer = Some(layer);
            self.isolation_active = true;
        }
    }

    /// Exit isolation mode entirely.
    pub fn show_all(&mut self) {
        self.isolated_layer = None;
        self.isolation_active = false;
    }

    /// Check if a layer should be rendered based on solo/mute state.
    ///
    /// Returns `true` if the layer should be rendered.
    pub fn should_render_layer(&self, layer: usize) -> bool {
        if layer >= 8 {
            return false;
        }

        // If a layer is soloed, only render that layer
        if self.isolation_active {
            if let Some(soloed) = self.isolated_layer {
                return layer == soloed;
            }
        }

        // Otherwise, render if not muted
        !self.is_muted(layer)
    }

    /// Generate a layer visibility mask (8 bits, one per layer).
    ///
    /// Can be passed to the GPU to control which layers are rendered.
    pub fn visibility_mask(&self) -> u8 {
        let mut mask = 0u8;

        for i in 0..8 {
            if self.should_render_layer(i) {
                mask |= 1 << i;
            }
        }

        mask
    }

    /// Get the count of visible layers.
    pub fn visible_count(&self) -> usize {
        self.visibility_mask().count_ones() as usize
    }
}

// =============================================================================
// Layer Contribution Description
// =============================================================================

/// Describes what a layer contributes to the final result.
#[derive(Clone, Debug)]
pub struct LayerContribution {
    /// Short summary (e.g., "Adds sky gradient")
    pub summary: String,
    /// Opcode category
    pub category: LayerCategory,
    /// Primary effect color (for visual swatch)
    pub primary_color: [u8; 3],
    /// Secondary effect color (for gradient swatch)
    pub secondary_color: Option<[u8; 3]>,
    /// Which regions this affects
    pub regions: Vec<&'static str>,
    /// Whether this layer is active (non-NOP opcode)
    pub is_active: bool,
}

/// Category of layer effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerCategory {
    /// Layer is disabled (NOP)
    Disabled,
    /// Bounds opcode (defines geometry/regions)
    Bounds,
    /// Radiance opcode (adds color/light)
    Radiance,
}

impl LayerContribution {
    /// Generate a contribution description from a layer edit state.
    pub fn from_layer(layer: &LayerEditState) -> Self {
        // Handle NOP/disabled layers
        if layer.opcode == 0 {
            return Self {
                summary: "Disabled".to_string(),
                category: LayerCategory::Disabled,
                primary_color: [64, 64, 64],
                secondary_color: None,
                regions: vec![],
                is_active: false,
            };
        }

        // Get opcode info
        let opcode_str = opcode_name(layer.opcode);
        let kind = opcode_kind(layer.opcode);
        let variant = variant_name(layer.opcode, layer.variant_id);

        // Build region list
        let regions = region_names(layer.region_mask);

        // Determine category
        let category = match kind {
            Some(OpcodeKind::Bounds) => LayerCategory::Bounds,
            Some(OpcodeKind::Radiance) => LayerCategory::Radiance,
            None => LayerCategory::Disabled,
        };

        // Generate summary based on opcode
        let summary = generate_layer_summary(layer, opcode_str, variant, &regions);

        // Determine if we need secondary color
        let secondary_color = if layer.color_a != layer.color_b {
            Some(layer.color_b)
        } else {
            None
        };

        Self {
            summary,
            category,
            primary_color: layer.color_a,
            secondary_color,
            regions,
            is_active: true,
        }
    }

    /// Get a short status string for display in tabs.
    pub fn status_icon(&self) -> &'static str {
        match self.category {
            LayerCategory::Disabled => "-",
            LayerCategory::Bounds => "B",
            LayerCategory::Radiance => "R",
        }
    }
}

/// Get region names from a region mask.
fn region_names(mask: u8) -> Vec<&'static str> {
    let mut regions = Vec::with_capacity(3);

    if mask == REGION_ALL {
        regions.push("All");
    } else {
        if (mask & REGION_SKY) != 0 {
            regions.push("Sky");
        }
        if (mask & REGION_WALLS) != 0 {
            regions.push("Walls");
        }
        if (mask & REGION_FLOOR) != 0 {
            regions.push("Floor");
        }
    }

    if regions.is_empty() {
        regions.push("None");
    }

    regions
}

/// Generate a human-readable summary of what a layer does.
fn generate_layer_summary(
    layer: &LayerEditState,
    opcode: &str,
    variant: &str,
    regions: &[&str],
) -> String {
    let region_str = regions.join("+");
    let color_desc = describe_color(layer.color_a);

    // Build variant suffix if present
    let variant_str = if variant.is_empty() {
        String::new()
    } else {
        format!(" ({})", variant.to_lowercase())
    };

    // Generate opcode-specific descriptions
    match layer.opcode {
        0x01 => {
            // RAMP
            format!("Vertical gradient on {}", region_str)
        }
        0x02 => {
            // SECTOR
            format!("Sector bounds{} on {}", variant_str, region_str)
        }
        0x03 => {
            // SILHOUETTE
            format!("Horizon silhouette{} on {}", variant_str, region_str)
        }
        0x04 => {
            // SPLIT
            format!("Split pattern{} on {}", variant_str, region_str)
        }
        0x05 => {
            // CELL
            format!("Cell pattern{} on {}", variant_str, region_str)
        }
        0x06 => {
            // PATCHES
            format!("Patch pattern{} on {}", variant_str, region_str)
        }
        0x07 => {
            // APERTURE
            format!("Aperture shape{} on {}", variant_str, region_str)
        }
        0x08 => {
            // DECAL
            format!("Adds {} decal to {}", color_desc, region_str)
        }
        0x09 => {
            // GRID
            format!("Adds {} grid lines to {}", color_desc, region_str)
        }
        0x0A => {
            // SCATTER
            format!("Scatters {} points{} on {}", color_desc, variant_str, region_str)
        }
        0x0B => {
            // FLOW
            format!("Adds {} flow pattern to {}", color_desc, region_str)
        }
        0x0C => {
            // TRACE
            format!("Draws {} traces{} on {}", color_desc, variant_str, region_str)
        }
        0x0D => {
            // VEIL
            format!("Adds {} veil{} to {}", color_desc, variant_str, region_str)
        }
        0x0E => {
            // ATMOSPHERE
            format!("Applies {} atmosphere{} to {}", color_desc, variant_str, region_str)
        }
        0x0F => {
            // PLANE
            format!("Adds {} plane texture{} to {}", color_desc, variant_str, region_str)
        }
        0x10 => {
            // CELESTIAL
            format!("Renders {} celestial body{} on {}", color_desc, variant_str, region_str)
        }
        0x11 => {
            // PORTAL
            format!("Creates {} portal{} on {}", color_desc, variant_str, region_str)
        }
        0x12 => {
            // LOBE
            format!("Adds {} directional lobe to {}", color_desc, region_str)
        }
        0x13 => {
            // BAND
            format!("Adds {} horizontal band to {}", color_desc, region_str)
        }
        _ => {
            // Unknown/generic
            format!("{}{}: {} on {}", opcode, variant_str, color_desc, region_str)
        }
    }
}

/// Generate a short color description.
fn describe_color(rgb: [u8; 3]) -> &'static str {
    let [r, g, b] = rgb;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    // Check for grayscale
    if max - min < 30 {
        return if max < 64 {
            "dark"
        } else if max < 192 {
            "gray"
        } else {
            "bright"
        };
    }

    // Determine dominant color
    if r > g && r > b {
        if r > 200 && g > 150 && b < 100 {
            "orange"
        } else if r > 200 && g < 100 && b > 150 {
            "pink"
        } else {
            "red"
        }
    } else if g > r && g > b {
        if g > 200 && r > 150 {
            "yellow"
        } else if b > 150 {
            "cyan"
        } else {
            "green"
        }
    } else if b > r && b > g {
        if r > 150 {
            "purple"
        } else if g > 150 {
            "cyan"
        } else {
            "blue"
        }
    } else if r > 200 && g > 200 {
        "yellow"
    } else if r > 200 && b > 200 {
        "magenta"
    } else if g > 200 && b > 200 {
        "cyan"
    } else {
        "colored"
    }
}

// =============================================================================
// UI Rendering for Isolation Controls
// =============================================================================

/// Render the isolation banner when a layer is isolated.
///
/// Returns `true` if "Show All" was clicked.
pub fn render_isolation_banner(ui: &mut egui::Ui, state: &LayerIsolationState) -> bool {
    let mut show_all_clicked = false;

    if state.isolation_active {
        if let Some(layer_idx) = state.isolated_layer {
            ui.horizontal(|ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::YELLOW);
                ui.label(format!("Layer {} isolated", layer_idx));
                ui.visuals_mut().override_text_color = None;

                if ui.button("Show All").clicked() {
                    show_all_clicked = true;
                }
            });
            ui.separator();
        }
    }

    show_all_clicked
}

/// Render solo/mute controls for a single layer.
///
/// Returns (solo_toggled, mute_toggled)
pub fn render_layer_isolation_controls(
    ui: &mut egui::Ui,
    layer_idx: usize,
    state: &LayerIsolationState,
) -> (bool, bool) {
    let mut solo_toggled = false;
    let mut mute_toggled = false;

    let is_soloed = state.is_soloed(layer_idx);
    let is_muted = state.is_muted(layer_idx);

    // Solo button
    let solo_text = if is_soloed { "S" } else { "s" };
    let solo_color = if is_soloed {
        egui::Color32::from_rgb(255, 200, 50)
    } else {
        egui::Color32::GRAY
    };

    if ui
        .add(
            egui::Button::new(egui::RichText::new(solo_text).color(solo_color))
                .min_size(egui::vec2(20.0, 20.0)),
        )
        .on_hover_text("Solo this layer (isolate)")
        .clicked()
    {
        solo_toggled = true;
    }

    // Mute button
    let mute_text = if is_muted { "M" } else { "m" };
    let mute_color = if is_muted {
        egui::Color32::from_rgb(255, 80, 80)
    } else {
        egui::Color32::GRAY
    };

    if ui
        .add(
            egui::Button::new(egui::RichText::new(mute_text).color(mute_color))
                .min_size(egui::vec2(20.0, 20.0)),
        )
        .on_hover_text("Mute this layer")
        .clicked()
    {
        mute_toggled = true;
    }

    (solo_toggled, mute_toggled)
}

/// Render a compact contribution preview for a layer.
pub fn render_contribution_preview(
    ui: &mut egui::Ui,
    contribution: &LayerContribution,
    compact: bool,
) {
    ui.horizontal(|ui| {
        // Color swatch
        render_color_swatch(
            ui,
            contribution.primary_color,
            contribution.secondary_color,
            if compact { 16.0 } else { 24.0 },
        );

        // Status/category indicator
        let category_color = match contribution.category {
            LayerCategory::Disabled => egui::Color32::DARK_GRAY,
            LayerCategory::Bounds => egui::Color32::from_rgb(100, 180, 255),
            LayerCategory::Radiance => egui::Color32::from_rgb(255, 200, 100),
        };

        ui.colored_label(category_color, contribution.status_icon());

        // Summary text
        if compact {
            ui.label(&contribution.summary);
        } else {
            ui.vertical(|ui| {
                ui.label(&contribution.summary);
                if !contribution.regions.is_empty() && contribution.is_active {
                    ui.weak(format!("Regions: {}", contribution.regions.join(", ")));
                }
            });
        }
    });
}

/// Render a color swatch (with optional gradient).
pub fn render_color_swatch(
    ui: &mut egui::Ui,
    primary: [u8; 3],
    secondary: Option<[u8; 3]>,
    size: f32,
) {
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        let primary_color =
            egui::Color32::from_rgb(primary[0], primary[1], primary[2]);

        if let Some(sec) = secondary {
            // Draw gradient swatch
            let secondary_color = egui::Color32::from_rgb(sec[0], sec[1], sec[2]);

            // Left half
            painter.rect_filled(
                egui::Rect::from_min_max(
                    rect.min,
                    egui::pos2(rect.center().x, rect.max.y),
                ),
                0.0,
                primary_color,
            );

            // Right half
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(rect.center().x, rect.min.y),
                    rect.max,
                ),
                0.0,
                secondary_color,
            );
        } else {
            // Solid color swatch
            painter.rect_filled(rect, 0.0, primary_color);
        }

        // Border
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
            egui::StrokeKind::Inside,
        );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_state_default() {
        let state = LayerIsolationState::new();
        assert!(!state.isolation_active);
        assert!(state.isolated_layer.is_none());
        assert_eq!(state.mute_mask, 0);
    }

    #[test]
    fn test_mute_toggle() {
        let mut state = LayerIsolationState::new();

        assert!(!state.is_muted(0));
        state.toggle_mute(0);
        assert!(state.is_muted(0));
        state.toggle_mute(0);
        assert!(!state.is_muted(0));
    }

    #[test]
    fn test_solo_toggle() {
        let mut state = LayerIsolationState::new();

        assert!(!state.is_soloed(2));
        state.toggle_solo(2);
        assert!(state.is_soloed(2));
        assert!(state.isolation_active);

        // Toggle again should un-solo
        state.toggle_solo(2);
        assert!(!state.is_soloed(2));
        assert!(!state.isolation_active);
    }

    #[test]
    fn test_visibility_mask_normal() {
        let state = LayerIsolationState::new();
        // All layers visible by default
        assert_eq!(state.visibility_mask(), 0xFF);
    }

    #[test]
    fn test_visibility_mask_with_mute() {
        let mut state = LayerIsolationState::new();
        state.set_muted(0, true);
        state.set_muted(3, true);
        // Layers 0 and 3 muted = bits 0 and 3 clear
        assert_eq!(state.visibility_mask(), 0b11110110);
    }

    #[test]
    fn test_visibility_mask_with_solo() {
        let mut state = LayerIsolationState::new();
        state.toggle_solo(2);
        // Only layer 2 visible
        assert_eq!(state.visibility_mask(), 0b00000100);
    }

    #[test]
    fn test_should_render_layer() {
        let mut state = LayerIsolationState::new();

        // All layers render by default
        assert!(state.should_render_layer(0));
        assert!(state.should_render_layer(7));

        // Muted layer doesn't render
        state.set_muted(3, true);
        assert!(!state.should_render_layer(3));
        assert!(state.should_render_layer(4));

        // Solo overrides mute
        state.toggle_solo(5);
        assert!(!state.should_render_layer(3)); // Still not rendered (not soloed)
        assert!(!state.should_render_layer(4)); // Not rendered (not soloed)
        assert!(state.should_render_layer(5)); // Rendered (soloed)
    }

    #[test]
    fn test_show_all() {
        let mut state = LayerIsolationState::new();
        state.toggle_solo(2);
        assert!(state.isolation_active);

        state.show_all();
        assert!(!state.isolation_active);
        assert!(state.isolated_layer.is_none());
    }

    #[test]
    fn test_contribution_from_disabled_layer() {
        let layer = LayerEditState::default();
        let contrib = LayerContribution::from_layer(&layer);

        assert!(!contrib.is_active);
        assert_eq!(contrib.category, LayerCategory::Disabled);
        assert_eq!(contrib.status_icon(), "-");
    }

    #[test]
    fn test_region_names() {
        assert_eq!(region_names(REGION_ALL), vec!["All"]);
        assert_eq!(region_names(REGION_SKY), vec!["Sky"]);
        assert_eq!(region_names(REGION_SKY | REGION_WALLS), vec!["Sky", "Walls"]);
        assert_eq!(region_names(0), vec!["None"]);
    }

    #[test]
    fn test_describe_color() {
        assert_eq!(describe_color([255, 0, 0]), "red");
        assert_eq!(describe_color([0, 255, 0]), "green");
        assert_eq!(describe_color([0, 0, 255]), "blue");
        assert_eq!(describe_color([128, 128, 128]), "gray");
        assert_eq!(describe_color([255, 255, 0]), "yellow");
    }
}
