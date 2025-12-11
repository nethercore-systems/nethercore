//! Debug value serialization for reading/writing WASM memory
//!
//! Provides functions to convert between byte slices and debug values
//! for the debug inspection panel.

use emberware_core::debug::{DebugValue, ValueType};

/// Read a debug value from a byte slice
///
/// Returns `None` if the data slice is too short for the value type.
pub fn read_from_slice(data: &[u8], value_type: ValueType) -> Option<DebugValue> {
    Some(match value_type {
        ValueType::I8 | ValueType::U8 | ValueType::Bool => {
            if data.is_empty() {
                return None;
            }
            match value_type {
                ValueType::I8 => DebugValue::I8(data[0] as i8),
                ValueType::U8 => DebugValue::U8(data[0]),
                ValueType::Bool => DebugValue::Bool(data[0] != 0),
                _ => unreachable!(),
            }
        }
        ValueType::I16 => {
            let bytes: [u8; 2] = data.get(..2)?.try_into().ok()?;
            DebugValue::I16(i16::from_le_bytes(bytes))
        }
        ValueType::U16 => {
            let bytes: [u8; 2] = data.get(..2)?.try_into().ok()?;
            DebugValue::U16(u16::from_le_bytes(bytes))
        }
        ValueType::I32 => {
            let bytes: [u8; 4] = data.get(..4)?.try_into().ok()?;
            DebugValue::I32(i32::from_le_bytes(bytes))
        }
        ValueType::U32 => {
            let bytes: [u8; 4] = data.get(..4)?.try_into().ok()?;
            DebugValue::U32(u32::from_le_bytes(bytes))
        }
        ValueType::F32 => {
            let bytes: [u8; 4] = data.get(..4)?.try_into().ok()?;
            DebugValue::F32(f32::from_le_bytes(bytes))
        }
        ValueType::Vec2 => {
            let x = f32::from_le_bytes(data.get(0..4)?.try_into().ok()?);
            let y = f32::from_le_bytes(data.get(4..8)?.try_into().ok()?);
            DebugValue::Vec2 { x, y }
        }
        ValueType::Vec3 => {
            let x = f32::from_le_bytes(data.get(0..4)?.try_into().ok()?);
            let y = f32::from_le_bytes(data.get(4..8)?.try_into().ok()?);
            let z = f32::from_le_bytes(data.get(8..12)?.try_into().ok()?);
            DebugValue::Vec3 { x, y, z }
        }
        ValueType::Rect => {
            let x = i16::from_le_bytes(data.get(0..2)?.try_into().ok()?);
            let y = i16::from_le_bytes(data.get(2..4)?.try_into().ok()?);
            let w = i16::from_le_bytes(data.get(4..6)?.try_into().ok()?);
            let h = i16::from_le_bytes(data.get(6..8)?.try_into().ok()?);
            DebugValue::Rect { x, y, w, h }
        }
        ValueType::Color => {
            if data.len() < 4 {
                return None;
            }
            DebugValue::Color {
                r: data[0],
                g: data[1],
                b: data[2],
                a: data[3],
            }
        }
        ValueType::FixedI16Q8 => {
            let bytes: [u8; 2] = data.get(..2)?.try_into().ok()?;
            DebugValue::FixedI16Q8(i16::from_le_bytes(bytes))
        }
        ValueType::FixedI32Q16 => {
            let bytes: [u8; 4] = data.get(..4)?.try_into().ok()?;
            DebugValue::FixedI32Q16(i32::from_le_bytes(bytes))
        }
        ValueType::FixedI32Q8 => {
            let bytes: [u8; 4] = data.get(..4)?.try_into().ok()?;
            DebugValue::FixedI32Q8(i32::from_le_bytes(bytes))
        }
        ValueType::FixedI32Q24 => {
            let bytes: [u8; 4] = data.get(..4)?.try_into().ok()?;
            DebugValue::FixedI32Q24(i32::from_le_bytes(bytes))
        }
    })
}

/// Write a debug value to a byte slice
pub fn write_to_slice(data: &mut [u8], value: &DebugValue) {
    match value {
        DebugValue::I8(v) => data[0] = *v as u8,
        DebugValue::U8(v) => data[0] = *v,
        DebugValue::Bool(v) => data[0] = if *v { 1 } else { 0 },
        DebugValue::I16(v) => data[..2].copy_from_slice(&v.to_le_bytes()),
        DebugValue::U16(v) => data[..2].copy_from_slice(&v.to_le_bytes()),
        DebugValue::I32(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::U32(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::F32(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::Vec2 { x, y } => {
            data[0..4].copy_from_slice(&x.to_le_bytes());
            data[4..8].copy_from_slice(&y.to_le_bytes());
        }
        DebugValue::Vec3 { x, y, z } => {
            data[0..4].copy_from_slice(&x.to_le_bytes());
            data[4..8].copy_from_slice(&y.to_le_bytes());
            data[8..12].copy_from_slice(&z.to_le_bytes());
        }
        DebugValue::Rect { x, y, w, h } => {
            data[0..2].copy_from_slice(&x.to_le_bytes());
            data[2..4].copy_from_slice(&y.to_le_bytes());
            data[4..6].copy_from_slice(&w.to_le_bytes());
            data[6..8].copy_from_slice(&h.to_le_bytes());
        }
        DebugValue::Color { r, g, b, a } => {
            data[0] = *r;
            data[1] = *g;
            data[2] = *b;
            data[3] = *a;
        }
        DebugValue::FixedI16Q8(v) => data[..2].copy_from_slice(&v.to_le_bytes()),
        DebugValue::FixedI32Q16(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::FixedI32Q8(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
        DebugValue::FixedI32Q24(v) => data[..4].copy_from_slice(&v.to_le_bytes()),
    }
}
