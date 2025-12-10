//! Debug inspection FFI functions
//!
//! These functions are registered with the WASM linker and called by games
//! to register values for debug inspection.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::console::ConsoleInput;
use crate::wasm::GameStateWithConsole;

use super::registry::DebugRegistry;
use super::types::{Constraints, ValueType};

/// Trait to allow generic access to the debug registry
///
/// This trait is implemented for any type that has a `debug_registry` field.
pub trait HasDebugRegistry {
    fn debug_registry(&self) -> &DebugRegistry;
    fn debug_registry_mut(&mut self) -> &mut DebugRegistry;
}

/// Register all debug FFI functions with the linker
///
/// These functions are always registered (even for release builds), but games
/// built in release mode won't import them, so they won't be linked.
pub fn register_debug_ffi<I, S>(linker: &mut Linker<GameStateWithConsole<I, S>>) -> Result<()>
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    // Value registration functions (primitives)
    linker.func_wrap("env", "debug_register_i8", debug_register_i8::<I, S>)?;
    linker.func_wrap("env", "debug_register_i16", debug_register_i16::<I, S>)?;
    linker.func_wrap("env", "debug_register_i32", debug_register_i32::<I, S>)?;
    linker.func_wrap("env", "debug_register_u8", debug_register_u8::<I, S>)?;
    linker.func_wrap("env", "debug_register_u16", debug_register_u16::<I, S>)?;
    linker.func_wrap("env", "debug_register_u32", debug_register_u32::<I, S>)?;
    linker.func_wrap("env", "debug_register_f32", debug_register_f32::<I, S>)?;
    linker.func_wrap("env", "debug_register_bool", debug_register_bool::<I, S>)?;

    // Value registration functions with range constraints
    linker.func_wrap(
        "env",
        "debug_register_i32_range",
        debug_register_i32_range::<I, S>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_f32_range",
        debug_register_f32_range::<I, S>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_u8_range",
        debug_register_u8_range::<I, S>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_u16_range",
        debug_register_u16_range::<I, S>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_i16_range",
        debug_register_i16_range::<I, S>,
    )?;

    // Compound type registration
    linker.func_wrap("env", "debug_register_vec2", debug_register_vec2::<I, S>)?;
    linker.func_wrap("env", "debug_register_vec3", debug_register_vec3::<I, S>)?;
    linker.func_wrap("env", "debug_register_rect", debug_register_rect::<I, S>)?;
    linker.func_wrap("env", "debug_register_color", debug_register_color::<I, S>)?;

    // Fixed-point type registration
    linker.func_wrap(
        "env",
        "debug_register_fixed_i16_q8",
        debug_register_fixed_i16_q8::<I, S>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_fixed_i32_q16",
        debug_register_fixed_i32_q16::<I, S>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_fixed_i32_q8",
        debug_register_fixed_i32_q8::<I, S>,
    )?;
    linker.func_wrap(
        "env",
        "debug_register_fixed_i32_q24",
        debug_register_fixed_i32_q24::<I, S>,
    )?;

    // Grouping functions
    linker.func_wrap("env", "debug_group_begin", debug_group_begin::<I, S>)?;
    linker.func_wrap("env", "debug_group_end", debug_group_end::<I, S>)?;

    // State query functions
    linker.func_wrap("env", "debug_is_paused", debug_is_paused::<I, S>)?;
    linker.func_wrap("env", "debug_get_time_scale", debug_get_time_scale::<I, S>)?;

    // Callback registration
    linker.func_wrap(
        "env",
        "debug_set_change_callback",
        debug_set_change_callback::<I, S>,
    )?;

    Ok(())
}

// ============================================================================
// Helper functions
// ============================================================================

/// Read a null-terminated C string from WASM memory
fn read_c_string<I, S>(caller: &Caller<'_, GameStateWithConsole<I, S>>, ptr: u32) -> Option<String>
where
    I: ConsoleInput,
{
    let memory = caller.data().game.memory?;
    let data = memory.data(caller);
    let start = ptr as usize;

    // Find null terminator
    let mut end = start;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }

    if end >= data.len() {
        return None; // No null terminator found
    }

    std::str::from_utf8(&data[start..end])
        .ok()
        .map(String::from)
}

// ============================================================================
// Primitive type registration
// ============================================================================

fn debug_register_i8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I8, None);
    }
}

fn debug_register_i16<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I16, None);
    }
}

fn debug_register_i32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I32, None);
    }
}

fn debug_register_u8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U8, None);
    }
}

fn debug_register_u16<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U16, None);
    }
}

fn debug_register_u32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U32, None);
    }
}

fn debug_register_f32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::F32, None);
    }
}

fn debug_register_bool<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Bool, None);
    }
}

// ============================================================================
// Range-constrained registration
// ============================================================================

fn debug_register_i32_range<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
    min: i32,
    max: i32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I32, constraints);
    }
}

fn debug_register_f32_range<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
    min: f32,
    max: f32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::F32, constraints);
    }
}

fn debug_register_u8_range<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
    min: u32,
    max: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U8, constraints);
    }
}

fn debug_register_u16_range<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
    min: u32,
    max: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        let constraints = Some(Constraints::new(min as f64, max as f64));
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U16, constraints);
    }
}

fn debug_register_i16_range<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
    min: i32,
    max: i32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
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

fn debug_register_vec2<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Vec2, None);
    }
}

fn debug_register_vec3<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Vec3, None);
    }
}

fn debug_register_rect<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Rect, None);
    }
}

fn debug_register_color<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Color, None);
    }
}

// ============================================================================
// Fixed-point type registration
// ============================================================================

fn debug_register_fixed_i16_q8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI16Q8, None);
    }
}

fn debug_register_fixed_i32_q16<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q16, None);
    }
}

fn debug_register_fixed_i32_q8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q8, None);
    }
}

fn debug_register_fixed_i32_q24<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q24, None);
    }
}

// ============================================================================
// Grouping functions
// ============================================================================

fn debug_group_begin<I, S>(mut caller: Caller<'_, GameStateWithConsole<I, S>>, name_ptr: u32)
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_c_string(&caller, name_ptr) {
        caller.data_mut().debug_registry_mut().group_begin(&name);
    }
}

fn debug_group_end<I, S>(mut caller: Caller<'_, GameStateWithConsole<I, S>>)
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    caller.data_mut().debug_registry_mut().group_end();
}

// ============================================================================
// State query functions
// ============================================================================

/// Query if the game is currently paused (debug mode)
///
/// Returns 1 if paused, 0 if running normally.
/// Note: This reads from frame controller state, which is stored separately.
/// For now, always returns 0 (not paused) - actual implementation requires
/// integration with the frame controller.
fn debug_is_paused<I, S>(_caller: Caller<'_, GameStateWithConsole<I, S>>) -> i32
where
    I: ConsoleInput,
    S: Send + Default + 'static,
{
    // TODO: Read from frame controller state once integrated
    0
}

/// Get the current time scale (1.0 = normal, 0.5 = half speed, etc.)
///
/// Note: This reads from frame controller state, which is stored separately.
/// For now, always returns 1.0 - actual implementation requires
/// integration with the frame controller.
fn debug_get_time_scale<I, S>(_caller: Caller<'_, GameStateWithConsole<I, S>>) -> f32
where
    I: ConsoleInput,
    S: Send + Default + 'static,
{
    // TODO: Read from frame controller state once integrated
    1.0
}

// ============================================================================
// Callback registration
// ============================================================================

fn debug_set_change_callback<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    callback_ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    caller
        .data_mut()
        .debug_registry_mut()
        .set_change_callback(callback_ptr);
}
