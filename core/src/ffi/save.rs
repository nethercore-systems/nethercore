//! Save data FFI functions

use wasmtime::Caller;

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::save_store::{PERSISTENT_SLOTS, map_session_slot_to_controller_slot};
use crate::wasm::{
    MAX_SAVE_SIZE, MAX_SAVE_SLOTS, WasmGameContext, read_bytes_from_memory, write_bytes_to_memory,
};

pub(super) fn persist_controller_mapped_slot_from_state<
    I: ConsoleInput,
    S,
    R: ConsoleRollbackState,
>(
    ctx: &mut WasmGameContext<I, S, R>,
    session_slot: usize,
) {
    // Only session slots 0..3 are persisted.
    if session_slot >= PERSISTENT_SLOTS {
        return;
    }

    let Some(store) = ctx.save_store.as_mut() else {
        return;
    };

    let player_count = ctx.game.player_count.min(crate::wasm::MAX_PLAYERS as u32);
    let Some(controller_slot) = map_session_slot_to_controller_slot(
        ctx.game.local_player_mask,
        player_count,
        session_slot as u32,
    ) else {
        // Remote session slot (or out of range): succeed, but do not touch disk.
        return;
    };

    if controller_slot >= PERSISTENT_SLOTS {
        return;
    }

    store.set_controller_slot(controller_slot, ctx.game.save_data[session_slot].clone());
    if let Err(e) = store.flush() {
        tracing::warn!(error = %e, session_slot, controller_slot, "Failed to flush SaveStore");
    }
}

/// Save data to a slot (0-3)
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
    {
        let ctx = caller.data_mut();
        ctx.game.save_data[slot] = Some(data);
        persist_controller_mapped_slot_from_state(ctx, slot);
    }
    0
}

/// Load data from a slot (0-3)
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

/// Delete data in a slot (0-3)
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

    {
        let ctx = caller.data_mut();
        ctx.game.save_data[slot] = None;
        persist_controller_mapped_slot_from_state(ctx, slot);
    }
    0
}
