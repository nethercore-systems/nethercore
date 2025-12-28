//! 3D rendering for NEON DRIFT
//!
//! Track, cars, environment, and visual effects.

use crate::ffi::*;
use crate::types::*;
use crate::state::*;
use crate::particles::{render_particles, render_speed_lines, render_vignette};
use crate::hud::render_hud;

pub fn load_rom_mesh(id: &[u8]) -> u32 {
    unsafe { rom_mesh(id.as_ptr() as u32, id.len() as u32) }
}

pub fn load_rom_texture(id: &[u8]) -> u32 {
    unsafe { rom_texture(id.as_ptr() as u32, id.len() as u32) }
}

pub fn load_rom_sound(id: &[u8]) -> u32 {
    unsafe { rom_sound(id.as_ptr() as u32, id.len() as u32) }
}

pub fn get_viewport_layout(player_count: u32) -> [(u32, u32, u32, u32); 4] {
    match player_count {
        1 => [
            (0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
            (0, 0, 0, 0),
            (0, 0, 0, 0),
            (0, 0, 0, 0),
        ],
        2 => [
            (0, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT),
            (SCREEN_WIDTH / 2, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT),
            (0, 0, 0, 0),
            (0, 0, 0, 0),
        ],
        3 => [
            (0, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (SCREEN_WIDTH / 2, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (0, SCREEN_HEIGHT / 2, SCREEN_WIDTH, SCREEN_HEIGHT / 2),
            (0, 0, 0, 0),
        ],
        _ => [
            (0, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (SCREEN_WIDTH / 2, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (0, SCREEN_HEIGHT / 2, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
        ],
    }
}

pub fn setup_environment(track: TrackId) {
    unsafe {
        match track {
            TrackId::SunsetStrip => {
                env_gradient(0, 0xFF6B35FF, 0xF72585FF, 0x7209B7FF, 0x1A0533FF, 0.0, 0.2);
                env_lines(1, 0, 2, 3, 2.5, 100.0, 0xFF00FFFF, 0x00FFFFFF, 5, GRID_PHASE);
                env_blend(0);
            }
            TrackId::NeonCity => {
                env_gradient(0, 0x0D0221FF, 0x0D0221FF, 0x190A3DFF, 0x000000FF, 0.0, 0.0);
                env_rectangles(1, 1, 200, 180, 10, 28, 3, 0xFF00FFFF, 0x00FFFFFF, 120, WINDOW_PHASE);
                env_lines(2, 0, 2, 2, 2.0, 80.0, 0x00FFFFFF, 0xFF00FFFF, 6, GRID_PHASE);
                env_blend(1);
            }
            TrackId::VoidTunnel => {
                env_gradient(0, 0x000000FF, 0x000000FF, 0x000000FF, 0x000000FF, 0.0, 0.0);
                env_rings(1, 50, 4, 0xFF00FFFF, 0x00FFFFFF, 0xFFFFFFFF, 220, 10.0, 0.0, 0.0, 1.0, RING_PHASE);
                env_scatter(2, 3, 150, 3, 128, 20, 0x00FFFFFF, 0xFF00FFFF, 200, 150, SPEED_PHASE);
                env_blend(1);
            }
            TrackId::CrystalCavern => {
                env_gradient(0, 0x1A0533FF, 0x2D1B4EFF, 0x0D0221FF, 0x000000FF, 0.0, 0.0);
                env_scatter(1, 2, 180, 8, 200, 5, 0x00FFFFFF, 0xFF00FFFF, 250, 180, SPEED_PHASE);
                env_lines(2, 1, 1, 2, 3.0, 120.0, 0x8B5CF6FF, 0x00FFFFFF, 4, GRID_PHASE);
                env_blend(1);
            }
            TrackId::SolarHighway => {
                env_gradient(0, 0xFFFFFFFF, 0xFFAA00FF, 0xFF4400FF, 0x330000FF, 0.3, 0.0);
                env_scatter(1, 0, 100, 12, 255, 30, 0xFFFF00FF, 0xFF8800FF, 180, 200, SPEED_PHASE);
                env_rings(2, 30, 5, 0xFFAA00FF, 0xFFFFAAFF, 0xFFFFFFFF, 180, 5.0, 0.0, 0.2, 1.0, RING_PHASE);
                env_blend(1);
            }
        }
    }
}

pub fn render_track() {
    unsafe {
        // Get track-specific colors
        let (track_color_a, track_color_b, lane_color) = get_track_colors(SELECTED_TRACK);

        // Track surface - dark with neon grid lines
        material_metallic(0.1);
        material_roughness(0.8);
        material_emissive(0.2);

        // Render each track segment with its position and rotation
        for i in 0..TRACK_SEGMENT_COUNT {
            let segment = &TRACK_SEGMENTS[i];

            // Select mesh based on segment type
            let mesh = match segment.segment_type {
                SegmentType::Straight => MESH_TRACK_STRAIGHT,
                SegmentType::CurveLeft => MESH_TRACK_CURVE_LEFT,  // Could be custom curve mesh
                SegmentType::CurveRight => MESH_TRACK_STRAIGHT,   // Using straight for now
                SegmentType::Tunnel => MESH_TRACK_TUNNEL,
                SegmentType::Jump => MESH_TRACK_JUMP,
            };

            // Alternate track segment colors for visibility
            let track_color = if i % 2 == 0 { track_color_a } else { track_color_b };

            // Special colors for segment types
            let segment_color = match segment.segment_type {
                SegmentType::Tunnel => 0x1A1A30FF,  // Darker for tunnel
                SegmentType::Jump => 0x302018FF,    // Brown/orange tint for jump
                _ => track_color,
            };

            set_color(segment_color);

            push_identity();
            push_translate(segment.x, 0.0, segment.z);
            push_rotate_y(segment.rotation);
            draw_mesh(mesh);

            // Draw lane markings with track-specific color
            set_color(lane_color);
            material_emissive(2.0);

            // Left lane marker
            push_identity();
            push_translate(segment.x, 0.01, segment.z);
            push_rotate_y(segment.rotation);
            push_translate(-4.0, 0.0, 5.0);
            push_scale(0.1, 0.01, 8.0);
            draw_mesh(MESH_PROP_BARRIER);

            // Right lane marker
            push_identity();
            push_translate(segment.x, 0.01, segment.z);
            push_rotate_y(segment.rotation);
            push_translate(4.0, 0.0, 5.0);
            push_scale(0.1, 0.01, 8.0);
            draw_mesh(MESH_PROP_BARRIER);

            // Add special visuals for segment types
            match segment.segment_type {
                SegmentType::Tunnel => {
                    // Draw tunnel walls/ceiling hint
                    set_color(0x3030AAFF);
                    material_emissive(3.0);
                    push_identity();
                    push_translate(segment.x, 0.0, segment.z);
                    push_rotate_y(segment.rotation);
                    push_translate(-5.5, 2.0, 5.0);
                    push_scale(0.3, 4.0, 10.0);
                    draw_mesh(MESH_PROP_BARRIER);

                    push_identity();
                    push_translate(segment.x, 0.0, segment.z);
                    push_rotate_y(segment.rotation);
                    push_translate(5.5, 2.0, 5.0);
                    push_scale(0.3, 4.0, 10.0);
                    draw_mesh(MESH_PROP_BARRIER);
                }
                SegmentType::Jump => {
                    // Draw ramp indicator
                    set_color(0xFFAA00FF);
                    material_emissive(4.0);
                    push_identity();
                    push_translate(segment.x, 0.0, segment.z);
                    push_rotate_y(segment.rotation);
                    push_translate(0.0, 0.3, 5.0);
                    push_rotate_x(-15.0);  // Tilted ramp surface
                    push_scale(5.0, 0.1, 5.0);
                    draw_mesh(MESH_TRACK_STRAIGHT);
                }
                SegmentType::CurveLeft | SegmentType::CurveRight => {
                    // Add corner markers
                    let marker_color = if segment.segment_type == SegmentType::CurveLeft {
                        0xFF0000FF  // Red for left curve
                    } else {
                        0x0000FFFF  // Blue for right curve
                    };
                    set_color(marker_color);
                    material_emissive(3.0);

                    // Outer corner marker
                    let outer_x = if segment.segment_type == SegmentType::CurveLeft { 5.0 } else { -5.0 };
                    push_identity();
                    push_translate(segment.x, 0.0, segment.z);
                    push_rotate_y(segment.rotation);
                    push_translate(outer_x, 1.0, 8.0);
                    push_scale(0.5, 2.0, 0.5);
                    draw_mesh(MESH_PROP_BARRIER);
                }
                _ => {}
            }

            material_emissive(0.2);
        }

        set_color(0xFFFFFFFF);
        render_track_props_along_segments();
    }
}

/// Returns (track_color_a, track_color_b, lane_color) for the selected track
fn get_track_colors(track: TrackId) -> (u32, u32, u32) {
    match track {
        TrackId::SunsetStrip => (0x2A1520FF, 0x251018FF, 0xFF6B35FF),   // Dark red/orange, orange lanes
        TrackId::NeonCity => (0x1A1A2EFF, 0x16162AFF, 0x00FFFFFF),      // Dark blue/purple, cyan lanes
        TrackId::VoidTunnel => (0x0A0A0AFF, 0x050505FF, 0xFF00FFFF),    // Near black, magenta lanes
        TrackId::CrystalCavern => (0x1A2030FF, 0x152028FF, 0x8B5CF6FF), // Dark blue/grey, purple lanes
        TrackId::SolarHighway => (0x2A2010FF, 0x251A08FF, 0xFFAA00FF),  // Dark orange/brown, gold lanes
    }
}

pub fn render_track_props() {
    render_track_props_along_segments();
}

/// Render props positioned along track segments
fn render_track_props_along_segments() {
    unsafe {
        // Get track-specific prop configuration
        let (building_color_l, building_color_r, barrier_color_l, barrier_color_r,
             billboard_color, boost_color, has_buildings, prop_density) = get_track_props(SELECTED_TRACK);

        material_metallic(0.5);
        material_roughness(0.5);

        // Place props along segments
        for i in 0..TRACK_SEGMENT_COUNT {
            let segment = &TRACK_SEGMENTS[i];

            // Barriers at each segment
            material_emissive(3.0);
            set_color(barrier_color_l);
            push_identity();
            push_translate(segment.x, 0.0, segment.z);
            push_rotate_y(segment.rotation);
            push_translate(-5.5, 0.0, 5.0);
            draw_mesh(MESH_PROP_BARRIER);

            set_color(barrier_color_r);
            push_identity();
            push_translate(segment.x, 0.0, segment.z);
            push_rotate_y(segment.rotation);
            push_translate(5.5, 0.0, 5.0);
            draw_mesh(MESH_PROP_BARRIER);

            // Buildings every few segments (if track has them)
            if has_buildings && i % 3 == 0 {
                material_emissive(0.5);
                let height_var = ((i * 17) % 5) as f32;

                set_color(building_color_l);
                push_identity();
                push_translate(segment.x, 0.0, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(-15.0, 0.0, 5.0);
                push_scale(1.5, 2.0 + height_var, 1.5);
                draw_mesh(MESH_PROP_BUILDING);

                set_color(building_color_r);
                push_identity();
                push_translate(segment.x, 0.0, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(15.0, 0.0, 5.0);
                push_scale(1.2, 1.5 + height_var * 0.5, 1.2);
                draw_mesh(MESH_PROP_BUILDING);
            }

            // Billboards every few segments
            if prop_density > 0 && i % 5 == 2 {
                set_color(billboard_color);
                material_emissive(4.0);
                push_identity();
                push_translate(segment.x, 0.0, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(-12.0, 8.0, 5.0);
                draw_mesh(MESH_PROP_BILLBOARD);
            }

            // Boost pads on specific segments
            if i % 4 == 1 && segment.segment_type == SegmentType::Straight {
                set_color(boost_color);
                material_emissive(5.0);
                let x_offset = if i % 8 < 4 { -2.0 } else { 2.0 };
                push_identity();
                push_translate(segment.x, 0.0, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(x_offset, 0.02, 5.0);
                draw_mesh(MESH_PROP_BOOST_PAD);
            }
        }

        set_color(0xFFFFFFFF);
    }
}

/// Returns track-specific prop configuration:
/// (building_l, building_r, barrier_l, barrier_r, billboard, boost, has_buildings, prop_density)
fn get_track_props(track: TrackId) -> (u32, u32, u32, u32, u32, u32, bool, u32) {
    match track {
        TrackId::SunsetStrip => (
            0x3A2030FF, 0x302028FF,  // Warm dark buildings
            0xFF6B35FF, 0xF72585FF,  // Orange/pink barriers
            0xFF6B35FF,              // Orange billboards
            0xFFAA00FF,              // Gold boost
            true, 4                   // Has buildings, full props
        ),
        TrackId::NeonCity => (
            0x2A1A4AFF, 0x1A2A4AFF,  // Purple/blue buildings
            0xFF00FFFF, 0x00FFFFFF,  // Magenta/cyan barriers
            0xFF00AAFF,              // Pink billboards
            0x00FFFFFF,              // Cyan boost
            true, 4                   // Has buildings, full props
        ),
        TrackId::VoidTunnel => (
            0x101010FF, 0x080808FF,  // Nearly invisible buildings
            0xFF00FFFF, 0x00FFFFFF,  // Magenta/cyan barriers (more visible in void)
            0xFFFFFFFF,              // White billboards
            0xFF00FFFF,              // Magenta boost
            false, 0                  // No buildings, minimal props (it's a void!)
        ),
        TrackId::CrystalCavern => (
            0x2A2040FF, 0x1A2A50FF,  // Purple/blue crystal-like buildings
            0x8B5CF6FF, 0x00FFFFFF,  // Purple/cyan barriers
            0x8B5CF6FF,              // Purple billboards
            0x00FFFFFF,              // Cyan boost
            true, 2                   // Sparse props (it's a cavern)
        ),
        TrackId::SolarHighway => (
            0x3A2A10FF, 0x302008FF,  // Warm brown buildings (sun-baked)
            0xFFAA00FF, 0xFFFF00FF,  // Gold/yellow barriers
            0xFFAA00FF,              // Gold billboards
            0xFFFF00FF,              // Yellow boost
            true, 3                   // Medium props
        ),
    }
}

pub fn render_all_cars() {
    unsafe {
        // Render ALL 4 cars (both players and AI)
        for i in 0..4 {
            let car = &CARS[i];
            let is_ai = i >= ACTIVE_PLAYER_COUNT as usize;

            // Car colors based on type (using set_color since textures may not exist)
            let (mesh, mut color) = match car.car_type {
                CarType::Speedster => (MESH_SPEEDSTER, 0xFF3333FF),  // Red
                CarType::Muscle => (MESH_MUSCLE, 0x3366FFFF),        // Blue
                CarType::Racer => (MESH_RACER, 0x33FF33FF),          // Green
                CarType::Drift => (MESH_DRIFT, 0xFF9900FF),          // Orange
                CarType::Phantom => (MESH_PHANTOM, 0x9933FFFF),      // Purple
                CarType::Titan => (MESH_TITAN, 0xFFCC00FF),          // Gold
                CarType::Viper => (MESH_VIPER, 0x00FFFFFF),          // Cyan
            };

            // AI cars have slightly darker/desaturated colors to distinguish them
            if is_ai {
                // Darken AI car colors by reducing RGB components
                let r = ((color >> 24) & 0xFF) * 7 / 10;
                let g = ((color >> 16) & 0xFF) * 7 / 10;
                let b = ((color >> 8) & 0xFF) * 7 / 10;
                let a = color & 0xFF;
                color = (r << 24) | (g << 16) | (b << 8) | a;
            }

            // Use vertex color instead of texture for now
            set_color(color);
            material_metallic(0.8);
            material_roughness(0.2);
            material_emissive(if is_ai { 1.0 } else { 1.5 });

            push_identity();
            push_translate(car.x, car.y + 0.2, car.z);  // Lift car slightly above ground
            push_rotate_y(car.rotation_y);
            draw_mesh(mesh);
        }

        // Reset color
        set_color(0xFFFFFFFF);
    }
}

pub fn render_racing_view() {
    unsafe {
        let viewports = get_viewport_layout(ACTIVE_PLAYER_COUNT);

        for player_id in 0..ACTIVE_PLAYER_COUNT as usize {
            let (vp_x, vp_y, vp_w, vp_h) = viewports[player_id];

            viewport(vp_x, vp_y, vp_w, vp_h);

            let camera = &CAMERAS[player_id];
            camera_set(
                camera.current_pos_x, camera.current_pos_y, camera.current_pos_z,
                camera.current_target_x, camera.current_target_y, camera.current_target_z
            );
            camera_fov(75.0);

            setup_environment(SELECTED_TRACK);
            draw_env();

            // Set up lighting for the scene (required for PBR mode)
            light_set(0, -0.3, -0.8, -0.5);
            light_color(0, 0xFFFFFFFF);
            light_intensity(0, 2.0);
            light_enable(0);

            // Unbind any texture to use vertex colors for 3D meshes
            texture_bind(0);

            render_track();
            render_all_cars();
        }

        viewport_clear();
    }
}

pub fn render_racing() {
    unsafe {
        let viewports = get_viewport_layout(ACTIVE_PLAYER_COUNT);

        for player_id in 0..ACTIVE_PLAYER_COUNT as usize {
            let (vp_x, vp_y, vp_w, vp_h) = viewports[player_id];

            viewport(vp_x, vp_y, vp_w, vp_h);

            let camera = &CAMERAS[player_id];
            let shake_x = camera.shake_offset_x;
            let shake_y = camera.shake_offset_y;
            camera_set(
                camera.current_pos_x + shake_x,
                camera.current_pos_y + shake_y,
                camera.current_pos_z,
                camera.current_target_x + shake_x,
                camera.current_target_y + shake_y,
                camera.current_target_z
            );
            camera_fov(75.0);

            setup_environment(SELECTED_TRACK);
            draw_env();

            // Set up lighting for the scene (required for PBR mode)
            light_set(0, -0.3, -0.8, -0.5);
            light_color(0, 0xFFFFFFFF);
            light_intensity(0, 2.0);
            light_enable(0);

            // Unbind any texture to use vertex colors for 3D meshes
            texture_bind(0);

            render_track();
            render_all_cars();
            render_particles();

            render_speed_lines(player_id, vp_w, vp_h);

            if BOOST_GLOW_INTENSITY[player_id] > 0.1 {
                render_vignette(BOOST_GLOW_INTENSITY[player_id]);
            }

            render_hud(player_id as u32, vp_w, vp_h);
        }

        viewport_clear();
    }
}

pub fn render_attract_mode() {
    unsafe {
        render_racing();

        font_bind(0);  // Use built-in font for readability
        depth_test(0);

        let t = TITLE_ANIM_TIME;
        let demo_pulse = (libm::sinf(t * 3.0) * 0.2 + 0.8) as f32;
        let demo_alpha = (demo_pulse * 255.0) as u32;
        let demo_color = 0xFF00FF00 | demo_alpha;

        let demo_text = b"DEMO MODE";
        draw_text(demo_text.as_ptr(), demo_text.len() as u32, 400.0, 20.0, 32.0, demo_color);

        let prompt = b"PRESS ANY BUTTON";
        let blink = if (t * 2.0) as u32 % 2 == 0 { 0xFFFFFFFF } else { 0x666666FF };
        draw_text(prompt.as_ptr(), prompt.len() as u32, 360.0, 500.0, 20.0, blink);

        depth_test(1);
    }
}
