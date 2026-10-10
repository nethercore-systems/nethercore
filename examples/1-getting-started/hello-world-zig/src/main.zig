//! Same first-cartridge lesson as Rust/C. `nether build` uses Zig 0.13.0.
const std = @import("std");
const zx = @import("zx");
var square_y: f32 = 240.0;

export fn init() void {
    zx.set_clear_color(0x172334ff);
    zx.debug_watch_f32("square_y".ptr, 8, std.mem.asBytes(&square_y).ptr);
}

export fn update() void {
    if (zx.button_pressed(0, zx.Button.up) != 0) square_y -= 10.0;
    if (zx.button_pressed(0, zx.Button.down) != 0) square_y += 10.0;
    if (zx.button_pressed(0, zx.Button.a) != 0) square_y = 240.0;
    square_y = @max(160.0, @min(square_y, 384.0));
}

fn text(s: []const u8, x: f32, y: f32, size: f32, color: u32) void {
    zx.set_color(color);
    zx.draw_text(s.ptr, @intCast(s.len), x, y, size);
}

export fn render() void {
    text("Hello Nethercore!", 48, 36, 32, 0xffffffff);
    text("960 x 540 | top-left origin | x=448 | 64 x 64 | y=160..384", 48, 92, 14, 0xabc1d6ff);
    zx.set_color(0x344c66ff);
    zx.draw_rect(96, 156, 768, 4);
    zx.draw_rect(96, 448, 768, 4);
    zx.set_color(0xff967aff);
    zx.draw_rect(448, square_y, 64, 64);
    const y: u32 = @intFromFloat(square_y);
    const label = [_]u8{ 'y', '=', '0' + @as(u8, @intCast(y / 100)), '0' + @as(u8, @intCast((y / 10) % 10)), '0' + @as(u8, @intCast(y % 10)) };
    text(&label, 532, square_y + 22, 18, 0xffffffff);
    text("D-pad Up/Down: step 10 pixels   A: reset", 48, 480, 18, 0xffffffff);
    text("Keyboard: arrows / Z   |   Hold does not repeat.   F3: inspect", 48, 512, 14, 0xabc1d6ff);
}
