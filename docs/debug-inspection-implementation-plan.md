# Debug Inspection System: Implementation Plan

This document provides a phased implementation plan for the Debug Inspection System specified in `debug-inspection-spec.md`. The plan prioritizes robust architecture over quick implementation.

**Target audience:** AI agents implementing this feature. Each phase includes complete code, file paths, integration points, and test cases.

---

## Design Decisions (from Q&A)

| Topic | Decision |
|-------|----------|
| Memory safety | WASM can't access outside its memory; pre-allocate max RAM, disallow buffer growth |
| Threading | Single-threaded; no synchronization needed |
| Registration lifetime | `init()` only; no dynamic registration for now |
| egui location | Core dependency (enables all future consoles automatically) |
| Group errors | Panic on mismatch; auto-close unclosed groups at registration end |
| Time scale + GGRS | Debug features disabled during netplay; single-player/local only |
| Console scope | Console-agnostic in core; no console-specific code needed |
| Callback re-entrancy | Allowed; if WASM changes values, that's fine |
| Release linking | Wasmtime ignores unused definitions; always register debug FFI |
| Float precision | Full round-trip precision for export (use `{:?}` format) |
| Fixed-point | Support `fixed` crate types (e.g., `FixedI32<U16>` for Q16.16) |

---

## Existing Code Context

Before implementing, understand these existing patterns:

### Key Files to Reference

| File | Contains | Relevant Patterns |
|------|----------|-------------------|
| `core/src/ffi.rs` | Common FFI functions | How to register FFI with `Linker`, how to access `GameStateWithConsole` via `Caller` |
| `core/src/wasm/state.rs` | `GameState`, `GameStateWithConsole` | Where to add `debug_registry` field |
| `core/src/wasm/mod.rs` | `GameInstance` | Where to finalize registration after `init()`, how to access WASM memory |
| `core/src/console.rs` | `Console` trait, `ConsoleInput` | Generic bounds needed for FFI functions |
| `core/src/app/config.rs` | `Config` struct | Pattern for adding `DebugConfig` |
| `core/src/rollback/session.rs` | `RollbackSession`, `SessionType` | How to check if networked (for disabling debug) |
| `core/src/test_utils.rs` | `TestConsole`, `TestInput` | Test utilities for unit tests |

### FFI Pattern (from `core/src/ffi.rs`)

All FFI functions follow this pattern:

```rust
fn some_ffi_function<I: ConsoleInput, S: Send + Default + 'static>(
    caller: Caller<'_, GameStateWithConsole<I, S>>,  // or `mut caller` if modifying state
    // ... WASM params (u32, f32, etc.)
) -> ReturnType {
    // Access game state
    let state = caller.data();      // immutable
    let state = caller.data_mut();  // mutable

    // Access WASM memory
    let memory = caller.data().game.memory.unwrap();
    let mem_data = memory.data(&caller);           // &[u8]
    let mem_data = memory.data_mut(&mut caller);   // &mut [u8]
}
```

### Generic Bounds

The `GameStateWithConsole<I, S>` type requires:
- `I: ConsoleInput` (which implies `Clone + Copy + Default + Pod + Zeroable + Send + Sync + 'static`)
- `S: Send + Default + 'static` (console-specific state)

### Current GameStateWithConsole Structure (from `core/src/wasm/state.rs`)

```rust
pub struct GameStateWithConsole<I: ConsoleInput, S> {
    pub game: GameState<I>,
    pub console: S,
    // debug_registry will be added here in Phase 7
}
```

---

## Architecture Overview

```
+---------------------------------------------------------------------+
|                         core/src/debug/                             |
|                                                                     |
|  +-------------+  +-------------+  +----------------------------+   |
|  |   registry  |  |    types    |  |          ffi               |   |
|  |             |  |             |  |                            |   |
|  | DebugRegist |  | ValueType   |  | debug_register_f32()       |   |
|  | RegisterVal |  | DebugValue  |  | debug_register_i32()       |   |
|  | GroupStack  |  | Constraints |  | debug_group_begin()        |   |
|  +------+------+  +-------------+  | ...                        |   |
|         |                          +-------------+--------------+   |
|         |                                        |                  |
|         v                                        |                  |
|  +-------------+  +-------------+                |                  |
|  |    panel    |  | frame_ctrl  |                |                  |
|  |             |  |             |                |                  |
|  | DebugPanel  |  | FrameContrl |<---------------+                  |
|  | (egui UI)   |  | pause/step  |                                   |
|  | tree render |  | time_scale  |                                   |
|  +------+------+  +-------------+                                   |
|         |                                                           |
|         v                                                           |
|  +-------------+                                                    |
|  |    export   |                                                    |
|  |             |                                                    |
|  | Rust const  |                                                    |
|  | formatting  |                                                    |
|  +-------------+                                                    |
+---------------------------------------------------------------------+
```

---

## Phase 1: Core Types

**Goal:** Define foundational types with no dependencies on other debug modules.

**File to create:** `core/src/debug/types.rs`

**Dependencies:** None

### Complete Implementation

```rust
//! Debug inspection value types
//!
//! Defines the types of values that can be registered for debug inspection,
//! along with runtime value representations and constraints.

/// Supported value types for debug inspection
///
/// Each variant corresponds to a specific memory layout in WASM linear memory.
/// The byte size and serialization format are fixed per variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    // ========== Primitives ==========
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,

    // ========== Compound types (known layouts) ==========
    /// Vec2: { x: f32, y: f32 } - 8 bytes
    Vec2,
    /// Vec3: { x: f32, y: f32, z: f32 } - 12 bytes
    Vec3,
    /// Vec4: { x: f32, y: f32, z: f32, w: f32 } - 16 bytes
    Vec4,
    /// Rect: { x: i16, y: i16, w: i16, h: i16 } - 8 bytes
    Rect,
    /// Color: { r: u8, g: u8, b: u8, a: u8 } - 4 bytes
    Color,

    // ========== Fixed-point (compatible with `fixed` crate) ==========
    /// Q8.8: FixedI16<U8> - 16-bit signed, 8 fractional bits
    FixedI16F8,
    /// Q24.8: FixedI32<U8> - 32-bit signed, 8 fractional bits
    FixedI32F8,
    /// Q16.16: FixedI32<U16> - 32-bit signed, 16 fractional bits (most common)
    FixedI32F16,
    /// Q8.24: FixedI32<U24> - 32-bit signed, 24 fractional bits
    FixedI32F24,
}

impl ValueType {
    /// Get the byte size of this value type in WASM memory
    #[inline]
    pub const fn byte_size(&self) -> usize {
        match self {
            Self::I8 | Self::U8 | Self::Bool => 1,
            Self::I16 | Self::U16 | Self::FixedI16F8 => 2,
            Self::I32 | Self::U32 | Self::F32 | Self::FixedI32F8
            | Self::FixedI32F16 | Self::FixedI32F24 | Self::Color => 4,
            Self::I64 | Self::U64 | Self::F64 | Self::Vec2 | Self::Rect => 8,
            Self::Vec3 => 12,
            Self::Vec4 => 16,
        }
    }

    /// Get the display name for this type (used in export)
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Bool => "bool",
            Self::Vec2 => "Vec2",
            Self::Vec3 => "Vec3",
            Self::Vec4 => "Vec4",
            Self::Rect => "Rect",
            Self::Color => "Color",
            Self::FixedI16F8 => "FixedI16<U8>",
            Self::FixedI32F8 => "FixedI32<U8>",
            Self::FixedI32F16 => "FixedI32<U16>",
            Self::FixedI32F24 => "FixedI32<U24>",
        }
    }

    /// Get the fractional bits for fixed-point types (0 for non-fixed types)
    pub const fn fractional_bits(&self) -> u32 {
        match self {
            Self::FixedI16F8 | Self::FixedI32F8 => 8,
            Self::FixedI32F16 => 16,
            Self::FixedI32F24 => 24,
            _ => 0,
        }
    }
}

/// Runtime value representation for reading/editing
///
/// This enum holds the actual value read from WASM memory, allowing
/// type-safe manipulation in the debug panel.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Vec2 { x: f32, y: f32 },
    Vec3 { x: f32, y: f32, z: f32 },
    Vec4 { x: f32, y: f32, z: f32, w: f32 },
    Rect { x: i16, y: i16, w: i16, h: i16 },
    Color { r: u8, g: u8, b: u8, a: u8 },
    /// Fixed-point stored as raw bits
    FixedI16F8(i16),
    FixedI32F8(i32),
    FixedI32F16(i32),
    FixedI32F24(i32),
}

impl DebugValue {
    /// Convert fixed-point raw bits to f64 for display
    ///
    /// Returns `None` for non-fixed-point types.
    pub fn fixed_to_f64(&self) -> Option<f64> {
        match self {
            Self::FixedI16F8(raw) => Some(*raw as f64 / 256.0),
            Self::FixedI32F8(raw) => Some(*raw as f64 / 256.0),
            Self::FixedI32F16(raw) => Some(*raw as f64 / 65536.0),
            Self::FixedI32F24(raw) => Some(*raw as f64 / 16777216.0),
            _ => None,
        }
    }

    /// Create a fixed-point value from f64 display value
    pub fn fixed_from_f64(value_type: ValueType, display: f64) -> Option<Self> {
        match value_type {
            ValueType::FixedI16F8 => {
                let raw = (display * 256.0).round().clamp(i16::MIN as f64, i16::MAX as f64) as i16;
                Some(Self::FixedI16F8(raw))
            }
            ValueType::FixedI32F8 => {
                let raw = (display * 256.0).round().clamp(i32::MIN as f64, i32::MAX as f64) as i32;
                Some(Self::FixedI32F8(raw))
            }
            ValueType::FixedI32F16 => {
                let raw = (display * 65536.0).round().clamp(i32::MIN as f64, i32::MAX as f64) as i32;
                Some(Self::FixedI32F16(raw))
            }
            ValueType::FixedI32F24 => {
                let raw = (display * 16777216.0).round().clamp(i32::MIN as f64, i32::MAX as f64) as i32;
                Some(Self::FixedI32F24(raw))
            }
            _ => None,
        }
    }
}

/// Optional constraints for numeric values
///
/// When present, the debug panel renders a slider instead of a drag value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    /// Minimum allowed value (inclusive)
    pub min: f64,
    /// Maximum allowed value (inclusive)
    pub max: f64,
}

impl Constraints {
    /// Create new constraints
    pub fn new(min: f64, max: f64) -> Self {
        debug_assert!(min <= max, "Constraints min must be <= max");
        Self { min, max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== ValueType Tests ==========

    #[test]
    fn test_byte_sizes() {
        assert_eq!(ValueType::I8.byte_size(), 1);
        assert_eq!(ValueType::U8.byte_size(), 1);
        assert_eq!(ValueType::Bool.byte_size(), 1);
        assert_eq!(ValueType::I16.byte_size(), 2);
        assert_eq!(ValueType::U16.byte_size(), 2);
        assert_eq!(ValueType::I32.byte_size(), 4);
        assert_eq!(ValueType::F32.byte_size(), 4);
        assert_eq!(ValueType::I64.byte_size(), 8);
        assert_eq!(ValueType::F64.byte_size(), 8);
        assert_eq!(ValueType::Vec2.byte_size(), 8);
        assert_eq!(ValueType::Vec3.byte_size(), 12);
        assert_eq!(ValueType::Vec4.byte_size(), 16);
        assert_eq!(ValueType::Rect.byte_size(), 8);
        assert_eq!(ValueType::Color.byte_size(), 4);
        assert_eq!(ValueType::FixedI16F8.byte_size(), 2);
        assert_eq!(ValueType::FixedI32F16.byte_size(), 4);
    }

    #[test]
    fn test_fractional_bits() {
        assert_eq!(ValueType::F32.fractional_bits(), 0);
        assert_eq!(ValueType::I32.fractional_bits(), 0);
        assert_eq!(ValueType::FixedI16F8.fractional_bits(), 8);
        assert_eq!(ValueType::FixedI32F8.fractional_bits(), 8);
        assert_eq!(ValueType::FixedI32F16.fractional_bits(), 16);
        assert_eq!(ValueType::FixedI32F24.fractional_bits(), 24);
    }

    // ========== Fixed-Point Conversion Tests ==========

    #[test]
    fn test_fixed_i32f16_roundtrip() {
        // Q16.16: 1.5 = 1.5 * 65536 = 98304
        let raw = 98304i32;
        let value = DebugValue::FixedI32F16(raw);
        let display = value.fixed_to_f64().unwrap();
        assert!((display - 1.5).abs() < 0.0001);

        // Convert back
        let restored = DebugValue::fixed_from_f64(ValueType::FixedI32F16, display).unwrap();
        assert_eq!(restored, DebugValue::FixedI32F16(raw));
    }

    #[test]
    fn test_fixed_i32f16_negative() {
        // -2.25 in Q16.16
        let raw = (-2.25 * 65536.0) as i32;
        let value = DebugValue::FixedI32F16(raw);
        let display = value.fixed_to_f64().unwrap();
        assert!((display - (-2.25)).abs() < 0.0001);
    }

    #[test]
    fn test_fixed_i16f8_range() {
        // Q8.8 max positive: 127.99609375
        let max_raw = i16::MAX;
        let value = DebugValue::FixedI16F8(max_raw);
        let display = value.fixed_to_f64().unwrap();
        assert!(display > 127.0 && display < 128.0);
    }

    // ========== Constraints Tests ==========

    #[test]
    fn test_constraints_creation() {
        let c = Constraints::new(0.0, 100.0);
        assert_eq!(c.min, 0.0);
        assert_eq!(c.max, 100.0);
    }
}
```

### Test Cases Summary for Phase 1

| Test | Purpose |
|------|---------|
| `test_byte_sizes` | Verify each ValueType returns correct byte size |
| `test_fractional_bits` | Verify fixed-point types return correct fractional bits |
| `test_fixed_i32f16_roundtrip` | Verify Q16.16 conversion preserves value |
| `test_fixed_i32f16_negative` | Verify negative fixed-point works |
| `test_fixed_i16f8_range` | Verify Q8.8 range boundaries |
| `test_constraints_creation` | Verify Constraints struct construction |

---

## Phase 2: Registry

**Goal:** Implement value registration and memory access.

**File to create:** `core/src/debug/registry.rs`

**Dependencies:** Phase 1 (`types.rs`)

### Complete Implementation

```rust
//! Debug value registry
//!
//! Stores registered debug values and provides memory access operations.

use crate::debug::types::{Constraints, DebugValue, ValueType};

/// A value registered for debug inspection
#[derive(Debug, Clone)]
pub struct RegisteredValue {
    /// Display name (from game)
    pub name: String,
    /// Full hierarchical path (e.g., "player/attacks/punch_hitbox")
    pub full_path: String,
    /// Pointer into WASM linear memory
    pub wasm_ptr: u32,
    /// Type of the value
    pub value_type: ValueType,
    /// Optional min/max constraints (renders as slider)
    pub constraints: Option<Constraints>,
}

/// The debug registry storing all registered values
///
/// Values are registered during `init()` via FFI calls. After init completes,
/// `finalize_registration()` is called to close any unclosed groups and
/// mark registration as complete.
#[derive(Debug, Default)]
pub struct DebugRegistry {
    /// All registered values (flat list, tree built from full_path at render time)
    values: Vec<RegisteredValue>,
    /// Current group stack during registration
    group_stack: Vec<String>,
    /// Whether registration is complete (after init() returns)
    registration_complete: bool,
    /// Optional callback function pointer in WASM (index into function table)
    change_callback: Option<u32>,
}

impl DebugRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a value for debug inspection
    ///
    /// # Arguments
    /// * `name` - Display name for the value
    /// * `wasm_ptr` - Pointer into WASM linear memory
    /// * `value_type` - Type of the value (determines byte size and UI widget)
    /// * `constraints` - Optional min/max constraints (renders as slider if present)
    ///
    /// # Panics
    /// Panics if called after `finalize_registration()`.
    pub fn register(
        &mut self,
        name: &str,
        wasm_ptr: u32,
        value_type: ValueType,
        constraints: Option<Constraints>,
    ) {
        assert!(
            !self.registration_complete,
            "Cannot register debug values after init() completes"
        );

        // Build full path from group stack
        let full_path = if self.group_stack.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", self.group_stack.join("/"), name)
        };

        self.values.push(RegisteredValue {
            name: name.to_string(),
            full_path,
            wasm_ptr,
            value_type,
            constraints,
        });
    }

    /// Begin a new group (creates hierarchy in the debug panel)
    ///
    /// Groups can be nested. Call `group_end()` to close.
    ///
    /// # Panics
    /// Panics if called after `finalize_registration()`.
    pub fn group_begin(&mut self, name: &str) {
        assert!(
            !self.registration_complete,
            "Cannot modify groups after init() completes"
        );
        self.group_stack.push(name.to_string());
    }

    /// End the current group
    ///
    /// # Panics
    /// Panics if there's no matching `group_begin()` or if called after finalization.
    pub fn group_end(&mut self) {
        assert!(
            !self.registration_complete,
            "Cannot modify groups after init() completes"
        );
        assert!(
            !self.group_stack.is_empty(),
            "debug_group_end() called without matching debug_group_begin()"
        );
        self.group_stack.pop();
    }

    /// Finalize registration after init() completes
    ///
    /// Auto-closes any unclosed groups and marks registration as complete.
    /// After this, no more values can be registered.
    pub fn finalize_registration(&mut self) {
        if !self.group_stack.is_empty() {
            log::warn!(
                "Debug registration: {} unclosed group(s), auto-closing: {:?}",
                self.group_stack.len(),
                self.group_stack
            );
            self.group_stack.clear();
        }
        self.registration_complete = true;
        log::info!("Debug registration complete: {} values registered", self.values.len());
    }

    /// Set the change callback function pointer
    pub fn set_change_callback(&mut self, callback_ptr: u32) {
        self.change_callback = Some(callback_ptr);
    }

    /// Get the change callback function pointer
    pub fn change_callback(&self) -> Option<u32> {
        self.change_callback
    }

    /// Get all registered values
    pub fn values(&self) -> &[RegisteredValue] {
        &self.values
    }

    /// Get the number of registered values
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Read a value from WASM memory
    ///
    /// Returns `None` if the pointer is out of bounds or memory access fails.
    pub fn read_value(&self, memory: &[u8], value: &RegisteredValue) -> Option<DebugValue> {
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check with overflow protection
        let end = ptr.checked_add(size)?;
        if end > memory.len() {
            log::warn!(
                "Debug read out of bounds: '{}' at 0x{:08x} (size {}, memory len {})",
                value.full_path,
                ptr,
                size,
                memory.len()
            );
            return None;
        }

        let bytes = &memory[ptr..end];
        Self::parse_value(value.value_type, bytes)
    }

    /// Write a value to WASM memory
    ///
    /// Returns `true` if successful, `false` if out of bounds.
    pub fn write_value(
        &self,
        memory: &mut [u8],
        value: &RegisteredValue,
        new_val: &DebugValue,
    ) -> bool {
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();

        // Bounds check with overflow protection
        let end = match ptr.checked_add(size) {
            Some(e) if e <= memory.len() => e,
            _ => {
                log::warn!(
                    "Debug write out of bounds: '{}' at 0x{:08x} (size {}, memory len {})",
                    value.full_path,
                    ptr,
                    size,
                    memory.len()
                );
                return false;
            }
        };

        let bytes = &mut memory[ptr..end];
        Self::serialize_value(new_val, bytes);
        true
    }

    /// Parse bytes into a DebugValue based on type
    fn parse_value(value_type: ValueType, bytes: &[u8]) -> Option<DebugValue> {
        Some(match value_type {
            ValueType::I8 => DebugValue::I8(i8::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::I16 => DebugValue::I16(i16::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::I32 => DebugValue::I32(i32::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::I64 => DebugValue::I64(i64::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::U8 => DebugValue::U8(bytes[0]),
            ValueType::U16 => DebugValue::U16(u16::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::U32 => DebugValue::U32(u32::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::U64 => DebugValue::U64(u64::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::F32 => DebugValue::F32(f32::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::F64 => DebugValue::F64(f64::from_le_bytes(bytes.try_into().ok()?)),
            ValueType::Bool => DebugValue::Bool(bytes[0] != 0),
            ValueType::Vec2 => {
                let x = f32::from_le_bytes(bytes[0..4].try_into().ok()?);
                let y = f32::from_le_bytes(bytes[4..8].try_into().ok()?);
                DebugValue::Vec2 { x, y }
            }
            ValueType::Vec3 => {
                let x = f32::from_le_bytes(bytes[0..4].try_into().ok()?);
                let y = f32::from_le_bytes(bytes[4..8].try_into().ok()?);
                let z = f32::from_le_bytes(bytes[8..12].try_into().ok()?);
                DebugValue::Vec3 { x, y, z }
            }
            ValueType::Vec4 => {
                let x = f32::from_le_bytes(bytes[0..4].try_into().ok()?);
                let y = f32::from_le_bytes(bytes[4..8].try_into().ok()?);
                let z = f32::from_le_bytes(bytes[8..12].try_into().ok()?);
                let w = f32::from_le_bytes(bytes[12..16].try_into().ok()?);
                DebugValue::Vec4 { x, y, z, w }
            }
            ValueType::Rect => {
                let x = i16::from_le_bytes(bytes[0..2].try_into().ok()?);
                let y = i16::from_le_bytes(bytes[2..4].try_into().ok()?);
                let w = i16::from_le_bytes(bytes[4..6].try_into().ok()?);
                let h = i16::from_le_bytes(bytes[6..8].try_into().ok()?);
                DebugValue::Rect { x, y, w, h }
            }
            ValueType::Color => {
                DebugValue::Color {
                    r: bytes[0],
                    g: bytes[1],
                    b: bytes[2],
                    a: bytes[3],
                }
            }
            ValueType::FixedI16F8 => {
                DebugValue::FixedI16F8(i16::from_le_bytes(bytes.try_into().ok()?))
            }
            ValueType::FixedI32F8 => {
                DebugValue::FixedI32F8(i32::from_le_bytes(bytes.try_into().ok()?))
            }
            ValueType::FixedI32F16 => {
                DebugValue::FixedI32F16(i32::from_le_bytes(bytes.try_into().ok()?))
            }
            ValueType::FixedI32F24 => {
                DebugValue::FixedI32F24(i32::from_le_bytes(bytes.try_into().ok()?))
            }
        })
    }

    /// Serialize a DebugValue into bytes
    fn serialize_value(value: &DebugValue, bytes: &mut [u8]) {
        match value {
            DebugValue::I8(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::I16(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::I32(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::I64(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::U8(v) => bytes[0] = *v,
            DebugValue::U16(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::U32(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::U64(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::F32(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::F64(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::Bool(v) => bytes[0] = if *v { 1 } else { 0 },
            DebugValue::Vec2 { x, y } => {
                bytes[0..4].copy_from_slice(&x.to_le_bytes());
                bytes[4..8].copy_from_slice(&y.to_le_bytes());
            }
            DebugValue::Vec3 { x, y, z } => {
                bytes[0..4].copy_from_slice(&x.to_le_bytes());
                bytes[4..8].copy_from_slice(&y.to_le_bytes());
                bytes[8..12].copy_from_slice(&z.to_le_bytes());
            }
            DebugValue::Vec4 { x, y, z, w } => {
                bytes[0..4].copy_from_slice(&x.to_le_bytes());
                bytes[4..8].copy_from_slice(&y.to_le_bytes());
                bytes[8..12].copy_from_slice(&z.to_le_bytes());
                bytes[12..16].copy_from_slice(&w.to_le_bytes());
            }
            DebugValue::Rect { x, y, w, h } => {
                bytes[0..2].copy_from_slice(&x.to_le_bytes());
                bytes[2..4].copy_from_slice(&y.to_le_bytes());
                bytes[4..6].copy_from_slice(&w.to_le_bytes());
                bytes[6..8].copy_from_slice(&h.to_le_bytes());
            }
            DebugValue::Color { r, g, b, a } => {
                bytes[0] = *r;
                bytes[1] = *g;
                bytes[2] = *b;
                bytes[3] = *a;
            }
            DebugValue::FixedI16F8(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::FixedI32F8(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::FixedI32F16(v) => bytes.copy_from_slice(&v.to_le_bytes()),
            DebugValue::FixedI32F24(v) => bytes.copy_from_slice(&v.to_le_bytes()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Registration Tests ==========

    #[test]
    fn test_register_basic() {
        let mut reg = DebugRegistry::new();
        reg.register("health", 0x100, ValueType::I32, None);
        reg.finalize_registration();

        assert_eq!(reg.len(), 1);
        assert_eq!(reg.values()[0].name, "health");
        assert_eq!(reg.values()[0].full_path, "health");
        assert_eq!(reg.values()[0].wasm_ptr, 0x100);
    }

    #[test]
    fn test_register_with_groups() {
        let mut reg = DebugRegistry::new();
        reg.group_begin("player");
        reg.register("speed", 0x100, ValueType::F32, None);
        reg.group_begin("attacks");
        reg.register("punch_damage", 0x104, ValueType::U8, None);
        reg.group_end();
        reg.group_end();
        reg.finalize_registration();

        assert_eq!(reg.len(), 2);
        assert_eq!(reg.values()[0].full_path, "player/speed");
        assert_eq!(reg.values()[1].full_path, "player/attacks/punch_damage");
    }

    #[test]
    fn test_auto_close_groups() {
        let mut reg = DebugRegistry::new();
        reg.group_begin("unclosed");
        reg.register("value", 0x100, ValueType::I32, None);
        // Don't call group_end()
        reg.finalize_registration(); // Should auto-close and warn

        assert_eq!(reg.len(), 1);
        assert_eq!(reg.values()[0].full_path, "unclosed/value");
    }

    #[test]
    #[should_panic(expected = "without matching")]
    fn test_group_end_without_begin() {
        let mut reg = DebugRegistry::new();
        reg.group_end(); // Should panic
    }

    #[test]
    #[should_panic(expected = "after init()")]
    fn test_register_after_finalize() {
        let mut reg = DebugRegistry::new();
        reg.finalize_registration();
        reg.register("late", 0x100, ValueType::I32, None); // Should panic
    }

    // ========== Memory Read/Write Tests ==========

    #[test]
    fn test_read_write_i32() {
        let reg = DebugRegistry::new();
        let value = RegisteredValue {
            name: "test".to_string(),
            full_path: "test".to_string(),
            wasm_ptr: 0,
            value_type: ValueType::I32,
            constraints: None,
        };

        let mut memory = vec![0u8; 64];
        memory[0..4].copy_from_slice(&42i32.to_le_bytes());

        let read = reg.read_value(&memory, &value).unwrap();
        assert_eq!(read, DebugValue::I32(42));

        // Write new value
        let new_val = DebugValue::I32(-123);
        assert!(reg.write_value(&mut memory, &value, &new_val));

        // Verify write
        let read_back = reg.read_value(&memory, &value).unwrap();
        assert_eq!(read_back, DebugValue::I32(-123));
    }

    #[test]
    fn test_read_write_f32() {
        let reg = DebugRegistry::new();
        let value = RegisteredValue {
            name: "test".to_string(),
            full_path: "test".to_string(),
            wasm_ptr: 4,
            value_type: ValueType::F32,
            constraints: None,
        };

        let mut memory = vec![0u8; 64];
        memory[4..8].copy_from_slice(&3.14159f32.to_le_bytes());

        let read = reg.read_value(&memory, &value).unwrap();
        if let DebugValue::F32(v) = read {
            assert!((v - 3.14159).abs() < 0.00001);
        } else {
            panic!("Expected F32");
        }
    }

    #[test]
    fn test_read_write_vec2() {
        let reg = DebugRegistry::new();
        let value = RegisteredValue {
            name: "pos".to_string(),
            full_path: "pos".to_string(),
            wasm_ptr: 0,
            value_type: ValueType::Vec2,
            constraints: None,
        };

        let mut memory = vec![0u8; 64];
        memory[0..4].copy_from_slice(&1.5f32.to_le_bytes());
        memory[4..8].copy_from_slice(&2.5f32.to_le_bytes());

        let read = reg.read_value(&memory, &value).unwrap();
        assert_eq!(read, DebugValue::Vec2 { x: 1.5, y: 2.5 });

        // Write and verify
        let new_val = DebugValue::Vec2 { x: -1.0, y: 99.0 };
        assert!(reg.write_value(&mut memory, &value, &new_val));
        let read_back = reg.read_value(&memory, &value).unwrap();
        assert_eq!(read_back, new_val);
    }

    #[test]
    fn test_read_write_color() {
        let reg = DebugRegistry::new();
        let value = RegisteredValue {
            name: "color".to_string(),
            full_path: "color".to_string(),
            wasm_ptr: 0,
            value_type: ValueType::Color,
            constraints: None,
        };

        let mut memory = vec![0u8; 64];
        memory[0..4].copy_from_slice(&[255, 128, 64, 200]);

        let read = reg.read_value(&memory, &value).unwrap();
        assert_eq!(read, DebugValue::Color { r: 255, g: 128, b: 64, a: 200 });
    }

    #[test]
    fn test_read_out_of_bounds() {
        let reg = DebugRegistry::new();
        let value = RegisteredValue {
            name: "test".to_string(),
            full_path: "test".to_string(),
            wasm_ptr: 60,  // Near end
            value_type: ValueType::I64,  // 8 bytes would exceed
            constraints: None,
        };

        let memory = vec![0u8; 64];  // Only 64 bytes
        assert!(reg.read_value(&memory, &value).is_none());
    }

    #[test]
    fn test_read_ptr_overflow() {
        let reg = DebugRegistry::new();
        let value = RegisteredValue {
            name: "test".to_string(),
            full_path: "test".to_string(),
            wasm_ptr: u32::MAX,  // Would overflow when adding size
            value_type: ValueType::I32,
            constraints: None,
        };

        let memory = vec![0u8; 64];
        assert!(reg.read_value(&memory, &value).is_none());
    }

    #[test]
    fn test_write_out_of_bounds() {
        let reg = DebugRegistry::new();
        let value = RegisteredValue {
            name: "test".to_string(),
            full_path: "test".to_string(),
            wasm_ptr: 100,  // Beyond memory
            value_type: ValueType::I32,
            constraints: None,
        };

        let mut memory = vec![0u8; 64];
        assert!(!reg.write_value(&mut memory, &value, &DebugValue::I32(42)));
    }

    // ========== Fixed-Point Memory Tests ==========

    #[test]
    fn test_read_write_fixed_i32f16() {
        let reg = DebugRegistry::new();
        let value = RegisteredValue {
            name: "pos_x".to_string(),
            full_path: "pos_x".to_string(),
            wasm_ptr: 0,
            value_type: ValueType::FixedI32F16,
            constraints: None,
        };

        let mut memory = vec![0u8; 64];
        // 2.5 in Q16.16 = 2.5 * 65536 = 163840
        let raw: i32 = 163840;
        memory[0..4].copy_from_slice(&raw.to_le_bytes());

        let read = reg.read_value(&memory, &value).unwrap();
        assert_eq!(read, DebugValue::FixedI32F16(163840));

        // Check the display value
        let display = read.fixed_to_f64().unwrap();
        assert!((display - 2.5).abs() < 0.0001);
    }
}
```

### Test Cases Summary for Phase 2

| Test | Purpose |
|------|---------|
| `test_register_basic` | Basic registration without groups |
| `test_register_with_groups` | Nested group hierarchy |
| `test_auto_close_groups` | Unclosed groups auto-close on finalize |
| `test_group_end_without_begin` | Panic on mismatched group_end |
| `test_register_after_finalize` | Panic on late registration |
| `test_read_write_i32` | Read/write integer values |
| `test_read_write_f32` | Read/write float values |
| `test_read_write_vec2` | Read/write compound types |
| `test_read_write_color` | Read/write color values |
| `test_read_out_of_bounds` | Graceful handling of out-of-bounds reads |
| `test_read_ptr_overflow` | Handle pointer arithmetic overflow |
| `test_write_out_of_bounds` | Graceful handling of out-of-bounds writes |
| `test_read_write_fixed_i32f16` | Fixed-point read/write |

---

## Phases 3-10: Summary

Due to document length, phases 3-10 are summarized with key details:

### Phase 3: FFI Functions (`core/src/debug/ffi.rs`)

- Implement all `debug_register_*` functions using the pattern from `core/src/ffi.rs`
- Use `Caller<'_, GameStateWithConsole<I, S>>` pattern
- Add helper `read_c_string()` function for reading null-terminated strings
- **Integration:** Add `crate::debug::ffi::register_debug_ffi(linker)?` to `core/src/ffi.rs:register_common_ffi()`

**Key test:** WAT module that calls debug FFI and verifies registration (see test at end of document)

### Phase 4: Frame Controller (`core/src/debug/frame_control.rs`)

- Implement pause/resume, single-step, time scale
- Time scale presets: `[0.1, 0.25, 0.5, 1.0, 2.0]`
- `disable()` method for netplay (resets all state)
- **No dependencies on other modules** - can be tested in isolation

### Phase 5: Debug Panel (`core/src/debug/panel.rs`)

- egui `SidePanel::right` for UI
- Tree-building from `full_path` strings
- Value widgets based on `ValueType` and `Constraints`
- Returns `bool` indicating if any value changed

### Phase 6: Export (`core/src/debug/export.rs`)

- `export_rust_flat()` function
- Use `{:?}` for float formatting (round-trip safe)
- Fixed-point exports as `FixedI32::from_bits(raw)`

### Phase 7: GameState Integration

**Modify `core/src/wasm/state.rs`:**
```rust
pub struct GameStateWithConsole<I: ConsoleInput, S> {
    pub game: GameState<I>,
    pub console: S,
    pub debug_registry: DebugRegistry,  // ADD
}
```

**Modify `core/src/wasm/mod.rs` in `init()`:**
```rust
// After existing init logic:
self.store.data_mut().debug_registry.finalize_registration();
```

### Phase 8: Config (`core/src/app/config.rs`)

Add `DebugConfig` struct with hotkey strings (F3, F5, F6, F7, F8 defaults)

### Phase 9: Runtime Integration

- Check `session.is_networked()` to disable debug features
- Integrate `FrameController` with game loop

### Phase 10: Change Callback

- Return `changed` bool from panel render
- Invoke WASM callback via stored function pointer

---

## Module Root and Exports

**Create `core/src/debug/mod.rs`:**

```rust
//! Debug inspection system for runtime value editing

mod export;
mod ffi;
mod frame_control;
mod panel;
mod registry;
mod types;

pub use export::{export_rust_flat, ExportFormat};
pub use ffi::register_debug_ffi;
pub use frame_control::FrameController;
pub use panel::DebugPanel;
pub use registry::{DebugRegistry, RegisteredValue};
pub use types::{Constraints, DebugValue, ValueType};
```

**Modify `core/src/lib.rs`:**

```rust
pub mod debug;  // ADD this line
```

---

## Integration Test WAT Module

Use this for end-to-end testing:

```wat
(module
    ;; Import debug FFI functions
    (import "env" "debug_register_f32" (func $debug_register_f32 (param i32 i32)))
    (import "env" "debug_register_f32_range" (func $debug_register_f32_range (param i32 i32 f32 f32)))
    (import "env" "debug_register_i32" (func $debug_register_i32 (param i32 i32)))
    (import "env" "debug_register_bool" (func $debug_register_bool (param i32 i32)))
    (import "env" "debug_register_color" (func $debug_register_color (param i32 i32)))
    (import "env" "debug_group_begin" (func $debug_group_begin (param i32)))
    (import "env" "debug_group_end" (func $debug_group_end))

    (memory (export "memory") 1)

    ;; String data (null-terminated)
    (data (i32.const 0) "player\00")
    (data (i32.const 16) "speed\00")
    (data (i32.const 32) "health\00")
    (data (i32.const 48) "invincible\00")
    (data (i32.const 80) "tint\00")

    ;; Value data (starting at 256)
    (data (i32.const 256) "\00\00\20\41")  ;; speed: 10.0f
    (data (i32.const 260) "\64\00\00\00")  ;; health: 100
    (data (i32.const 264) "\00")           ;; invincible: false
    (data (i32.const 280) "\ff\80\40\ff")  ;; tint: {255, 128, 64, 255}

    (func (export "init")
        (call $debug_group_begin (i32.const 0))  ;; "player"

        (call $debug_register_f32_range
            (i32.const 16)   ;; "speed"
            (i32.const 256)  ;; ptr
            (f32.const 0.0)  ;; min
            (f32.const 20.0) ;; max
        )

        (call $debug_register_i32
            (i32.const 32)   ;; "health"
            (i32.const 260)  ;; ptr
        )

        (call $debug_register_bool
            (i32.const 48)   ;; "invincible"
            (i32.const 264)  ;; ptr
        )

        (call $debug_register_color
            (i32.const 80)   ;; "tint"
            (i32.const 280)  ;; ptr
        )

        (call $debug_group_end)
    )

    (func (export "update"))
    (func (export "render"))
)
```

---

## File Checklist

| Phase | Action | File | Description |
|-------|--------|------|-------------|
| 1 | Create | `core/src/debug/types.rs` | ValueType, DebugValue, Constraints |
| 2 | Create | `core/src/debug/registry.rs` | DebugRegistry, RegisteredValue |
| 3 | Create | `core/src/debug/ffi.rs` | FFI function implementations |
| 3 | Modify | `core/src/ffi.rs` | Add `register_debug_ffi()` call |
| 4 | Create | `core/src/debug/frame_control.rs` | FrameController |
| 5 | Create | `core/src/debug/panel.rs` | DebugPanel egui UI |
| 6 | Create | `core/src/debug/export.rs` | Export formatting |
| 7 | Modify | `core/src/wasm/state.rs` | Add `debug_registry` field |
| 7 | Modify | `core/src/wasm/mod.rs` | Add `finalize_registration()` in init() |
| 8 | Modify | `core/src/app/config.rs` | Add `DebugConfig` |
| 9 | Modify | `core/src/runtime.rs` | Integrate FrameController |
| - | Create | `core/src/debug/mod.rs` | Module root |
| - | Modify | `core/src/lib.rs` | Add `pub mod debug` |

---

**End of Implementation Plan**
