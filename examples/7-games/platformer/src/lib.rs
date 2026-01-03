//! Platformer Example
//!
//! A full mini-game demonstrating multiple Nethercore ZX features:
//! - 2D gameplay using 3D renderer (side-scrolling view)
//! - Textured sprites for player/enemies
//! - Billboarded sprites in 3D space
//! - Simple physics (gravity, friction)
//! - AABB collision detection (platforms, collectibles)
//! - Multiple players with analog stick input
//! - 2D UI overlay with `draw_text()`, `draw_rect()`
//! - Sky background with `set_sky()`
//! - Rollback-safe game state (all state in statics)
//!
//! Note: Rollback state is automatic (entire WASM memory is snapshotted). No save_state/load_state needed.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Trigger a WASM trap so runtime can catch the error
    // instead of infinite loop which freezes the game
    core::arch::wasm32::unreachable()
}

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;


// === Constants ===

// Button indices
const BUTTON_A: u32 = 4;
const BUTTON_START: u32 = 12;

// Billboard modes
const MODE_CYLINDRICAL_Y: u32 = 2;

// Game constants
const MAX_PLAYERS: usize = 4;
const GRAVITY: f32 = 0.5;
const JUMP_FORCE: f32 = 12.0;
const MOVE_SPEED: f32 = 5.0;
const FRICTION: f32 = 0.85;
const PLAYER_WIDTH: f32 = 0.8;
const PLAYER_HEIGHT: f32 = 1.2;

// Level bounds
const LEVEL_LEFT: f32 = -12.0;
const LEVEL_RIGHT: f32 = 12.0;
const LEVEL_BOTTOM: f32 = -2.0;

// Platforms
const MAX_PLATFORMS: usize = 12;
const MAX_COLLECTIBLES: usize = 8;

// Player colors (RGBA)
const PLAYER_COLORS: [u32; 4] = [
    0x4a9fffFF, // Blue
    0xff6b6bFF, // Red
    0x6bff6bFF, // Green
    0xffff6bFF, // Yellow
];

// === Game State (all rollback-safe) ===

#[derive(Clone, Copy)]
struct Player {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    on_ground: bool,
    facing_right: bool,
    score: u32,
    active: bool,
}

impl Player {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            on_ground: false,
            facing_right: true,
            score: 0,
            active: false,
        }
    }
}

#[derive(Clone, Copy)]
struct Platform {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    active: bool,
}

impl Platform {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            active: false,
        }
    }
}

#[derive(Clone, Copy)]
struct Collectible {
    x: f32,
    y: f32,
    collected: bool,
    bob_offset: f32,
}

impl Collectible {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            collected: false,
            bob_offset: 0.0,
        }
    }
}

// Game state (static for rollback safety)
static mut PLAYERS: [Player; MAX_PLAYERS] = [Player::new(); MAX_PLAYERS];
static mut PLATFORMS: [Platform; MAX_PLATFORMS] = [Platform::new(); MAX_PLATFORMS];
static mut COLLECTIBLES: [Collectible; MAX_COLLECTIBLES] = [Collectible::new(); MAX_COLLECTIBLES];
static mut TICK: u32 = 0;
static mut GAME_OVER: bool = false;

// Texture handles
static mut PLAYER_TEXTURE: u32 = 0;
static mut PLATFORM_TEXTURE: u32 = 0;
static mut COIN_TEXTURE: u32 = 0;

// === Textures (8x8 pixel art) ===

// Player sprite (simple character silhouette)
const PLAYER_PIXELS: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let white = [0xFF, 0xFF, 0xFF, 0xFF];
    let trans = [0x00, 0x00, 0x00, 0x00];

    // Simple character shape
    let pattern: [[u8; 8]; 8] = [
        [0, 0, 1, 1, 1, 1, 0, 0], // Head top
        [0, 1, 1, 1, 1, 1, 1, 0], // Head
        [0, 1, 1, 1, 1, 1, 1, 0], // Head
        [0, 0, 1, 1, 1, 1, 0, 0], // Neck
        [0, 1, 1, 1, 1, 1, 1, 0], // Body
        [1, 1, 1, 1, 1, 1, 1, 1], // Body + arms
        [0, 0, 1, 0, 0, 1, 0, 0], // Legs
        [0, 0, 1, 0, 0, 1, 0, 0], // Feet
    ];

    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = if pattern[y][x] == 1 { white } else { trans };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};

// Platform texture (brick pattern)
const PLATFORM_PIXELS: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let brown1 = [0x8B, 0x45, 0x13, 0xFF]; // Dark brown
    let brown2 = [0xA0, 0x52, 0x2D, 0xFF]; // Lighter brown
    let mortar = [0x60, 0x40, 0x30, 0xFF]; // Mortar color

    // Brick pattern
    let pattern: [[u8; 8]; 8] = [
        [1, 1, 1, 0, 2, 2, 2, 0],
        [1, 1, 1, 0, 2, 2, 2, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [2, 0, 1, 1, 1, 0, 2, 2],
        [2, 0, 1, 1, 1, 0, 2, 2],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 1, 0, 2, 2, 2, 0],
        [1, 1, 1, 0, 2, 2, 2, 0],
    ];

    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = match pattern[y][x] {
                0 => mortar,
                1 => brown1,
                _ => brown2,
            };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};

// Coin texture (golden circle)
const COIN_PIXELS: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let gold = [0xFF, 0xD7, 0x00, 0xFF];
    let dark_gold = [0xDA, 0xA5, 0x20, 0xFF];
    let trans = [0x00, 0x00, 0x00, 0x00];

    // Coin shape with highlight
    let pattern: [[u8; 8]; 8] = [
        [0, 0, 1, 1, 1, 1, 0, 0],
        [0, 1, 2, 2, 1, 1, 1, 0],
        [1, 2, 2, 1, 1, 1, 1, 1],
        [1, 2, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 0, 1, 1, 1, 1, 0, 0],
    ];

    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = match pattern[y][x] {
                0 => trans,
                1 => gold,
                _ => dark_gold,
            };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};

// === Helper Functions ===

fn draw_text_str(s: &str, x: f32, y: f32, size: f32) {
    unsafe {
        draw_text(s.as_ptr(), s.len() as u32, x, y, size);
    }
}

// Using libm for accurate no_std math
#[inline]
fn sin_approx(x: f32) -> f32 {
    libm::sinf(x)
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min { min } else if v > max { max } else { v }
}

fn abs(v: f32) -> f32 {
    if v < 0.0 { -v } else { v }
}

// AABB collision check
fn aabb_overlap(
    x1: f32, y1: f32, w1: f32, h1: f32,
    x2: f32, y2: f32, w2: f32, h2: f32,
) -> bool {
    x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
}

// === Initialization ===

fn init_level() {
    unsafe {
        // Initialize platforms
        // Ground platform (wide)
        PLATFORMS[0] = Platform {
            x: -10.0,
            y: LEVEL_BOTTOM,
            width: 20.0,
            height: 0.5,
            active: true,
        };

        // Floating platforms
        PLATFORMS[1] = Platform { x: -8.0, y: 0.0, width: 3.0, height: 0.4, active: true };
        PLATFORMS[2] = Platform { x: -3.0, y: 1.5, width: 2.5, height: 0.4, active: true };
        PLATFORMS[3] = Platform { x: 2.0, y: 0.5, width: 3.0, height: 0.4, active: true };
        PLATFORMS[4] = Platform { x: 6.0, y: 2.0, width: 2.5, height: 0.4, active: true };
        PLATFORMS[5] = Platform { x: -5.0, y: 3.5, width: 2.0, height: 0.4, active: true };
        PLATFORMS[6] = Platform { x: 0.0, y: 4.0, width: 3.0, height: 0.4, active: true };
        PLATFORMS[7] = Platform { x: 5.0, y: 4.5, width: 2.5, height: 0.4, active: true };

        // Initialize collectibles (coins)
        COLLECTIBLES[0] = Collectible { x: -7.0, y: 1.0, collected: false, bob_offset: 0.0 };
        COLLECTIBLES[1] = Collectible { x: -2.0, y: 2.5, collected: false, bob_offset: 0.5 };
        COLLECTIBLES[2] = Collectible { x: 3.0, y: 1.5, collected: false, bob_offset: 1.0 };
        COLLECTIBLES[3] = Collectible { x: 7.0, y: 3.0, collected: false, bob_offset: 1.5 };
        COLLECTIBLES[4] = Collectible { x: -4.0, y: 4.5, collected: false, bob_offset: 2.0 };
        COLLECTIBLES[5] = Collectible { x: 1.0, y: 5.0, collected: false, bob_offset: 2.5 };
        COLLECTIBLES[6] = Collectible { x: 6.0, y: 5.5, collected: false, bob_offset: 3.0 };
        COLLECTIBLES[7] = Collectible { x: 0.0, y: 0.0, collected: false, bob_offset: 3.5 };

        // Initialize players
        let count = player_count().min(MAX_PLAYERS as u32) as usize;
        for i in 0..MAX_PLAYERS {
            if i < count {
                PLAYERS[i] = Player {
                    x: -8.0 + (i as f32 * 2.0),
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    on_ground: false,
                    facing_right: true,
                    score: 0,
                    active: true,
                };
            } else {
                PLAYERS[i].active = false;
            }
        }

        TICK = 0;
        GAME_OVER = false;
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Sky blue clear color
        set_clear_color(0x87CEEBFF);

        // Note: Sky uses reasonable defaults (blue gradient with sun) from the renderer
        // No need to set sky explicitly unless you want custom sky settings

        // Load textures
        PLAYER_TEXTURE = load_texture(8, 8, PLAYER_PIXELS.as_ptr());
        PLATFORM_TEXTURE = load_texture(8, 8, PLATFORM_PIXELS.as_ptr());
        COIN_TEXTURE = load_texture(8, 8, COIN_PIXELS.as_ptr());

        // Nearest-neighbor for crisp pixels
        texture_filter(0);

        // Initialize level
        init_level();
    }
}

// === Update ===

fn update_player(player_idx: usize) {
    unsafe {
        let p = &mut PLAYERS[player_idx];
        if !p.active {
            return;
        }

        // Read input
        let stick_x = left_stick_x(player_idx as u32);
        let jump_pressed = button_pressed(player_idx as u32, BUTTON_A) != 0;
        let jump_held = button_held(player_idx as u32, BUTTON_A) != 0;

        // Horizontal movement
        p.vx += stick_x * MOVE_SPEED * 0.1;
        p.vx *= FRICTION;

        // Clamp horizontal velocity
        p.vx = clamp(p.vx, -MOVE_SPEED, MOVE_SPEED);

        // Update facing direction
        if abs(stick_x) > 0.3 {
            p.facing_right = stick_x > 0.0;
        }

        // Jump (only when on ground)
        if jump_pressed && p.on_ground {
            p.vy = JUMP_FORCE;
            p.on_ground = false;
        }

        // Variable jump height (release early = lower jump)
        if !jump_held && p.vy > 0.0 {
            p.vy *= 0.5;
        }

        // Apply gravity
        p.vy -= GRAVITY;

        // Apply velocity
        let new_x = p.x + p.vx * 0.1;
        let new_y = p.y + p.vy * 0.1;

        // Collision detection with platforms
        p.on_ground = false;

        // Check collision with each platform
        for platform in &PLATFORMS {
            if !platform.active {
                continue;
            }

            // Player AABB (centered at x, y is bottom)
            let px = new_x - PLAYER_WIDTH / 2.0;
            let py = new_y;
            let pw = PLAYER_WIDTH;
            let ph = PLAYER_HEIGHT;

            // Platform AABB
            let plx = platform.x;
            let ply = platform.y;
            let plw = platform.width;
            let plh = platform.height;

            // Check if overlapping
            if aabb_overlap(px, py, pw, ph, plx, ply, plw, plh) {
                // Determine collision side
                // Coming from above (landing)
                if p.vy <= 0.0 && p.y >= platform.y + platform.height - 0.1 {
                    p.y = platform.y + platform.height;
                    p.vy = 0.0;
                    p.on_ground = true;
                }
                // Coming from below (hitting head)
                else if p.vy > 0.0 && p.y + PLAYER_HEIGHT <= platform.y + 0.2 {
                    p.y = platform.y - PLAYER_HEIGHT;
                    p.vy = 0.0;
                }
            }
        }

        // Update position if no collision stopped it
        if !p.on_ground || p.vy > 0.0 {
            p.y = new_y;
        }
        p.x = new_x;

        // Clamp to level bounds
        p.x = clamp(p.x, LEVEL_LEFT, LEVEL_RIGHT);

        // Fall off bottom - respawn
        if p.y < LEVEL_BOTTOM - 5.0 {
            p.x = 0.0;
            p.y = 5.0;
            p.vx = 0.0;
            p.vy = 0.0;
        }

        // Check collectible collection
        for collectible in &mut COLLECTIBLES {
            if collectible.collected {
                continue;
            }

            // Simple distance check
            let dx = p.x - collectible.x;
            let dy = (p.y + PLAYER_HEIGHT / 2.0) - collectible.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < 1.0 {
                collectible.collected = true;
                p.score += 100;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if GAME_OVER {
            // Check for restart
            for i in 0..MAX_PLAYERS {
                if PLAYERS[i].active && button_pressed(i as u32, BUTTON_START) != 0 {
                    init_level();
                    return;
                }
            }
            return;
        }

        TICK += 1;

        // Update all active players
        for i in 0..MAX_PLAYERS {
            update_player(i);
        }

        // Check if all collectibles collected
        let mut all_collected = true;
        for collectible in &COLLECTIBLES {
            if !collectible.collected {
                all_collected = false;
                break;
            }
        }

        if all_collected {
            GAME_OVER = true;
        }
    }
}

// === Render ===

fn render_platforms() {
    unsafe {
        texture_bind(PLATFORM_TEXTURE);

        for platform in &PLATFORMS {
            if !platform.active {
                continue;
            }

            // Draw platform as multiple billboards (tiled)
            let tile_size = 1.0;
            let tiles_x = libm::ceilf(platform.width / tile_size) as i32;
            let tiles_y = libm::ceilf(platform.height / tile_size) as i32;

            set_color(0xFFFFFFFF);
            for ty in 0..tiles_y {
                for tx in 0..tiles_x {
                    let tile_x = platform.x + (tx as f32 + 0.5) * tile_size;
                    let tile_y = platform.y + (ty as f32 + 0.5) * tile_size;

                    push_identity();
                    push_translate(tile_x, tile_y, 0.0);
                    draw_billboard(tile_size, tile_size, MODE_CYLINDRICAL_Y);
                }
            }
        }
    }
}

fn render_collectibles() {
    unsafe {
        texture_bind(COIN_TEXTURE);
        set_color(0xFFFFFFFF);

        let time = TICK as f32 / 60.0;

        for collectible in &COLLECTIBLES {
            if collectible.collected {
                continue;
            }

            // Bob up and down
            let bob = sin_approx(time * 3.0 + collectible.bob_offset) * 0.15;

            push_identity();
            push_translate(collectible.x, collectible.y + bob, 0.1);
            draw_billboard(0.6, 0.6, MODE_CYLINDRICAL_Y);
        }
    }
}

fn render_players() {
    unsafe {
        texture_bind(PLAYER_TEXTURE);

        for (i, player) in PLAYERS.iter().enumerate() {
            if !player.active {
                continue;
            }

            // Flip sprite based on facing direction
            let scale_x = if player.facing_right { PLAYER_WIDTH } else { -PLAYER_WIDTH };

            push_identity();
            push_translate(player.x, player.y + PLAYER_HEIGHT / 2.0, 0.2);

            // Use player color as tint
            set_color(PLAYER_COLORS[i]);
            draw_billboard(scale_x, PLAYER_HEIGHT, MODE_CYLINDRICAL_Y);
        }
    }
}

fn render_ui() {
    unsafe {
        // Background panel for scores
        set_color(0x000000AA);
        draw_rect(10.0, 10.0, 300.0, 80.0 + (player_count() as f32 * 70.0));

        set_color(0xFFFFFFFF);
        draw_text_str("PLATFORMER", 20.0, 30.0, 24.0);

        // Player scores
        let mut y_offset = 100.0;
        for (i, player) in PLAYERS.iter().enumerate() {
            if !player.active {
                continue;
            }

            // Format score (simple approach - just show the number)
            let score = player.score;
            let digits = [
                b'0' + ((score / 1000) % 10) as u8,
                b'0' + ((score / 100) % 10) as u8,
                b'0' + ((score / 10) % 10) as u8,
                b'0' + (score % 10) as u8,
            ];

            // "P1: 0000" format
            let label = match i {
                0 => b"P1: ",
                1 => b"P2: ",
                2 => b"P3: ",
                _ => b"P4: ",
            };

            set_color(PLAYER_COLORS[i]);
        draw_text(label.as_ptr(), 4, 20.0, y_offset, 20.0);
            set_color(0xFFFFFFFF);
        draw_text(digits.as_ptr(), 4, 100.0, y_offset, 20.0);
            y_offset += 70.0;
        }

        // Coin counter
        let mut coins_left = 0u32;
        for collectible in &COLLECTIBLES {
            if !collectible.collected {
                coins_left += 1;
            }
        }

        let coins_text = [
            b'C', b'o', b'i', b'n', b's', b':', b' ',
            b'0' + (coins_left % 10) as u8,
        ];
        set_color(0xFFD700FF);
        draw_text(coins_text.as_ptr(), 8, 20.0, y_offset + 20.0, 18.0);

        // Controls hint
        set_color(0x000000AA);
        draw_rect(10.0, 480.0, 480.0, 90.0);
        set_color(0xCCCCCCFF);
        draw_text_str("L-Stick: Move  A: Jump", 20.0, 500.0, 16.0);
        set_color(0xFFD700FF);
        draw_text_str("Collect all coins!", 20.0, 540.0, 16.0);

        // Game over overlay
        if GAME_OVER {
            set_color(0x000000DD);
            draw_rect(150.0, 200.0, 660.0, 140.0);
            set_color(0xFFD700FF);
            draw_text_str("ALL COINS COLLECTED!", 200.0, 240.0, 28.0);
            set_color(0xCCCCCCFF);
            draw_text_str("Press START to restart", 240.0, 290.0, 20.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 2.0, 15.0, 0.0, 2.0, 0.0);
        camera_fov(45.0);

        set_color(0xFFFFFFFF);

        // Render game objects (back to front)
        render_platforms();
        render_collectibles();
        render_players();

        // UI on top
        render_ui();
    }
}

// Note: Rollback state is automatic (entire WASM memory is snapshotted). No save_state/load_state needed.
