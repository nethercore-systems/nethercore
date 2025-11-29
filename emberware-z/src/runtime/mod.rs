//! WASM game runtime with deterministic rollback netcode
//!
//! Architecture:
//! - wasmtime runs the game WASM
//! - GGRS handles rollback netcode (save/load state, input sync)
//! - wgpu renders graphics
//! - Tick rate (update) is separate from frame rate (render)
//!
//! Rollback flow:
//! 1. GGRS requests confirmed inputs for frame N
//! 2. If prediction was wrong, GGRS calls load_state(N-X)
//! 3. Re-run update() for frames N-X to N with corrected inputs
//! 4. render() only called for final confirmed frame

// TODO: Implement runtime modules
// - wasm.rs: wasmtime setup, memory management, state serialization
// - ffi.rs: Host functions (clear, draw_*, save_state, load_state, random)
// - graphics.rs: wgpu rendering pipeline
// - audio.rs: rodio audio playback (skip during rollback)
// - input.rs: Keyboard/gamepad input → GGRS input
// - rollback.rs: GGRS integration, state save/load
