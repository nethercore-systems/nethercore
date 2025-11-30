//! WASM game runtime with deterministic rollback netcode
//!
//! Architecture:
//! - wasmtime runs the game WASM (see `core/src/wasm.rs`)
//! - GGRS handles rollback netcode (see `core/src/rollback/`)
//! - wgpu renders graphics (see `emberware-z/src/graphics.rs`)
//! - Tick rate (update) is separate from frame rate (render)
//!
//! Rollback flow:
//! 1. GGRS requests confirmed inputs for frame N
//! 2. If prediction was wrong, GGRS calls load_state(N-X)
//! 3. Re-run update() for frames N-X to N with corrected inputs
//! 4. render() only called for final confirmed frame
//!
//! Module layout (implemented in parent crates):
//! - `core/src/wasm.rs`: wasmtime setup, memory management, state serialization
//! - `core/src/ffi.rs`: Common host functions (log, save, load, random)
//! - `core/src/rollback/`: GGRS integration, state save/load
//! - `core/src/runtime.rs`: Game loop orchestration
//! - `emberware-z/src/ffi/`: Z-specific FFI (draw_*, input, camera, etc.)
//! - `emberware-z/src/graphics.rs`: wgpu rendering pipeline
//! - `emberware-z/src/input.rs`: Keyboard/gamepad input → GGRS input
//! - Audio: Not yet implemented (see TASKS.md)
