//! Rendering functions for Prism Survivors
//!
//! Contains all rendering code for game phases and UI.

use crate::types::*;
use crate::state::*;
use crate::math::*;

#[link(wasm_import_module = "env")]
extern "C" {
    fn player_count() -> u32;

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Transforms
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_deg: f32);
    fn push_scale_uniform(s: f32);

    // Textures
    fn texture_bind(handle: u32);

    // Mesh Drawing
    fn draw_mesh(handle: u32);

    // Render State
    fn set_color(color: u32);

    // Materials (Mode 3)
    fn material_shininess(value: f32);
    fn material_specular(color: u32);

    // Environment
    fn env_gradient(layer: u32, zenith: u32, sky_h: u32, ground_h: u32, nadir: u32, rot: f32, shift: f32);
    fn env_scatter(layer: u32, variant: u32, density: u32, size: u32, glow: u32, streak: u32, c1: u32, c2: u32, parallax: u32, psize: u32, phase: u32);
    fn draw_env();

    // 2D Drawing
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn layer(n: u32);

    // Font
    fn font_bind(handle: u32);

    // Billboard (for VFX) - canonical API uses transform stack for position
    fn draw_billboard(w: f32, h: f32, mode: u32, color: u32);

    // Procedural shapes
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
}

// =============================================================================
// Stage Environments
// =============================================================================

pub fn render_env_crystal_cavern() {
    unsafe {
        env_gradient(0, 0x1a2a4aFF, 0x2a3a5aFF, 0x101828FF, 0x080810FF, 0.0, 0.0);
        env_scatter(1, 0, 30, 3, 200, 0, 0x8080FFFF, 0x4040AAFF, 20, 50, (GAME_TIME * 20.0) as u32);
    }
}

pub fn render_env_forest() {
    unsafe {
        env_gradient(0, 0x102810FF, 0x1a3a1aFF, 0x081808FF, 0x040804FF, 0.0, 0.0);
        env_scatter(1, 0, 40, 2, 150, 0, 0x40FF40FF, 0x208020FF, 15, 40, (GAME_TIME * 15.0) as u32);
    }
}

pub fn render_env_volcano() {
    unsafe {
        env_gradient(0, 0x2a1010FF, 0x4a2020FF, 0x200808FF, 0x100404FF, 0.0, 0.0);
        env_scatter(1, 0, 25, 4, 255, 10, 0xFF8040FF, 0xFF4020FF, 30, 60, (GAME_TIME * 25.0) as u32);
    }
}

pub fn render_env_void() {
    unsafe {
        env_gradient(0, 0x0a0a1aFF, 0x1a1a3aFF, 0x050510FF, 0x000008FF, 0.0, 0.0);
        env_scatter(1, 0, 50, 2, 180, 5, 0x8040FFFF, 0x4020AAFF, 40, 80, (GAME_TIME * 30.0) as u32);
    }
}

// =============================================================================
// Title Screen
// =============================================================================

pub fn render_title() {
    unsafe {
        let t = TITLE_TIMER;

        // Animated background gradient
        let phase = (t * 0.3) % 6.28;
        let bg_shift = (sin(phase) * 0.2 + 0.5) as f32;
        env_gradient(0, 0x0a0a18FF, 0x1a1a3eFF, 0x0e0e20FF, 0x000000FF, 0.0, bg_shift);
        env_scatter(0, 1, 40, 2, 100, 0, 0x00FFFFFF, 0xB060FFFF, 30, 3, (t * 10.0) as u32);
        draw_env();

        // Showcase rotating heroes in background
        camera_set(0.0, 3.0, 8.0, 0.0, 0.5, 0.0);
        camera_fov(45.0);

        // Draw rotating showcase of all heroes
        let hero_angle = t * 30.0;
        let radius = 2.5;

        // Knight
        push_identity();
        push_translate(cos(hero_angle * 0.0175) * radius, 0.0, sin(hero_angle * 0.0175) * radius);
        push_rotate_y(hero_angle + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_KNIGHT);
        material_shininess(0.6);
        material_specular(0x8080C0FF);
        draw_mesh(MESH_KNIGHT);

        // Mage (offset 60 degrees)
        push_identity();
        push_translate(cos((hero_angle + 60.0) * 0.0175) * radius, 0.0, sin((hero_angle + 60.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 60.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_MAGE);
        draw_mesh(MESH_MAGE);

        // Ranger (offset 120 degrees)
        push_identity();
        push_translate(cos((hero_angle + 120.0) * 0.0175) * radius, 0.0, sin((hero_angle + 120.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 120.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_RANGER);
        draw_mesh(MESH_RANGER);

        // Cleric (offset 180 degrees)
        push_identity();
        push_translate(cos((hero_angle + 180.0) * 0.0175) * radius, 0.0, sin((hero_angle + 180.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 180.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_CLERIC);
        draw_mesh(MESH_CLERIC);

        // Necromancer (offset 240 degrees)
        push_identity();
        push_translate(cos((hero_angle + 240.0) * 0.0175) * radius, 0.0, sin((hero_angle + 240.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 240.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_NECROMANCER);
        draw_mesh(MESH_NECROMANCER);

        // Paladin (offset 300 degrees)
        push_identity();
        push_translate(cos((hero_angle + 300.0) * 0.0175) * radius, 0.0, sin((hero_angle + 300.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 300.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_PALADIN);
        draw_mesh(MESH_PALADIN);

        // HUD layer
        layer(10);

        // Dark overlay for text readability
        draw_rect(0.0, 0.0, SCREEN_WIDTH, 160.0, 0x000000A0);
        draw_rect(0.0, 380.0, SCREEN_WIDTH, 160.0, 0x000000A0);

        // Use custom font
        font_bind(FONT_PRISM);

        // Animated title with wave effect
        let title_y = 50.0 + sin(t * 2.0) * 5.0;
        let title = b"PRISM SURVIVORS";

        // Draw title letter by letter with wave
        for (i, &c) in title.iter().enumerate() {
            let wave = sin(t * 4.0 + i as f32 * 0.4) * 4.0;
            let x = 200.0 + i as f32 * 38.0;
            let y = title_y + wave;
            // Rainbow color per letter
            let hue = (t * 60.0 + i as f32 * 25.0) % 360.0;
            let color = hsv_to_rgb(hue);
            draw_text(&c as *const u8, 1, x, y, 56.0, color);
        }

        // Subtitle
        draw_str(b"ROGUELITE CO-OP SURVIVAL", 280.0, 120.0, 22.0, 0x00FFFFFF);

        // Hero classes info (fade in after 1 second)
        let info_alpha = clamp((t - 1.0) * 2.0, 0.0, 1.0);
        if info_alpha > 0.0 {
            let a = (info_alpha * 255.0) as u32;
            draw_str(b"6 UNIQUE HEROES", 120.0, 390.0, 16.0, 0x00FFFF00 | a);
            draw_str(b"ROGUELITE UPGRADES", 350.0, 390.0, 16.0, 0xB060FF00 | a);
            draw_str(b"4-PLAYER CO-OP", 620.0, 390.0, 16.0, 0xFFFF4000 | a);
        }

        // Difficulty selection
        draw_str(b"DIFFICULTY:", 380.0, 420.0, 16.0, 0xAAAAAAFF);
        let difficulties = [
            (b"EASY" as &[u8], 0x44FF44FF),
            (b"NORMAL" as &[u8], 0xFFFF44FF),
            (b"HARD" as &[u8], 0xFF8844FF),
            (b"NIGHTMARE" as &[u8], 0xFF4444FF),
        ];

        let base_x = 280.0;
        for (i, (name, color)) in difficulties.iter().enumerate() {
            let x = base_x + i as f32 * 110.0;
            let y = 440.0;
            let selected = i as u8 == DIFFICULTY_SELECTION;

            // Selection indicator
            if selected {
                let pulse = (sin(t * 6.0) + 1.0) * 0.3 + 0.7;
                let c = (*color & 0xFFFFFF00) | ((pulse * 255.0) as u32);
                draw_str(b">", x - 12.0, y, 20.0, c);
                draw_text(name.as_ptr(), name.len() as u32, x, y, 20.0, c);
            } else {
                draw_text(name.as_ptr(), name.len() as u32, x, y, 16.0, (*color & 0xFFFFFF00) | 0x88);
            }
        }

        // Instructions
        draw_str(b"UP/DOWN: SELECT DIFFICULTY", 320.0, 470.0, 14.0, 0x888888FF);

        // Blinking "Press Start" text
        let blink = (sin(t * 4.0) + 1.0) * 0.5;
        let alpha = ((0.3 + blink * 0.7) * 255.0) as u32;
        draw_str(b"PRESS A OR START", 340.0, 495.0, 24.0, 0xFFFFFF00 | alpha);

        // Player count
        let mut buf = [0u8; 16];
        buf[0..9].copy_from_slice(b"PLAYERS: ");
        buf[9] = b'0' + player_count() as u8;
        draw_text(buf.as_ptr(), 10, 780.0, 500.0, 16.0, 0x88FF88FF);

        // Reset font
        font_bind(0);
    }
}

// =============================================================================
// Game Rendering
// =============================================================================

pub fn render_game() {
    unsafe {
        // Fixed height camera (25 units up) following players, looking down at arena
        camera_set(CAM_X, 25.0, CAM_Y + 15.0, CAM_X, 0.0, CAM_Y);
        camera_fov(45.0);

        // Arena floor
        push_identity(); push_translate(0.0, -0.1, 0.0);
        texture_bind(TEX_ARENA);
        material_shininess(0.1); material_specular(0x404040FF);
        set_color(0x808080FF);
        draw_mesh(MESH_ARENA);

        // Environmental hazard
        if HAZARD_ACTIVE {
            let hazard_color = match STAGE {
                Stage::CrystalCavern => 0x8080FFFF,   // Blue crystal
                Stage::EnchantedForest => 0x40FF40FF, // Green poison
                Stage::VolcanicDepths => 0xFF4000FF,  // Orange lava
                Stage::VoidRealm => 0x8000FFFF,       // Purple void
            };
            let pulse = 0.8 + (sin(GAME_TIME * 4.0) + 1.0) * 0.2;
            let alpha = ((HAZARD_TIMER / 5.0).min(1.0) * pulse * 200.0) as u32;

            // Draw hazard circle on ground
            push_identity();
            push_translate(HAZARD_X, 0.05, HAZARD_Y);
            push_scale_uniform(HAZARD_RADIUS * 2.0);
            set_color((hazard_color & 0xFFFFFF00) | alpha);
            material_shininess(0.8);
            material_specular(hazard_color);
            draw_mesh(MESH_ARENA);  // Reuse arena mesh as circle indicator
        }

        // XP gems
        for g in &XP_GEMS {
            if !g.active { continue; }
            let bob = sin(GAME_TIME * 4.0 + g.phase) * 0.1;
            push_identity(); push_translate(g.x, 0.3 + bob, g.y);
            push_rotate_y(GAME_TIME * 90.0); push_scale_uniform(0.3);
            texture_bind(TEX_XP);
            material_shininess(0.8); material_specular(0x8080FFFF);
            set_color(0x4080FFFF);
            draw_mesh(MESH_XP);
        }

        // Enemies
        for e in &ENEMIES {
            if !e.active { continue; }
            push_identity(); push_translate(e.x, 0.0, e.y);
            let angle = atan2(-e.y, -e.x) * 57.3;
            push_rotate_y(angle);
            let scale = match e.enemy_type {
                // Basic enemies
                EnemyType::Crawler => 0.6, EnemyType::Skeleton => 0.8, EnemyType::Wisp => 0.5,
                EnemyType::Golem => 1.2, EnemyType::Shade => 0.7, EnemyType::Berserker => 1.0,
                EnemyType::ArcaneSentinel => 0.9,
                // Elites (larger)
                EnemyType::CrystalKnight => 1.3, EnemyType::VoidMage => 1.1,
                EnemyType::GolemTitan => 1.6, EnemyType::SpecterLord => 1.2,
                // Bosses (much larger)
                EnemyType::PrismColossus => 2.5, EnemyType::VoidDragon => 2.2,
            };
            push_scale_uniform(scale);
            let (tex, mesh) = match e.enemy_type {
                // Basic enemies
                EnemyType::Golem => (TEX_GOLEM, MESH_GOLEM),
                EnemyType::Crawler => (TEX_CRAWLER, MESH_CRAWLER),
                EnemyType::Wisp => (TEX_WISP, MESH_WISP),
                EnemyType::Skeleton => (TEX_SKELETON, MESH_SKELETON),
                EnemyType::Shade => (TEX_SHADE, MESH_SHADE),
                EnemyType::Berserker => (TEX_BERSERKER, MESH_BERSERKER),
                EnemyType::ArcaneSentinel => (TEX_ARCANE_SENTINEL, MESH_ARCANE_SENTINEL),
                // Elites
                EnemyType::CrystalKnight => (TEX_CRYSTAL_KNIGHT, MESH_CRYSTAL_KNIGHT),
                EnemyType::VoidMage => (TEX_VOID_MAGE, MESH_VOID_MAGE),
                EnemyType::GolemTitan => (TEX_GOLEM_TITAN, MESH_GOLEM_TITAN),
                EnemyType::SpecterLord => (TEX_SPECTER_LORD, MESH_SPECTER_LORD),
                // Bosses
                EnemyType::PrismColossus => (TEX_PRISM_COLOSSUS, MESH_PRISM_COLOSSUS),
                EnemyType::VoidDragon => (TEX_VOID_DRAGON, MESH_VOID_DRAGON),
            };
            texture_bind(tex);
            // Elites glow purple, bosses glow red
            let (shin, spec) = match e.tier {
                EnemyTier::Normal => (0.3, 0x606060FF),
                EnemyTier::Elite => (0.6, 0x8040FFFF),
                EnemyTier::Boss => (0.8, 0xFF4040FF),
            };
            material_shininess(shin); material_specular(spec);
            set_color(if e.hit_timer > 0.0 { 0xFF8080FF } else { 0xFFFFFFFF });
            draw_mesh(mesh);
        }

        // Players (living and dead)
        for (_i, p) in PLAYERS.iter().enumerate() {
            if !p.active { continue; }

            let (tex, mesh) = match p.class {
                HeroClass::Knight => (TEX_KNIGHT, MESH_KNIGHT), HeroClass::Mage => (TEX_MAGE, MESH_MAGE),
                HeroClass::Ranger => (TEX_RANGER, MESH_RANGER), HeroClass::Cleric => (TEX_CLERIC, MESH_CLERIC),
                HeroClass::Necromancer => (TEX_NECROMANCER, MESH_NECROMANCER), HeroClass::Paladin => (TEX_PALADIN, MESH_PALADIN),
            };

            if p.dead {
                // Draw downed player (lying down, semi-transparent)
                push_identity(); push_translate(p.x, 0.1, p.y);
                push_rotate_y(p.facing_angle - 90.0);
                // Tilt forward to show they're downed
                push_scale_uniform(0.8);
                texture_bind(tex);
                material_shininess(0.2); material_specular(0x404040FF);
                set_color(0x808080A0);  // Gray and transparent
                draw_mesh(mesh);

                // Revive progress indicator (ring around player)
                if p.revive_progress > 0.0 {
                    let progress = p.revive_progress / REVIVE_TIME;
                    // Draw progress using a spinning effect via repeated positioning
                    let angle = GAME_TIME * 180.0;
                    push_identity(); push_translate(p.x, 0.5, p.y);
                    push_rotate_y(angle);
                    push_scale_uniform(0.15);
                    material_shininess(1.0); material_specular(0x40FF40FF);
                    let color = 0x40FF4000 | ((progress * 255.0) as u32);
                    set_color(color);
                    draw_mesh(MESH_PROJ);  // Simple sphere as indicator
                }
            } else {
                // Draw living player
                push_identity(); push_translate(p.x, 0.0, p.y);
                push_rotate_y(p.facing_angle - 90.0); push_scale_uniform(0.8);
                texture_bind(tex);
                material_shininess(0.5); material_specular(0x808080FF);
                let flash = p.invuln_timer > 0.0 && (GAME_TIME * 10.0) as i32 % 2 == 0;
                set_color(if flash { 0xFFFFFF80 } else { 0xFFFFFFFF });
                draw_mesh(mesh);
            }
        }

        // Projectiles
        for pr in &PROJECTILES {
            if !pr.active { continue; }
            push_identity(); push_translate(pr.x, 0.4, pr.y); push_scale_uniform(1.2);
            material_shininess(1.0); material_specular(0xFFFF80FF);
            set_color(0xFFFF40FF);
            draw_mesh(MESH_PROJ);
        }

        // Visual Effects (VFX)
        render_vfx();
    }
}

// =============================================================================
// VFX Rendering
// =============================================================================

pub fn render_vfx() {
    unsafe {
        for v in &VFX_POOL {
            if !v.active { continue; }

            let progress = v.progress();
            let alpha = ((1.0 - progress) * 255.0) as u32;
            let color_with_alpha = (v.color & 0xFFFFFF00) | alpha;

            match v.vtype {
                VfxType::CleaveSlash => {
                    // Arc/slash effect - draw as expanding ring on ground
                    let scale = v.scale * (1.0 + progress * 0.5);  // Expand outward
                    push_identity();
                    push_translate(v.x, 0.3, v.y);
                    push_rotate_y(v.angle);
                    push_scale_uniform(scale);
                    material_shininess(1.0);
                    material_specular(v.color);
                    set_color(color_with_alpha);
                    draw_mesh(MESH_ARENA);  // Flat disc as cleave arc
                }
                VfxType::HitSpark => {
                    // Small spark particle
                    let size = 0.3 * v.scale * (1.0 - progress);
                    push_identity();
                    push_translate(v.x, 0.5, v.y);
                    draw_billboard(size, size, 1, color_with_alpha);
                }
                VfxType::DamageNumber => {
                    // Floating damage text - rises up and fades
                    let y_offset = progress * 1.5;  // Float upward

                    // Size based on damage (crits are larger via scale)
                    let size = 0.4 * v.scale * (1.0 - progress * 0.3);

                    // Draw as billboard in world space (damage value affects size)
                    let dmg_scale = (v.value / 20.0).min(2.0).max(0.5);
                    push_identity();
                    push_translate(v.x, 0.5 + y_offset, v.y);
                    draw_billboard(size * dmg_scale, size * dmg_scale, 1, color_with_alpha);
                }
                VfxType::HolyBurst => {
                    // Expanding ring
                    let scale = v.scale * progress;
                    push_identity();
                    push_translate(v.x, 0.2, v.y);
                    push_scale_uniform(scale);
                    material_shininess(1.0);
                    material_specular(0xFFFFAAFF);
                    set_color(color_with_alpha);
                    draw_mesh(MESH_ARENA);
                }
                VfxType::SoulDrainBeam => {
                    // Beam effect (would need line drawing)
                    push_identity();
                    push_translate(v.x, 0.5, v.y);
                    draw_billboard(0.3, 0.3, 1, color_with_alpha);
                }
                VfxType::PlayerHurt => {
                    // Handled separately as screen flash
                }
                VfxType::LevelUp => {
                    // Golden rising particles
                    let y_offset = progress * 2.0;
                    let scale = v.scale * (1.0 - progress * 0.5);
                    push_identity();
                    push_translate(v.x - 0.5, 0.5 + y_offset, v.y);
                    draw_billboard(scale * 0.2, scale * 0.4, 1, color_with_alpha);
                    push_identity();
                    push_translate(v.x + 0.5, 0.5 + y_offset * 0.8, v.y);
                    draw_billboard(scale * 0.15, scale * 0.3, 1, color_with_alpha);
                    push_identity();
                    push_translate(v.x, 0.5 + y_offset * 1.2, v.y - 0.3);
                    draw_billboard(scale * 0.25, scale * 0.5, 1, color_with_alpha);
                }
                VfxType::EnemyDeath => {
                    // Burst of particles
                    let scale = v.scale * (1.0 + progress);
                    for i in 0..4 {
                        let angle = (i as f32 * 90.0 + progress * 180.0) * 0.0175;
                        let dist = progress * 0.8;
                        let px = v.x + cos(angle) * dist;
                        let pz = v.y + sin(angle) * dist;
                        push_identity();
                        push_translate(px, 0.3 + progress * 0.5, pz);
                        draw_billboard(scale * 0.15, scale * 0.15, 1, color_with_alpha);
                    }
                }
            }
        }

        // Player hurt screen flash (drawn as 2D overlay)
        if PLAYER_HURT_FLASH > 0.0 {
            layer(20);  // Above everything
            let flash_alpha = ((PLAYER_HURT_FLASH / 0.3).min(1.0) * 80.0) as u32;
            draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0xFF000000 | flash_alpha);
        }
    }
}

fn fmt_damage(n: u32, buf: &mut [u8]) -> usize {
    if n == 0 { buf[0] = b'0'; return 1; }
    let mut v = n;
    let mut i = 0;
    while v > 0 && i < buf.len() {
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        i += 1;
    }
    // Reverse
    for j in 0..i/2 {
        let t = buf[j];
        buf[j] = buf[i - 1 - j];
        buf[i - 1 - j] = t;
    }
    i
}

// =============================================================================
// HUD Rendering
// =============================================================================

pub fn render_hud() {
    unsafe {
        layer(10);

        // Stage & Wave
        draw_rect(10.0, 10.0, 200.0, 55.0, 0x00000088);
        let stage_name: &[u8] = match STAGE {
            Stage::CrystalCavern => b"Crystal Cavern",
            Stage::EnchantedForest => b"Enchanted Forest",
            Stage::VolcanicDepths => b"Volcanic Depths",
            Stage::VoidRealm => b"Void Realm",
        };
        draw_text(stage_name.as_ptr(), stage_name.len() as u32, 20.0, 22.0, 16.0, 0xFFFFFFFF);

        let mut buf = [0u8; 16];
        buf[0..6].copy_from_slice(b"Wave: ");
        let len = fmt_num(WAVE, &mut buf[6..]);
        draw_text(buf.as_ptr(), 6 + len as u32, 20.0, 44.0, 14.0, 0xCCCCCCFF);

        // Time
        let ts = GAME_TIME as u32;
        buf[0..5].copy_from_slice(b"Time ");
        buf[5] = b'0' + ((ts / 60) / 10) as u8; buf[6] = b'0' + ((ts / 60) % 10) as u8;
        buf[7] = b':'; buf[8] = b'0' + ((ts % 60) / 10) as u8; buf[9] = b'0' + ((ts % 60) % 10) as u8;
        draw_text(buf.as_ptr(), 10, 130.0, 44.0, 14.0, 0xAAAAAAFF);

        // Player health/level
        let mut y = 75.0;
        for (_i, p) in PLAYERS.iter().enumerate() {
            if !p.active { continue; }
            let color = match p.class { HeroClass::Knight => 0x4080FFFF, HeroClass::Mage => 0x8040FFFF, HeroClass::Ranger => 0x40FF40FF, HeroClass::Cleric => 0xFFFF40FF, HeroClass::Necromancer => 0x6040A0FF, HeroClass::Paladin => 0xDDCC66FF };
            draw_rect(10.0, y, 150.0, 35.0, 0x00000088);
            let name: &[u8] = match p.class { HeroClass::Knight => b"Knight", HeroClass::Mage => b"Mage", HeroClass::Ranger => b"Ranger", HeroClass::Cleric => b"Cleric", HeroClass::Necromancer => b"Necromancer", HeroClass::Paladin => b"Paladin" };

            if p.dead {
                // Show "DOWNED" status
                draw_text(name.as_ptr(), name.len() as u32, 15.0, y + 5.0, 12.0, 0x808080FF);
                draw_str(b"DOWNED", 90.0, y + 5.0, 12.0, 0xFF4040FF);

                // Revive progress bar (green)
                if p.revive_progress > 0.0 {
                    let progress = p.revive_progress / REVIVE_TIME;
                    draw_rect(15.0, y + 20.0, 130.0, 8.0, 0x004000FF);
                    draw_rect(15.0, y + 20.0, 130.0 * progress, 8.0, 0x40FF40FF);
                } else {
                    draw_rect(15.0, y + 20.0, 130.0, 8.0, 0x400000FF);
                    draw_str(b"Get close!", 30.0, y + 19.0, 8.0, 0xCCCCCCFF);
                }
            } else {
                draw_text(name.as_ptr(), name.len() as u32, 15.0, y + 5.0, 12.0, color);
                buf[0..3].copy_from_slice(b"Lv "); let len = fmt_num(p.level, &mut buf[3..]); draw_text(buf.as_ptr(), 3 + len as u32, 90.0, y + 5.0, 12.0, 0xFFFFFFFF);
                let hp = (p.health / p.max_health).max(0.0);
                draw_rect(15.0, y + 20.0, 130.0, 8.0, 0x400000FF);
                draw_rect(15.0, y + 20.0, 130.0 * hp, 8.0, 0xFF4040FF);
            }
            y += 40.0;
        }

        // Kill counter
        draw_rect(SCREEN_WIDTH - 120.0, 10.0, 110.0, 30.0, 0x00000088);
        buf[0..7].copy_from_slice(b"Kills: ");
        let len = fmt_num(KILLS, &mut buf[7..]);
        draw_text(buf.as_ptr(), 7 + len as u32, SCREEN_WIDTH - 110.0, 22.0, 16.0, 0xFF8080FF);

        // Combo display (when active)
        if COMBO_COUNT >= 5 {
            let combo_y = 50.0;
            draw_rect(SCREEN_WIDTH - 140.0, combo_y, 130.0, 45.0, 0x00000088);

            // Combo count with pulsing effect
            let pulse = 1.0 + (sin(GAME_TIME * 8.0) + 1.0) * 0.15;
            let size = 20.0 * pulse;
            buf[0..6].copy_from_slice(b"COMBO ");
            let len = fmt_num(COMBO_COUNT, &mut buf[6..]);
            let color = if COMBO_COUNT >= 50 { 0xFF00FFFF }  // Legendary (purple)
                else if COMBO_COUNT >= 25 { 0xFFFF00FF }     // Epic (gold)
                else if COMBO_COUNT >= 10 { 0xFF8000FF }     // Rare (orange)
                else { 0x00FF00FF };                          // Common (green)
            draw_text(buf.as_ptr(), 6 + len as u32, SCREEN_WIDTH - 130.0, combo_y + 8.0, size, color);

            // Multiplier
            buf[0] = b'x';
            let mult_int = (COMBO_MULT * 10.0) as u32;
            buf[1] = b'0' + (mult_int / 10) as u8;
            buf[2] = b'.';
            buf[3] = b'0' + (mult_int % 10) as u8;
            draw_text(buf.as_ptr(), 4, SCREEN_WIDTH - 90.0, combo_y + 30.0, 14.0, 0xFFFF88FF);

            // Timer bar
            let timer_pct = (COMBO_TIMER / (COMBO_WINDOW + 3.0)).max(0.0);
            draw_rect(SCREEN_WIDTH - 130.0, combo_y + 40.0, 110.0, 4.0, 0x404040FF);
            draw_rect(SCREEN_WIDTH - 130.0, combo_y + 40.0, 110.0 * timer_pct, 4.0, color);
        }

        // Difficulty indicator (top center)
        draw_rect(SCREEN_WIDTH / 2.0 - 50.0, 10.0, 100.0, 20.0, 0x00000088);
        let diff_name = DIFFICULTY.name();
        let diff_color = DIFFICULTY.color();
        draw_text(diff_name.as_ptr(), diff_name.len() as u32, SCREEN_WIDTH / 2.0 - 40.0, 14.0, 14.0, diff_color);

        // Hazard warning (when active)
        if HAZARD_ACTIVE {
            let warn_pulse = (sin(GAME_TIME * 6.0) + 1.0) * 0.5;
            let warn_alpha = ((0.5 + warn_pulse * 0.5) * 255.0) as u32;
            let hazard_name: &[u8] = match STAGE {
                Stage::CrystalCavern => b"CRYSTAL SHARDS!",
                Stage::EnchantedForest => b"POISON CLOUD!",
                Stage::VolcanicDepths => b"LAVA POOL!",
                Stage::VoidRealm => b"VOID RIFT!",
            };
            draw_rect(SCREEN_WIDTH / 2.0 - 80.0, 35.0, 160.0, 22.0, 0xFF000044 | (warn_alpha / 2));
            draw_text(hazard_name.as_ptr(), hazard_name.len() as u32, SCREEN_WIDTH / 2.0 - 65.0, 40.0, 14.0, 0xFF404000 | warn_alpha);
        }

        // Boss health bar (bottom center of screen)
        if let Some(bi) = ACTIVE_BOSS {
            let boss = &ENEMIES[bi];
            if boss.active {
                let bar_w = 400.0;
                let bar_h = 20.0;
                let bar_x = (SCREEN_WIDTH - bar_w) / 2.0;
                let bar_y = SCREEN_HEIGHT - 50.0;

                // Background
                draw_rect(bar_x - 5.0, bar_y - 5.0, bar_w + 10.0, bar_h + 30.0, 0x00000088);

                // Boss name
                let boss_name: &[u8] = match boss.enemy_type {
                    EnemyType::PrismColossus => b"PRISM COLOSSUS",
                    EnemyType::VoidDragon => b"VOID DRAGON",
                    _ => b"BOSS",
                };
                draw_text(boss_name.as_ptr(), boss_name.len() as u32, bar_x + (bar_w - (boss_name.len() as f32 * 12.0)) / 2.0, bar_y - 2.0, 16.0, 0xFF4040FF);

                // Health bar
                let hp = (boss.health / boss.max_health).max(0.0);
                draw_rect(bar_x, bar_y + 15.0, bar_w, bar_h, 0x400000FF);
                draw_rect(bar_x, bar_y + 15.0, bar_w * hp, bar_h, 0xFF2020FF);

                // Health percentage
                let pct = (hp * 100.0) as u32;
                buf[0..3].copy_from_slice(b"HP ");
                let len = fmt_num(pct, &mut buf[3..]);
                buf[3 + len] = b'%';
                draw_text(buf.as_ptr(), 4 + len as u32, bar_x + bar_w / 2.0 - 25.0, bar_y + 17.0, 14.0, 0xFFFFFFFF);
            }
        }
    }
}

// =============================================================================
// UI Overlays
// =============================================================================

pub fn render_levelup() {
    unsafe {
        draw_rect(250.0, 150.0, 460.0, 280.0, 0x000000DD);
        draw_str(b"LEVEL UP!", 380.0, 170.0, 32.0, 0xFFFF40FF);
        draw_str(b"Choose a power-up:", 340.0, 210.0, 18.0, 0xCCCCCCFF);

        for i in 0..3 {
            let y = 250.0 + i as f32 * 60.0;
            let selected = i == LEVELUP_SELECTION;
            let bg = if selected { 0x404080FF } else { 0x202040FF };
            draw_rect(270.0, y, 420.0, 50.0, bg);

            let name: &[u8] = powerup_name(LEVELUP_CHOICES[i]);
            let color = if selected { 0xFFFFFFFF } else { 0xAAAAAAFF };
            draw_text(name.as_ptr(), name.len() as u32, 280.0, y + 18.0, 16.0, color);
        }

        draw_str(b"[Up/Down] Select  [A] Confirm", 310.0, 440.0, 14.0, 0x888888FF);
    }
}

pub fn render_pause() {
    unsafe {
        draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x00000088);
        draw_str(b"PAUSED", 400.0, 250.0, 48.0, 0xFFFFFFFF);
        draw_str(b"Press START to continue", 340.0, 320.0, 20.0, 0xAAAAAAFF);
    }
}

pub fn render_gameover() {
    unsafe {
        draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x000000CC);
        draw_str(b"GAME OVER", 350.0, 180.0, 48.0, 0xFF4040FF);

        let mut buf = [0u8; 32];
        buf[0..7].copy_from_slice(b"Wave: "); let len = fmt_num(WAVE, &mut buf[7..]); draw_text(buf.as_ptr(), 7 + len as u32, 400.0, 260.0, 22.0, 0xFFFFFFFF);
        buf[0..7].copy_from_slice(b"Kills: "); let len = fmt_num(KILLS, &mut buf[7..]); draw_text(buf.as_ptr(), 7 + len as u32, 400.0, 295.0, 22.0, 0xFFFFFFFF);

        let ts = GAME_TIME as u32;
        buf[0..6].copy_from_slice(b"Time: ");
        buf[6] = b'0' + ((ts / 60) / 10) as u8; buf[7] = b'0' + ((ts / 60) % 10) as u8; buf[8] = b':';
        buf[9] = b'0' + ((ts % 60) / 10) as u8; buf[10] = b'0' + ((ts % 60) % 10) as u8;
        draw_text(buf.as_ptr(), 11, 400.0, 330.0, 22.0, 0xFFFFFFFF);

        draw_str(b"Press A to restart", 370.0, 420.0, 20.0, 0xAAAAAAFF);
    }
}

pub fn render_victory() {
    unsafe {
        draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x000000CC);
        draw_str(b"VICTORY!", 380.0, 180.0, 48.0, 0x40FF40FF);
        draw_str(b"You survived the Void Realm!", 300.0, 250.0, 22.0, 0xFFFFFFFF);

        let mut buf = [0u8; 32];
        buf[0..7].copy_from_slice(b"Kills: "); let len = fmt_num(KILLS, &mut buf[7..]); draw_text(buf.as_ptr(), 7 + len as u32, 400.0, 300.0, 22.0, 0xFFFFFFFF);

        let ts = GAME_TIME as u32;
        buf[0..6].copy_from_slice(b"Time: ");
        buf[6] = b'0' + ((ts / 60) / 10) as u8; buf[7] = b'0' + ((ts / 60) % 10) as u8; buf[8] = b':';
        buf[9] = b'0' + ((ts % 60) / 10) as u8; buf[10] = b'0' + ((ts % 60) % 10) as u8;
        draw_text(buf.as_ptr(), 11, 400.0, 335.0, 22.0, 0xFFFFFFFF);

        draw_str(b"Press A to play again", 360.0, 420.0, 20.0, 0xAAAAAAFF);
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

pub fn powerup_name(ptype: PowerUpType) -> &'static [u8] {
    match ptype {
        PowerUpType::Cleave => b"Cleave - Melee sweep attack",
        PowerUpType::MagicMissile => b"Magic Missile - Homing shots",
        PowerUpType::PiercingArrow => b"Piercing Arrow - Pass through",
        PowerUpType::HolyNova => b"Holy Nova - Radial burst",
        PowerUpType::SoulDrain => b"Soul Drain - Life steal beam",
        PowerUpType::DivineCrush => b"Divine Crush - Holy smash",
        PowerUpType::Fireball => b"Fireball - AoE explosion",
        PowerUpType::IceShards => b"Ice Shards - Multi-shot spread",
        PowerUpType::LightningBolt => b"Lightning - Chain damage",
        PowerUpType::ShadowOrb => b"Shadow Orb - Orbiting damage",
        PowerUpType::Might => b"Might - +20% Damage",
        PowerUpType::Swiftness => b"Swiftness - +15% Speed",
        PowerUpType::Vitality => b"Vitality - +25 Max HP",
        PowerUpType::Haste => b"Haste - +20% Attack Speed",
        PowerUpType::Magnetism => b"Magnetism - +50% Pickup Range",
        PowerUpType::Armor => b"Armor - -15% Damage Taken",
        PowerUpType::Luck => b"Luck - +25% XP Gain",
        PowerUpType::Fury => b"Fury - Damage when low HP",
    }
}

// =============================================================================
// Class Selection Screen
// =============================================================================

pub fn render_class_select() {
    unsafe {
        use crate::gameplay::class_starter_weapon;

        let active_count = player_count().min(MAX_PLAYERS as u32) as usize;

        // Class data
        let class_names: [&[u8]; 6] = [b"KNIGHT", b"MAGE", b"RANGER", b"CLERIC", b"NECROMANCER", b"PALADIN"];
        let class_colors: [u32; 6] = [0x4488FFFF, 0xAA44FFFF, 0x44FF88FF, 0xFFFF44FF, 0x8844AAFF, 0xFFDD44FF];
        let class_stats: [&[u8]; 6] = [
            b"HP:150 SPD:3.5 ARM:10%",
            b"HP:80  SPD:4.0 DMG:+30%",
            b"HP:100 SPD:5.0 ATK:+30%",
            b"HP:120 SPD:3.8 PICK:+66%",
            b"HP:90  SPD:3.6 DMG:+20%",
            b"HP:180 SPD:3.0 ARM:20%",
        ];
        let class_desc: [&[u8]; 6] = [
            b"Tank - High armor and health",
            b"DPS - High damage spellcaster",
            b"DPS - Fast attacks and movement",
            b"Support - Healing and pickup",
            b"DPS - Life steal specialist",
            b"Tank - Ultimate survivability",
        ];

        // Panel layout
        let panel_width = 200.0;
        let panel_spacing = 20.0;
        let total_width = active_count as f32 * panel_width + (active_count as f32 - 1.0) * panel_spacing;
        let start_x = (SCREEN_WIDTH - total_width) / 2.0;

        // ===== PHASE 1: Render 3D character models =====
        // Set up camera for character showcase
        camera_fov(35.0);

        // Rotation speeds per class (personality)
        let rotation_speeds: [f32; 6] = [20.0, 40.0, 35.0, 25.0, 45.0, 15.0];

        // Render each player's selected character in 3D
        for i in 0..active_count {
            let cursor = CLASS_SELECT_CURSOR[i] as usize;

            // Center characters based on active player count
            // For 1 player: center at 0.0
            // For 2 players: positions at -1.0 and 1.0
            // For 3 players: positions at -2.0, 0.0, 2.0
            // For 4 players: positions at -3.0, -1.0, 1.0, 3.0
            let total_span = (active_count as f32 - 1.0) * 2.0;
            let char_x = -total_span / 2.0 + (i as f32 * 2.0);
            let char_z = 0.0;

            // Set camera to view all characters (centered)
            camera_set(0.0, 1.8, 6.0, 0.0, 0.6, 0.0);

            // Calculate rotation
            let rotation_angle = (GAME_TIME * rotation_speeds[cursor]) % 360.0;
            let bob = sin(GAME_TIME * 2.0 + i as f32) * 0.05;

            // Transform for this character
            push_identity();
            push_translate(char_x, bob, char_z);
            push_rotate_y(rotation_angle);
            push_scale_uniform(0.8);

            // Get mesh and texture for this class
            let (tex, mesh) = match cursor {
                0 => (TEX_KNIGHT, MESH_KNIGHT),
                1 => (TEX_MAGE, MESH_MAGE),
                2 => (TEX_RANGER, MESH_RANGER),
                3 => (TEX_CLERIC, MESH_CLERIC),
                4 => (TEX_NECROMANCER, MESH_NECROMANCER),
                _ => (TEX_PALADIN, MESH_PALADIN),
            };

            // Render character
            texture_bind(tex);
            material_shininess(0.6);
            material_specular(class_colors[cursor]);
            set_color(0xFFFFFFFF);
            draw_mesh(mesh);
        }

        // ===== PHASE 2: Switch to 2D layer for UI =====
        layer(10);

        // Background overlay (semi-transparent to see 3D behind)
        draw_rect(0.0, 0.0, SCREEN_WIDTH, 100.0, 0x000000CC);
        draw_rect(0.0, 440.0, SCREEN_WIDTH, 140.0, 0x000000CC);

        // Title
        draw_str(b"SELECT YOUR CLASS", 290.0, 30.0, 36.0, 0xFFD700FF);
        draw_str(b"LEFT/RIGHT: Choose  A: Confirm  B: Back", 260.0, 70.0, 16.0, 0xAAAAAAFF);

        // Draw each player's info panel (below 3D models)
        for i in 0..active_count {
            let px = start_x + i as f32 * (panel_width + panel_spacing);
            let py = 450.0; // Moved lower to not overlap 3D models
            let cursor = CLASS_SELECT_CURSOR[i] as usize;
            let confirmed = CLASS_CONFIRMED[i];

            // Panel background
            let panel_color = if confirmed { 0x44FF4466 } else { 0x33333388 };
            draw_rect(px, py, panel_width, 120.0, panel_color);

            // Player number
            let player_label = match i {
                0 => b"P1",
                1 => b"P2",
                2 => b"P3",
                _ => b"P4",
            };
            draw_str(player_label, px + 5.0, py + 5.0, 16.0, 0xFFFFFFFF);

            // Selected class name
            draw_str(class_names[cursor], px + 40.0, py + 5.0, 18.0, class_colors[cursor]);

            // Class stats
            draw_str(class_stats[cursor], px + 10.0, py + 30.0, 11.0, 0xCCCCCCFF);

            // Class description
            draw_str(class_desc[cursor], px + 10.0, py + 48.0, 10.0, 0x888888FF);

            // Starting weapon
            let starter = class_starter_weapon(CLASS_SELECTIONS[i]);
            let weapon_name: &[u8] = match starter {
                PowerUpType::Cleave => b"Cleave",
                PowerUpType::MagicMissile => b"Magic Missile",
                PowerUpType::PiercingArrow => b"Piercing Arrow",
                PowerUpType::HolyNova => b"Holy Nova",
                PowerUpType::SoulDrain => b"Soul Drain",
                PowerUpType::DivineCrush => b"Divine Crush",
                _ => b"???",
            };
            draw_str(b"Weapon:", px + 10.0, py + 68.0, 12.0, 0xFFAA44FF);
            draw_str(weapon_name, px + 70.0, py + 68.0, 12.0, 0xFFFFFFFF);

            // Ready indicator
            if confirmed {
                draw_str(b"READY!", px + 70.0, py + 90.0, 18.0, 0x44FF44FF);
            } else {
                draw_str(b"Press A", px + 70.0, py + 95.0, 12.0, 0xAAAAAAAA);
            }

            // Selection indicator dots
            let dot_y = py + 105.0;
            for c in 0..6u8 {
                let dot_x = px + 25.0 + c as f32 * 25.0;
                let dot_color = if c == cursor as u8 { class_colors[cursor] } else { 0x555555FF };
                draw_rect(dot_x, dot_y, 15.0, 6.0, dot_color);
            }
        }

        // Countdown display when all ready
        if ALL_PLAYERS_READY {
            // Manual ceil: add 0.999 and truncate (no_std compatible)
            let countdown = ((CLASS_SELECT_COUNTDOWN + 0.999) as u32).max(1);
            let mut buf = [0u8; 16];
            buf[0..10].copy_from_slice(b"Starting: ");
            buf[10] = b'0' + (countdown % 10) as u8;
            draw_text(buf.as_ptr(), 11, 420.0, 410.0, 28.0, 0x44FF44FF);
        }
    }
}
