//! PRISM SURVIVORS - 4-Player Co-op Fantasy Survival
//!
//! A showcase game for the Nethercore ZX console demonstrating:
//! - Render Mode 3 (Specular-Shininess Blinn-Phong with rim lighting)
//! - Procedurally generated assets via mesh-gen tool
//! - Alpha dithering for transparency effects
//! - Environment Processing Unit (EPU) for dynamic backgrounds
//! - 4-player rollback netcode multiplayer
//!
//! Run `cargo run -p mesh-gen` to generate assets before building.

#![no_std]

// TODO: Implement game logic
// - Player movement and combat
// - Enemy AI and wave spawning
// - Power-up system
// - Stage progression
// - UI/HUD

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Game initialization - called once at startup
#[unsafe(no_mangle)]
pub extern "C" fn init() {
    // TODO: Load assets from ROM
    // TODO: Initialize game state
}

/// Game update - called each frame
#[unsafe(no_mangle)]
pub extern "C" fn update() {
    // TODO: Handle input
    // TODO: Update player positions
    // TODO: Update enemies
    // TODO: Check collisions
    // TODO: Process power-ups
}

/// Game render - called each frame after update
#[unsafe(no_mangle)]
pub extern "C" fn render() {
    // TODO: Set up environment (EPU)
    // TODO: Set up lighting
    // TODO: Render stage
    // TODO: Render enemies
    // TODO: Render players
    // TODO: Render effects
    // TODO: Render UI
}
