//! Netplay Demo - Arena Combat
//!
//! A complete example demonstrating Nethercore's multiplayer capabilities:
//!
//! ## Features Demonstrated
//!
//! ### Rollback Mechanics
//! - All game state in static variables (WASM memory is snapshotted)
//! - Deterministic `update()` function (same inputs = same state)
//! - No state changes in `render()` (skipped during rollback replay)
//! - Use of `random()` for deterministic randomness
//! - Fixed iteration order for all game objects
//!
//! ### Connection Handling
//! - `player_count()` to detect connected players
//! - `local_player_mask()` to identify which players are local
//! - Automatic transition between 1-4 player modes
//! - Connection status display in UI
//!
//! ### Save Sync
//! - `save()` / `load()` for persistent player stats
//! - Per-player save slots (0-3)
//! - Save data only persists for local players
//! - Remote player slots never overwrite local saves
//!
//! ## Game Rules
//!
//! - 2-4 players compete in an arena
//! - Each player has health and can shoot projectiles
//! - Last player standing wins the round
//! - Stats (wins, total kills) are saved between sessions
//!
//! ## Controls
//!
//! - Left Stick: Move
//! - Right Stick: Aim
//! - A Button: Shoot
//! - Start: Pause / Ready up
//!
//! ## Rollback Safety Notes
//!
//! This example follows all the rules for rollback-safe code:
//!
//! 1. **All state in WASM memory**: Every game variable is `static mut`
//! 2. **Deterministic update**: `update()` produces identical results given identical inputs
//! 3. **No state in render**: `render()` only reads state, never modifies it
//! 4. **Synchronized randomness**: All random values from `random()`
//! 5. **Fixed iteration order**: Arrays, not hashmaps; indexed loops
//!
//! When testing multiplayer, give identical inputs and verify identical states.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// === FFI Imports ===
#[path = "../../../../include/zx/mod.rs"]
mod ffi;
use ffi::*;

// === Constants ===

// Screen dimensions
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Arena bounds (with margins)
const ARENA_MARGIN: f32 = 50.0;
const ARENA_LEFT: f32 = ARENA_MARGIN;
const ARENA_RIGHT: f32 = SCREEN_WIDTH - ARENA_MARGIN;
const ARENA_TOP: f32 = ARENA_MARGIN + 40.0; // Extra space for HUD
const ARENA_BOTTOM: f32 = SCREEN_HEIGHT - ARENA_MARGIN;

// Player constants
const MAX_PLAYERS: usize = 4;
const PLAYER_RADIUS: f32 = 20.0;
const PLAYER_SPEED: f32 = 4.0;
const PLAYER_MAX_HEALTH: i32 = 100;
const PLAYER_INVULN_TICKS: u32 = 60; // 1 second of invulnerability after spawn
const SHOOT_COOLDOWN: u32 = 15; // Ticks between shots

// Projectile constants
const MAX_PROJECTILES: usize = 64;
const PROJECTILE_SPEED: f32 = 8.0;
const PROJECTILE_RADIUS: f32 = 5.0;
const PROJECTILE_DAMAGE: i32 = 25;
const PROJECTILE_LIFETIME: u32 = 120; // 2 seconds

// Button indices
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;
const BUTTON_A: u32 = 4;
const BUTTON_START: u32 = 12;

// Colors for each player
const PLAYER_COLORS: [u32; MAX_PLAYERS] = [
    0x4a9fffFF, // Player 1: Blue
    0xff6b6bFF, // Player 2: Red
    0x6bff6bFF, // Player 3: Green
    0xffff6bFF, // Player 4: Yellow
];

// UI Colors
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0x888888FF;
const COLOR_DARK: u32 = 0x1a1a2eFF;
const COLOR_ARENA_BG: u32 = 0x2a2a4eFF;
const COLOR_ARENA_BORDER: u32 = 0x4a4a6eFF;

// === Save Data Structure ===

/// Player statistics saved to disk.
/// Uses `#[repr(C)]` for stable memory layout across sessions.
#[repr(C)]
#[derive(Clone, Copy)]
struct PlayerStats {
    magic: u32,       // Magic number to validate save
    version: u32,     // Save format version
    total_wins: u32,  // Total round wins
    total_kills: u32, // Total kills
    total_deaths: u32,
    total_shots: u32,
    total_hits: u32,
    checksum: u32, // Simple checksum for validation
}

impl PlayerStats {
    const MAGIC: u32 = 0x4E504C59; // "NPLY"
    const VERSION: u32 = 1;

    const fn new() -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            total_wins: 0,
            total_kills: 0,
            total_deaths: 0,
            total_shots: 0,
            total_hits: 0,
            checksum: 0,
        }
    }

    fn calculate_checksum(&self) -> u32 {
        // Simple XOR checksum of all fields except checksum itself
        self.magic
            ^ self.version
            ^ self.total_wins
            ^ self.total_kills
            ^ self.total_deaths
            ^ self.total_shots
            ^ self.total_hits
    }

    fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
            && self.version == Self::VERSION
            && self.checksum == self.calculate_checksum()
    }
}

// === Game State Structures ===

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Lobby,     // Waiting for players to ready up
    Countdown, // 3-2-1 countdown
    Playing,   // Active gameplay
    RoundOver, // Someone won, showing results
}

#[derive(Clone, Copy)]
struct Player {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    health: i32,
    alive: bool,
    active: bool, // Is this player slot in use?
    ready: bool,  // Ready in lobby?
    shoot_cooldown: u32,
    invuln_timer: u32, // Invulnerability frames after spawn
    // Session stats (not saved, reset each game)
    kills_this_round: u32,
}

impl Player {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            health: PLAYER_MAX_HEALTH,
            alive: false,
            active: false,
            ready: false,
            shoot_cooldown: 0,
            invuln_timer: 0,
            kills_this_round: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Projectile {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    owner: u32, // Player index who fired this
    lifetime: u32,
    active: bool,
}

impl Projectile {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            owner: 0,
            lifetime: 0,
            active: false,
        }
    }
}

// === All Game State (Static for Rollback Safety) ===
//
// IMPORTANT: All game state MUST be in static variables.
// The Nethercore runtime snapshots all WASM memory for rollback.
// Any state outside WASM memory (impossible in no_std anyway) would desync.

static mut STATE: GameState = GameState::Lobby;
static mut PLAYERS: [Player; MAX_PLAYERS] = [Player::new(); MAX_PLAYERS];
static mut PROJECTILES: [Projectile; MAX_PROJECTILES] = [Projectile::new(); MAX_PROJECTILES];

// Player stats (loaded from save at init, saved on win)
static mut PLAYER_STATS: [PlayerStats; MAX_PLAYERS] = [PlayerStats::new(); MAX_PLAYERS];

// Countdown timer (in ticks)
static mut COUNTDOWN_TIMER: u32 = 0;
static mut ROUND_WINNER: u32 = 0;
static mut ROUND_OVER_TIMER: u32 = 0;

// Connection state tracking
// NOTE: We cache player_count() at specific moments (lobby enter, round start)
// rather than polling every frame. Polling in update() is non-deterministic
// during rollback and causes desync.
static mut CACHED_PLAYER_COUNT: u32 = 1;
static mut CACHED_LOCAL_MASK: u32 = 1;

// === Helper Functions ===

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

fn abs(v: f32) -> f32 {
    if v < 0.0 {
        -v
    } else {
        v
    }
}

fn sqrt(v: f32) -> f32 {
    libm::sqrtf(v)
}

fn normalize(x: f32, y: f32) -> (f32, f32) {
    let len = sqrt(x * x + y * y);
    if len > 0.001 {
        (x / len, y / len)
    } else {
        (0.0, 0.0)
    }
}

fn distance_sq(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}

fn draw_text_str(s: &[u8], x: f32, y: f32, size: f32) {
    unsafe {
        draw_text(s.as_ptr(), s.len() as u32, x, y, size);
    }
}

fn log_str(s: &[u8]) {
    unsafe {
        log(s.as_ptr(), s.len() as u32);
    }
}

// === Save/Load Functions ===

/// Load player stats from save slot.
/// Each player has their own save slot (0-3).
fn load_player_stats(player_idx: u32) {
    unsafe {
        let stats = &mut PLAYER_STATS[player_idx as usize];
        let bytes = stats as *mut PlayerStats as *mut u8;
        let size = core::mem::size_of::<PlayerStats>() as u32;

        let read = load(player_idx, bytes, size);
        if read != size || !stats.is_valid() {
            // Invalid or empty save - reset to defaults
            *stats = PlayerStats::new();
            log_str(b"[netplay-demo] No valid save for player, using defaults");
        } else {
            log_str(b"[netplay-demo] Loaded player stats from save");
        }
    }
}

/// Save player stats to save slot.
/// Only saves for local players (remote slots don't persist).
fn save_player_stats(player_idx: u32) {
    unsafe {
        // Check if this player is local
        let local_mask = CACHED_LOCAL_MASK;
        if (local_mask & (1 << player_idx)) == 0 {
            // Not a local player - skip save
            return;
        }

        let stats = &mut PLAYER_STATS[player_idx as usize];
        stats.checksum = stats.calculate_checksum();

        let bytes = stats as *const PlayerStats as *const u8;
        let size = core::mem::size_of::<PlayerStats>() as u32;

        let result = save(player_idx, bytes, size);
        if result == 0 {
            log_str(b"[netplay-demo] Saved player stats");
        } else {
            log_str(b"[netplay-demo] Failed to save player stats");
        }
    }
}

// === Spawn Positions ===

/// Get spawn position for a player.
/// Players spawn in corners of the arena.
fn get_spawn_position(player_idx: u32) -> (f32, f32) {
    let margin = 80.0;
    match player_idx % 4 {
        0 => (ARENA_LEFT + margin, ARENA_TOP + margin), // Top-left
        1 => (ARENA_RIGHT - margin, ARENA_BOTTOM - margin), // Bottom-right
        2 => (ARENA_RIGHT - margin, ARENA_TOP + margin), // Top-right
        3 => (ARENA_LEFT + margin, ARENA_BOTTOM - margin), // Bottom-left
        _ => (SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
    }
}

// === Game Logic ===

fn spawn_player(player_idx: u32) {
    unsafe {
        let (x, y) = get_spawn_position(player_idx);
        let player = &mut PLAYERS[player_idx as usize];

        player.x = x;
        player.y = y;
        player.vx = 0.0;
        player.vy = 0.0;
        player.health = PLAYER_MAX_HEALTH;
        player.alive = true;
        player.active = true;
        player.shoot_cooldown = 0;
        player.invuln_timer = PLAYER_INVULN_TICKS;
        player.kills_this_round = 0;
    }
}

fn spawn_projectile(owner: u32, x: f32, y: f32, dir_x: f32, dir_y: f32) {
    unsafe {
        // Find inactive projectile slot
        for i in 0..MAX_PROJECTILES {
            if !PROJECTILES[i].active {
                let (nx, ny) = normalize(dir_x, dir_y);
                PROJECTILES[i] = Projectile {
                    x,
                    y,
                    vx: nx * PROJECTILE_SPEED,
                    vy: ny * PROJECTILE_SPEED,
                    owner,
                    lifetime: PROJECTILE_LIFETIME,
                    active: true,
                };

                // Update stats
                PLAYER_STATS[owner as usize].total_shots += 1;
                return;
            }
        }
    }
}

fn reset_for_new_round() {
    unsafe {
        // Deactivate all projectiles
        for i in 0..MAX_PROJECTILES {
            PROJECTILES[i].active = false;
        }

        // Spawn all active players
        for i in 0..MAX_PLAYERS {
            if PLAYERS[i].active {
                spawn_player(i as u32);
                PLAYERS[i].ready = false;
            }
        }
    }
}

fn start_round() {
    unsafe {
        reset_for_new_round();
        STATE = GameState::Countdown;
        COUNTDOWN_TIMER = 180; // 3 seconds
    }
}

fn count_alive_players() -> u32 {
    unsafe {
        let mut count = 0;
        for i in 0..MAX_PLAYERS {
            if PLAYERS[i].active && PLAYERS[i].alive {
                count += 1;
            }
        }
        count
    }
}

fn get_last_alive_player() -> Option<u32> {
    unsafe {
        for i in 0..MAX_PLAYERS {
            if PLAYERS[i].active && PLAYERS[i].alive {
                return Some(i as u32);
            }
        }
        None
    }
}

// === Update Functions ===

fn update_player_input(player_idx: u32) {
    unsafe {
        let player = &mut PLAYERS[player_idx as usize];
        if !player.active || !player.alive {
            return;
        }

        // Movement (left stick or d-pad)
        let mut move_x = left_stick_x(player_idx);
        let mut move_y = left_stick_y(player_idx);

        // D-pad fallback
        if button_held(player_idx, BUTTON_LEFT) != 0 {
            move_x = -1.0;
        }
        if button_held(player_idx, BUTTON_RIGHT) != 0 {
            move_x = 1.0;
        }
        if button_held(player_idx, BUTTON_UP) != 0 {
            move_y = -1.0;
        }
        if button_held(player_idx, BUTTON_DOWN) != 0 {
            move_y = 1.0;
        }

        // Apply movement
        let (nx, ny) = normalize(move_x, move_y);
        player.x += nx * PLAYER_SPEED;
        player.y += ny * PLAYER_SPEED;

        // Clamp to arena bounds
        player.x = clamp(
            player.x,
            ARENA_LEFT + PLAYER_RADIUS,
            ARENA_RIGHT - PLAYER_RADIUS,
        );
        player.y = clamp(
            player.y,
            ARENA_TOP + PLAYER_RADIUS,
            ARENA_BOTTOM - PLAYER_RADIUS,
        );

        // Shooting (A button + right stick aim)
        if player.shoot_cooldown > 0 {
            player.shoot_cooldown -= 1;
        }

        if button_held(player_idx, BUTTON_A) != 0 && player.shoot_cooldown == 0 {
            // Get aim direction from right stick
            let aim_x = right_stick_x(player_idx);
            let aim_y = right_stick_y(player_idx);

            // If no aim input, shoot in movement direction or default right
            let (dir_x, dir_y) = if abs(aim_x) > 0.3 || abs(aim_y) > 0.3 {
                (aim_x, aim_y)
            } else if abs(move_x) > 0.1 || abs(move_y) > 0.1 {
                (move_x, move_y)
            } else {
                (1.0, 0.0) // Default: shoot right
            };

            spawn_projectile(player_idx, player.x, player.y, dir_x, dir_y);
            player.shoot_cooldown = SHOOT_COOLDOWN;
        }

        // Update invulnerability
        if player.invuln_timer > 0 {
            player.invuln_timer -= 1;
        }
    }
}

fn update_projectiles() {
    unsafe {
        for i in 0..MAX_PROJECTILES {
            let proj = &mut PROJECTILES[i];
            if !proj.active {
                continue;
            }

            // Move projectile
            proj.x += proj.vx;
            proj.y += proj.vy;

            // Update lifetime
            proj.lifetime -= 1;
            if proj.lifetime == 0 {
                proj.active = false;
                continue;
            }

            // Check arena bounds
            if proj.x < ARENA_LEFT
                || proj.x > ARENA_RIGHT
                || proj.y < ARENA_TOP
                || proj.y > ARENA_BOTTOM
            {
                proj.active = false;
                continue;
            }

            // Check collision with players
            for p in 0..MAX_PLAYERS {
                let player = &mut PLAYERS[p];
                if !player.active || !player.alive {
                    continue;
                }
                if p as u32 == proj.owner {
                    continue; // Can't hit self
                }
                if player.invuln_timer > 0 {
                    continue; // Invulnerable
                }

                let dist_sq = distance_sq(proj.x, proj.y, player.x, player.y);
                let hit_radius = PLAYER_RADIUS + PROJECTILE_RADIUS;

                if dist_sq < hit_radius * hit_radius {
                    // Hit!
                    player.health -= PROJECTILE_DAMAGE;
                    proj.active = false;

                    // Update shooter stats
                    PLAYER_STATS[proj.owner as usize].total_hits += 1;

                    if player.health <= 0 {
                        player.alive = false;
                        PLAYER_STATS[p].total_deaths += 1;
                        PLAYER_STATS[proj.owner as usize].total_kills += 1;
                        PLAYERS[proj.owner as usize].kills_this_round += 1;
                    }
                    break;
                }
            }
        }
    }
}

fn update_lobby() {
    unsafe {
        // Update player count (safe to poll in lobby - not during active gameplay)
        CACHED_PLAYER_COUNT = player_count();
        CACHED_LOCAL_MASK = local_player_mask();

        // Mark active players based on current player count
        for i in 0..MAX_PLAYERS {
            let should_be_active = (i as u32) < CACHED_PLAYER_COUNT;
            if should_be_active && !PLAYERS[i].active {
                // New player joined - load their stats
                PLAYERS[i].active = true;
                PLAYERS[i].ready = false;
                load_player_stats(i as u32);
            } else if !should_be_active {
                PLAYERS[i].active = false;
                PLAYERS[i].ready = false;
            }
        }

        // Check for ready input (Start button)
        for i in 0..CACHED_PLAYER_COUNT {
            if button_pressed(i, BUTTON_START) != 0 {
                PLAYERS[i as usize].ready = !PLAYERS[i as usize].ready;
            }
        }

        // Check if all active players are ready and we have at least 2
        if CACHED_PLAYER_COUNT >= 2 {
            let mut all_ready = true;
            for i in 0..CACHED_PLAYER_COUNT as usize {
                if !PLAYERS[i].ready {
                    all_ready = false;
                    break;
                }
            }

            if all_ready {
                start_round();
            }
        }
    }
}

fn update_countdown() {
    unsafe {
        COUNTDOWN_TIMER -= 1;
        if COUNTDOWN_TIMER == 0 {
            STATE = GameState::Playing;
        }
    }
}

fn update_playing() {
    unsafe {
        // Process input for each player
        // IMPORTANT: Fixed iteration order for determinism
        for i in 0..CACHED_PLAYER_COUNT {
            update_player_input(i);
        }

        // Update projectiles
        update_projectiles();

        // Check for round end
        let alive = count_alive_players();
        if alive <= 1 {
            STATE = GameState::RoundOver;
            ROUND_OVER_TIMER = 180; // 3 seconds

            if let Some(winner) = get_last_alive_player() {
                ROUND_WINNER = winner;
                PLAYER_STATS[winner as usize].total_wins += 1;
                // Save winner's stats
                save_player_stats(winner);
            } else {
                ROUND_WINNER = 0xFF; // Draw
            }
        }
    }
}

fn update_round_over() {
    unsafe {
        ROUND_OVER_TIMER -= 1;
        if ROUND_OVER_TIMER == 0 {
            // Return to lobby
            STATE = GameState::Lobby;
            for i in 0..MAX_PLAYERS {
                PLAYERS[i].ready = false;
            }
        }
    }
}

// === Render Functions ===

fn render_arena() {
    unsafe {
        // Arena background
        set_color(COLOR_ARENA_BG);
        draw_rect(
            ARENA_LEFT,
            ARENA_TOP,
            ARENA_RIGHT - ARENA_LEFT,
            ARENA_BOTTOM - ARENA_TOP,
        );

        // Arena border
        set_color(COLOR_ARENA_BORDER);
        let border = 3.0;

        // Top
        draw_rect(ARENA_LEFT, ARENA_TOP, ARENA_RIGHT - ARENA_LEFT, border);
        // Bottom
        draw_rect(
            ARENA_LEFT,
            ARENA_BOTTOM - border,
            ARENA_RIGHT - ARENA_LEFT,
            border,
        );
        // Left
        draw_rect(ARENA_LEFT, ARENA_TOP, border, ARENA_BOTTOM - ARENA_TOP);
        // Right
        draw_rect(
            ARENA_RIGHT - border,
            ARENA_TOP,
            border,
            ARENA_BOTTOM - ARENA_TOP,
        );
    }
}

fn render_player(player_idx: usize) {
    unsafe {
        let player = &PLAYERS[player_idx];
        if !player.active || !player.alive {
            return;
        }

        let color = PLAYER_COLORS[player_idx];

        // Flash when invulnerable
        if player.invuln_timer > 0 && (player.invuln_timer / 5) % 2 == 0 {
            set_color(COLOR_WHITE);
        } else {
            set_color(color);
        }

        // Draw player as circle (using rect approximation)
        draw_rect(
            player.x - PLAYER_RADIUS,
            player.y - PLAYER_RADIUS,
            PLAYER_RADIUS * 2.0,
            PLAYER_RADIUS * 2.0,
        );

        // Health bar above player
        let bar_width = 40.0;
        let bar_height = 4.0;
        let bar_y = player.y - PLAYER_RADIUS - 10.0;

        // Background
        set_color(0x333333FF);
        draw_rect(player.x - bar_width / 2.0, bar_y, bar_width, bar_height);

        // Health fill
        let health_pct = player.health as f32 / PLAYER_MAX_HEALTH as f32;
        set_color(if health_pct > 0.5 {
            0x44FF44FF
        } else if health_pct > 0.25 {
            0xFFFF44FF
        } else {
            0xFF4444FF
        });
        draw_rect(
            player.x - bar_width / 2.0,
            bar_y,
            bar_width * health_pct,
            bar_height,
        );

        // Player number
        let num = [b'1' + player_idx as u8];
        set_color(COLOR_WHITE);
        draw_text(num.as_ptr(), 1, player.x - 4.0, player.y - 6.0, 16.0);
    }
}

fn render_projectiles() {
    unsafe {
        for i in 0..MAX_PROJECTILES {
            let proj = &PROJECTILES[i];
            if !proj.active {
                continue;
            }

            // Color based on owner
            set_color(PLAYER_COLORS[proj.owner as usize]);
            draw_rect(
                proj.x - PROJECTILE_RADIUS,
                proj.y - PROJECTILE_RADIUS,
                PROJECTILE_RADIUS * 2.0,
                PROJECTILE_RADIUS * 2.0,
            );
        }
    }
}

fn render_hud() {
    unsafe {
        // Title
        set_color(COLOR_WHITE);
        draw_text_str(b"NETPLAY DEMO", 10.0, 10.0, 20.0);

        // Connection status
        set_color(COLOR_GRAY);
        let count = CACHED_PLAYER_COUNT;
        let status = match count {
            1 => b"1 Player (Waiting...)" as &[u8],
            2 => b"2 Players Connected" as &[u8],
            3 => b"3 Players Connected" as &[u8],
            4 => b"4 Players Connected" as &[u8],
            _ => b"Unknown" as &[u8],
        };
        draw_text_str(status, SCREEN_WIDTH - 200.0, 10.0, 14.0);

        // Player indicators with scores
        let mut x = 250.0;
        for i in 0..MAX_PLAYERS {
            if !PLAYERS[i].active {
                continue;
            }

            // Player color box
            set_color(PLAYER_COLORS[i]);
            draw_rect(x, 8.0, 16.0, 16.0);

            // "P1", "P2", etc.
            set_color(COLOR_WHITE);
            let label = [b'P', b'1' + i as u8];
            draw_text(label.as_ptr(), 2, x + 20.0, 10.0, 14.0);

            // Wins count
            set_color(COLOR_GRAY);
            let wins = PLAYER_STATS[i].total_wins;
            let win_label = [b'W', b':', b'0' + (wins % 10) as u8];
            draw_text(win_label.as_ptr(), 3, x + 45.0, 10.0, 12.0);

            x += 90.0;
        }
    }
}

fn render_lobby() {
    unsafe {
        render_arena();
        render_hud();

        // Center text
        set_color(COLOR_WHITE);
        draw_text_str(b"ARENA COMBAT", SCREEN_WIDTH / 2.0 - 100.0, 150.0, 32.0);

        // Instructions
        set_color(COLOR_GRAY);
        if CACHED_PLAYER_COUNT < 2 {
            draw_text_str(
                b"Waiting for players...",
                SCREEN_WIDTH / 2.0 - 100.0,
                200.0,
                18.0,
            );
            draw_text_str(
                b"Connect another controller or player online",
                SCREEN_WIDTH / 2.0 - 180.0,
                230.0,
                14.0,
            );
        } else {
            draw_text_str(
                b"Press START when ready",
                SCREEN_WIDTH / 2.0 - 110.0,
                200.0,
                18.0,
            );
        }

        // Player ready status
        let mut y = 280.0;
        for i in 0..MAX_PLAYERS {
            if !PLAYERS[i].active {
                continue;
            }

            set_color(PLAYER_COLORS[i]);
            let label = [b'P', b'1' + i as u8, b':'];
            draw_text(label.as_ptr(), 3, SCREEN_WIDTH / 2.0 - 60.0, y, 20.0);

            if PLAYERS[i].ready {
                set_color(0x44FF44FF);
                draw_text_str(b"READY", SCREEN_WIDTH / 2.0, y, 20.0);
            } else {
                set_color(COLOR_GRAY);
                draw_text_str(b"NOT READY", SCREEN_WIDTH / 2.0, y, 20.0);
            }

            // Show if local or remote
            let is_local = (CACHED_LOCAL_MASK & (1 << i)) != 0;
            set_color(if is_local { 0x88FF88FF } else { 0xFF8888FF });
            let locality = if is_local {
                b"(Local)" as &[u8]
            } else {
                b"(Remote)" as &[u8]
            };
            draw_text_str(locality, SCREEN_WIDTH / 2.0 + 100.0, y, 12.0);

            y += 30.0;
        }

        // Controls hint
        set_color(COLOR_GRAY);
        draw_text_str(
            b"Controls: Left Stick = Move, Right Stick = Aim, A = Shoot",
            SCREEN_WIDTH / 2.0 - 220.0,
            SCREEN_HEIGHT - 60.0,
            14.0,
        );
    }
}

fn render_countdown() {
    unsafe {
        render_arena();
        render_hud();

        // Render players at spawn positions
        for i in 0..MAX_PLAYERS {
            render_player(i);
        }

        // Countdown number
        let seconds = (COUNTDOWN_TIMER / 60) + 1;
        set_color(COLOR_WHITE);
        let digit = [b'0' + seconds as u8];
        draw_text(
            digit.as_ptr(),
            1,
            SCREEN_WIDTH / 2.0 - 20.0,
            SCREEN_HEIGHT / 2.0 - 40.0,
            80.0,
        );

        if seconds <= 1 {
            set_color(0x44FF44FF);
            draw_text_str(
                b"FIGHT!",
                SCREEN_WIDTH / 2.0 - 50.0,
                SCREEN_HEIGHT / 2.0 + 40.0,
                32.0,
            );
        }
    }
}

fn render_playing() {
    render_arena();
    render_hud();

    // Render projectiles first (under players)
    render_projectiles();

    // Render players
    for i in 0..MAX_PLAYERS {
        render_player(i);
    }
}

fn render_round_over() {
    unsafe {
        render_arena();
        render_hud();
        render_projectiles();

        for i in 0..MAX_PLAYERS {
            render_player(i);
        }

        // Overlay
        set_color(0x000000AA);
        draw_rect(
            SCREEN_WIDTH / 4.0,
            SCREEN_HEIGHT / 3.0,
            SCREEN_WIDTH / 2.0,
            SCREEN_HEIGHT / 3.0,
        );

        // Winner announcement
        if ROUND_WINNER < MAX_PLAYERS as u32 {
            set_color(PLAYER_COLORS[ROUND_WINNER as usize]);
            let label = [b'P', b'1' + ROUND_WINNER as u8];
            draw_text(
                label.as_ptr(),
                2,
                SCREEN_WIDTH / 2.0 - 100.0,
                SCREEN_HEIGHT / 2.0 - 30.0,
                40.0,
            );
            set_color(COLOR_WHITE);
            draw_text_str(
                b"WINS!",
                SCREEN_WIDTH / 2.0 - 10.0,
                SCREEN_HEIGHT / 2.0 - 30.0,
                40.0,
            );
        } else {
            set_color(COLOR_WHITE);
            draw_text_str(
                b"DRAW!",
                SCREEN_WIDTH / 2.0 - 50.0,
                SCREEN_HEIGHT / 2.0 - 30.0,
                40.0,
            );
        }

        // Stats for winner
        if ROUND_WINNER < MAX_PLAYERS as u32 {
            set_color(COLOR_GRAY);
            let kills = PLAYERS[ROUND_WINNER as usize].kills_this_round;
            let label = [
                b'K',
                b'i',
                b'l',
                b'l',
                b's',
                b':',
                b' ',
                b'0' + (kills % 10) as u8,
            ];
            draw_text(
                label.as_ptr(),
                8,
                SCREEN_WIDTH / 2.0 - 40.0,
                SCREEN_HEIGHT / 2.0 + 20.0,
                18.0,
            );
        }

        // Return hint
        set_color(COLOR_GRAY);
        draw_text_str(
            b"Returning to lobby...",
            SCREEN_WIDTH / 2.0 - 80.0,
            SCREEN_HEIGHT / 2.0 + 60.0,
            14.0,
        );
    }
}

// === Entry Points ===

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(COLOR_DARK);

        // Initialize state
        STATE = GameState::Lobby;
        CACHED_PLAYER_COUNT = player_count();
        CACHED_LOCAL_MASK = local_player_mask();

        // Load stats for initially connected players
        for i in 0..CACHED_PLAYER_COUNT {
            PLAYERS[i as usize].active = true;
            load_player_stats(i);
        }

        log_str(b"[netplay-demo] Initialized");
    }
}

/// Update function - called every tick.
///
/// CRITICAL FOR ROLLBACK:
/// - This function must be deterministic
/// - Same inputs must produce same outputs
/// - Do NOT poll player_count() here during gameplay (non-deterministic during rollback)
/// - Use CACHED_PLAYER_COUNT which is set at safe moments (lobby, round start)
#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        match STATE {
            GameState::Lobby => update_lobby(),
            GameState::Countdown => update_countdown(),
            GameState::Playing => update_playing(),
            GameState::RoundOver => update_round_over(),
        }
    }
}

/// Render function - called every frame.
///
/// CRITICAL FOR ROLLBACK:
/// - This function is SKIPPED during rollback replay
/// - Do NOT modify any game state here
/// - Only READ state and issue draw calls
#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        match STATE {
            GameState::Lobby => render_lobby(),
            GameState::Countdown => render_countdown(),
            GameState::Playing => render_playing(),
            GameState::RoundOver => render_round_over(),
        }
    }
}
