//! 3D rendering for NEON DRIFT
//!
//! Track, cars, environment, and visual effects.

use crate::ffi::*;
use crate::types::*;
use crate::state::*;
use crate::particles::{render_particles, render_speed_lines, render_vignette};
use crate::hud::render_hud;

pub fn load_rom_texture(id: &[u8]) -> u32 {
    unsafe { rom_texture(id.as_ptr() as u32, id.len() as u32) }
}

pub fn load_rom_mesh(id: &[u8]) -> u32 {
    unsafe { rom_mesh(id.as_ptr() as u32, id.len() as u32) }
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

        // Render each track segment with full 3D transformation
        for i in 0..TRACK_SEGMENT_COUNT {
            let segment = &TRACK_SEGMENTS[i];
            let half_len = segment.length * 0.5;

            // Determine base color from style
            let style_color = match segment.style {
                SegmentStyle::Tunnel => 0x1A1A30FF,   // Dark blue/purple
                SegmentStyle::Bridge => 0x2A2A3AFF,   // Metallic grey
                SegmentStyle::Canyon => 0x302820FF,   // Rocky brown
                SegmentStyle::Coastal => 0x201830FF,  // Ocean-tinted
                SegmentStyle::Open => if i % 2 == 0 { track_color_a } else { track_color_b },
            };

            // Modify color based on elevation
            let segment_color = match segment.elevation {
                Elevation::Jump => blend_color(style_color, 0xFFAA00FF, 0.3),   // Orange tint for jumps
                Elevation::Bridge => blend_color(style_color, 0x4444FFFF, 0.2), // Blue tint for bridges
                Elevation::SteepUp | Elevation::SteepDown => blend_color(style_color, 0x00FF00FF, 0.1), // Green tint for slopes
                _ => style_color,
            };

            set_color(segment_color);

            // Full 3D transform: position, rotation, pitch, roll
            // Track mesh is 10x10 centered at origin, scale to fit segment length
            push_identity();
            push_translate(segment.x, segment.y, segment.z);
            push_rotate_y(segment.rotation);
            push_translate(0.0, 0.0, half_len);  // Center the mesh on the segment
            push_rotate_x(segment.pitch());      // Elevation tilt
            push_rotate_z(segment.roll());       // Banking
            push_scale(1.0, 1.0, segment.length / 10.0);  // Scale for length
            draw_mesh(MESH_TRACK_STRAIGHT);

            // Lane markings
            set_color(lane_color);
            material_emissive(2.0);

            // Left lane marker (follows terrain)
            push_identity();
            push_translate(segment.x, segment.y + 0.02, segment.z);
            push_rotate_y(segment.rotation);
            push_rotate_x(segment.pitch());
            push_rotate_z(segment.roll());
            push_translate(-4.0, 0.0, half_len);
            push_scale(0.1, 0.02, segment.length * 0.8);
            draw_mesh(MESH_PROP_BARRIER);

            // Right lane marker
            push_identity();
            push_translate(segment.x, segment.y + 0.02, segment.z);
            push_rotate_y(segment.rotation);
            push_rotate_x(segment.pitch());
            push_rotate_z(segment.roll());
            push_translate(4.0, 0.0, half_len);
            push_scale(0.1, 0.02, segment.length * 0.8);
            draw_mesh(MESH_PROP_BARRIER);

            // Style-specific decorations
            render_segment_style(segment, lane_color);

            // Turn markers for curves
            render_turn_markers(segment);

            // Elevation indicators
            render_elevation_markers(segment);

            material_emissive(0.2);
        }

        set_color(0xFFFFFFFF);
        render_track_props_along_segments();
    }
}

/// Blend two RGBA colors
fn blend_color(base: u32, tint: u32, amount: f32) -> u32 {
    let r1 = ((base >> 24) & 0xFF) as f32;
    let g1 = ((base >> 16) & 0xFF) as f32;
    let b1 = ((base >> 8) & 0xFF) as f32;
    let r2 = ((tint >> 24) & 0xFF) as f32;
    let g2 = ((tint >> 16) & 0xFF) as f32;
    let b2 = ((tint >> 8) & 0xFF) as f32;

    let r = (r1 * (1.0 - amount) + r2 * amount) as u32;
    let g = (g1 * (1.0 - amount) + g2 * amount) as u32;
    let b = (b1 * (1.0 - amount) + b2 * amount) as u32;

    (r << 24) | (g << 16) | (b << 8) | 0xFF
}

/// Render style-specific decorations (tunnels, bridges, canyons)
fn render_segment_style(segment: &TrackSegment, lane_color: u32) {
    unsafe {
        let half_len = segment.length * 0.5;

        match segment.style {
            SegmentStyle::Tunnel => {
                // Tunnel walls
                set_color(0x2020AAFF);
                material_emissive(2.0);

                // Left wall
                push_identity();
                push_translate(segment.x, segment.y, segment.z);
                push_rotate_y(segment.rotation);
                push_rotate_x(segment.pitch());
                push_translate(-5.5, 2.0, half_len);
                push_scale(0.3, 4.0, segment.length);
                draw_mesh(MESH_PROP_BARRIER);

                // Right wall
                push_identity();
                push_translate(segment.x, segment.y, segment.z);
                push_rotate_y(segment.rotation);
                push_rotate_x(segment.pitch());
                push_translate(5.5, 2.0, half_len);
                push_scale(0.3, 4.0, segment.length);
                draw_mesh(MESH_PROP_BARRIER);

                // Ceiling (neon strip)
                set_color(lane_color);
                material_emissive(5.0);
                push_identity();
                push_translate(segment.x, segment.y, segment.z);
                push_rotate_y(segment.rotation);
                push_rotate_x(segment.pitch());
                push_translate(0.0, 4.5, half_len);
                push_scale(0.3, 0.1, segment.length);
                draw_mesh(MESH_PROP_BARRIER);
            }
            SegmentStyle::Bridge => {
                // Bridge railings
                set_color(0x666688FF);
                material_emissive(1.5);

                // Left railing
                push_identity();
                push_translate(segment.x, segment.y, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(-5.2, 0.5, half_len);
                push_scale(0.15, 1.0, segment.length);
                draw_mesh(MESH_PROP_BARRIER);

                // Right railing
                push_identity();
                push_translate(segment.x, segment.y, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(5.2, 0.5, half_len);
                push_scale(0.15, 1.0, segment.length);
                draw_mesh(MESH_PROP_BARRIER);

                // Support pillars (every other segment)
                if (segment.x as i32 / 20) % 2 == 0 {
                    set_color(0x444466FF);
                    push_identity();
                    push_translate(segment.x, segment.y - 5.0, segment.z);
                    push_rotate_y(segment.rotation);
                    push_translate(-4.0, 0.0, half_len);
                    push_scale(0.5, 10.0, 0.5);
                    draw_mesh(MESH_PROP_BARRIER);

                    push_identity();
                    push_translate(segment.x, segment.y - 5.0, segment.z);
                    push_rotate_y(segment.rotation);
                    push_translate(4.0, 0.0, half_len);
                    push_scale(0.5, 10.0, 0.5);
                    draw_mesh(MESH_PROP_BARRIER);
                }
            }
            SegmentStyle::Canyon => {
                // Canyon walls
                set_color(0x403020FF);
                material_emissive(0.5);

                // Left cliff wall
                push_identity();
                push_translate(segment.x, segment.y, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(-7.0, 4.0, half_len);
                push_scale(1.5, 8.0, segment.length);
                draw_mesh(MESH_PROP_BUILDING);

                // Right cliff wall
                push_identity();
                push_translate(segment.x, segment.y, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(7.0, 4.0, half_len);
                push_scale(1.5, 8.0, segment.length);
                draw_mesh(MESH_PROP_BUILDING);
            }
            SegmentStyle::Coastal => {
                // Water hint on one side
                set_color(0x004488AA);
                material_emissive(2.0);

                push_identity();
                push_translate(segment.x, segment.y - 2.0, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(-15.0, 0.0, half_len);
                push_scale(10.0, 0.1, segment.length * 2.0);
                draw_mesh(MESH_TRACK_STRAIGHT);
            }
            SegmentStyle::Open => {}
        }
    }
}

/// Render turn apex markers
fn render_turn_markers(segment: &TrackSegment) {
    unsafe {
        let turn_deg = segment.turn.degrees();
        if turn_deg.abs() < 10.0 { return; }  // No markers for straights

        let half_len = segment.length * 0.5;

        // Color based on turn direction
        let marker_color = if turn_deg < 0.0 { 0xFF3333FF } else { 0x3333FFFF };
        set_color(marker_color);
        material_emissive(4.0);

        // Outer apex marker
        let outer_x = if turn_deg < 0.0 { 5.0 } else { -5.0 };

        push_identity();
        push_translate(segment.x, segment.y, segment.z);
        push_rotate_y(segment.rotation);
        push_translate(outer_x, 1.0, half_len);
        push_scale(0.4, 2.0, 0.4);
        draw_mesh(MESH_PROP_BARRIER);

        // For hairpins, add extra warning markers
        if turn_deg.abs() >= 90.0 {
            set_color(0xFFFF00FF);  // Yellow warning
            material_emissive(5.0);

            push_identity();
            push_translate(segment.x, segment.y, segment.z);
            push_rotate_y(segment.rotation);
            push_translate(outer_x * 0.8, 2.5, half_len);
            push_scale(0.6, 0.6, 0.6);
            draw_mesh(MESH_PROP_BARRIER);
        }
    }
}

/// Render elevation change indicators
fn render_elevation_markers(segment: &TrackSegment) {
    unsafe {
        let half_len = segment.length * 0.5;

        match segment.elevation {
            Elevation::Jump => {
                // Chevron arrows pointing up for jump
                set_color(0xFFAA00FF);
                material_emissive(6.0);

                for j in 0..3 {
                    push_identity();
                    push_translate(segment.x, segment.y + 0.1, segment.z);
                    push_rotate_y(segment.rotation);
                    push_translate(0.0, 0.0, half_len - 2.0 + (j as f32) * 2.0);
                    push_rotate_x(-20.0);
                    push_scale(3.0, 0.1, 1.0);
                    draw_mesh(MESH_PROP_BOOST_PAD);
                }
            }
            Elevation::SteepUp | Elevation::GentleUp => {
                // Uphill indicator
                set_color(0x00FF66FF);
                material_emissive(3.0);

                push_identity();
                push_translate(segment.x, segment.y + 0.1, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(-4.5, 0.0, half_len);
                push_rotate_x(-segment.pitch());
                push_scale(0.2, 0.1, 2.0);
                draw_mesh(MESH_PROP_BARRIER);
            }
            Elevation::SteepDown | Elevation::GentleDown => {
                // Downhill indicator
                set_color(0xFF6600FF);
                material_emissive(3.0);

                push_identity();
                push_translate(segment.x, segment.y + 0.1, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(-4.5, 0.0, half_len);
                push_rotate_x(-segment.pitch());
                push_scale(0.2, 0.1, 2.0);
                draw_mesh(MESH_PROP_BARRIER);
            }
            Elevation::Crest => {
                // Peak indicator
                set_color(0xFFFF00FF);
                material_emissive(4.0);

                push_identity();
                push_translate(segment.x, segment.y + 1.0, segment.z);
                push_rotate_y(segment.rotation);
                push_translate(0.0, 0.0, half_len);
                push_scale(0.5, 0.5, 0.5);
                draw_mesh(MESH_PROP_BARRIER);
            }
            _ => {}
        }
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

            // Boost pads on straight segments only
            if i % 4 == 1 && segment.turn == TurnAngle::Straight {
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

            // Get mesh and textures for car type
            let (mesh, tex_albedo, tex_emissive) = match car.car_type {
                CarType::Speedster => (MESH_SPEEDSTER, TEX_SPEEDSTER, TEX_SPEEDSTER_EMISSIVE),
                CarType::Muscle => (MESH_MUSCLE, TEX_MUSCLE, TEX_MUSCLE_EMISSIVE),
                CarType::Racer => (MESH_RACER, TEX_RACER, TEX_RACER_EMISSIVE),
                CarType::Drift => (MESH_DRIFT, TEX_DRIFT, TEX_DRIFT_EMISSIVE),
                CarType::Phantom => (MESH_PHANTOM, TEX_PHANTOM, TEX_PHANTOM_EMISSIVE),
                CarType::Titan => (MESH_TITAN, TEX_TITAN, TEX_TITAN_EMISSIVE),
                CarType::Viper => (MESH_VIPER, TEX_VIPER, TEX_VIPER_EMISSIVE),
            };

            // Bind textures for PBR rendering
            set_color(0xFFFFFFFF);  // White base - let texture show through
            texture_bind(tex_albedo);
            texture_bind_slot(tex_emissive, 1);  // Slot 1 for emissive/MRE

            // PBR material properties
            material_metallic(0.7);
            material_roughness(0.3);
            material_emissive(if is_ai { 1.5 } else { 2.5 });  // Neon glow

            push_identity();
            push_translate(car.x, car.y + 0.2, car.z);  // Lift car slightly above ground
            push_rotate_y(car.rotation_y);  // Physics and mesh both use -Z as forward
            draw_mesh(mesh);
        }

        // Reset textures and color
        texture_bind(0);
        texture_bind_slot(0, 1);
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
