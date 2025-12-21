///! Hello World - Emberware ZX (Zig Version)
///!
///! A simple game that draws a colored square and responds to input.
///! Demonstrates the core concepts of Emberware game development in Zig.
///!
///! Build with: zig build
///! Run with: ember run zig-out/bin/game.wasm

// =============================================================================
// FFI Imports from Emberware Runtime
// =============================================================================

extern fn set_clear_color(color: u32) void;
extern fn button_pressed(player: u32, button: u32) u32;
extern fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32) void;
extern fn draw_text(ptr: [*]const u8, len: u32, x: f32, y: f32, size: f32, color: u32) void;

// =============================================================================
// Constants
// =============================================================================

const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;

const WHITE: u32 = 0xFFFFFFFF;
const SALMON: u32 = 0xFF6B6BFF;
const GRAY: u32 = 0x888888FF;
const DARK_BLUE: u32 = 0x1a1a2eFF;

// =============================================================================
// Game State
// =============================================================================

// Game state - stored in static variables for rollback safety
var square_y: f32 = 200.0;

// =============================================================================
// Game Lifecycle Functions
// =============================================================================

export fn init() void {
    // Set the background color (dark blue)
    set_clear_color(DARK_BLUE);
}

export fn update() void {
    // Move square with D-pad
    if (button_pressed(0, BUTTON_UP) != 0) {
        square_y -= 10.0;
    }
    if (button_pressed(0, BUTTON_DOWN) != 0) {
        square_y += 10.0;
    }

    // Reset position with A button
    if (button_pressed(0, BUTTON_A) != 0) {
        square_y = 200.0;
    }

    // Keep square on screen
    square_y = @max(20.0, @min(square_y, 450.0));
}

export fn render() void {
    // Draw title text
    const title = "Hello Emberware!";
    draw_text(title.ptr, title.len, 80.0, 50.0, 32.0, WHITE);

    // Draw the moving square
    draw_rect(200.0, square_y, 80.0, 80.0, SALMON);

    // Draw instructions
    const hint = "D-pad: Move   A: Reset";
    draw_text(hint.ptr, hint.len, 60.0, 500.0, 18.0, GRAY);
}
