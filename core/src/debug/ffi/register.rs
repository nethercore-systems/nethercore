//! Debug value registration FFI functions
//!
//! Functions for registering editable debug values from WASM games.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::debug::registry::DebugRegistry;
use crate::debug::types::{Constraints, ValueType};
use crate::wasm::WasmGameContext;

use super::read_string;

/// Trait to allow generic access to the debug registry
///
/// This trait is implemented for any type that has a `debug_registry` field.
pub trait HasDebugRegistry {
    fn debug_registry(&self) -> &DebugRegistry;
    fn debug_registry_mut(&mut self) -> &mut DebugRegistry;
}

/// Register debug value registration FFI functions
pub(super) fn register<I, S, R>(linker: &mut Linker<WasmGameContext<I, S, R>>) -> Result<()>
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    // Value registration functions (primitives)
    linker.func_wrap("env", "debug_register_i8", debug_register_i8::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_i16", debug_register_i16::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_i32", debug_register_i32::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_u8", debug_register_u8::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_u16", debug_register_u16::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_u32", debug_register_u32::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_f32", debug_register_f32::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_bool", debug_register_bool::<I, S, R>)?;

    // Value registration functions with range constraints
    linker.func_wrap(
        "env",
        "debug_register_i32_range",
        debug_register_i32_range::<I, S, R>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_f32_range",
        debug_register_f32_range::<I, S, R>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_u8_range",
        debug_register_u8_range::<I, S, R>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_u16_range",
        debug_register_u16_range::<I, S, R>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_i16_range",
        debug_register_i16_range::<I, S, R>,
    )?;

    // Compound type registration
    linker.func_wrap("env", "debug_register_vec2", debug_register_vec2::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_vec3", debug_register_vec3::<I, S, R>)?;
    linker.func_wrap("env", "debug_register_rect", debug_register_rect::<I, S, R>)?;
    linker.func_wrap(
        "env",
        "debug_register_color",
        debug_register_color::<I, S, R>,
    )?;

    // Fixed-point type registration
    linker.func_wrap(
        "env",
        "debug_register_fixed_i16_q8",
        debug_register_fixed_i16_q8::<I, S, R>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_fixed_i32_q16",
        debug_register_fixed_i32_q16::<I, S, R>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_fixed_i32_q8",
        debug_register_fixed_i32_q8::<I, S, R>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_fixed_i32_q24",
        debug_register_fixed_i32_q24::<I, S, R>,
    )?;

    Ok(())
}

// ============================================================================
// Primitive type registration
// ============================================================================

fn debug_register_i8<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I8, None);
    }
}

fn debug_register_i16<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I16, None);
    }
}

fn debug_register_i32<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I32, None);
    }
}

fn debug_register_u8<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U8, None);
    }
}

fn debug_register_u16<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U16, None);
    }
}

fn debug_register_u32<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U32, None);
    }
}

fn debug_register_f32<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::F32, None);
    }
}

fn debug_register_bool<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Bool, None);
    }
}

// ============================================================================
// Range-constrained registration
// ============================================================================

fn debug_register_i32_range<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
    min: i32,
    max: i32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I32, constraints);
    }
}

fn debug_register_f32_range<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
    min: f32,
    max: f32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::F32, constraints);
    }
}

fn debug_register_u8_range<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
    min: u32,
    max: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U8, constraints);
    }
}

fn debug_register_u16_range<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
    min: u32,
    max: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U16, constraints);
    }
}

fn debug_register_i16_range<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
    min: i32,
    max: i32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I16, constraints);
    }
}

// ============================================================================
// Compound type registration
// ============================================================================

fn debug_register_vec2<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Vec2, None);
    }
}

fn debug_register_vec3<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Vec3, None);
    }
}

fn debug_register_rect<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Rect, None);
    }
}

fn debug_register_color<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Color, None);
    }
}

// ============================================================================
// Fixed-point type registration
// ============================================================================

fn debug_register_fixed_i16_q8<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI16Q8, None);
    }
}

fn debug_register_fixed_i32_q16<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q16, None);
    }
}

fn debug_register_fixed_i32_q8<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q8, None);
    }
}

fn debug_register_fixed_i32_q24<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q24, None);
    }
}
