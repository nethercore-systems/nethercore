//! Save data FFI functions

use wasmtime::Caller;

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::{
    MAX_SAVE_SIZE, MAX_SAVE_SLOTS, WasmGameContext, read_bytes_from_memory, write_bytes_to_memory,
};

/// Save data to a slot (0-7)
///
/// Returns: 0 = success, 1 = invalid slot, 2 = data too large
pub(super) fn save<I: ConsoleInput, S, R: ConsoleRollbackState>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    slot: u32,
    data_ptr: u32,
    data_len: u32,
) -> u32 {
    let slot = slot as usize;
    let data_len = data_len as usize;

    // Validate slot
    if slot >= MAX_SAVE_SLOTS {
        return 1;
    }

    // Validate data size
    if data_len > MAX_SAVE_SIZE {
        return 2;
    }

    // Read data from WASM memory
    let memory = match caller.data().game.memory {
        Some(memory) => memory,
        None => return 2,
    };
    let data = match read_bytes_from_memory(memory, &caller, data_ptr, data_len as u32) {
        Ok(data) => data,
        Err(_) => return 2, // Invalid memory access
    };

    // Store the data
    caller.data_mut().game.save_data[slot] = Some(data);
    0
}

/// Load data from a slot (0-7)
///
/// Returns: bytes read, or 0 if slot is empty/invalid
pub(super) fn load<I: ConsoleInput, S, R: ConsoleRollbackState>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    slot: u32,
    data_ptr: u32,
    max_len: u32,
) -> u32 {
    let slot = slot as usize;
    let max_len = max_len as usize;

    // Validate slot
    if slot >= MAX_SAVE_SLOTS {
        return 0;
    }

    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => return 0,
    };

    let (buffer, copy_len) = {
        let data = match caller.data().game.save_data[slot].as_ref() {
            Some(data) => data,
            None => return 0,
        };
        let copy_len = data.len().min(max_len);
        (data[..copy_len].to_vec(), copy_len)
    };

    match write_bytes_to_memory(memory, &mut caller, data_ptr, &buffer) {
        Ok(()) => copy_len as u32,
        Err(_) => 0,
    }
}

/// Delete data in a slot (0-7)
///
/// Returns: 0 = success, 1 = invalid slot
pub(super) fn delete<I: ConsoleInput, S, R: ConsoleRollbackState>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    slot: u32,
) -> u32 {
    let slot = slot as usize;

    // Validate slot
    if slot >= MAX_SAVE_SLOTS {
        return 1;
    }

    caller.data_mut().game.save_data[slot] = None;
    0
}
