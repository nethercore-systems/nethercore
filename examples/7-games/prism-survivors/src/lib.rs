//! PRISM SURVIVORS - 4-Player Co-op Roguelite Survival
//!
//! A run-based survival roguelite for Nethercore ZX featuring:
//!
//! ## Core Features
//! - 6 unique hero classes with distinct starting weapons
//! - 18 power-ups (10 weapons + 8 passives) with roguelite level-up choices
//! - 10 synergy combinations for emergent build variety
//! - Wave-based enemy spawning with 7 basic, 4 elite, and 2 boss types
//! - 4 distinct stage environments with unique hazards
//! - Full 4-player co-op with rollback netcode and revive system
//!
//! ## Difficulty System
//! - Easy: 0.6x HP, 0.7x damage, 0.8x spawns, elites wave 7+
//! - Normal: Balanced gameplay, elites wave 5+
//! - Hard: 1.4x HP, 1.3x damage, 1.2x spawns, +10% XP bonus, elites wave 4+
//! - Nightmare: 2.0x HP, 1.6x damage, 1.5x spawns, +25% XP bonus, elites wave 3+
//!
//! ## Player Scaling
//! Enemy HP and spawn rates scale with player count (1-4 players)
//! for balanced challenge regardless of party size.
//!
//! ## Combo System
//! Chain kills to build combo multiplier (up to 2.5x XP bonus).
//! Combo tiers: 5+ (green), 10+ (orange), 25+ (gold), 50+ (purple).
//!
//! ## Synergies
//! - Steam Explosion: Fireball + Ice Shards (extra shards + AoE slow)
//! - Chain Lightning: Magic Missile + Lightning (chain damage)
//! - Whirlwind: Cleave + Shadow Orb (spinning blade attack)
//! - Holy Fire: Holy Nova + Fireball (ignite enemies)
//! - Frost Arrow: Piercing Arrow + Ice Shards (slow + pierce)
//! - Void Drain: Soul Drain + Shadow Orb (orbs heal)
//! - Divine Bolt: Divine Crush + Lightning (stun + chain)
//! - Berserk Fury: Cleave + Fury (damage scales with missing HP)
//! - Vampire Touch: Soul Drain + Vitality (double lifesteal)
//! - Speed Demon: Swiftness + Haste (+30% proj speed, -20% cooldown)
//!
//! ## Environmental Hazards
//! Each stage has unique hazards that spawn periodically:
//! - Crystal Cavern: Crystal shards
//! - Enchanted Forest: Poison clouds
//! - Volcanic Depths: Lava pools
//! - Void Realm: Void rifts
//!
//! Render Mode 3 (Specular-Shininess Blinn-Phong)

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// =============================================================================
// Modules
// =============================================================================

mod state;
mod types;
mod math;
mod powerups;
mod spawning;
mod weapons;
mod gameplay;
mod rendering;

use types::*;
use state::*;
use rendering::*;
use gameplay::*;

// =============================================================================
// FFI Declarations
// =============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration (init-only)
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);
    fn set_tick_rate(rate: u32);

    // System
    fn delta_time() -> f32;

    // ROM Asset Loading (init-only)
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;

    // Procedural Meshes (init-only)
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;

    // Textures
    fn texture_filter(filter: u32);

    // Render State
    fn depth_test(enabled: u32);
    fn cull_mode(mode: u32);

    // Lighting
    fn light_set(index: u32, x: f32, y: f32, z: f32);
    fn light_color(index: u32, color: u32);
    fn light_intensity(index: u32, intensity: f32);
    fn light_enable(index: u32);

    // Environment
    fn env_gradient(layer: u32, zenith: u32, sky_h: u32, ground_h: u32, nadir: u32, rot: f32, shift: f32);
    fn env_scatter(layer: u32, variant: u32, density: u32, size: u32, glow: u32, streak: u32, c1: u32, c2: u32, parallax: u32, psize: u32, phase: u32);
    fn draw_env();

    // Font
    fn load_font(texture: u32, char_width: u32, char_height: u32, first_codepoint: u32, char_count: u32) -> u32;
}

// =============================================================================
// Entry Points
// =============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        render_mode(3);
        set_tick_rate(2);
        set_clear_color(0x1a1a2eFF);
        depth_test(1);
        cull_mode(1);
        texture_filter(0);

        // Load textures - Heroes
        TEX_KNIGHT = rom_texture(b"knight".as_ptr(), 6);
        TEX_MAGE = rom_texture(b"mage".as_ptr(), 4);
        TEX_RANGER = rom_texture(b"ranger".as_ptr(), 6);
        TEX_CLERIC = rom_texture(b"cleric".as_ptr(), 6);
        TEX_NECROMANCER = rom_texture(b"necromancer".as_ptr(), 11);
        TEX_PALADIN = rom_texture(b"paladin".as_ptr(), 7);
        // Load textures - Enemies
        TEX_GOLEM = rom_texture(b"golem".as_ptr(), 5);
        TEX_CRAWLER = rom_texture(b"crawler".as_ptr(), 7);
        TEX_WISP = rom_texture(b"wisp".as_ptr(), 4);
        TEX_SKELETON = rom_texture(b"skeleton".as_ptr(), 8);
        TEX_SHADE = rom_texture(b"shade".as_ptr(), 5);
        TEX_BERSERKER = rom_texture(b"berserker".as_ptr(), 9);
        TEX_ARCANE_SENTINEL = rom_texture(b"arcane_sentinel".as_ptr(), 15);
        // Load textures - Elites
        TEX_CRYSTAL_KNIGHT = rom_texture(b"crystal_knight".as_ptr(), 14);
        TEX_VOID_MAGE = rom_texture(b"void_mage".as_ptr(), 9);
        TEX_GOLEM_TITAN = rom_texture(b"golem_titan".as_ptr(), 11);
        TEX_SPECTER_LORD = rom_texture(b"specter_lord".as_ptr(), 12);
        // Load textures - Bosses
        TEX_PRISM_COLOSSUS = rom_texture(b"prism_colossus".as_ptr(), 14);
        TEX_VOID_DRAGON = rom_texture(b"void_dragon".as_ptr(), 11);
        // Load textures - Other
        TEX_XP = rom_texture(b"xp_gem".as_ptr(), 6);
        TEX_ARENA = rom_texture(b"arena_floor".as_ptr(), 11);

        // Load meshes - Heroes
        MESH_KNIGHT = rom_mesh(b"knight".as_ptr(), 6);
        MESH_MAGE = rom_mesh(b"mage".as_ptr(), 4);
        MESH_RANGER = rom_mesh(b"ranger".as_ptr(), 6);
        MESH_CLERIC = rom_mesh(b"cleric".as_ptr(), 6);
        MESH_NECROMANCER = rom_mesh(b"necromancer".as_ptr(), 11);
        MESH_PALADIN = rom_mesh(b"paladin".as_ptr(), 7);
        // Load meshes - Enemies
        MESH_GOLEM = rom_mesh(b"golem".as_ptr(), 5);
        MESH_CRAWLER = rom_mesh(b"crawler".as_ptr(), 7);
        MESH_WISP = rom_mesh(b"wisp".as_ptr(), 4);
        MESH_SKELETON = rom_mesh(b"skeleton".as_ptr(), 8);
        MESH_SHADE = rom_mesh(b"shade".as_ptr(), 5);
        MESH_BERSERKER = rom_mesh(b"berserker".as_ptr(), 9);
        MESH_ARCANE_SENTINEL = rom_mesh(b"arcane_sentinel".as_ptr(), 15);
        // Load meshes - Elites
        MESH_CRYSTAL_KNIGHT = rom_mesh(b"crystal_knight".as_ptr(), 14);
        MESH_VOID_MAGE = rom_mesh(b"void_mage".as_ptr(), 9);
        MESH_GOLEM_TITAN = rom_mesh(b"golem_titan".as_ptr(), 11);
        MESH_SPECTER_LORD = rom_mesh(b"specter_lord".as_ptr(), 12);
        // Load meshes - Bosses
        MESH_PRISM_COLOSSUS = rom_mesh(b"prism_colossus".as_ptr(), 14);
        MESH_VOID_DRAGON = rom_mesh(b"void_dragon".as_ptr(), 11);
        // Load meshes - Other
        MESH_XP = rom_mesh(b"xp_gem".as_ptr(), 6);
        MESH_ARENA = rom_mesh(b"arena_floor".as_ptr(), 11);
        MESH_PROJ = sphere(0.12, 6, 4);

        // Load sounds
        SFX_SHOOT = rom_sound(b"shoot".as_ptr(), 5);
        SFX_HIT = rom_sound(b"hit".as_ptr(), 3);
        SFX_DEATH = rom_sound(b"death".as_ptr(), 5);
        SFX_XP = rom_sound(b"xp".as_ptr(), 2);
        SFX_LEVELUP = rom_sound(b"level_up".as_ptr(), 8);
        SFX_HURT = rom_sound(b"hurt".as_ptr(), 4);
        SFX_SELECT = rom_sound(b"select".as_ptr(), 6);

        // Load font
        TEX_FONT = rom_texture(b"prism_font".as_ptr(), 10);
        FONT_PRISM = load_font(TEX_FONT, 8, 12, 32, 96);

        // Lighting
        light_set(0, -0.5, -1.0, -0.3);
        light_color(0, 0xFFFFFFFF);
        light_intensity(0, 1.5);
        light_enable(0);

        PHASE = GamePhase::Title;
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        match PHASE {
            GamePhase::Title => update_title(),
            GamePhase::ClassSelect => update_class_select(),
            GamePhase::Playing => update_playing(),
            GamePhase::LevelUp => update_levelup(),
            GamePhase::Paused => update_paused(),
            GamePhase::GameOver | GamePhase::Victory => update_endgame(),
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Stage-specific environment (use GAME_TIME for visual sync across clients)
        match STAGE {
            Stage::CrystalCavern => render_env_crystal_cavern(),
            Stage::EnchantedForest => render_env_forest(),
            Stage::VolcanicDepths => render_env_volcano(),
            Stage::VoidRealm => render_env_void(),
        }
        draw_env();

        match PHASE {
            GamePhase::Title => render_title(),
            GamePhase::ClassSelect => render_class_select(),
            GamePhase::Playing | GamePhase::Paused | GamePhase::LevelUp => {
                render_game();
                render_hud();
                if PHASE == GamePhase::Paused { render_pause(); }
                if PHASE == GamePhase::LevelUp { render_levelup(); }
            }
            GamePhase::GameOver => { render_game(); render_gameover(); }
            GamePhase::Victory => { render_game(); render_victory(); }
            _ => {}
        }
    }
}
