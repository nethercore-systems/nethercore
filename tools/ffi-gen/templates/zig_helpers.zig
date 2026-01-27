// =============================================================================
// MANUALLY MAINTAINED HELPER FUNCTIONS
// =============================================================================
// These helpers provide Zig-specific conveniences using slices and native types

/// Color packing helpers
pub fn rgba(r: u8, g: u8, b: u8, a: u8) u32 {
    return (@as(u32, r) << 24) | (@as(u32, g) << 16) | (@as(u32, b) << 8) | @as(u32, a);
}

pub fn rgb(r: u8, g: u8, b: u8) u32 {
    return rgba(r, g, b, 255);
}

/// Math helpers using Zig built-ins
pub fn clampf(val: f32, min: f32, max: f32) f32 {
    return @max(min, @min(val, max));
}

pub fn lerpf(a: f32, b: f32, t: f32) f32 {
    return a + (b - a) * t;
}

/// String helpers using Zig slices
pub fn logStr(msg: []const u8) void {
    log(msg.ptr, @intCast(msg.len));
}

pub fn drawTextStr(msg: []const u8, x: f32, y: f32, size: f32, col: u32) void {
    set_color(col);
    draw_text(msg.ptr, @intCast(msg.len), x, y, size);
}

/// ROM loading helpers
pub fn romTexture(id: []const u8) u32 {
    return rom_texture(id.ptr, @intCast(id.len));
}

pub fn romMesh(id: []const u8) u32 {
    return rom_mesh(id.ptr, @intCast(id.len));
}

pub fn romSound(id: []const u8) u32 {
    return rom_sound(id.ptr, @intCast(id.len));
}

pub fn romFont(id: []const u8) u32 {
    return rom_font(id.ptr, @intCast(id.len));
}

pub fn romSkeleton(id: []const u8) u32 {
    return rom_skeleton(id.ptr, @intCast(id.len));
}
