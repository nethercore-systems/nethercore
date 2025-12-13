//! Debug inspection panel UI
//!
//! Provides an egui panel for viewing and editing registered debug values.

use std::collections::HashSet;

use super::export::export_as_rust_flat;
use super::frame_control::{FrameController, TIME_SCALE_OPTIONS};
use super::registry::{DebugRegistry, RegisteredValue, TreeNode};
use super::types::{DebugValue, ValueType};

/// Debug inspection panel state
pub struct DebugPanel {
    /// Whether the panel is visible
    pub visible: bool,
    /// Set of collapsed group paths
    collapsed_groups: HashSet<String>,
    /// Cached tree structure (rebuilt when values change)
    tree_cache: Option<Vec<TreeNode>>,
    /// Whether tree needs to be rebuilt
    tree_dirty: bool,
}

impl Default for DebugPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugPanel {
    /// Create a new debug panel
    pub fn new() -> Self {
        Self {
            visible: false,
            collapsed_groups: HashSet::new(),
            tree_cache: None,
            tree_dirty: true,
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

    /// Mark tree as needing rebuild (call after registry changes)
    pub fn invalidate_tree(&mut self) {
        self.tree_dirty = true;
    }

    /// Render the debug panel
    ///
    /// Returns true if any value was changed (caller should invoke callback).
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        registry: &DebugRegistry,
        frame_controller: &mut FrameController,
        read_value: impl Fn(&RegisteredValue) -> Option<DebugValue>,
        write_value: impl Fn(&RegisteredValue, &DebugValue) -> bool,
    ) -> bool {
        if !self.visible || registry.is_empty() {
            return false;
        }

        // Rebuild tree if needed
        if self.tree_dirty || self.tree_cache.is_none() {
            self.tree_cache = Some(registry.build_tree());
            self.tree_dirty = false;
        }

        let mut any_changed = false;

        egui::Window::new("Debug Inspector")
            .id(egui::Id::new("debug_inspection_window"))
            .default_pos([10.0, 10.0])
            .default_size([320.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // Frame controls
                if !frame_controller.is_disabled() {
                    any_changed |= self.render_frame_controls(ui, frame_controller);
                    ui.separator();
                }

                // Scrollable area for values
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Clone tree to avoid borrow conflict with render_tree(&mut self)
                        if let Some(tree) = self.tree_cache.clone() {
                            any_changed |= self.render_tree(
                                ui,
                                &tree,
                                registry,
                                "",
                                &read_value,
                                &write_value,
                            );
                        }
                    });

                ui.separator();

                // Export buttons
                self.render_export_buttons(ui, registry, &read_value);
            });

        any_changed
    }

    /// Render frame control buttons
    fn render_frame_controls(
        &self,
        ui: &mut egui::Ui,
        frame_controller: &mut FrameController,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Pause/Play toggle
            let pause_text = if frame_controller.is_paused() {
                "▶ Play"
            } else {
                "⏸ Pause"
            };
            if ui.button(pause_text).clicked() {
                frame_controller.toggle_pause();
                changed = true;
            }

            // Step frame (only when paused)
            ui.add_enabled_ui(frame_controller.is_paused(), |ui| {
                if ui.button("⏭ Step").clicked() {
                    frame_controller.request_step();
                    changed = true;
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label("Speed:");

            // Time scale dropdown
            let current_scale = frame_controller.time_scale();
            egui::ComboBox::from_id_salt("time_scale")
                .selected_text(format!("{}x", current_scale))
                .show_ui(ui, |ui| {
                    for &scale in &TIME_SCALE_OPTIONS {
                        if ui
                            .selectable_value(
                                &mut frame_controller.time_scale_index(),
                                TIME_SCALE_OPTIONS.iter().position(|&s| s == scale).unwrap(),
                                format!("{}x", scale),
                            )
                            .clicked()
                        {
                            frame_controller.set_time_scale(scale);
                            changed = true;
                        }
                    }
                });
        });

        changed
    }

    /// Render the value tree recursively
    fn render_tree(
        &mut self,
        ui: &mut egui::Ui,
        nodes: &[TreeNode],
        registry: &DebugRegistry,
        parent_path: &str,
        read_value: &impl Fn(&RegisteredValue) -> Option<DebugValue>,
        write_value: &impl Fn(&RegisteredValue, &DebugValue) -> bool,
    ) -> bool {
        let mut any_changed = false;

        for node in nodes {
            match node {
                TreeNode::Group { name, children } => {
                    let path = if parent_path.is_empty() {
                        name.clone()
                    } else {
                        format!("{}/{}", parent_path, name)
                    };

                    let is_collapsed = self.collapsed_groups.contains(&path);
                    let header = egui::CollapsingHeader::new(name)
                        .default_open(!is_collapsed)
                        .show(ui, |ui| {
                            any_changed |= self.render_tree(
                                ui,
                                children,
                                registry,
                                &path,
                                read_value,
                                write_value,
                            );
                        });

                    // Track collapse state
                    if header.header_response.clicked() {
                        if is_collapsed {
                            self.collapsed_groups.remove(&path);
                        } else {
                            self.collapsed_groups.insert(path);
                        }
                    }
                }
                TreeNode::Value(idx) => {
                    if let Some(reg_value) = registry.values.get(*idx) {
                        if let Some(current) = read_value(reg_value) {
                            if let Some(new_val) = self.render_value_widget(ui, reg_value, current)
                            {
                                if write_value(reg_value, &new_val) {
                                    any_changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        any_changed
    }

    /// Render a widget for a single value
    ///
    /// Returns Some(new_value) if the value was changed, None otherwise.
    fn render_value_widget(
        &self,
        ui: &mut egui::Ui,
        reg_value: &RegisteredValue,
        current: DebugValue,
    ) -> Option<DebugValue> {
        // Read-only values just display as labels
        if reg_value.read_only {
            self.render_watch_value(ui, reg_value, &current);
            return None;
        }

        match (&reg_value.value_type, &reg_value.constraints) {
            // Float with range -> slider
            (ValueType::F32, Some(c)) => {
                let mut v = current.as_f32();
                let changed = ui
                    .add(
                        egui::Slider::new(&mut v, c.min as f32..=c.max as f32)
                            .text(&reg_value.name),
                    )
                    .changed();
                if changed {
                    Some(DebugValue::F32(v))
                } else {
                    None
                }
            }
            // Float without range -> drag value
            (ValueType::F32, None) => {
                let mut v = current.as_f32();
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).speed(0.1));
                });
                if v != current.as_f32() {
                    Some(DebugValue::F32(v))
                } else {
                    None
                }
            }
            // I32 with range -> slider
            (ValueType::I32, Some(c)) => {
                let mut v = current.as_f32() as i32;
                let changed = ui
                    .add(
                        egui::Slider::new(&mut v, c.min as i32..=c.max as i32)
                            .text(&reg_value.name),
                    )
                    .changed();
                if changed {
                    Some(DebugValue::I32(v))
                } else {
                    None
                }
            }
            // I32 without range -> drag value
            (ValueType::I32, None) => {
                let mut v = current.as_f32() as i32;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v));
                });
                if v != current.as_f32() as i32 {
                    Some(DebugValue::I32(v))
                } else {
                    None
                }
            }
            // U32 with range -> slider
            (ValueType::U32, Some(c)) => {
                let mut v = current.as_f32() as u32;
                let changed = ui
                    .add(
                        egui::Slider::new(&mut v, c.min as u32..=c.max as u32)
                            .text(&reg_value.name),
                    )
                    .changed();
                if changed {
                    Some(DebugValue::U32(v))
                } else {
                    None
                }
            }
            // U32 without range -> drag value
            (ValueType::U32, None) => {
                let mut v = current.as_f32() as u32;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v));
                });
                if v != current.as_f32() as u32 {
                    Some(DebugValue::U32(v))
                } else {
                    None
                }
            }
            // U8 with range -> slider
            (ValueType::U8, Some(c)) => {
                let mut v = current.as_f32() as u8;
                let changed = ui
                    .add(egui::Slider::new(&mut v, c.min as u8..=c.max as u8).text(&reg_value.name))
                    .changed();
                if changed {
                    Some(DebugValue::U8(v))
                } else {
                    None
                }
            }
            // U8 without range
            (ValueType::U8, None) => {
                let mut v = current.as_f32() as i32; // Use i32 for DragValue
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).range(0..=255));
                });
                let v = v.clamp(0, 255) as u8;
                if v != current.as_f32() as u8 {
                    Some(DebugValue::U8(v))
                } else {
                    None
                }
            }
            // U16 with range -> slider
            (ValueType::U16, Some(c)) => {
                let mut v = current.as_f32() as u16;
                let changed = ui
                    .add(
                        egui::Slider::new(&mut v, c.min as u16..=c.max as u16)
                            .text(&reg_value.name),
                    )
                    .changed();
                if changed {
                    Some(DebugValue::U16(v))
                } else {
                    None
                }
            }
            // U16 without range
            (ValueType::U16, None) => {
                let mut v = current.as_f32() as i32;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).range(0..=65535));
                });
                let v = v.clamp(0, 65535) as u16;
                if v != current.as_f32() as u16 {
                    Some(DebugValue::U16(v))
                } else {
                    None
                }
            }
            // I16 with range -> slider
            (ValueType::I16, Some(c)) => {
                let mut v = current.as_f32() as i16;
                let changed = ui
                    .add(
                        egui::Slider::new(&mut v, c.min as i16..=c.max as i16)
                            .text(&reg_value.name),
                    )
                    .changed();
                if changed {
                    Some(DebugValue::I16(v))
                } else {
                    None
                }
            }
            // I16 without range
            (ValueType::I16, None) => {
                let mut v = current.as_f32() as i32;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).range(-32768..=32767));
                });
                let v = v.clamp(-32768, 32767) as i16;
                if v != current.as_f32() as i16 {
                    Some(DebugValue::I16(v))
                } else {
                    None
                }
            }
            // I8 with range -> slider
            (ValueType::I8, Some(c)) => {
                let mut v = current.as_f32() as i8;
                let changed = ui
                    .add(egui::Slider::new(&mut v, c.min as i8..=c.max as i8).text(&reg_value.name))
                    .changed();
                if changed {
                    Some(DebugValue::I8(v))
                } else {
                    None
                }
            }
            // I8 without range
            (ValueType::I8, None) => {
                let mut v = current.as_f32() as i32;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).range(-128..=127));
                });
                let v = v.clamp(-128, 127) as i8;
                if v != current.as_f32() as i8 {
                    Some(DebugValue::I8(v))
                } else {
                    None
                }
            }
            // Bool -> checkbox
            (ValueType::Bool, _) => {
                let mut v = current.as_bool();
                if ui.checkbox(&mut v, &reg_value.name).changed() {
                    Some(DebugValue::Bool(v))
                } else {
                    None
                }
            }
            // Vec2 -> two drag values
            (ValueType::Vec2, _) => {
                let (mut x, mut y) = current.as_vec2();
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    changed |= ui
                        .add(egui::DragValue::new(&mut x).speed(0.1).prefix("x:"))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut y).speed(0.1).prefix("y:"))
                        .changed();
                });
                if changed {
                    Some(DebugValue::Vec2 { x, y })
                } else {
                    None
                }
            }
            // Vec3 -> three drag values
            (ValueType::Vec3, _) => {
                let (mut x, mut y, mut z) = current.as_vec3();
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    changed |= ui
                        .add(egui::DragValue::new(&mut x).speed(0.1).prefix("x:"))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut y).speed(0.1).prefix("y:"))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut z).speed(0.1).prefix("z:"))
                        .changed();
                });
                if changed {
                    Some(DebugValue::Vec3 { x, y, z })
                } else {
                    None
                }
            }
            // Rect -> four drag values
            (ValueType::Rect, _) => {
                let (mut x, mut y, mut w, mut h) = current.as_rect();
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                });
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    changed |= ui.add(egui::DragValue::new(&mut x).prefix("x:")).changed();
                    changed |= ui.add(egui::DragValue::new(&mut y).prefix("y:")).changed();
                    changed |= ui.add(egui::DragValue::new(&mut w).prefix("w:")).changed();
                    changed |= ui.add(egui::DragValue::new(&mut h).prefix("h:")).changed();
                });
                if changed {
                    Some(DebugValue::Rect { x, y, w, h })
                } else {
                    None
                }
            }
            // Color -> color picker using Color32 directly (avoids f32 conversion issues)
            (ValueType::Color, _) => {
                let (r, g, b, a) = current.as_color();
                let mut color = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    changed = ui.color_edit_button_srgba(&mut color).changed();
                });
                if changed {
                    Some(DebugValue::Color {
                        r: color.r(),
                        g: color.g(),
                        b: color.b(),
                        a: color.a(),
                    })
                } else {
                    None
                }
            }
            // Fixed-point types - display as float, edit as float, convert back
            (ValueType::FixedI16Q8, _) => {
                let mut v = current.as_f32();
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).speed(0.01));
                    ui.weak("(Q8.8)");
                });
                if v != current.as_f32() {
                    // Convert float back to fixed-point
                    let raw = (v * 256.0).round() as i16;
                    Some(DebugValue::FixedI16Q8(raw))
                } else {
                    None
                }
            }
            (ValueType::FixedI32Q16, _) => {
                let mut v = current.as_f32();
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).speed(0.001));
                    ui.weak("(Q16.16)");
                });
                if v != current.as_f32() {
                    let raw = (v * 65536.0).round() as i32;
                    Some(DebugValue::FixedI32Q16(raw))
                } else {
                    None
                }
            }
            (ValueType::FixedI32Q8, _) => {
                let mut v = current.as_f32();
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).speed(0.01));
                    ui.weak("(Q24.8)");
                });
                if v != current.as_f32() {
                    let raw = (v * 256.0).round() as i32;
                    Some(DebugValue::FixedI32Q8(raw))
                } else {
                    None
                }
            }
            (ValueType::FixedI32Q24, _) => {
                let mut v = current.as_f32();
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    ui.add(egui::DragValue::new(&mut v).speed(0.0001));
                    ui.weak("(Q8.24)");
                });
                if v != current.as_f32() {
                    let raw = (v * 16777216.0).round() as i32;
                    Some(DebugValue::FixedI32Q24(raw))
                } else {
                    None
                }
            }
        }
    }

    /// Render a read-only watch value as a label
    fn render_watch_value(
        &self,
        ui: &mut egui::Ui,
        reg_value: &RegisteredValue,
        current: &DebugValue,
    ) {
        let value_str = match current {
            DebugValue::I8(v) => format!("{}", v),
            DebugValue::U8(v) => format!("{}", v),
            DebugValue::I16(v) => format!("{}", v),
            DebugValue::U16(v) => format!("{}", v),
            DebugValue::I32(v) => format!("{}", v),
            DebugValue::U32(v) => format!("{}", v),
            DebugValue::F32(v) => format!("{:.3}", v),
            DebugValue::Bool(v) => format!("{}", v),
            DebugValue::Vec2 { x, y } => format!("({:.2}, {:.2})", x, y),
            DebugValue::Vec3 { x, y, z } => format!("({:.2}, {:.2}, {:.2})", x, y, z),
            DebugValue::Rect { x, y, w, h } => format!("({}, {}, {}x{})", x, y, w, h),
            DebugValue::Color { r, g, b, a } => format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a),
            DebugValue::FixedI16Q8(v) => format!("{:.3}", *v as f32 / 256.0),
            DebugValue::FixedI32Q16(v) => format!("{:.3}", *v as f32 / 65536.0),
            DebugValue::FixedI32Q8(v) => format!("{:.3}", *v as f32 / 256.0),
            DebugValue::FixedI32Q24(v) => format!("{:.6}", *v as f32 / 16777216.0),
        };

        ui.horizontal(|ui| {
            // Dimmed color for read-only indicator
            ui.weak(format!("{}: {}", reg_value.name, value_str));
        });
    }

    /// Render export buttons
    fn render_export_buttons(
        &self,
        ui: &mut egui::Ui,
        registry: &DebugRegistry,
        read_value: &impl Fn(&RegisteredValue) -> Option<DebugValue>,
    ) {
        ui.horizontal(|ui| {
            if ui.button("📋 Copy as Rust").clicked() {
                let text = export_as_rust_flat(registry, read_value);
                ui.ctx().copy_text(text);
            }
        });
    }
}
