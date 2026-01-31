//! EPU Inspector - Live EPU Editor Playground
//!
//! A debug-panel-driven editor for tweaking EPU layer values in real-time.
//!
//! ## Usage
//!
//! 1. Run the game: `cargo run -- examples/3-inspectors/epu-inspector`
//! 2. Press F4 to open the Debug Panel
//! 3. Adjust layer_index (1-8) to select which layer to edit
//! 4. Modify any field - changes apply immediately
//! 5. Toggle "isolate" to view only the selected layer
//! 6. Click "export hex" to print all layers to console
//!
//! ## Features
//!
//! - **Layer editing**: All 8 EPU layers accessible via layer selector
//! - **Live preview**: Changes reflect immediately in the viewport
//! - **Isolation mode**: View single layer contribution
//! - **Direction helpers**: Azimuth/elevation instead of raw octahedral encoding
//! - **Param hints**: Dynamic hints showing what each param does per opcode
//! - **Export**: Copy hex values for use in preset files

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[path = "../../../../include/zx/mod.rs"]
mod ffi;
use ffi::*;

// ============================================================================
// Editor State - Unpacked layer values for debug panel editing
// ============================================================================

/// Unpacked representation of a single EPU layer for editing
#[derive(Clone, Copy)]
struct EditorState {
    // Hi word fields
    opcode: u8,
    region_sky: u8,      // bool as u8 for FFI
    region_walls: u8,
    region_floor: u8,
    blend: u8,
    domain_id: u8,
    variant_id: u8,
    color_a: u32,        // RGBA for color picker
    color_b: u32,

    // Lo word fields
    intensity: u8,
    param_a: u8,
    param_b: u8,
    param_c: u8,
    param_d: u8,
    azimuth: f32,        // 0-360 degrees
    elevation: f32,      // -90 to +90 degrees
    alpha_a: u8,
    alpha_b: u8,
}

impl EditorState {
    const fn default() -> Self {
        Self {
            opcode: 1,           // RAMP
            region_sky: 1,
            region_walls: 1,
            region_floor: 1,
            blend: 0,            // ADD
            domain_id: 0,
            variant_id: 0,
            color_a: 0x6496DCFF, // Sky blue
            color_b: 0x285028FF, // Ground green
            intensity: 180,
            param_a: 180,
            param_b: 165,
            param_c: 0,
            param_d: 128,
            azimuth: 0.0,
            elevation: 0.0,
            alpha_a: 15,
            alpha_b: 15,
        }
    }
}

// ============================================================================
// Global State
// ============================================================================

/// The 8-layer EPU configuration (16 u64 values = 128 bytes)
static mut LAYERS: [[u64; 2]; 8] = [[0; 2]; 8];

/// Current editor state (unpacked from selected layer)
static mut EDITOR: EditorState = EditorState::default();

/// Control state
static mut LAYER_INDEX: u8 = 1;      // 1-8 (user-facing)
static mut ISOLATE_LAYER: u8 = 0;    // bool
static mut SHOW_HINTS: u8 = 1;       // bool

/// Track previous layer index for change detection
static mut PREV_LAYER_INDEX: u8 = 1;

/// Mesh handles for reference objects
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;

/// Shape selection
static mut SHAPE_INDEX: u8 = 0;  // 0=Sphere, 1=Cube, 2=Torus
const SHAPE_COUNT: u8 = 3;
const SHAPE_NAMES: [&[u8]; 3] = [b"Sphere", b"Cube", b"Torus"];

/// Camera orbit state
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 15.0;

/// Material properties
static mut MATERIAL_METALLIC: f32 = 1.0;
static mut MATERIAL_ROUGHNESS: f32 = 0.38;

// ============================================================================
// Pack/Unpack Helpers
// ============================================================================

const PI: f32 = 3.14159265358979323846;

/// Convert octahedral-encoded direction to (azimuth, elevation) in degrees
fn octahedral_to_angles(dir16: u16) -> (f32, f32) {
    // Unpack: lo byte = u, hi byte = v (after byte swap in pack)
    let u_byte = (dir16 & 0xFF) as f32;
    let v_byte = ((dir16 >> 8) & 0xFF) as f32;

    // Convert to [-1, 1] range
    let u = u_byte / 127.5 - 1.0;
    let v = v_byte / 127.5 - 1.0;

    // Octahedral decode
    let z = 1.0 - libm::fabsf(u) - libm::fabsf(v);
    let (x, y) = if z >= 0.0 {
        (u, v)
    } else {
        // Reflect for lower hemisphere
        let sign_u = if u >= 0.0 { 1.0 } else { -1.0 };
        let sign_v = if v >= 0.0 { 1.0 } else { -1.0 };
        ((1.0 - libm::fabsf(v)) * sign_u, (1.0 - libm::fabsf(u)) * sign_v)
    };

    // Normalize
    let len = libm::sqrtf(x * x + y * y + z * z);
    let (nx, ny, nz) = if len > 0.0001 {
        (x / len, y / len, z / len)
    } else {
        (0.0, 0.0, 1.0)
    };

    // Convert to spherical
    let elevation = libm::asinf(if nz > 1.0 { 1.0 } else if nz < -1.0 { -1.0 } else { nz }) * 180.0 / PI;
    let azimuth = libm::atan2f(ny, nx) * 180.0 / PI;
    let azimuth = if azimuth < 0.0 { azimuth + 360.0 } else { azimuth };

    (azimuth, elevation)
}

/// Convert (azimuth, elevation) in degrees to octahedral encoding
fn angles_to_octahedral(azimuth: f32, elevation: f32) -> u16 {
    let az_rad = azimuth * PI / 180.0;
    let el_rad = elevation * PI / 180.0;

    // Spherical to Cartesian
    let cos_el = libm::cosf(el_rad);
    let x = libm::cosf(az_rad) * cos_el;
    let y = libm::sinf(az_rad) * cos_el;
    let z = libm::sinf(el_rad);

    // Octahedral encode
    let sum = libm::fabsf(x) + libm::fabsf(y) + libm::fabsf(z);
    let (mut u, mut v) = if sum > 0.0001 {
        (x / sum, y / sum)
    } else {
        (0.0, 0.0)
    };

    if z < 0.0 {
        let sign_u = if u >= 0.0 { 1.0 } else { -1.0 };
        let sign_v = if v >= 0.0 { 1.0 } else { -1.0 };
        let new_u = (1.0 - libm::fabsf(v)) * sign_u;
        let new_v = (1.0 - libm::fabsf(u)) * sign_v;
        u = new_u;
        v = new_v;
    }

    // Convert to [0, 255] range
    let u_byte = ((u + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
    let v_byte = ((v + 1.0) * 127.5).clamp(0.0, 255.0) as u8;

    // Pack: lo byte = u, hi byte = v
    (u_byte as u16) | ((v_byte as u16) << 8)
}

/// Unpack a layer's [hi, lo] into the EDITOR state
unsafe fn unpack_layer(hi: u64, lo: u64) {
    // Hi word extraction
    EDITOR.opcode = ((hi >> 59) & 0x1F) as u8;
    let region = ((hi >> 56) & 0x7) as u8;
    EDITOR.region_sky = if region & 0b100 != 0 { 1 } else { 0 };
    EDITOR.region_walls = if region & 0b010 != 0 { 1 } else { 0 };
    EDITOR.region_floor = if region & 0b001 != 0 { 1 } else { 0 };
    EDITOR.blend = ((hi >> 53) & 0x7) as u8;

    let meta_hi = ((hi >> 49) & 0xF) as u8;
    let meta_lo = ((hi >> 48) & 0x1) as u8;
    let meta5 = (meta_hi << 1) | meta_lo;
    EDITOR.domain_id = (meta5 >> 3) & 0x3;
    EDITOR.variant_id = meta5 & 0x7;

    // Colors: stored as RGB24, convert to RGBA for color picker
    let rgb_a = ((hi >> 24) & 0xFFFFFF) as u32;
    let rgb_b = (hi & 0xFFFFFF) as u32;
    EDITOR.color_a = (rgb_a << 8) | 0xFF;
    EDITOR.color_b = (rgb_b << 8) | 0xFF;

    // Lo word extraction
    EDITOR.intensity = ((lo >> 56) & 0xFF) as u8;
    EDITOR.param_a = ((lo >> 48) & 0xFF) as u8;
    EDITOR.param_b = ((lo >> 40) & 0xFF) as u8;
    EDITOR.param_c = ((lo >> 32) & 0xFF) as u8;
    EDITOR.param_d = ((lo >> 24) & 0xFF) as u8;

    // Direction: extract and convert to angles
    let dir_packed = ((lo >> 8) & 0xFFFF) as u16;
    let (az, el) = octahedral_to_angles(dir_packed);
    EDITOR.azimuth = az;
    EDITOR.elevation = el;

    EDITOR.alpha_a = ((lo >> 4) & 0xF) as u8;
    EDITOR.alpha_b = (lo & 0xF) as u8;
}

/// Pack the EDITOR state back into [hi, lo]
unsafe fn pack_layer() -> (u64, u64) {
    let opcode = (EDITOR.opcode as u64 & 0x1F) << 59;
    let region = ((if EDITOR.region_sky != 0 { 0b100u64 } else { 0 })
        | (if EDITOR.region_walls != 0 { 0b010 } else { 0 })
        | (if EDITOR.region_floor != 0 { 0b001 } else { 0 }))
        << 56;
    let blend = (EDITOR.blend as u64 & 0x7) << 53;

    let meta5 = ((EDITOR.domain_id & 0x3) << 3) | (EDITOR.variant_id & 0x7);
    let meta_hi = ((meta5 >> 1) & 0xF) as u64;
    let meta_lo = (meta5 & 0x1) as u64;
    let meta = (meta_hi << 49) | (meta_lo << 48);

    // Colors: RGBA -> RGB24
    let rgb_a = ((EDITOR.color_a >> 8) & 0xFFFFFF) as u64;
    let rgb_b = ((EDITOR.color_b >> 8) & 0xFFFFFF) as u64;
    let colors = (rgb_a << 24) | rgb_b;

    let hi = opcode | region | blend | meta | colors;

    // Lo word
    let intensity = (EDITOR.intensity as u64) << 56;
    let param_a = (EDITOR.param_a as u64) << 48;
    let param_b = (EDITOR.param_b as u64) << 40;
    let param_c = (EDITOR.param_c as u64) << 32;
    let param_d = (EDITOR.param_d as u64) << 24;

    let dir16 = angles_to_octahedral(EDITOR.azimuth, EDITOR.elevation);
    let direction = (dir16 as u64) << 8;

    let alpha_a = (EDITOR.alpha_a as u64 & 0xF) << 4;
    let alpha_b = EDITOR.alpha_b as u64 & 0xF;

    let lo = intensity | param_a | param_b | param_c | param_d | direction | alpha_a | alpha_b;

    (hi, lo)
}

// ============================================================================
// Debug Panel Registration
// ============================================================================

unsafe fn register_debug_panel() {
    // Control group
    debug_group_begin(b"control".as_ptr(), 7);
    debug_register_u8_range(b"layer (1-8)".as_ptr(), 11, &LAYER_INDEX, 1, 8);
    debug_register_bool(b"isolate".as_ptr(), 7, &ISOLATE_LAYER);
    debug_register_bool(b"hints".as_ptr(), 5, &SHOW_HINTS);
    debug_group_end();

    // Hi word group
    debug_group_begin(b"hi word".as_ptr(), 7);
    debug_register_u8_range(b"opcode".as_ptr(), 6, &EDITOR.opcode, 0, 31);
    debug_register_bool(b"region_sky".as_ptr(), 10, &EDITOR.region_sky);
    debug_register_bool(b"region_walls".as_ptr(), 12, &EDITOR.region_walls);
    debug_register_bool(b"region_floor".as_ptr(), 12, &EDITOR.region_floor);
    debug_register_u8_range(b"blend".as_ptr(), 5, &EDITOR.blend, 0, 7);
    debug_register_u8_range(b"domain_id".as_ptr(), 9, &EDITOR.domain_id, 0, 3);
    debug_register_u8_range(b"variant_id".as_ptr(), 10, &EDITOR.variant_id, 0, 7);
    debug_register_color(b"color_a".as_ptr(), 7, &EDITOR.color_a as *const u32 as *const u8);
    debug_register_color(b"color_b".as_ptr(), 7, &EDITOR.color_b as *const u32 as *const u8);
    debug_group_end();

    // Lo word group
    debug_group_begin(b"lo word".as_ptr(), 7);
    debug_register_u8(b"intensity".as_ptr(), 9, &EDITOR.intensity);
    debug_register_u8(b"param_a".as_ptr(), 7, &EDITOR.param_a);
    debug_register_u8(b"param_b".as_ptr(), 7, &EDITOR.param_b);
    debug_register_u8(b"param_c".as_ptr(), 7, &EDITOR.param_c);
    debug_register_u8(b"param_d".as_ptr(), 7, &EDITOR.param_d);
    debug_register_f32_range(b"azimuth".as_ptr(), 7, &EDITOR.azimuth as *const f32 as *const u8, 0.0, 360.0);
    debug_register_f32_range(b"elevation".as_ptr(), 9, &EDITOR.elevation as *const f32 as *const u8, -90.0, 90.0);
    debug_register_u8_range(b"alpha_a".as_ptr(), 7, &EDITOR.alpha_a, 0, 15);
    debug_register_u8_range(b"alpha_b".as_ptr(), 7, &EDITOR.alpha_b, 0, 15);
    debug_group_end();

    // Export action
    debug_group_begin(b"export".as_ptr(), 6);
    debug_register_action(b"export hex".as_ptr(), 10, b"do_export".as_ptr(), 9);
    debug_group_end();

    // Shape selection
    debug_group_begin(b"shape".as_ptr(), 5);
    debug_register_u8_range(b"index (0-2)".as_ptr(), 11, &SHAPE_INDEX, 0, 2);
    debug_group_end();

    // Camera controls
    debug_group_begin(b"camera".as_ptr(), 6);
    debug_register_f32(b"angle".as_ptr(), 5, &CAM_ANGLE as *const f32 as *const u8);
    debug_register_f32_range(b"elevation".as_ptr(), 9, &CAM_ELEVATION as *const f32 as *const u8, -60.0, 60.0);
    debug_group_end();

    // Material properties
    debug_group_begin(b"material".as_ptr(), 8);
    debug_register_f32_range(b"metallic".as_ptr(), 8, &MATERIAL_METALLIC as *const f32 as *const u8, 0.0, 1.0);
    debug_register_f32_range(b"roughness".as_ptr(), 9, &MATERIAL_ROUGHNESS as *const f32 as *const u8, 0.0, 1.0);
    debug_group_end();
}

// ============================================================================
// Export Action
// ============================================================================

#[no_mangle]
pub extern "C" fn do_export() {
    unsafe {
        // Log each layer as hex
        for i in 0..8 {
            let hi = LAYERS[i][0];
            let lo = LAYERS[i][1];
            log_layer_hex(i, hi, lo);
        }
    }
}

unsafe fn log_layer_hex(_index: usize, hi: u64, lo: u64) {
    // Format: "[0x{hi:016X}, 0x{lo:016X}],"
    // We need to build this string manually in no_std
    let mut buf = [0u8; 48];
    buf[0] = b'[';
    buf[1] = b'0';
    buf[2] = b'x';
    write_hex_u64(&mut buf[3..19], hi);
    buf[19] = b',';
    buf[20] = b' ';
    buf[21] = b'0';
    buf[22] = b'x';
    write_hex_u64(&mut buf[23..39], lo);
    buf[39] = b']';
    buf[40] = b',';

    log(buf.as_ptr(), 41);
}

fn write_hex_u64(buf: &mut [u8], val: u64) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for i in 0..16 {
        let nibble = ((val >> (60 - i * 4)) & 0xF) as usize;
        buf[i] = HEX[nibble];
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);

        // Initialize layer 0 with a simple RAMP (sky gradient)
        // hi: opcode=1(RAMP), region=7(ALL), blend=0(ADD), meta5=0, colors
        // lo: intensity, params, direction, alphas
        LAYERS[0] = [
            0x0F00_6496_DC28_5028, // RAMP, ALL, ADD, sky-blue / ground-green
            0xB4B4_A500_8000_FF,   // intensity=180, param_a=180, param_b=165, dir=center, alpha=15/15
        ];

        // Unpack layer 0 into editor state
        unpack_layer(LAYERS[0][0], LAYERS[0][1]);

        // Create reference meshes
        SPHERE_MESH = sphere(1.3, 32, 24);
        CUBE_MESH = cube(1.2, 1.2, 1.2);
        TORUS_MESH = torus(1.0, 0.4, 32, 16);

        // Register debug panel
        register_debug_panel();
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Check if layer index changed
        if LAYER_INDEX != PREV_LAYER_INDEX {
            // Save current editor state to previous layer
            let prev_idx = (PREV_LAYER_INDEX - 1) as usize;
            let (hi, lo) = pack_layer();
            LAYERS[prev_idx] = [hi, lo];

            // Load new layer into editor
            let new_idx = (LAYER_INDEX - 1) as usize;
            unpack_layer(LAYERS[new_idx][0], LAYERS[new_idx][1]);

            PREV_LAYER_INDEX = LAYER_INDEX;
        } else {
            // Layer index unchanged - pack editor state back to current layer
            let idx = (LAYER_INDEX - 1) as usize;
            let (hi, lo) = pack_layer();
            LAYERS[idx] = [hi, lo];
        }

        // Cycle shapes with X button
        if button_pressed(0, button::X) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % SHAPE_COUNT;
        }

        // Camera orbit control via left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 {
            CAM_ANGLE += stick_x * 2.0;
        }
        if stick_y.abs() > 0.1 {
            CAM_ELEVATION -= stick_y * 2.0;
            CAM_ELEVATION = CAM_ELEVATION.clamp(-60.0, 60.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Calculate orbit camera position
        let angle_rad = CAM_ANGLE * 0.0174533;
        let elev_rad = CAM_ELEVATION * 0.0174533;
        let dist = 5.0;

        let cam_x = dist * libm::cosf(elev_rad) * libm::sinf(angle_rad);
        let cam_y = dist * libm::sinf(elev_rad) + 1.0;
        let cam_z = dist * libm::cosf(elev_rad) * libm::cosf(angle_rad);

        camera_set(cam_x, cam_y, cam_z, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Build EPU config for rendering
        if ISOLATE_LAYER != 0 {
            // Isolation mode: only show selected layer
            let mut isolated: [[u64; 2]; 8] = [[0; 2]; 8];
            let idx = (LAYER_INDEX - 1) as usize;
            isolated[idx] = LAYERS[idx];
            epu_set(isolated.as_ptr() as *const u64);
        } else {
            // Full composition
            epu_set(LAYERS.as_ptr() as *const u64);
        }

        // Draw reference mesh with material properties
        push_identity();
        set_color(0xAAAAAAFF);
        material_metallic(MATERIAL_METALLIC);
        material_roughness(MATERIAL_ROUGHNESS);

        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        // Draw environment
        draw_epu();

        // Draw UI overlay
        draw_ui();
    }
}

unsafe fn draw_ui() {
    // Title
    let title = b"EPU Inspector";
    set_color(0xFFFFFFFF);
    draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

    // Layer indicator
    let mut layer_text = [0u8; 16];
    layer_text[0..7].copy_from_slice(b"Layer: ");
    layer_text[7] = b'0' + LAYER_INDEX;
    set_color(0xCCCCCCFF);
    draw_text(layer_text.as_ptr(), 8, 10.0, 35.0, 16.0);

    // Isolation indicator
    if ISOLATE_LAYER != 0 {
        let iso = b"[ISOLATED]";
        set_color(0xFFFF00FF);
        draw_text(iso.as_ptr(), iso.len() as u32, 100.0, 35.0, 16.0);
    }

    // Shape indicator
    let shape_name = SHAPE_NAMES[SHAPE_INDEX as usize];
    let mut shape_text = [0u8; 24];
    shape_text[0..7].copy_from_slice(b"Shape: ");
    let slen = copy_slice(&mut shape_text[7..], shape_name);
    set_color(0xAAAAAAFF);
    draw_text(shape_text.as_ptr(), (7 + slen) as u32, 200.0, 35.0, 16.0);

    // Hints
    if SHOW_HINTS != 0 {
        draw_hints();
    }

    // Control hints at bottom
    set_color(0x666666FF);
    let hint1 = b"F4: Debug Panel | X: Cycle shapes | Left stick: Orbit";
    draw_text(hint1.as_ptr(), hint1.len() as u32, 10.0, 200.0, 12.0);
    let hint2 = b"Edit layer values to see live changes";
    draw_text(hint2.as_ptr(), hint2.len() as u32, 10.0, 214.0, 12.0);
}

// ============================================================================
// Opcode Hint Data
// ============================================================================

/// Get parameter hints for a given opcode
/// Returns: (name, intensity hint, param_a hint, param_b hint, param_c hint, param_d hint)
fn get_opcode_hints(opcode: u8) -> (&'static [u8], &'static [u8], &'static [u8], &'static [u8], &'static [u8], &'static [u8]) {
    match opcode {
        0x00 => (b"NOP", b"-", b"-", b"-", b"-", b"-"),
        0x01 => (b"RAMP", b"softness", b"wall_r", b"wall_g", b"wall_b", b"thresholds"),
        0x02 => (b"SECTOR", b"opening", b"azimuth", b"width", b"-", b"-"),
        0x03 => (b"SILHOUETTE", b"edge_soft", b"height", b"roughness", b"octaves", b"-"),
        0x04 => (b"SPLIT", b"-", b"blend_width", b"angle", b"sides", b"offset"),
        0x05 => (b"CELL", b"outline", b"density", b"fill", b"gap_width", b"seed"),
        0x06 => (b"PATCHES", b"-", b"scale", b"coverage", b"sharpness", b"seed"),
        0x07 => (b"APERTURE", b"edge_soft", b"half_width", b"half_height", b"frame", b"(varies)"),
        0x08 => (b"DECAL", b"brightness", b"shape+soft", b"size", b"glow_soft", b"phase"),
        0x09 => (b"GRID", b"brightness", b"scale", b"thickness", b"pat+scroll", b"phase"),
        0x0A => (b"SCATTER", b"brightness", b"density", b"size", b"twinkle", b"seed"),
        0x0B => (b"FLOW", b"brightness", b"scale", b"turbulence", b"oct+pat", b"phase"),
        0x0C => (b"TRACE", b"brightness", b"count", b"thickness", b"jitter", b"seed+shape"),
        0x0D => (b"VEIL", b"brightness", b"count", b"thickness", b"sway", b"phase"),
        0x0E => (b"ATMOSPHERE", b"strength", b"falloff", b"horizon_y", b"mie_conc", b"mie_exp"),
        0x0F => (b"PLANE", b"contrast", b"scale", b"gap_width", b"roughness", b"phase"),
        0x10 => (b"CELESTIAL", b"brightness", b"ang_size", b"limb_dark", b"phase_ang", b"(varies)"),
        0x11 => (b"PORTAL", b"glow", b"size", b"glow_width", b"roughness", b"phase"),
        0x12 => (b"LOBE", b"brightness", b"exponent", b"falloff", b"waveform", b"phase"),
        0x13 => (b"BAND", b"brightness", b"width", b"y_offset", b"softness", b"phase"),
        _ => (b"UNKNOWN", b"-", b"-", b"-", b"-", b"-"),
    }
}

fn copy_slice(dst: &mut [u8], src: &[u8]) -> usize {
    let len = dst.len().min(src.len());
    dst[..len].copy_from_slice(&src[..len]);
    len
}

unsafe fn draw_hints() {
    let (name, hint_i, hint_a, hint_b, hint_c, hint_d) = get_opcode_hints(EDITOR.opcode);

    let y_base = 60.0;
    let line_height = 14.0;
    set_color(0x88FF88FF);

    // Opcode name
    draw_text(name.as_ptr(), name.len() as u32, 10.0, y_base, 14.0);

    set_color(0x888888FF);

    // intensity
    let mut buf = [0u8; 32];
    buf[0..3].copy_from_slice(b"i: ");
    let len_i = 3 + copy_slice(&mut buf[3..], hint_i);
    draw_text(buf.as_ptr(), len_i as u32, 10.0, y_base + line_height, 12.0);

    // param_a
    buf[0..3].copy_from_slice(b"a: ");
    let len_a = 3 + copy_slice(&mut buf[3..], hint_a);
    draw_text(buf.as_ptr(), len_a as u32, 10.0, y_base + line_height * 2.0, 12.0);

    // param_b
    buf[0..3].copy_from_slice(b"b: ");
    let len_b = 3 + copy_slice(&mut buf[3..], hint_b);
    draw_text(buf.as_ptr(), len_b as u32, 10.0, y_base + line_height * 3.0, 12.0);

    // param_c
    buf[0..3].copy_from_slice(b"c: ");
    let len_c = 3 + copy_slice(&mut buf[3..], hint_c);
    draw_text(buf.as_ptr(), len_c as u32, 10.0, y_base + line_height * 4.0, 12.0);

    // param_d
    buf[0..3].copy_from_slice(b"d: ");
    let len_d = 3 + copy_slice(&mut buf[3..], hint_d);
    draw_text(buf.as_ptr(), len_d as u32, 10.0, y_base + line_height * 5.0, 12.0);
}
