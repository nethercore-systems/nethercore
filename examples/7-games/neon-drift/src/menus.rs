//! Menu screens for NEON DRIFT
//!
//! Main menu, car selection, track selection, and pause screens.

use crate::ffi::*;
use crate::types::*;
use crate::state::*;
use crate::racing::{init_race, start_attract_mode};

// === Menu Update Functions ===

pub fn update_main_menu(dt: f32) {
    unsafe {
        let any_input = buttons_held(0) != 0;

        if any_input {
            IDLE_TIMER = 0.0;
        } else {
            IDLE_TIMER += dt;
        }

        if IDLE_TIMER >= ATTRACT_MODE_DELAY {
            start_attract_mode();
            return;
        }

        if button_pressed(0, 0) != 0 {
            if MENU_SELECTION > 0 { MENU_SELECTION -= 1; }
        }
        if button_pressed(0, 1) != 0 {
            if MENU_SELECTION < 2 { MENU_SELECTION += 1; }
        }

        if button_pressed(0, BUTTON_A) != 0 {
            match MENU_SELECTION {
                0 => {
                    GAME_MODE = GameMode::CarSelect;
                    for i in 0..4 { PLAYER_CONFIRMED[i] = false; }
                }
                1 => {
                    GAME_MODE = GameMode::CountdownReady;
                    COUNTDOWN_TIMER = 240;
                    init_race();
                }
                _ => {}
            }
        }
    }
}

pub fn update_car_select() {
    unsafe {
        let pcount = player_count();
        ACTIVE_PLAYER_COUNT = pcount;
        MENU_TIME += 1.0 / 60.0;

        for p in 0..pcount as usize {
            if PLAYER_CONFIRMED[p] { continue; }

            if button_pressed(p as u32, 2) != 0 {
                if CAR_SELECTIONS[p] > 0 { CAR_SELECTIONS[p] -= 1; }
            }
            if button_pressed(p as u32, 3) != 0 {
                if CAR_SELECTIONS[p] < 6 { CAR_SELECTIONS[p] += 1; }
            }

            if button_pressed(p as u32, BUTTON_A) != 0 {
                PLAYER_CONFIRMED[p] = true;
                CARS[p].car_type = match CAR_SELECTIONS[p] {
                    0 => CarType::Speedster,
                    1 => CarType::Muscle,
                    2 => CarType::Racer,
                    3 => CarType::Drift,
                    4 => CarType::Phantom,
                    5 => CarType::Titan,
                    _ => CarType::Viper,
                };
                CARS[p].init_stats();
            }
        }

        let mut all_confirmed = true;
        for p in 0..pcount as usize {
            if !PLAYER_CONFIRMED[p] { all_confirmed = false; break; }
        }

        if all_confirmed {
            GAME_MODE = GameMode::TrackSelect;
            MENU_SELECTION = 0;
        }

        if button_pressed(0, BUTTON_B) != 0 {
            GAME_MODE = GameMode::MainMenu;
        }
    }
}

pub fn update_track_select() {
    unsafe {
        MENU_TIME += 1.0 / 60.0;

        if button_pressed(0, 0) != 0 {
            if MENU_SELECTION > 0 { MENU_SELECTION -= 1; }
        }
        if button_pressed(0, 1) != 0 {
            if MENU_SELECTION < 4 { MENU_SELECTION += 1; }
        }

        if button_pressed(0, BUTTON_A) != 0 {
            SELECTED_TRACK = match MENU_SELECTION {
                0 => TrackId::SunsetStrip,
                1 => TrackId::NeonCity,
                2 => TrackId::VoidTunnel,
                3 => TrackId::CrystalCavern,
                _ => TrackId::SolarHighway,
            };
            GAME_MODE = GameMode::CountdownReady;
            COUNTDOWN_TIMER = 240;
            init_race();
        }

        if button_pressed(0, BUTTON_B) != 0 {
            GAME_MODE = GameMode::CarSelect;
            for p in 0..4 { PLAYER_CONFIRMED[p] = false; }
        }
    }
}

pub fn update_results() {
    unsafe {
        if button_pressed(0, BUTTON_A) != 0 {
            music_stop();  // Stop race music
            GAME_MODE = GameMode::MainMenu;
            MENU_SELECTION = 0;
        }
    }
}

pub fn update_paused() {
    unsafe {
        if button_pressed(0, BUTTON_START) != 0 {
            GAME_MODE = GameMode::Racing;
        }
        if button_pressed(0, BUTTON_SELECT) != 0 {
            music_stop();  // Stop race music
            GAME_MODE = GameMode::MainMenu;
            MENU_SELECTION = 0;
        }
    }
}

// === Menu Render Functions ===

pub fn render_main_menu() {
    unsafe {
        let t = TITLE_ANIM_TIME;

        // Set up synthwave environment background (EPU supports 2 layers: 0 and 1)
        env_gradient(0, 0x0D0221FF, 0x1A0533FF, 0x7209B7FF, 0x000000FF, 0.0, 0.1);
        env_lines(1, 0, 2, 3, 2.5, 100.0, 0xFF00FFFF, 0x00FFFFFF, 5, GRID_PHASE);
        env_blend(1);
        draw_env();

        // Use built-in font for better readability (0 = default 8x8 font)
        // font_bind(NEON_FONT);  // Custom font disabled - hard to read
        font_bind(0);

        // Glowing title
        let title = b"NEON DRIFT";
        let title_scale = 64.0;
        let title_x = 480.0 - (title.len() as f32 * title_scale * 0.4);
        let title_y = 80.0;

        let glow_pulse = (libm::sinf(t * 2.0) * 0.3 + 0.7) as f32;
        let glow_alpha = (glow_pulse * 100.0) as u32;
        let glow_color = 0x00FFFF00 | glow_alpha;
        draw_text(title.as_ptr(), title.len() as u32,
                  title_x - 2.0, title_y - 2.0, title_scale + 4.0, glow_color);

        let hue_shift = (libm::sinf(t * 0.5) * 0.5 + 0.5) as f32;
        let r = (255.0 * (1.0 - hue_shift * 0.5)) as u32;
        let g = 255u32;
        let b = (255.0 * (0.5 + hue_shift * 0.5)) as u32;
        let title_color = (r << 24) | (g << 16) | (b << 8) | 0xFF;
        draw_text(title.as_ptr(), title.len() as u32,
                  title_x, title_y, title_scale, title_color);

        // Subtitle
        let subtitle = b"ARCADE RACING";
        let sub_x = 480.0 - (subtitle.len() as f32 * 16.0 * 0.4);
        let sub_alpha = (libm::sinf(t * 1.5) * 0.3 + 0.7) * 255.0;
        let sub_color = 0xFF00FF00 | (sub_alpha as u32);
        draw_text(subtitle.as_ptr(), subtitle.len() as u32, sub_x, 160.0, 24.0, sub_color);

        // Menu options
        let options = [b"SINGLE RACE" as &[u8], b"QUICK RACE", b"TIME TRIAL"];
        for (i, opt) in options.iter().enumerate() {
            let y = 240.0 + (i as f32 * 50.0);
            let is_selected = MENU_SELECTION == i as u32;

            if is_selected {
                let box_pulse = (libm::sinf(t * 4.0) * 0.1 + 0.9) as f32;
                let box_width = 300.0 * box_pulse;
                let box_x = 480.0 - box_width / 2.0;
                draw_rect(box_x, y - 5.0, box_width, 35.0, 0x00FFFF30);

                let arrow_offset = (libm::sinf(t * 6.0) * 5.0) as f32;
                draw_text(b">".as_ptr(), 1, 330.0 - arrow_offset, y, 24.0, COLOR_CYAN);
                draw_text(b"<".as_ptr(), 1, 620.0 + arrow_offset, y, 24.0, COLOR_CYAN);
            }

            let color = if is_selected { COLOR_CYAN } else { 0xAAAAAAFF };
            let opt_x = 480.0 - (opt.len() as f32 * 24.0 * 0.4);
            draw_text(opt.as_ptr(), opt.len() as u32, opt_x, y, 24.0, color);
        }

        // Prompt
        let prompt = b"PRESS A TO SELECT";
        let prompt_x = 480.0 - (prompt.len() as f32 * 14.0 * 0.4);
        let blink = if (t * 2.0) as u32 % 2 == 0 { 0xFFFFFFFF } else { 0x888888FF };
        draw_text(prompt.as_ptr(), prompt.len() as u32, prompt_x, 420.0, 14.0, blink);

        // Footer
        let footer = b"(C) 2024 NETHERCORE";
        let footer_x = 480.0 - (footer.len() as f32 * 12.0 * 0.4);
        draw_text(footer.as_ptr(), footer.len() as u32, footer_x, 500.0, 12.0, 0x444444FF);

        // Idle timer indicator
        if IDLE_TIMER > ATTRACT_MODE_DELAY - 5.0 {
            let countdown = (ATTRACT_MODE_DELAY - IDLE_TIMER) as u32;
            let demo_text = b"DEMO IN ";
            let demo_x = 800.0;
            draw_text(demo_text.as_ptr(), demo_text.len() as u32, demo_x, 520.0, 10.0, 0x666666FF);
            let digit = [b'0' + countdown as u8];
            draw_text(digit.as_ptr(), 1, demo_x + 65.0, 520.0, 10.0, 0x666666FF);
        }
    }
}

/// Returns (mesh_handle, color) for the selected car
pub fn get_car_assets(sel: u32) -> (u32, u32) {
    unsafe {
        match sel {
            0 => (MESH_SPEEDSTER, 0xFF3333FF),  // Red
            1 => (MESH_MUSCLE, 0x3366FFFF),     // Blue
            2 => (MESH_RACER, 0x33FF33FF),      // Green
            3 => (MESH_DRIFT, 0xFF9900FF),      // Orange
            4 => (MESH_PHANTOM, 0x9933FFFF),    // Purple
            5 => (MESH_TITAN, 0xFFCC00FF),      // Gold
            _ => (MESH_VIPER, 0x00FFFFFF),      // Cyan
        }
    }
}

fn draw_stat_bar(x: f32, y: f32, width: f32, height: f32, value: f32, color: u32, anim_phase: f32) {
    unsafe {
        // Background
        draw_rect(x, y, width, height, 0x333333FF);

        // Animated fill
        let fill_width = width * value * (0.9 + libm::sinf(anim_phase) * 0.1);
        draw_rect(x, y, fill_width, height, color);

        // Border
        draw_rect(x, y, width, 1.0, 0x666666FF);
        draw_rect(x, y + height - 1.0, width, 1.0, 0x666666FF);
    }
}

fn render_car_preview_3d(mesh: u32, color: u32, x: f32, y: f32, size: f32, rotation: f32, pulse: f32) {
    unsafe {
        // Setup a mini viewport for the car preview
        let preview_size = (size * 1.5) as u32;
        let vp_x = (x - size * 0.75) as u32;
        let vp_y = (y - size * 0.5) as u32;

        viewport(vp_x, vp_y, preview_size, preview_size);

        // Set up lighting for the preview (required for PBR mode)
        light_set(0, -0.3, -0.8, -0.5);
        light_color(0, 0xFFFFFFFF);
        light_intensity(0, 1.5);
        light_enable(0);

        // Camera looking at car from front-side angle
        camera_set(2.0, 1.0, -3.0, 0.0, 0.0, 0.0);
        camera_fov(45.0);

        set_color(color);
        material_metallic(0.9);
        material_roughness(0.1);
        material_emissive(2.0 + pulse * 0.5);

        push_identity();
        push_rotate_y(rotation);
        push_scale(0.8, 0.8, 0.8);
        draw_mesh(mesh);

        set_color(0xFFFFFFFF);
        viewport_clear();
    }
}

pub fn render_car_select() {
    unsafe {
        let t = MENU_TIME;

        // Set up environment background
        env_gradient(0, 0x0D0221FF, 0x1A0533FF, 0x4A1A6EFF, 0x000000FF, 0.0, 0.0);
        env_rectangles(1, 1, 120, 150, 8, 20, 2, 0xFF00FFFF, 0x00FFFFFF, 80, WINDOW_PHASE);
        env_blend(1);
        draw_env();

        // Use built-in font for readability
        font_bind(0);

        // Title
        let title = b"SELECT YOUR RIDE";
        draw_text(title.as_ptr(), title.len() as u32, 320.0, 30.0, 36.0, COLOR_CYAN);

        let pcount = ACTIVE_PLAYER_COUNT;

        // Layout based on player count
        let card_width = if pcount <= 2 { 400.0 } else { 200.0 };
        let card_height = 350.0;
        let start_x = (SCREEN_WIDTH as f32 - card_width * pcount as f32) / 2.0;

        for p in 0..pcount as usize {
            let card_x = start_x + p as f32 * card_width;
            let confirmed = PLAYER_CONFIRMED[p];
            let sel = CAR_SELECTIONS[p];

            // Card background
            let bg_color = if confirmed { 0x00333380 } else { 0x22222280 };
            draw_rect(card_x + 10.0, 80.0, card_width - 20.0, card_height, bg_color);

            // Player label
            let plabel = [b'P', b'1' + p as u8];
            let label_color = if confirmed { 0x00FF00FF } else { COLOR_CYAN };
            draw_text(plabel.as_ptr(), 2, card_x + card_width / 2.0 - 20.0, 90.0, 28.0, label_color);

            // Car name
            let car_names = [
                b"SPEEDSTER" as &[u8], b"MUSCLE", b"RACER", b"DRIFT",
                b"PHANTOM", b"TITAN", b"VIPER"
            ];
            let name = car_names[sel as usize];
            let name_x = card_x + card_width / 2.0 - (name.len() as f32 * 10.0);
            draw_text(name.as_ptr(), name.len() as u32, name_x, 130.0, 20.0, COLOR_WHITE);

            // 3D car preview
            let (mesh, color) = get_car_assets(sel);
            let rotation = t * 30.0 + p as f32 * 90.0;
            let pulse = libm::sinf(t * 2.0 + p as f32) * 0.5 + 0.5;
            render_car_preview_3d(mesh, color, card_x + card_width / 2.0, 220.0, 100.0, rotation, pulse);

            // Stats
            let stats_y = 300.0;
            let bar_width = card_width - 60.0;

            // Get stats for selected car
            let (spd, acc, hnd) = match sel {
                0 => (0.95, 0.90, 1.00),  // Speedster
                1 => (1.10, 0.80, 0.85),  // Muscle
                2 => (0.95, 1.10, 0.95),  // Racer
                3 => (0.90, 1.00, 1.20),  // Drift
                4 => (1.05, 0.95, 0.90),  // Phantom
                5 => (0.85, 0.85, 0.75),  // Titan
                _ => (1.20, 0.75, 1.05),  // Viper
            };

            let spd_label = b"SPD";
            draw_text(spd_label.as_ptr(), 3, card_x + 30.0, stats_y, 12.0, 0x888888FF);
            draw_stat_bar(card_x + 60.0, stats_y, bar_width - 30.0, 12.0, spd / 1.2, 0x00AAFFFF, t * 0.5);

            let acc_label = b"ACC";
            draw_text(acc_label.as_ptr(), 3, card_x + 30.0, stats_y + 20.0, 12.0, 0x888888FF);
            draw_stat_bar(card_x + 60.0, stats_y + 20.0, bar_width - 30.0, 12.0, acc / 1.2, 0xFF6600FF, t * 0.5 + 1.0);

            let hnd_label = b"HND";
            draw_text(hnd_label.as_ptr(), 3, card_x + 30.0, stats_y + 40.0, 12.0, 0x888888FF);
            draw_stat_bar(card_x + 60.0, stats_y + 40.0, bar_width - 30.0, 12.0, hnd / 1.2, 0xFF00FFFF, t * 0.5 + 2.0);

            // Confirmed indicator or arrows
            if confirmed {
                let ready = b"READY!";
                draw_text(ready.as_ptr(), 6, card_x + card_width / 2.0 - 40.0, 380.0, 18.0, 0x00FF00FF);
            } else {
                let arrows = b"< A >";
                draw_text(arrows.as_ptr(), 5, card_x + card_width / 2.0 - 30.0, 380.0, 14.0, 0x888888FF);
            }
        }

        // Back prompt
        let back = b"B: BACK";
        draw_text(back.as_ptr(), 7, 20.0, 500.0, 14.0, 0x666666FF);
    }
}

pub fn render_track_select() {
    unsafe {
        // Set up environment background
        env_gradient(0, 0x0D0221FF, 0x1A0533FF, 0x4A1A6EFF, 0x000000FF, 0.0, 0.0);
        env_lines(1, 0, 2, 2, 3.0, 80.0, 0x00FFFFFF, 0xFF00FFFF, 4, GRID_PHASE);
        env_blend(1);
        draw_env();

        // Use built-in font for readability
        font_bind(0);

        let title = b"SELECT TRACK";
        draw_text(title.as_ptr(), title.len() as u32, 380.0, 40.0, 32.0, COLOR_CYAN);

        let tracks: [(&[u8], &[u8], u32); 5] = [
            (b"SUNSET STRIP", b"*", 0xFF6B35FF),
            (b"NEON CITY", b"**", 0xFF00FFFF),
            (b"VOID TUNNEL", b"***", 0x9933FFFF),
            (b"CRYSTAL CAVERN", b"***", 0x00FFFFFF),
            (b"SOLAR HIGHWAY", b"****", 0xFFAA00FF),
        ];

        for (i, (name, diff, color)) in tracks.iter().enumerate() {
            let y = 120.0 + (i as f32 * 60.0);
            let is_selected = MENU_SELECTION == i as u32;

            if is_selected {
                draw_rect(300.0, y - 5.0, 360.0, 50.0, 0x00FFFF20);
            }

            let text_color = if is_selected { COLOR_WHITE } else { 0xAAAAAAFF };
            draw_text(name.as_ptr(), name.len() as u32, 320.0, y, 24.0, text_color);

            // Difficulty stars
            draw_text(diff.as_ptr(), diff.len() as u32, 580.0, y + 5.0, 16.0, *color);
        }

        let prompt = b"A: SELECT    B: BACK";
        draw_text(prompt.as_ptr(), prompt.len() as u32, 350.0, 450.0, 14.0, 0x666666FF);
    }
}

pub fn render_countdown() {
    unsafe {
        crate::rendering::render_racing_view();

        font_bind(0);  // Use built-in font for readability
        viewport_clear();

        let number = (COUNTDOWN_TIMER / 60) as i32;
        let text = match number {
            3 => b"3" as &[u8],
            2 => b"2",
            1 => b"1",
            _ => b"GO!",
        };

        let size = if number > 0 { 96.0 } else { 72.0 };
        let color = if number > 0 { COLOR_WHITE } else { 0x00FF00FF };

        draw_text(text.as_ptr(), text.len() as u32,
                  SCREEN_WIDTH as f32 / 2.0 - 40.0, SCREEN_HEIGHT as f32 / 2.0 - 50.0,
                  size, color);
    }
}

pub fn render_results() {
    unsafe {
        // Set up environment background
        env_gradient(0, 0x0D0221FF, 0x1A0533FF, 0x4A1A6EFF, 0x000000FF, 0.0, 0.0);
        env_scatter(1, 0, 100, 8, 200, 0, 0xFFD700FF, 0xFFFFFFFF, 80, 60, RING_PHASE);
        env_blend(1);
        draw_env();

        draw_rect(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32, 0x111122AA);

        font_bind(0);  // Use built-in font for readability

        let title = b"RACE COMPLETE";
        draw_text(title.as_ptr(), title.len() as u32, 320.0, 60.0, 36.0, COLOR_CYAN);

        let pos_text = [b"1ST" as &[u8], b"2ND", b"3RD", b"4TH"];
        let pos_colors = [0xFFD700FF, 0xC0C0C0FF, 0xCD7F32FF, COLOR_WHITE];

        for i in 0..4 {
            let car_idx = (0..4).find(|&c| CARS[c].race_position == (i + 1) as u32).unwrap_or(i);
            let y = 140.0 + (i as f32 * 60.0);

            draw_text(pos_text[i].as_ptr(), pos_text[i].len() as u32, 300.0, y, 28.0, pos_colors[i]);

            let car_name = match CARS[car_idx].car_type {
                CarType::Speedster => b"SPEEDSTER" as &[u8],
                CarType::Muscle => b"MUSCLE",
                CarType::Racer => b"RACER",
                CarType::Drift => b"DRIFT",
                CarType::Phantom => b"PHANTOM",
                CarType::Titan => b"TITAN",
                CarType::Viper => b"VIPER",
            };
            draw_text(car_name.as_ptr(), car_name.len() as u32, 400.0, y, 24.0, COLOR_WHITE);

            if car_idx < ACTIVE_PLAYER_COUNT as usize {
                let plabel = [b'P', b'1' + car_idx as u8];
                draw_text(plabel.as_ptr(), 2, 550.0, y, 20.0, COLOR_CYAN);
            } else {
                let cpu = b"CPU";
                draw_text(cpu.as_ptr(), 3, 550.0, y, 20.0, 0x888888FF);
            }
        }

        let time_label = b"TIME:";
        draw_text(time_label.as_ptr(), 5, 360.0, 400.0, 20.0, COLOR_WHITE);

        let mins = (RACE_TIME / 60.0) as u32;
        let secs = (RACE_TIME % 60.0) as u32;
        let mut time_str = [b'0', b'0', b':', b'0', b'0'];
        time_str[0] = b'0' + ((mins / 10) % 10) as u8;
        time_str[1] = b'0' + (mins % 10) as u8;
        time_str[3] = b'0' + ((secs / 10) % 10) as u8;
        time_str[4] = b'0' + (secs % 10) as u8;
        draw_text(time_str.as_ptr(), 5, 430.0, 400.0, 20.0, COLOR_CYAN);

        let prompt = b"Press A to Continue";
        draw_text(prompt.as_ptr(), prompt.len() as u32, 350.0, 480.0, 18.0, 0x888888FF);
    }
}

pub fn render_paused() {
    unsafe {
        crate::rendering::render_racing_view();

        viewport_clear();
        font_bind(0);  // Use built-in font for readability
        draw_rect(300.0, 180.0, 360.0, 180.0, 0x111122EE);

        let title = b"PAUSED";
        draw_text(title.as_ptr(), title.len() as u32, 420.0, 200.0, 32.0, COLOR_CYAN);

        let resume = b"START: Resume";
        draw_text(resume.as_ptr(), resume.len() as u32, 360.0, 280.0, 18.0, COLOR_WHITE);

        let quit = b"SELECT: Quit";
        draw_text(quit.as_ptr(), quit.len() as u32, 360.0, 320.0, 18.0, COLOR_WHITE);
    }
}
