//! Debug inspection FFI functions
//!
//! These functions are registered with the WASM linker and called by games
//! to register values for debug inspection.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::console::ConsoleInput;
use crate::wasm::{read_string_from_memory, GameStateWithConsole};

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

    // Watch (read-only) registration functions
    linker.func_wrap("env", "debug_watch_i8", debug_watch_i8::<I, S>)?;
    linker.func_wrap("env", "debug_watch_i16", debug_watch_i16::<I, S>)?;
    linker.func_wrap("env", "debug_watch_i32", debug_watch_i32::<I, S>)?;
    linker.func_wrap("env", "debug_watch_u8", debug_watch_u8::<I, S>)?;
    linker.func_wrap("env", "debug_watch_u16", debug_watch_u16::<I, S>)?;
    linker.func_wrap("env", "debug_watch_u32", debug_watch_u32::<I, S>)?;
    linker.func_wrap("env", "debug_watch_f32", debug_watch_f32::<I, S>)?;
    linker.func_wrap("env", "debug_watch_bool", debug_watch_bool::<I, S>)?;
    linker.func_wrap("env", "debug_watch_vec2", debug_watch_vec2::<I, S>)?;
    linker.func_wrap("env", "debug_watch_vec3", debug_watch_vec3::<I, S>)?;
    linker.func_wrap("env", "debug_watch_rect", debug_watch_rect::<I, S>)?;
    linker.func_wrap("env", "debug_watch_color", debug_watch_color::<I, S>)?;

    // Grouping functions
    linker.func_wrap("env", "debug_group_begin", debug_group_begin::<I, S>)?;
    linker.func_wrap("env", "debug_group_end", debug_group_end::<I, S>)?;

    // State query functions
    linker.func_wrap("env", "debug_is_paused", debug_is_paused::<I, S>)?;
    linker.func_wrap("env", "debug_get_time_scale", debug_get_time_scale::<I, S>)?;

    // Note: Change callbacks are handled via exported on_debug_change() function
    // No FFI registration needed - console looks for the export directly

    Ok(())
}

// ============================================================================
// Helper functions
// ============================================================================

/// Read a length-prefixed string from WASM memory
fn read_string<I, S>(
    caller: &Caller<'_, GameStateWithConsole<I, S>>,
    ptr: u32,
    len: u32,
) -> Option<String>
where
    I: ConsoleInput,
{
    let memory = caller.data().game.memory?;
    read_string_from_memory(memory, caller, ptr, len)
}

// ============================================================================
// Primitive type registration
// ============================================================================

fn debug_register_i8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I8, None);
    }
}

fn debug_register_i16<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I16, None);
    }
}

fn debug_register_i32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::I32, None);
    }
}

fn debug_register_u8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U8, None);
    }
}

fn debug_register_u16<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U16, None);
    }
}

fn debug_register_u32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::U32, None);
    }
}

fn debug_register_f32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::F32, None);
    }
}

fn debug_register_bool<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
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

fn debug_register_i32_range<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
    min: i32,
    max: i32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
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
    name_len: u32,
    ptr: u32,
    min: f32,
    max: f32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
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
    name_len: u32,
    ptr: u32,
    min: u32,
    max: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
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
    name_len: u32,
    ptr: u32,
    min: u32,
    max: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
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
    name_len: u32,
    ptr: u32,
    min: i32,
    max: i32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
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

fn debug_register_vec2<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Vec2, None);
    }
}

fn debug_register_vec3<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Vec3, None);
    }
}

fn debug_register_rect<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::Rect, None);
    }
}

fn debug_register_color<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
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

fn debug_register_fixed_i16_q8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI16Q8, None);
    }
}

fn debug_register_fixed_i32_q16<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q16, None);
    }
}

fn debug_register_fixed_i32_q8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q8, None);
    }
}

fn debug_register_fixed_i32_q24<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .register(&name, ptr, ValueType::FixedI32Q24, None);
    }
}

// ============================================================================
// Watch (read-only) registration functions
// ============================================================================

fn debug_watch_i8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::I8);
    }
}

fn debug_watch_i16<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::I16);
    }
}

fn debug_watch_i32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::I32);
    }
}

fn debug_watch_u8<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::U8);
    }
}

fn debug_watch_u16<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::U16);
    }
}

fn debug_watch_u32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::U32);
    }
}

fn debug_watch_f32<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::F32);
    }
}

fn debug_watch_bool<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::Bool);
    }
}

fn debug_watch_vec2<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::Vec2);
    }
}

fn debug_watch_vec3<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::Vec3);
    }
}

fn debug_watch_rect<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::Rect);
    }
}

fn debug_watch_color<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
    ptr: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller
            .data_mut()
            .debug_registry_mut()
            .watch(&name, ptr, ValueType::Color);
    }
}

// ============================================================================
// Grouping functions
// ============================================================================

fn debug_group_begin<I, S>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    name_ptr: u32,
    name_len: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    GameStateWithConsole<I, S>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
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

// Note: Change callbacks are handled via exported on_debug_change() function.
// Games export `on_debug_change` and the console calls it when values change,
// similar to how init/update/render work.
