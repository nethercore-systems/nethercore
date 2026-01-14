//! Widget rendering for debug values

use super::super::registry::RegisteredValue;
use super::super::types::DebugValue;

/// Handles rendering of value widgets
pub(super) struct ValueWidgetRenderer;

impl ValueWidgetRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render a widget for a single value
    ///
    /// Returns Some(new_value) if the value was changed, None otherwise.
    pub fn render_value_widget(
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
            (super::super::types::ValueType::F32, Some(c)) => {
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
            (super::super::types::ValueType::F32, None) => {
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
            (super::super::types::ValueType::I32, Some(c)) => {
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
            (super::super::types::ValueType::I32, None) => {
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
            (super::super::types::ValueType::U32, Some(c)) => {
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
            (super::super::types::ValueType::U32, None) => {
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
            (super::super::types::ValueType::U8, Some(c)) => {
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
            (super::super::types::ValueType::U8, None) => {
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
            (super::super::types::ValueType::U16, Some(c)) => {
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
            (super::super::types::ValueType::U16, None) => {
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
            (super::super::types::ValueType::I16, Some(c)) => {
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
            (super::super::types::ValueType::I16, None) => {
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
            (super::super::types::ValueType::I8, Some(c)) => {
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
            (super::super::types::ValueType::I8, None) => {
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
            (super::super::types::ValueType::Bool, _) => {
                let mut v = current.as_bool();
                if ui.checkbox(&mut v, &reg_value.name).changed() {
                    Some(DebugValue::Bool(v))
                } else {
                    None
                }
            }
            // Vec2 -> two drag values
            (super::super::types::ValueType::Vec2, _) => {
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
            (super::super::types::ValueType::Vec3, _) => {
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
            (super::super::types::ValueType::Rect, _) => {
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
            (super::super::types::ValueType::Color, _) => {
                let packed = if let DebugValue::Color(p) = current {
                    p
                } else {
                    0
                };
                // Unpack: 0xRRGGBBAA
                let r = ((packed >> 24) & 0xFF) as u8;
                let g = ((packed >> 16) & 0xFF) as u8;
                let b = ((packed >> 8) & 0xFF) as u8;
                let a = (packed & 0xFF) as u8;

                let mut color = egui::Rgba::from_rgba_unmultiplied(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                );
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label(&reg_value.name);
                    changed = egui::color_picker::color_edit_button_rgba(
                        ui,
                        &mut color,
                        egui::color_picker::Alpha::OnlyBlend,
                    )
                    .changed();
                });

                if changed {
                    // Repack: 0xRRGGBBAA
                    let r = (color.r() * 255.0).round() as u32;
                    let g = (color.g() * 255.0).round() as u32;
                    let b = (color.b() * 255.0).round() as u32;
                    let a = (color.a() * 255.0).round() as u32;
                    let packed = (r << 24) | (g << 16) | (b << 8) | a;
                    Some(DebugValue::Color(packed))
                } else {
                    None
                }
            }
            // Fixed-point types - display as float, edit as float, convert back
            (super::super::types::ValueType::FixedI16Q8, _) => {
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
            (super::super::types::ValueType::FixedI32Q16, _) => {
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
            (super::super::types::ValueType::FixedI32Q8, _) => {
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
            (super::super::types::ValueType::FixedI32Q24, _) => {
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
            DebugValue::Color(packed) => {
                let r = (packed >> 24) & 0xFF;
                let g = (packed >> 16) & 0xFF;
                let b = (packed >> 8) & 0xFF;
                let a = packed & 0xFF;
                format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
            }
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
}
