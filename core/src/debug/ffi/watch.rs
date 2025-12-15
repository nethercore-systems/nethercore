//! Debug watch (read-only) FFI functions
//!
//! Functions for registering read-only debug values from WASM games.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::debug::types::ValueType;
use crate::wasm::WasmGameContext;

use super::read_string;
use super::register::HasDebugRegistry;

/// Register debug watch FFI functions
pub(super) fn register<I, S, R>(linker: &mut Linker<WasmGameContext<I, S, R>>) -> Result<()>
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    linker.func_wrap("env", "debug_watch_i8", debug_watch_i8::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_i16", debug_watch_i16::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_i32", debug_watch_i32::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_u8", debug_watch_u8::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_u16", debug_watch_u16::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_u32", debug_watch_u32::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_f32", debug_watch_f32::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_bool", debug_watch_bool::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_vec2", debug_watch_vec2::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_vec3", debug_watch_vec3::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_rect", debug_watch_rect::<I, S, R>)?;
    linker.func_wrap("env", "debug_watch_color", debug_watch_color::<I, S, R>)?;
    Ok(())
}

fn debug_watch_i8<I, S, R>(
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
            .watch(&name, ptr, ValueType::I8);
    }
}

fn debug_watch_i16<I, S, R>(
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
            .watch(&name, ptr, ValueType::I16);
    }
}

fn debug_watch_i32<I, S, R>(
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
            .watch(&name, ptr, ValueType::I32);
    }
}

fn debug_watch_u8<I, S, R>(
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
            .watch(&name, ptr, ValueType::U8);
    }
}

fn debug_watch_u16<I, S, R>(
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
            .watch(&name, ptr, ValueType::U16);
    }
}

fn debug_watch_u32<I, S, R>(
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
            .watch(&name, ptr, ValueType::U32);
    }
}

fn debug_watch_f32<I, S, R>(
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
            .watch(&name, ptr, ValueType::F32);
    }
}

fn debug_watch_bool<I, S, R>(
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
            .watch(&name, ptr, ValueType::Bool);
    }
}

fn debug_watch_vec2<I, S, R>(
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
            .watch(&name, ptr, ValueType::Vec2);
    }
}

fn debug_watch_vec3<I, S, R>(
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
            .watch(&name, ptr, ValueType::Vec3);
    }
}

fn debug_watch_rect<I, S, R>(
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
            .watch(&name, ptr, ValueType::Rect);
    }
}

fn debug_watch_color<I, S, R>(
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
            .watch(&name, ptr, ValueType::Color);
    }
}
