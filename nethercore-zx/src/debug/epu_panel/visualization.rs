//! Direction gizmo visualization for EPU octahedral-encoded directions.
//!
//! This module provides a visual 3D direction gizmo widget that displays and allows
//! editing of octahedral-encoded direction values used in EPU layers.

use egui::{Color32, Pos2, Response, Sense, Stroke, Ui, Vec2};

// =============================================================================
// Direction Encoding/Decoding
// =============================================================================

/// Decode an octahedral u16 value to a normalized 3D direction vector.
///
/// The u16 is packed as: low byte = u (maps to X), high byte = v (maps to Y).
/// Each byte maps from [0, 255] to [-1, 1].
/// Z is computed from: z = 1 - |x| - |y| (with octahedral unwrapping for z < 0).
///
/// This mirrors the WGSL `decode_dir16` function in `epu_common.wgsl`.
pub fn decode_direction_u16(encoded: u16) -> [f32; 3] {
    let u_byte = (encoded & 0xFF) as f32;
    let v_byte = ((encoded >> 8) & 0xFF) as f32;

    // Map [0, 255] -> [-1, 1]
    let oct_x = u_byte / 255.0 * 2.0 - 1.0;
    let oct_y = v_byte / 255.0 * 2.0 - 1.0;

    octahedral_decode(oct_x, oct_y)
}

/// Encode a normalized 3D direction vector to octahedral u16.
///
/// This mirrors the Rust `encode_direction_u16` in `layer.rs`.
pub fn encode_direction_u16(dir: [f32; 3]) -> u16 {
    let [x, y, z] = normalize(dir);

    // Handle zero vector -> default to +Z
    if x == 0.0 && y == 0.0 && z == 0.0 {
        return 0x8080; // Center of octahedral map = +Z direction
    }

    let denom = x.abs() + y.abs() + z.abs();
    let mut p_x = x / denom;
    let mut p_y = y / denom;

    if z < 0.0 {
        let sign_x = if p_x >= 0.0 { 1.0 } else { -1.0 };
        let sign_y = if p_y >= 0.0 { 1.0 } else { -1.0 };
        let new_x = (1.0 - p_y.abs()) * sign_x;
        let new_y = (1.0 - p_x.abs()) * sign_y;
        p_x = new_x;
        p_y = new_y;
    }

    // Map [-1, 1] -> [0, 255]
    let u = ((p_x * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u16;
    let v = ((p_y * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u16;
    (u & 0xFF) | ((v & 0xFF) << 8)
}

/// Decode octahedral [-1, 1]^2 coordinates to a unit direction vector.
///
/// This mirrors the WGSL `octahedral_decode` function.
/// The oct coordinates map to X and Y, with Z computed as: z = 1 - |x| - |y|.
fn octahedral_decode(oct_x: f32, oct_y: f32) -> [f32; 3] {
    let mut n_x = oct_x;
    let mut n_y = oct_y;
    let n_z = 1.0 - oct_x.abs() - oct_y.abs();

    if n_z < 0.0 {
        let sign_x = if n_x >= 0.0 { 1.0 } else { -1.0 };
        let sign_y = if n_y >= 0.0 { 1.0 } else { -1.0 };
        // Note: the order matters here - we need to use the original n_y for n_x calculation
        let old_n_x = n_x;
        n_x = (1.0 - n_y.abs()) * sign_x;
        n_y = (1.0 - old_n_x.abs()) * sign_y;
    }

    normalize([n_x, n_y, n_z])
}

/// Normalize a 3D vector to unit length.
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

// =============================================================================
// Direction Gizmo Widget
// =============================================================================

/// Visual direction gizmo for octahedral-encoded directions.
///
/// Displays a hemisphere with an arrow indicating the direction, and allows
/// interactive editing by clicking/dragging on the gizmo surface.
pub struct DirectionGizmo {
    /// Size of the gizmo widget in pixels
    pub size: f32,
    /// Whether the gizmo is interactive (allows editing)
    pub interactive: bool,
    /// Show axis indicators (X, Y, Z labels)
    pub show_axes: bool,
}

impl Default for DirectionGizmo {
    fn default() -> Self {
        Self {
            size: 100.0,
            interactive: true,
            show_axes: true,
        }
    }
}

impl DirectionGizmo {
    /// Create a new direction gizmo with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the size of the gizmo.
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set whether the gizmo is interactive.
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Set whether to show axis indicators.
    pub fn with_axes(mut self, show_axes: bool) -> Self {
        self.show_axes = show_axes;
        self
    }

    /// Render the direction gizmo and return whether the value was changed.
    ///
    /// # Arguments
    /// * `ui` - The egui UI context
    /// * `direction` - Mutable reference to the octahedral-encoded direction (u16)
    ///
    /// # Returns
    /// `true` if the direction was modified by user interaction
    pub fn show(&self, ui: &mut Ui, direction: &mut u16) -> bool {
        let mut changed = false;

        // Decode current direction
        let dir = decode_direction_u16(*direction);

        // Allocate space for the gizmo
        let sense = if self.interactive {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), sense);

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let radius = self.size * 0.4;

            // Draw the gizmo
            self.draw_hemisphere(painter, center, radius);
            self.draw_direction_arrow(painter, center, radius, dir);

            if self.show_axes {
                self.draw_axis_indicators(painter, center, radius);
            }

            // Handle interaction
            if self.interactive {
                changed = self.handle_interaction(&response, center, radius, direction);
            }
        }

        // Show numerical values below the gizmo
        ui.horizontal(|ui| {
            ui.label(format!(
                "X: {:.2}  Y: {:.2}  Z: {:.2}",
                dir[0], dir[1], dir[2]
            ));
        });

        // Show raw hex value
        ui.horizontal(|ui| {
            ui.weak(format!("oct: 0x{:04X}", *direction));
        });

        changed
    }

    /// Draw the hemisphere outline and grid.
    fn draw_hemisphere(&self, painter: &egui::Painter, center: Pos2, radius: f32) {
        // Background circle (sphere outline)
        painter.circle_stroke(center, radius, Stroke::new(1.5, Color32::from_gray(100)));

        // Inner fill (slightly transparent)
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(40, 50, 60, 200),
        );

        // Draw latitude lines (circles at different Z levels / depths)
        // These appear as concentric circles when viewing the front hemisphere
        for i in 1..4 {
            let z_level = i as f32 / 4.0; // 0.25, 0.5, 0.75
            let circle_radius = radius * (1.0 - z_level * z_level).sqrt();

            painter.circle_stroke(
                center,
                circle_radius,
                Stroke::new(0.5, Color32::from_gray(60)),
            );
        }

        // Draw longitude lines (radial lines from center)
        let num_lines = 8;
        for i in 0..num_lines {
            let angle = i as f32 * std::f32::consts::PI / num_lines as f32;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            // Line from center to edge
            let end = Pos2::new(
                center.x + cos_a * radius,
                center.y - sin_a * radius, // Invert Y for screen coords
            );

            painter.line_segment([center, end], Stroke::new(0.5, Color32::from_gray(50)));
        }

        // Draw the equator (XY plane at Z=0) as a thicker line
        painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_gray(70)));
    }

    /// Draw the direction arrow.
    fn draw_direction_arrow(
        &self,
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        dir: [f32; 3],
    ) {
        let [x, y, z] = dir;

        // Project 3D direction to 2D
        // X -> horizontal, Y -> vertical, Z affects depth/brightness (coming out of screen)
        let scale = 1.0 + z * 0.3; // Depth scaling based on Z (Z+ = toward viewer)
        let end_x = center.x + x * radius * 0.85 * scale;
        let end_y = center.y - y * radius * 0.85 * scale; // Y maps to vertical (inverted for screen coords)

        let end = Pos2::new(end_x, end_y);

        // Draw arrow line
        // Color based on Z (front/back hemisphere)
        let arrow_color = if z >= 0.0 {
            Color32::from_rgb(255, 200, 80) // Yellow for front hemisphere (+Z)
        } else {
            Color32::from_rgb(150, 100, 200) // Purple for back hemisphere (-Z)
        };

        painter.line_segment([center, end], Stroke::new(2.5, arrow_color));

        // Draw arrowhead
        let dir_2d = Vec2::new(end_x - center.x, end_y - center.y);
        let dir_len = dir_2d.length();
        if dir_len > 1.0 {
            let dir_norm = dir_2d / dir_len;
            let perp = Vec2::new(-dir_norm.y, dir_norm.x);

            let arrow_size = 8.0;
            let tip1 = end - dir_norm * arrow_size + perp * arrow_size * 0.5;
            let tip2 = end - dir_norm * arrow_size - perp * arrow_size * 0.5;

            painter.add(egui::Shape::convex_polygon(
                vec![end, tip1, tip2],
                arrow_color,
                Stroke::NONE,
            ));
        }

        // Draw endpoint dot
        painter.circle_filled(end, 4.0, arrow_color);
    }

    /// Draw axis indicators (X, Y, Z labels).
    fn draw_axis_indicators(&self, painter: &egui::Painter, center: Pos2, radius: f32) {
        let font = egui::FontId::proportional(10.0);

        // +X axis (right)
        painter.text(
            Pos2::new(center.x + radius + 8.0, center.y),
            egui::Align2::LEFT_CENTER,
            "+X",
            font.clone(),
            Color32::from_rgb(255, 100, 100),
        );

        // -X axis (left)
        painter.text(
            Pos2::new(center.x - radius - 8.0, center.y),
            egui::Align2::RIGHT_CENTER,
            "-X",
            font.clone(),
            Color32::from_rgb(150, 80, 80),
        );

        // +Y axis (up in 2D view)
        painter.text(
            Pos2::new(center.x, center.y - radius - 8.0),
            egui::Align2::CENTER_BOTTOM,
            "+Y",
            font.clone(),
            Color32::from_rgb(100, 255, 100),
        );

        // -Y axis (down in 2D view)
        painter.text(
            Pos2::new(center.x, center.y + radius + 8.0),
            egui::Align2::CENTER_TOP,
            "-Y",
            font.clone(),
            Color32::from_rgb(80, 150, 80),
        );

        // +Z indicator (coming out of screen) - shown near center
        painter.text(
            Pos2::new(center.x + radius + 8.0, center.y - radius * 0.4),
            egui::Align2::LEFT_CENTER,
            "+Z",
            font,
            Color32::from_rgb(100, 100, 255),
        );
    }

    /// Handle mouse interaction for editing the direction.
    fn handle_interaction(
        &self,
        response: &Response,
        center: Pos2,
        radius: f32,
        direction: &mut u16,
    ) -> bool {
        let mut changed = false;

        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                // Convert screen position to normalized coordinates relative to center
                // X maps to X, screen-Y maps to -Y (inverted)
                let dx = (pos.x - center.x) / radius;
                let dy = -(pos.y - center.y) / radius; // Invert Y for screen coords

                // Clamp to unit circle
                let dist = (dx * dx + dy * dy).sqrt();
                let (nx, ny) = if dist > 1.0 {
                    (dx / dist, dy / dist)
                } else {
                    (dx, dy)
                };

                // Calculate Z from hemisphere equation: x^2 + y^2 + z^2 = 1
                // Since we're showing the front hemisphere (+Z), Z is positive
                let z_sq = 1.0 - nx * nx - ny * ny;
                let nz = if z_sq > 0.0 { z_sq.sqrt() } else { 0.0 };

                // Encode the new direction
                let new_dir = [nx, ny, nz];
                let new_encoded = encode_direction_u16(new_dir);

                if new_encoded != *direction {
                    *direction = new_encoded;
                    changed = true;
                }
            }
        }

        changed
    }
}

/// Convenience function to show a direction gizmo with default settings.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `direction` - Mutable reference to the octahedral-encoded direction (u16)
///
/// # Returns
/// `true` if the direction was modified
pub fn direction_gizmo(ui: &mut Ui, direction: &mut u16) -> bool {
    DirectionGizmo::default().show(ui, direction)
}

/// Convenience function to show a compact direction gizmo (smaller, no axes).
pub fn direction_gizmo_compact(ui: &mut Ui, direction: &mut u16) -> bool {
    DirectionGizmo::new()
        .with_size(60.0)
        .with_axes(false)
        .show(ui, direction)
}

// =============================================================================
// Layer Activity Indicators
// =============================================================================

/// Visual indicator for layer activity state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerActivityState {
    /// Layer is disabled (NOP opcode)
    Disabled,
    /// Layer is active but currently muted
    Muted,
    /// Layer is active but another layer is soloed
    Inactive,
    /// Layer is active and visible
    Active,
    /// Layer is soloed (isolated)
    Soloed,
}

impl LayerActivityState {
    /// Get the indicator color for this state.
    pub fn color(&self) -> Color32 {
        match self {
            LayerActivityState::Disabled => Color32::from_gray(50),
            LayerActivityState::Muted => Color32::from_rgb(100, 50, 50),
            LayerActivityState::Inactive => Color32::from_gray(80),
            LayerActivityState::Active => Color32::from_rgb(80, 180, 80),
            LayerActivityState::Soloed => Color32::from_rgb(255, 200, 50),
        }
    }

    /// Get a single character indicator.
    pub fn indicator(&self) -> &'static str {
        match self {
            LayerActivityState::Disabled => "-",
            LayerActivityState::Muted => "M",
            LayerActivityState::Inactive => ".",
            LayerActivityState::Active => "*",
            LayerActivityState::Soloed => "S",
        }
    }
}

/// Draw a layer activity indicator dot.
pub fn draw_activity_indicator(ui: &mut Ui, state: LayerActivityState, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();
        let radius = size * 0.4;

        painter.circle_filled(center, radius, state.color());

        // Add glow for soloed state
        if state == LayerActivityState::Soloed {
            painter.circle_stroke(
                center,
                radius + 2.0,
                Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 200, 50, 128)),
            );
        }
    }

    response
}

/// Draw a horizontal bar showing all 8 layer activity states.
pub fn draw_layer_activity_bar(
    ui: &mut Ui,
    states: &[LayerActivityState; 8],
    selected: Option<usize>,
) {
    let item_size = 12.0;
    let spacing = 4.0;

    ui.horizontal(|ui| {
        for (i, &state) in states.iter().enumerate() {
            let is_selected = selected == Some(i);

            let (rect, _response) =
                ui.allocate_exact_size(Vec2::new(item_size, item_size + 2.0), Sense::hover());

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                let center = rect.center();

                // Draw indicator
                let radius = item_size * 0.35;
                painter.circle_filled(center, radius, state.color());

                // Selection indicator (underline)
                if is_selected {
                    let underline_y = rect.max.y - 1.0;
                    painter.line_segment(
                        [
                            Pos2::new(rect.min.x, underline_y),
                            Pos2::new(rect.max.x, underline_y),
                        ],
                        Stroke::new(2.0, Color32::WHITE),
                    );
                }
            }

            if i < 7 {
                ui.add_space(spacing);
            }
        }
    });
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_roundtrip() {
        // Test various directions
        // Note: octahedral encoding maps X,Y to oct coords, Z is derived
        let test_cases = [
            [0.0, 0.0, 1.0],       // +Z (forward) - center of octahedral map
            [1.0, 0.0, 0.0],       // +X (right)
            [0.0, 1.0, 0.0],       // +Y (up)
            [-1.0, 0.0, 0.0],      // -X (left)
            [0.0, -1.0, 0.0],      // -Y (down)
            [0.577, 0.577, 0.577], // diagonal (front hemisphere)
        ];

        for original in test_cases {
            let original_norm = normalize(original);
            let encoded = encode_direction_u16(original);
            let decoded = decode_direction_u16(encoded);

            // Allow for some quantization error due to 8-bit precision
            let error = (original_norm[0] - decoded[0]).abs()
                + (original_norm[1] - decoded[1]).abs()
                + (original_norm[2] - decoded[2]).abs();
            assert!(
                error < 0.15,
                "Direction roundtrip failed: {:?} -> 0x{:04X} -> {:?} (error: {})",
                original_norm,
                encoded,
                decoded,
                error
            );
        }
    }

    #[test]
    fn test_default_direction() {
        // 0x8080 = center of octahedral map = +Z direction
        let dir = decode_direction_u16(0x8080);
        assert!(
            dir[2] > 0.9,
            "+Z component should be dominant for 0x8080, got {:?}",
            dir
        );
    }

    #[test]
    fn test_encode_zero_vector() {
        // Zero vector should encode to default (0x8080 = +Z)
        let encoded = encode_direction_u16([0.0, 0.0, 0.0]);
        assert_eq!(encoded, 0x8080);
    }

    #[test]
    fn test_cardinal_directions() {
        // Test encoding of cardinal directions
        let plus_z = encode_direction_u16([0.0, 0.0, 1.0]);
        // +Z should be near center (0x80, 0x80)
        assert!(
            (plus_z & 0xFF) >= 0x70 && (plus_z & 0xFF) <= 0x90,
            "+Z u byte should be near 0x80"
        );
        assert!(
            ((plus_z >> 8) & 0xFF) >= 0x70 && ((plus_z >> 8) & 0xFF) <= 0x90,
            "+Z v byte should be near 0x80"
        );

        let plus_x = encode_direction_u16([1.0, 0.0, 0.0]);
        // +X should have high u, mid v
        assert!((plus_x & 0xFF) > 0xC0, "+X u byte should be high");

        let plus_y = encode_direction_u16([0.0, 1.0, 0.0]);
        // +Y should have mid u, high v
        assert!(((plus_y >> 8) & 0xFF) > 0xC0, "+Y v byte should be high");
    }
}
