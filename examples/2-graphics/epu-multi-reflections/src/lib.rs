//! EPU Multi Reflections
//!
//! Draws two shiny spheres that each sample a different EPU environment (env_id)
//! to validate multiple simultaneous reflection environments.
//!
//! - Requires `render_mode = 2` in nether.toml.
//! - Uses `environment_index(...)` + `epu_set(...)` to push two environments and `draw_epu()` to draw the background.
//! - Uses `environment_index(...)` per draw to select which env is sampled.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

static mut SPHERE_MESH: u32 = 0;

// -----------------------------------------------------------------------------
// Minimal EPU packing helpers (128-bit instruction = 2 x u64)
// -----------------------------------------------------------------------------

const fn epu_hi(
    opcode: u64,
    region: u64,
    blend: u64,
    meta5: u64,
    color_a: u64,
    color_b: u64,
) -> u64 {
    ((opcode & 0x1F) << 59)
        | ((region & 0x7) << 56)
        | ((blend & 0x7) << 53)
        | ((meta5 & 0x1F) << 48)
        | ((color_a & 0xFFFFFF) << 24)
        | (color_b & 0xFFFFFF)
}

const fn epu_lo(
    intensity: u64,
    param_a: u64,
    param_b: u64,
    param_c: u64,
    param_d: u64,
    direction: u64,
    alpha_a: u64,
    alpha_b: u64,
) -> u64 {
    ((intensity & 0xFF) << 56)
        | ((param_a & 0xFF) << 48)
        | ((param_b & 0xFF) << 40)
        | ((param_c & 0xFF) << 32)
        | ((param_d & 0xFF) << 24)
        | ((direction & 0xFFFF) << 8)
        | ((alpha_a & 0xF) << 4)
        | (alpha_b & 0xF)
}

const OP_RAMP: u64 = 0x01;
const REGION_ALL: u64 = 0b111;
const BLEND_ADD: u64 = 0;
const DIR_UP: u64 = 0x80FF;

// Two very different sky gradients to make reflections obviously different.
static EPU_WARM: [[u64; 2]; 8] = [
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xFF8A3D, 0x2A0A2F),
        epu_lo(220, 200, 180, 0xA5, 0, DIR_UP, 15, 15),
    ],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
];

static EPU_COOL: [[u64; 2]; 8] = [
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x00D9FF, 0x001933),
        epu_lo(220, 200, 180, 0xA5, 0, DIR_UP, 15, 15),
    ],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);

        // Procedural sphere mesh (init-only).
        SPHERE_MESH = sphere(1.0, 64, 32);

        // Shiny metal for very visible environment reflections.
        material_metallic(1.0);
        material_roughness(0.05);
        material_emissive(0.0);

        // Use uniform material values (no textures).
        use_uniform_color(1);
        use_uniform_metallic(1);
        use_uniform_roughness(1);
        use_uniform_emissive(1);

        // Simple key light.
        light_set(0, -0.4, -1.0, -0.2);
        light_color(0, 0xFFFFFFFF);
        light_intensity(0, 1.5);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    // No simulation needed for this showcase.
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Camera framing both spheres.
        camera_set(0.0, 1.2, 6.0, 0.0, 0.3, 0.0);

        // Push two environments this frame:
        // - env_id 0: warm (also used as the visible background)
        // - env_id 1: cool (used only for the right sphere's reflections/ambient)
        environment_index(0);
        epu_set(EPU_WARM.as_ptr() as *const u64);
        environment_index(1);
        epu_set(EPU_COOL.as_ptr() as *const u64);

        // Left sphere: env 0 (warm).
        environment_index(0);
        set_color(0xFFFFFFFF);
        push_identity();
        push_translate(-1.6, 0.0, 0.0);
        draw_mesh(SPHERE_MESH);

        // Right sphere: env 1 (cool).
        environment_index(1);
        set_color(0xFFFFFFFF);
        push_identity();
        push_translate(1.6, 0.0, 0.0);
        draw_mesh(SPHERE_MESH);

        // Draw environment background after 3D so it fills only background pixels.
        environment_index(0);
        draw_epu();

        // Labels.
        set_color(0xFFFFFFFF);
        let label_left = b"env 0 (warm)";
        draw_text(
            label_left.as_ptr(),
            label_left.len() as u32,
            140.0,
            20.0,
            18.0,
        );
        let label_right = b"env 1 (cool)";
        draw_text(
            label_right.as_ptr(),
            label_right.len() as u32,
            620.0,
            20.0,
            18.0,
        );
    }
}
