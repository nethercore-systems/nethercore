//! Native (non-WASM) stubs for unit tests / tooling.
//! These allow `cargo test --target x86_64-pc-windows-msvc` to link without the ZX host.

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn tick_count() -> u64 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn random() -> u32 {
    // Deterministic LCG.
    static mut STATE: u32 = 0x1234_5678;
    unsafe {
        const MUL: u32 = 1664525;
        const INC: u32 = 1013904223;
        STATE = STATE.wrapping_mul(MUL).wrapping_add(INC);
        STATE
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn random_f32() -> f32 {
    random() as f32 / u32::MAX as f32
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn button_held(_player: u32, _button: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn set_clear_color(_color: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn set_color(_color: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn z_index(_n: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn push_identity() {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn transform_set(_matrix_ptr: *const f32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn push_view_matrix(
    _m0: f32,
    _m1: f32,
    _m2: f32,
    _m3: f32,
    _m4: f32,
    _m5: f32,
    _m6: f32,
    _m7: f32,
    _m8: f32,
    _m9: f32,
    _m10: f32,
    _m11: f32,
    _m12: f32,
    _m13: f32,
    _m14: f32,
    _m15: f32,
) {
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn push_projection_matrix(
    _m0: f32,
    _m1: f32,
    _m2: f32,
    _m3: f32,
    _m4: f32,
    _m5: f32,
    _m6: f32,
    _m7: f32,
    _m8: f32,
    _m9: f32,
    _m10: f32,
    _m11: f32,
    _m12: f32,
    _m13: f32,
    _m14: f32,
    _m15: f32,
) {
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn draw_rect(_x: f32, _y: f32, _w: f32, _h: f32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn draw_text(_ptr: *const u8, _len: u32, _x: f32, _y: f32, _size: f32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn draw_mesh(_handle: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn cube(_size_x: f32, _size_y: f32, _size_z: f32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn sphere(_radius: f32, _segments: u32, _rings: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn matcap_set(_slot: u32, _texture: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn skeleton_bind(_skeleton: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn keyframes_bone_count(_handle: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn keyframes_frame_count(_handle: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn keyframe_bind(_handle: u32, _index: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn play_sound(_sound: u32, _volume: f32, _pan: f32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn music_play(_handle: u32, _volume: f32, _looping: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn music_stop() {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn log(_ptr: *const u8, _len: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn rom_texture(_id_ptr: *const u8, _id_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn rom_mesh(_id_ptr: *const u8, _id_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn rom_sound(_id_ptr: *const u8, _id_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn rom_skeleton(_id_ptr: *const u8, _id_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn rom_tracker(_id_ptr: *const u8, _id_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn rom_keyframes(_id_ptr: *const u8, _id_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn rom_font(_id_ptr: *const u8, _id_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn rom_data_len(_id_ptr: *const u8, _id_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn debug_register_u8(_name_ptr: *const u8, _name_len: u32, _ptr: *const u8) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn debug_register_u16(_name_ptr: *const u8, _name_len: u32, _ptr: *const u8) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn debug_register_f32(_name_ptr: *const u8, _name_len: u32, _ptr: *const u8) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn debug_register_i32(_name_ptr: *const u8, _name_len: u32, _ptr: *const u8) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn debug_register_bool(_name_ptr: *const u8, _name_len: u32, _ptr: *const u8) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn debug_group_begin(_name_ptr: *const u8, _name_len: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn debug_group_end() {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn load_zskeleton(_data_ptr: *const u8, _data_len: u32) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn material_specular_damping(_value: f32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn material_spec_damping(_value: f32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn material_specular_color(_r: f32, _g: f32, _b: f32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn use_uniform_specular(_enabled: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn use_matcap_reflection(_enabled: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn use_uniform_specular_damping(_enabled: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn use_uniform_shininess(_enabled: u32) {}
