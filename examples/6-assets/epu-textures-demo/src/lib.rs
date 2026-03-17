#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const IMPORTED_SOURCE_COUNT: i32 = 3;
const SOURCE_COUNT: i32 = 4;

const SOURCE_AXIS: i32 = 0;
const SOURCE_STUDIO: i32 = 1;
const SOURCE_NEON: i32 = 2;
const SOURCE_PROCEDURAL: i32 = 3;

static mut MESHES: Option<ShapeMeshes> = None;
static mut FACE_SETS: [[u32; 6]; IMPORTED_SOURCE_COUNT as usize] = [[0; 6]; 3];

static mut SOURCE_INDEX: i32 = SOURCE_AXIS;
static mut SHAPE_INDEX: i32 = 0;
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;
static mut ROTATION_SPEED: f32 = 24.0;
static mut OBJECT_COLOR: u32 = 0xF2F6FFFF;
static mut METALLIC_U8: i32 = 255;
static mut ROUGHNESS_U8: i32 = 28;
static mut SHOW_BACKGROUND: u8 = 1;
static mut SHOW_UI: u8 = 0;

static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 0.2,
    target_z: 0.0,
    distance: 6.4,
    elevation: 16.0,
    azimuth: 0.0,
    auto_orbit_speed: 0.0,
    stick_control: StickControl::LeftStick,
    fov: 55.0,
};

static EPU_PROCEDURAL: [[u64; 2]; 8] = [
    [0x0880005A1A103500, 0xDC3C1F00A50080FF],
    [0x480CFFAA102A2C00, 0xA03C0A00000000FF],
    [0x5800FFF018080000, 0xB86C3400000000DD],
    [0x0A8000FFF0A8D600, 0xC8301400000000DD],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
];

const SOURCE_NAMES: [&str; SOURCE_COUNT as usize] = [
    "Calibration: Axis Room",
    "Lookdev: Studio Warm",
    "Lookdev: Neon Night",
    "Procedural Reference",
];

unsafe fn load_face_set(slot: usize, ids: [&[u8]; 6]) {
    for (i, id) in ids.iter().enumerate() {
        FACE_SETS[slot][i] = rom_texture(id.as_ptr(), id.len() as u32);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x06080EFF);
        MESHES = Some(ShapeMeshes::generate());

        load_face_set(
            SOURCE_AXIS as usize,
            [
                b"axis_px", b"axis_nx", b"axis_py", b"axis_ny", b"axis_pz", b"axis_nz",
            ],
        );
        load_face_set(
            SOURCE_STUDIO as usize,
            [
                b"studio_px",
                b"studio_nx",
                b"studio_py",
                b"studio_ny",
                b"studio_pz",
                b"studio_nz",
            ],
        );
        load_face_set(
            SOURCE_NEON as usize,
            [
                b"neon_px", b"neon_nx", b"neon_py", b"neon_ny", b"neon_pz", b"neon_nz",
            ],
        );

        use_uniform_color(1);
        use_uniform_metallic(1);
        use_uniform_roughness(1);
        use_uniform_emissive(1);

        material_metallic(1.0);
        material_roughness(ROUGHNESS_U8 as f32 / 255.0);
        material_emissive(0.0);

        light_intensity(0, 0.0);

        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    debug_group_begin(b"scene".as_ptr(), 5);
    debug_register_i32(
        b"source_index".as_ptr(),
        12,
        &raw const SOURCE_INDEX as *const i32 as *const u8,
    );
    debug_register_bool(b"show_background".as_ptr(), 15, &raw const SHOW_BACKGROUND);
    debug_register_bool(b"show_ui".as_ptr(), 7, &raw const SHOW_UI);
    debug_group_end();

    debug_group_begin(b"shape".as_ptr(), 5);
    debug_register_i32(
        b"index".as_ptr(),
        5,
        &raw const SHAPE_INDEX as *const i32 as *const u8,
    );
    debug_register_f32(
        b"rotation_speed".as_ptr(),
        14,
        &raw const ROTATION_SPEED as *const f32 as *const u8,
    );
    debug_register_color(
        b"color".as_ptr(),
        5,
        &raw const OBJECT_COLOR as *const u32 as *const u8,
    );
    debug_group_end();

    debug_group_begin(b"material".as_ptr(), 8);
    debug_register_i32(
        b"metallic_u8".as_ptr(),
        11,
        &raw const METALLIC_U8 as *const i32 as *const u8,
    );
    debug_register_i32(
        b"roughness_u8".as_ptr(),
        12,
        &raw const ROUGHNESS_U8 as *const i32 as *const u8,
    );
    debug_group_end();

    debug_group_begin(b"camera".as_ptr(), 6);
    debug_register_f32(
        b"distance".as_ptr(),
        8,
        &raw const CAMERA.distance as *const f32 as *const u8,
    );
    debug_register_f32(
        b"elevation".as_ptr(),
        9,
        &raw const CAMERA.elevation as *const f32 as *const u8,
    );
    debug_register_f32(
        b"azimuth".as_ptr(),
        7,
        &raw const CAMERA.azimuth as *const f32 as *const u8,
    );
    debug_register_f32(
        b"auto_orbit".as_ptr(),
        10,
        &raw const CAMERA.auto_orbit_speed as *const f32 as *const u8,
    );
    debug_group_end();
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        SOURCE_INDEX = SOURCE_INDEX.clamp(0, SOURCE_COUNT - 1);
        SHAPE_INDEX = SHAPE_INDEX.clamp(0, ShapeType::COUNT - 1);
        METALLIC_U8 = METALLIC_U8.clamp(0, 255);
        ROUGHNESS_U8 = ROUGHNESS_U8.clamp(0, 255);
        ROTATION_SPEED = ROTATION_SPEED.clamp(0.0, 120.0);
        SHOW_BACKGROUND = if SHOW_BACKGROUND != 0 { 1 } else { 0 };
        SHOW_UI = if SHOW_UI != 0 { 1 } else { 0 };
        CAMERA.distance = CAMERA.distance.clamp(2.0, 20.0);
        CAMERA.elevation = CAMERA.elevation.clamp(-80.0, 80.0);
        CAMERA.auto_orbit_speed = CAMERA.auto_orbit_speed.clamp(0.0, 60.0);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::A) != 0 {
            SOURCE_INDEX = (SOURCE_INDEX + 1) % SOURCE_COUNT;
        }
        if button_pressed(0, button::B) != 0 {
            SOURCE_INDEX = (SOURCE_INDEX + SOURCE_COUNT - 1) % SOURCE_COUNT;
        }
        if button_pressed(0, button::X) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % ShapeType::COUNT;
        }
        if button_pressed(0, button::Y) != 0 {
            SHOW_BACKGROUND = if SHOW_BACKGROUND == 0 { 1 } else { 0 };
        }
        if button_pressed(0, button::START) != 0 {
            reset_camera();
        }
        if button_pressed(0, button::L1) != 0 {
            CAMERA.auto_orbit_speed = if CAMERA.auto_orbit_speed > 0.0 { 0.0 } else { 12.0 };
        }

        update_object_rotation();
        CAMERA.update();
    }
}

unsafe fn reset_camera() {
    CAMERA.distance = 6.4;
    CAMERA.elevation = 16.0;
    CAMERA.azimuth = 0.0;
    CAMERA.auto_orbit_speed = 0.0;
}

unsafe fn update_object_rotation() {
    let stick_x = right_stick_x(0);
    let stick_y = right_stick_y(0);

    if stick_x.abs() > 0.1 || stick_y.abs() > 0.1 {
        ROTATION_Y += stick_x * 3.0;
        ROTATION_X += stick_y * 3.0;
    } else {
        ROTATION_Y += ROTATION_SPEED * delta_time();
    }

    if ROTATION_Y >= 360.0 {
        ROTATION_Y -= 360.0;
    }
    if ROTATION_X >= 360.0 {
        ROTATION_X -= 360.0;
    }
}

unsafe fn apply_current_epu_source() {
    match SOURCE_INDEX {
        SOURCE_AXIS | SOURCE_STUDIO | SOURCE_NEON => {
            let faces = FACE_SETS[SOURCE_INDEX as usize];
            epu_textures(faces[0], faces[1], faces[2], faces[3], faces[4], faces[5]);
        }
        SOURCE_PROCEDURAL => epu_set(EPU_PROCEDURAL.as_ptr() as *const u64),
        _ => epu_set(EPU_PROCEDURAL.as_ptr() as *const u64),
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        CAMERA.apply();
        apply_current_epu_source();

        let meshes = MESHES.as_ref().unwrap();

        draw_hero_probe(meshes);
        draw_reference_probes(meshes);

        if SHOW_BACKGROUND != 0 {
            apply_current_epu_source();
            draw_epu();
        }

        if SHOW_UI != 0 {
            draw_ui();
        }
    }
}

unsafe fn draw_hero_probe(meshes: &ShapeMeshes) {
    let metallic = METALLIC_U8 as f32 / 255.0;
    let roughness = ROUGHNESS_U8 as f32 / 255.0;

    set_color(OBJECT_COLOR);
    material_metallic(metallic);
    material_roughness(roughness);
    material_emissive(0.0);

    push_identity();
    push_translate(0.0, 0.25, 0.0);
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    draw_mesh(meshes.get_by_index(SHAPE_INDEX));
}

unsafe fn draw_reference_probes(meshes: &ShapeMeshes) {
    let roughnesses = [0.05f32, 0.28f32, 0.72f32];
    let offsets = [-2.7f32, 0.0f32, 2.7f32];

    set_color(0xE0E6EEFF);
    material_metallic(1.0);
    material_emissive(0.0);

    for i in 0..3 {
        material_roughness(roughnesses[i]);
        push_identity();
        push_translate(offsets[i], -1.9, 0.0);
        push_scale(0.36, 0.36, 0.36);
        push_rotate_y(ROTATION_Y * 0.65 + i as f32 * 18.0);
        draw_mesh(meshes.get(ShapeType::Sphere));
    }
}

unsafe fn draw_ui() {
    set_color(0xFFFFFFFF);
    let title = b"EPU Textures Demo";
    draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

    let source_name = SOURCE_NAMES[SOURCE_INDEX as usize].as_bytes();
    draw_text(
        source_name.as_ptr(),
        source_name.len() as u32,
        10.0,
        40.0,
        16.0,
    );

    let shape_name = ShapeType::from_index(SHAPE_INDEX).name().as_bytes();
    let mut shape_label = [0u8; 24];
    let prefix = b"Shape: ";
    shape_label[..prefix.len()].copy_from_slice(prefix);
    shape_label[prefix.len()..prefix.len() + shape_name.len()].copy_from_slice(shape_name);
    set_color(0xCCCCCCFF);
    draw_text(
        shape_label.as_ptr(),
        (prefix.len() + shape_name.len()) as u32,
        10.0,
        60.0,
        16.0,
    );

    let mut material_label = [0u8; 48];
    let prefix = b"Hero probe: metallic ";
    material_label[..prefix.len()].copy_from_slice(prefix);
    let metal = write_percent(&mut material_label[prefix.len()..], METALLIC_U8);
    let sep = b" roughness ";
    let sep_start = prefix.len() + metal;
    material_label[sep_start..sep_start + sep.len()].copy_from_slice(sep);
    let rough = write_percent(&mut material_label[sep_start + sep.len()..], ROUGHNESS_U8);
    set_color(0xAAAAFFFF);
    draw_text(
        material_label.as_ptr(),
        (sep_start + sep.len() + rough) as u32,
        10.0,
        82.0,
        14.0,
    );

    set_color(0x888888FF);
    let hint1 = b"A/B: cycle EPU textures or procedural";
    draw_text(hint1.as_ptr(), hint1.len() as u32, 10.0, 110.0, 14.0);

    let hint2 = b"X: cycle hero shape | Y: background on/off";
    draw_text(hint2.as_ptr(), hint2.len() as u32, 10.0, 130.0, 14.0);

    let hint3 = b"Left stick: orbit | Right stick: rotate probe | Triggers: zoom";
    draw_text(hint3.as_ptr(), hint3.len() as u32, 10.0, 150.0, 14.0);

    let hint4 = b"L1: auto-orbit on/off | START: reset camera";
    draw_text(hint4.as_ptr(), hint4.len() as u32, 10.0, 170.0, 14.0);

    let hint5 = b"Bottom spheres: roughness 0.05 / 0.28 / 0.72";
    draw_text(hint5.as_ptr(), hint5.len() as u32, 10.0, 190.0, 14.0);

    let hint6 = b"F4: Debug Inspector";
    draw_text(hint6.as_ptr(), hint6.len() as u32, 10.0, 210.0, 14.0);
}

fn write_percent(buf: &mut [u8], value: i32) -> usize {
    let v = value.clamp(0, 255);
    let pct = (v * 100 + 127) / 255;
    if pct >= 100 {
        buf[0] = b'1';
        buf[1] = b'0';
        buf[2] = b'0';
        buf[3] = b'%';
        4
    } else if pct >= 10 {
        buf[0] = b'0' + (pct / 10) as u8;
        buf[1] = b'0' + (pct % 10) as u8;
        buf[2] = b'%';
        3
    } else {
        buf[0] = b'0' + pct as u8;
        buf[1] = b'%';
        2
    }
}
