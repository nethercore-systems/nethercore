//! Debug inspection type definitions
//!
//! Core types for the debug value inspection system.

/// Value type identifier for registered debug values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    // Primitives
    I8,
    I16,
    I32,
    U8,
    U16,
    U32,
    F32,
    Bool,

    // Compound types
    Vec2,  // { x: f32, y: f32 }
    Vec3,  // { x: f32, y: f32, z: f32 }
    Rect,  // { x: i16, y: i16, w: i16, h: i16 }
    Color, // { r: u8, g: u8, b: u8, a: u8 }

    // Fixed-point types
    FixedI16Q8,  // Q8.8 - 8 integer bits, 8 fractional bits
    FixedI32Q16, // Q16.16 - 16 integer bits, 16 fractional bits
    FixedI32Q8,  // Q24.8 - 24 integer bits, 8 fractional bits
    FixedI32Q24, // Q8.24 - 8 integer bits, 24 fractional bits
}

impl ValueType {
    /// Get the byte size of this value type
    pub fn byte_size(&self) -> usize {
        match self {
            ValueType::I8 | ValueType::U8 | ValueType::Bool => 1,
            ValueType::I16 | ValueType::U16 | ValueType::FixedI16Q8 => 2,
            ValueType::I32
            | ValueType::U32
            | ValueType::F32
            | ValueType::FixedI32Q16
            | ValueType::FixedI32Q8
            | ValueType::FixedI32Q24 => 4,
            ValueType::Vec2 => 8,  // 2 * f32
            ValueType::Vec3 => 12, // 3 * f32
            ValueType::Rect => 8,  // 4 * i16
            ValueType::Color => 4, // 4 * u8
        }
    }

    /// Get a human-readable type name
    pub fn type_name(&self) -> &'static str {
        match self {
            ValueType::I8 => "i8",
            ValueType::I16 => "i16",
            ValueType::I32 => "i32",
            ValueType::U8 => "u8",
            ValueType::U16 => "u16",
            ValueType::U32 => "u32",
            ValueType::F32 => "f32",
            ValueType::Bool => "bool",
            ValueType::Vec2 => "Vec2",
            ValueType::Vec3 => "Vec3",
            ValueType::Rect => "Rect",
            ValueType::Color => "Color",
            ValueType::FixedI16Q8 => "FixedI16<U8>",
            ValueType::FixedI32Q16 => "FixedI32<U16>",
            ValueType::FixedI32Q8 => "FixedI32<U8>",
            ValueType::FixedI32Q24 => "FixedI32<U24>",
        }
    }

    /// Get the number of fractional bits for fixed-point types
    pub fn fractional_bits(&self) -> Option<u32> {
        match self {
            ValueType::FixedI16Q8 => Some(8),
            ValueType::FixedI32Q16 => Some(16),
            ValueType::FixedI32Q8 => Some(8),
            ValueType::FixedI32Q24 => Some(24),
            _ => None,
        }
    }
}

/// Runtime value representation for debug inspection
#[derive(Debug, Clone, PartialEq)]
pub enum DebugValue {
    I8(i8),
    I16(i16),
    I32(i32),
    U8(u8),
    U16(u16),
    U32(u32),
    F32(f32),
    Bool(bool),
    Vec2 { x: f32, y: f32 },
    Vec3 { x: f32, y: f32, z: f32 },
    Rect { x: i16, y: i16, w: i16, h: i16 },
    Color(u32), // 0xRRGGBBAA format
    // Fixed-point stored as raw bits, converted to float for display
    FixedI16Q8(i16),
    FixedI32Q16(i32),
    FixedI32Q8(i32),
    FixedI32Q24(i32),
}

impl DebugValue {
    /// Convert to f32 for numeric types (useful for sliders)
    pub fn as_f32(&self) -> f32 {
        match self {
            DebugValue::I8(v) => *v as f32,
            DebugValue::I16(v) => *v as f32,
            DebugValue::I32(v) => *v as f32,
            DebugValue::U8(v) => *v as f32,
            DebugValue::U16(v) => *v as f32,
            DebugValue::U32(v) => *v as f32,
            DebugValue::F32(v) => *v,
            DebugValue::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            DebugValue::FixedI16Q8(v) => *v as f32 / 256.0,
            DebugValue::FixedI32Q16(v) => *v as f32 / 65536.0,
            DebugValue::FixedI32Q8(v) => *v as f32 / 256.0,
            DebugValue::FixedI32Q24(v) => *v as f32 / 16777216.0,
            _ => 0.0, // Compound types don't have a single f32 representation
        }
    }

    /// Get bool value
    pub fn as_bool(&self) -> bool {
        match self {
            DebugValue::Bool(v) => *v,
            _ => false,
        }
    }

    /// Get Rect components (x, y, w, h)
    pub fn as_rect(&self) -> (i16, i16, i16, i16) {
        match self {
            DebugValue::Rect { x, y, w, h } => (*x, *y, *w, *h),
            _ => (0, 0, 0, 0),
        }
    }

    /// Get Vec2 components (x, y)
    pub fn as_vec2(&self) -> (f32, f32) {
        match self {
            DebugValue::Vec2 { x, y } => (*x, *y),
            _ => (0.0, 0.0),
        }
    }

    /// Get Vec3 components (x, y, z)
    pub fn as_vec3(&self) -> (f32, f32, f32) {
        match self {
            DebugValue::Vec3 { x, y, z } => (*x, *y, *z),
            _ => (0.0, 0.0, 0.0),
        }
    }

    /// Get the value type for this debug value
    pub fn value_type(&self) -> ValueType {
        match self {
            DebugValue::I8(_) => ValueType::I8,
            DebugValue::I16(_) => ValueType::I16,
            DebugValue::I32(_) => ValueType::I32,
            DebugValue::U8(_) => ValueType::U8,
            DebugValue::U16(_) => ValueType::U16,
            DebugValue::U32(_) => ValueType::U32,
            DebugValue::F32(_) => ValueType::F32,
            DebugValue::Bool(_) => ValueType::Bool,
            DebugValue::Vec2 { .. } => ValueType::Vec2,
            DebugValue::Vec3 { .. } => ValueType::Vec3,
            DebugValue::Rect { .. } => ValueType::Rect,
            DebugValue::Color(_) => ValueType::Color,
            DebugValue::FixedI16Q8(_) => ValueType::FixedI16Q8,
            DebugValue::FixedI32Q16(_) => ValueType::FixedI32Q16,
            DebugValue::FixedI32Q8(_) => ValueType::FixedI32Q8,
            DebugValue::FixedI32Q24(_) => ValueType::FixedI32Q24,
        }
    }
}

/// Range constraints for numeric values
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min: f64,
    pub max: f64,
}

impl Constraints {
    /// Create new constraints with min and max values
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// Clamp a value to these constraints
    pub fn clamp(&self, value: f64) -> f64 {
        value.clamp(self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_byte_sizes() {
        assert_eq!(ValueType::I8.byte_size(), 1);
        assert_eq!(ValueType::U8.byte_size(), 1);
        assert_eq!(ValueType::Bool.byte_size(), 1);
        assert_eq!(ValueType::I16.byte_size(), 2);
        assert_eq!(ValueType::U16.byte_size(), 2);
        assert_eq!(ValueType::I32.byte_size(), 4);
        assert_eq!(ValueType::U32.byte_size(), 4);
        assert_eq!(ValueType::F32.byte_size(), 4);
        assert_eq!(ValueType::Vec2.byte_size(), 8);
        assert_eq!(ValueType::Vec3.byte_size(), 12);
        assert_eq!(ValueType::Rect.byte_size(), 8);
        assert_eq!(ValueType::Color.byte_size(), 4);
        assert_eq!(ValueType::FixedI16Q8.byte_size(), 2);
        assert_eq!(ValueType::FixedI32Q16.byte_size(), 4);
    }

    #[test]
    fn test_debug_value_as_f32() {
        assert_eq!(DebugValue::I32(100).as_f32(), 100.0);
        assert_eq!(DebugValue::F32(3.14).as_f32(), 3.14);
        assert_eq!(DebugValue::Bool(true).as_f32(), 1.0);
        assert_eq!(DebugValue::Bool(false).as_f32(), 0.0);
        // Q16.16: 65536 raw = 1.0 float
        assert_eq!(DebugValue::FixedI32Q16(65536).as_f32(), 1.0);
        // Q8.8: 256 raw = 1.0 float
        assert_eq!(DebugValue::FixedI16Q8(256).as_f32(), 1.0);
    }

    #[test]
    fn test_constraints_clamp() {
        let c = Constraints::new(0.0, 100.0);
        assert_eq!(c.clamp(50.0), 50.0);
        assert_eq!(c.clamp(-10.0), 0.0);
        assert_eq!(c.clamp(150.0), 100.0);
    }

    #[test]
    fn test_debug_value_type() {
        assert_eq!(DebugValue::I32(0).value_type(), ValueType::I32);
        assert_eq!(DebugValue::F32(0.0).value_type(), ValueType::F32);
        assert_eq!(
            DebugValue::Vec2 { x: 0.0, y: 0.0 }.value_type(),
            ValueType::Vec2
        );
        assert_eq!(DebugValue::Color(0xFF0000FF).value_type(), ValueType::Color);
    }
}
