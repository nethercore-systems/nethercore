//! Memory read/write operations for debug values

use super::super::types::{DebugValue, ValueType};
use super::RegisteredValue;
use wasmtime::{Caller, Memory};

impl super::DebugRegistry {
    /// Read a value from WASM memory
    pub fn read_value<T>(
        &self,
        caller: &mut Caller<'_, T>,
        value: &RegisteredValue,
    ) -> Option<DebugValue> {
        let memory = caller.get_export("memory").and_then(|e| e.into_memory())?;
        let data = memory.data(caller);
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr + size > data.len() {
            tracing::warn!(
                "debug: out of bounds read for {} at ptr {}",
                value.name,
                ptr
            );
            return None;
        }

        Some(self.read_value_from_slice(&data[ptr..ptr + size], value.value_type))
    }

    /// Read a value from WASM memory using a Memory handle
    pub fn read_value_with_memory<T>(
        &self,
        memory: &Memory,
        caller: &Caller<'_, T>,
        value: &RegisteredValue,
    ) -> Option<DebugValue> {
        let data = memory.data(caller);
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr + size > data.len() {
            return None;
        }

        Some(self.read_value_from_slice(&data[ptr..ptr + size], value.value_type))
    }

    /// Read a value from a byte slice
    pub fn read_value_from_slice(&self, data: &[u8], value_type: ValueType) -> DebugValue {
        match value_type {
            ValueType::I8 => DebugValue::I8(data[0] as i8),
            ValueType::U8 => DebugValue::U8(data[0]),
            ValueType::Bool => DebugValue::Bool(data[0] != 0),
            ValueType::I16 => {
                let bytes: [u8; 2] = data[..2].try_into().unwrap();
                DebugValue::I16(i16::from_le_bytes(bytes))
            }
            ValueType::U16 => {
                let bytes: [u8; 2] = data[..2].try_into().unwrap();
                DebugValue::U16(u16::from_le_bytes(bytes))
            }
            ValueType::I32 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::I32(i32::from_le_bytes(bytes))
            }
            ValueType::U32 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::U32(u32::from_le_bytes(bytes))
            }
            ValueType::F32 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::F32(f32::from_le_bytes(bytes))
            }
            ValueType::Vec2 => {
                let x = f32::from_le_bytes(data[0..4].try_into().unwrap());
                let y = f32::from_le_bytes(data[4..8].try_into().unwrap());
                DebugValue::Vec2 { x, y }
            }
            ValueType::Vec3 => {
                let x = f32::from_le_bytes(data[0..4].try_into().unwrap());
                let y = f32::from_le_bytes(data[4..8].try_into().unwrap());
                let z = f32::from_le_bytes(data[8..12].try_into().unwrap());
                DebugValue::Vec3 { x, y, z }
            }
            ValueType::Rect => {
                let x = i16::from_le_bytes(data[0..2].try_into().unwrap());
                let y = i16::from_le_bytes(data[2..4].try_into().unwrap());
                let w = i16::from_le_bytes(data[4..6].try_into().unwrap());
                let h = i16::from_le_bytes(data[6..8].try_into().unwrap());
                DebugValue::Rect { x, y, w, h }
            }
            ValueType::Color => {
                // Colors stored as u32 in 0xRRGGBBAA format
                DebugValue::Color(u32::from_le_bytes(data[0..4].try_into().unwrap()))
            }
            ValueType::FixedI16Q8 => {
                let bytes: [u8; 2] = data[..2].try_into().unwrap();
                DebugValue::FixedI16Q8(i16::from_le_bytes(bytes))
            }
            ValueType::FixedI32Q16 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::FixedI32Q16(i32::from_le_bytes(bytes))
            }
            ValueType::FixedI32Q8 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::FixedI32Q8(i32::from_le_bytes(bytes))
            }
            ValueType::FixedI32Q24 => {
                let bytes: [u8; 4] = data[..4].try_into().unwrap();
                DebugValue::FixedI32Q24(i32::from_le_bytes(bytes))
            }
        }
    }

    /// Write a value to WASM memory
    ///
    /// Returns true if the value was written successfully and differed from the previous value.
    pub fn write_value<T>(
        &self,
        caller: &mut Caller<'_, T>,
        value: &RegisteredValue,
        new_val: &DebugValue,
    ) -> bool {
        let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
            return false;
        };

        let data = memory.data_mut(caller);
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr + size > data.len() {
            tracing::warn!(
                "debug: out of bounds write for {} at ptr {}",
                value.name,
                ptr
            );
            return false;
        }

        self.write_value_to_slice(&mut data[ptr..ptr + size], new_val)
    }

    /// Write a value to WASM memory using a Memory handle
    ///
    /// Returns true if the value was written successfully.
    pub fn write_value_with_memory<T>(
        &self,
        memory: &Memory,
        caller: &mut Caller<'_, T>,
        value: &RegisteredValue,
        new_val: &DebugValue,
    ) -> bool {
        let data = memory.data_mut(caller);
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check
        if ptr + size > data.len() {
            return false;
        }

        self.write_value_to_slice(&mut data[ptr..ptr + size], new_val)
    }

    /// Write a value to a byte slice, returns true if written
    pub fn write_value_to_slice(&self, data: &mut [u8], new_val: &DebugValue) -> bool {
        match new_val {
            DebugValue::I8(v) => {
                data[0] = *v as u8;
            }
            DebugValue::U8(v) => {
                data[0] = *v;
            }
            DebugValue::Bool(v) => {
                data[0] = if *v { 1 } else { 0 };
            }
            DebugValue::I16(v) => {
                data[..2].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::U16(v) => {
                data[..2].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::I32(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::U32(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::F32(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
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
            DebugValue::Color(packed) => {
                data[..4].copy_from_slice(&packed.to_le_bytes());
            }
            DebugValue::FixedI16Q8(v) => {
                data[..2].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::FixedI32Q16(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::FixedI32Q8(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
            DebugValue::FixedI32Q24(v) => {
                data[..4].copy_from_slice(&v.to_le_bytes());
            }
        }
        true
    }
}
