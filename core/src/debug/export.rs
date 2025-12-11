//! Debug value export utilities
//!
//! Formats registered debug values as Rust source code for easy copy-paste.

use super::registry::{DebugRegistry, RegisteredValue};
use super::types::{DebugValue, ValueType};

/// Export all registered values as flat Rust constants
///
/// Output format:
/// ```rust
/// // Exported from Debug Panel
/// const PLAYER_SPEED: f32 = 3.5;
/// const PLAYER_HEALTH: i32 = 100;
/// ```
pub fn export_as_rust_flat(
    registry: &DebugRegistry,
    read_value: impl Fn(&RegisteredValue) -> Option<DebugValue>,
) -> String {
    let mut output = String::from("// Exported from Debug Panel\n\n");

    for value in &registry.values {
        if let Some(current) = read_value(value) {
            let line = format_rust_const(&value.name, &value.value_type, &current);
            output.push_str(&line);
            output.push('\n');
        }
    }

    output
}

/// Format a single value as a Rust const declaration
fn format_rust_const(name: &str, value_type: &ValueType, value: &DebugValue) -> String {
    let const_name = name.to_uppercase().replace([' ', '-'], "_");

    match (value_type, value) {
        (ValueType::F32, DebugValue::F32(v)) => {
            // Use {:?} for round-trip safe formatting
            format!("const {}: f32 = {:?};", const_name, v)
        }
        (ValueType::I32, DebugValue::I32(v)) => {
            format!("const {}: i32 = {};", const_name, v)
        }
        (ValueType::U32, DebugValue::U32(v)) => {
            format!("const {}: u32 = {};", const_name, v)
        }
        (ValueType::I16, DebugValue::I16(v)) => {
            format!("const {}: i16 = {};", const_name, v)
        }
        (ValueType::U16, DebugValue::U16(v)) => {
            format!("const {}: u16 = {};", const_name, v)
        }
        (ValueType::I8, DebugValue::I8(v)) => {
            format!("const {}: i8 = {};", const_name, v)
        }
        (ValueType::U8, DebugValue::U8(v)) => {
            format!("const {}: u8 = {};", const_name, v)
        }
        (ValueType::Bool, DebugValue::Bool(v)) => {
            format!("const {}: bool = {};", const_name, v)
        }
        (ValueType::Vec2, DebugValue::Vec2 { x, y }) => {
            format!(
                "const {}: Vec2 = Vec2 {{ x: {:?}, y: {:?} }};",
                const_name, x, y
            )
        }
        (ValueType::Vec3, DebugValue::Vec3 { x, y, z }) => {
            format!(
                "const {}: Vec3 = Vec3 {{ x: {:?}, y: {:?}, z: {:?} }};",
                const_name, x, y, z
            )
        }
        (ValueType::Rect, DebugValue::Rect { x, y, w, h }) => {
            format!(
                "const {}: Rect = Rect {{ x: {}, y: {}, w: {}, h: {} }};",
                const_name, x, y, w, h
            )
        }
        (ValueType::Color, DebugValue::Color { r, g, b, a }) => {
            format!(
                "const {}: Color = Color {{ r: {}, g: {}, b: {}, a: {} }};",
                const_name, r, g, b, a
            )
        }
        // Fixed-point: export both raw value and float equivalent as comment
        (ValueType::FixedI16Q8, DebugValue::FixedI16Q8(raw)) => {
            let float_val = *raw as f32 / 256.0;
            format!(
                "const {}: FixedI16<U8> = FixedI16::from_bits({}); // ~{:.4}",
                const_name, raw, float_val
            )
        }
        (ValueType::FixedI32Q16, DebugValue::FixedI32Q16(raw)) => {
            let float_val = *raw as f32 / 65536.0;
            format!(
                "const {}: FixedI32<U16> = FixedI32::from_bits({}); // ~{:.6}",
                const_name, raw, float_val
            )
        }
        (ValueType::FixedI32Q8, DebugValue::FixedI32Q8(raw)) => {
            let float_val = *raw as f32 / 256.0;
            format!(
                "const {}: FixedI32<U8> = FixedI32::from_bits({}); // ~{:.4}",
                const_name, raw, float_val
            )
        }
        (ValueType::FixedI32Q24, DebugValue::FixedI32Q24(raw)) => {
            let float_val = *raw as f32 / 16777216.0;
            format!(
                "const {}: FixedI32<U24> = FixedI32::from_bits({}); // ~{:.8}",
                const_name, raw, float_val
            )
        }
        // Fallback for mismatched types
        _ => format!("// {} - type mismatch", const_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::registry::DebugRegistry;

    #[test]
    fn test_format_rust_const_f32() {
        let result = format_rust_const("speed", &ValueType::F32, &DebugValue::F32(3.5));
        assert_eq!(result, "const SPEED: f32 = 3.5;");
    }

    #[test]
    fn test_format_rust_const_i32() {
        let result = format_rust_const("health", &ValueType::I32, &DebugValue::I32(-100));
        assert_eq!(result, "const HEALTH: i32 = -100;");
    }

    #[test]
    fn test_format_rust_const_bool() {
        let result = format_rust_const("enabled", &ValueType::Bool, &DebugValue::Bool(true));
        assert_eq!(result, "const ENABLED: bool = true;");
    }

    #[test]
    fn test_format_rust_const_rect() {
        let result = format_rust_const(
            "hitbox",
            &ValueType::Rect,
            &DebugValue::Rect {
                x: 10,
                y: 20,
                w: 30,
                h: 40,
            },
        );
        assert_eq!(
            result,
            "const HITBOX: Rect = Rect { x: 10, y: 20, w: 30, h: 40 };"
        );
    }

    #[test]
    fn test_format_rust_const_vec2() {
        let result = format_rust_const(
            "position",
            &ValueType::Vec2,
            &DebugValue::Vec2 { x: 1.5, y: 2.5 },
        );
        assert_eq!(result, "const POSITION: Vec2 = Vec2 { x: 1.5, y: 2.5 };");
    }

    #[test]
    fn test_format_rust_const_color() {
        let result = format_rust_const(
            "tint",
            &ValueType::Color,
            &DebugValue::Color {
                r: 255,
                g: 128,
                b: 64,
                a: 255,
            },
        );
        assert_eq!(
            result,
            "const TINT: Color = Color { r: 255, g: 128, b: 64, a: 255 };"
        );
    }

    #[test]
    fn test_format_rust_const_fixed_point() {
        // Q16.16: 65536 raw = 1.0 float
        let result = format_rust_const(
            "scale",
            &ValueType::FixedI32Q16,
            &DebugValue::FixedI32Q16(65536),
        );
        assert!(result.contains("from_bits(65536)"));
        assert!(result.contains("1.0"));
    }

    #[test]
    fn test_export_flat() {
        let mut registry = DebugRegistry::new();
        registry.register("speed", 0x100, ValueType::F32, None);
        registry.register("health", 0x104, ValueType::I32, None);

        // Create a mock read function that returns test values
        let read_value = |reg_value: &RegisteredValue| -> Option<DebugValue> {
            match reg_value.name.as_str() {
                "speed" => Some(DebugValue::F32(3.5)),
                "health" => Some(DebugValue::I32(100)),
                _ => None,
            }
        };

        let output = export_as_rust_flat(&registry, read_value);
        assert!(output.contains("const SPEED: f32 = 3.5;"));
        assert!(output.contains("const HEALTH: i32 = 100;"));
    }

    #[test]
    fn test_name_sanitization() {
        // Names with spaces and dashes should be sanitized
        let result = format_rust_const("player speed", &ValueType::F32, &DebugValue::F32(1.0));
        assert!(result.contains("PLAYER_SPEED"));

        let result = format_rust_const("jump-force", &ValueType::F32, &DebugValue::F32(2.0));
        assert!(result.contains("JUMP_FORCE"));
    }
}
