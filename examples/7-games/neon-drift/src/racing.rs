//! Race logic for NEON DRIFT

use crate::ffi::*;
use crate::types::*;
use crate::state::*;
use crate::physics::*;
use crate::particles::update_particles;

/// Generates track layout for the selected track
pub fn generate_track_layout(track: TrackId) {
    unsafe {
        TRACK_SEGMENT_COUNT = 0;
        WAYPOINT_COUNT = 0;

        // Segment length for calculations
        let seg_len = 10.0;

        // Define track layouts for each track
        let layout: &[SegmentType] = match track {
            TrackId::SunsetStrip => &[
                // Beginner track: Wide curves, long straights
                SegmentType::Straight, SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight, SegmentType::CurveRight,
                SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight, SegmentType::CurveRight,
                SegmentType::Straight, SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight, SegmentType::CurveRight,
                SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight, SegmentType::CurveRight,
            ],
            TrackId::NeonCity => &[
                // Intermediate: Tight corners, S-curves
                SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveLeft, SegmentType::CurveLeft,
                SegmentType::Straight,
                SegmentType::CurveRight, SegmentType::CurveRight,
                SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight,
                SegmentType::Straight,
                SegmentType::CurveLeft,
                SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight, SegmentType::CurveRight,
                SegmentType::Straight,
                SegmentType::CurveRight,
            ],
            TrackId::VoidTunnel => &[
                // Advanced: Mostly tunnel, twisting
                SegmentType::Tunnel, SegmentType::Tunnel,
                SegmentType::CurveLeft,
                SegmentType::Tunnel, SegmentType::Tunnel,
                SegmentType::CurveRight, SegmentType::CurveRight,
                SegmentType::Tunnel,
                SegmentType::CurveLeft, SegmentType::CurveLeft,
                SegmentType::Tunnel, SegmentType::Tunnel,
                SegmentType::CurveRight,
                SegmentType::Tunnel,
                SegmentType::CurveLeft,
                SegmentType::Tunnel, SegmentType::Tunnel,
                SegmentType::CurveRight, SegmentType::CurveRight,
            ],
            TrackId::CrystalCavern => &[
                // Hard: Tight S-curves, jumps
                SegmentType::Straight,
                SegmentType::CurveLeft, SegmentType::CurveRight,  // S-curve
                SegmentType::Jump,
                SegmentType::Straight,
                SegmentType::CurveRight, SegmentType::CurveLeft,  // S-curve
                SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveLeft, SegmentType::CurveLeft,
                SegmentType::Jump,
                SegmentType::CurveRight, SegmentType::CurveRight,
                SegmentType::Straight,
                SegmentType::CurveRight, SegmentType::CurveLeft,  // S-curve
                SegmentType::CurveRight, SegmentType::CurveRight,
            ],
            TrackId::SolarHighway => &[
                // Expert: High speed, wide sweeping curves, big jumps
                SegmentType::Straight, SegmentType::Straight, SegmentType::Straight,
                SegmentType::Jump,
                SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight,
                SegmentType::Straight, SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight,
                SegmentType::Jump,
                SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight,
                SegmentType::Straight, SegmentType::Straight, SegmentType::Straight,
                SegmentType::CurveRight,
            ],
        };

        // Build segment positions from layout
        let mut cur_x: f32 = 0.0;
        let mut cur_z: f32 = 0.0;
        let mut cur_rot: f32 = 0.0;  // Current heading in degrees

        for (i, &seg_type) in layout.iter().enumerate() {
            if i >= MAX_TRACK_SEGMENTS { break; }

            // Store segment
            TRACK_SEGMENTS[i] = TrackSegment {
                x: cur_x,
                z: cur_z,
                rotation: cur_rot,
                segment_type: seg_type,
            };

            // Add waypoint at center of segment
            if WAYPOINT_COUNT < MAX_WAYPOINTS {
                let sin_r = libm::sinf(cur_rot * 3.14159 / 180.0);
                let cos_r = libm::cosf(cur_rot * 3.14159 / 180.0);
                WAYPOINTS[WAYPOINT_COUNT] = Waypoint {
                    x: cur_x + sin_r * seg_len * 0.5,
                    z: cur_z + cos_r * seg_len * 0.5,
                };
                WAYPOINT_COUNT += 1;
            }

            // Advance position based on segment type
            let sin_r = libm::sinf(cur_rot * 3.14159 / 180.0);
            let cos_r = libm::cosf(cur_rot * 3.14159 / 180.0);

            match seg_type {
                SegmentType::CurveLeft => {
                    // Move forward and rotate left
                    cur_x += sin_r * seg_len;
                    cur_z += cos_r * seg_len;
                    cur_rot -= 45.0;  // Turn left
                }
                SegmentType::CurveRight => {
                    // Move forward and rotate right
                    cur_x += sin_r * seg_len;
                    cur_z += cos_r * seg_len;
                    cur_rot += 45.0;  // Turn right
                }
                _ => {
                    // Straight, tunnel, jump - just move forward
                    cur_x += sin_r * seg_len;
                    cur_z += cos_r * seg_len;
                }
            }

            TRACK_SEGMENT_COUNT = i + 1;
        }

        // Calculate total track length (approximate)
        TRACK_LENGTH = TRACK_SEGMENT_COUNT as f32 * seg_len;

        // Update checkpoints based on track length
        for i in 0..NUM_CHECKPOINTS {
            CHECKPOINT_Z[i] = (i as f32 / NUM_CHECKPOINTS as f32) * TRACK_LENGTH;
        }
    }
}

pub fn init_race() {
    unsafe {
        RACE_TIME = 0.0;
        RACE_FINISHED = false;

        // Generate track layout for selected track
        generate_track_layout(SELECTED_TRACK);

        // Start background music for the selected track
        let music_handle = match SELECTED_TRACK {
            TrackId::SunsetStrip => MUSIC_SUNSET_STRIP,
            TrackId::NeonCity => MUSIC_NEON_CITY,
            TrackId::VoidTunnel => MUSIC_VOID_TUNNEL,
            TrackId::CrystalCavern => MUSIC_CRYSTAL_CAVERN,
            TrackId::SolarHighway => MUSIC_SOLAR_HIGHWAY,
        };
        if music_handle != 0 {
            music_play(music_handle, 0.7, 1);  // volume 70%, loop
        }

        // Position cars on starting grid
        for i in 0..4 {
            CARS[i].x = (i as f32 - 1.5) * 2.5;
            CARS[i].y = 0.0;
            CARS[i].z = 5.0 + (i as f32) * 3.0;  // Start at Z=5, facing +Z
            CARS[i].rotation_y = 0.0;  // Facing +Z direction
            CARS[i].velocity_forward = 0.0;
            CARS[i].velocity_lateral = 0.0;
            CARS[i].angular_velocity = 0.0;
            CARS[i].boost_meter = 0.5;
            CARS[i].is_boosting = false;
            CARS[i].is_drifting = false;
            CARS[i].current_lap = 0;
            CARS[i].last_checkpoint = 0;
            CARS[i].race_position = (i + 1) as u32;
            CARS[i].collision_pushback_x = 0.0;
            CARS[i].collision_pushback_z = 0.0;
            CARS[i].current_waypoint = 0;

            // Initialize camera directly behind and above the car
            let car = &CARS[i];
            let offset_distance = 8.0;
            let offset_height = 3.0;

            CAMERAS[i] = Camera::new();
            CAMERAS[i].current_pos_x = car.x;
            CAMERAS[i].current_pos_y = car.y + offset_height;
            CAMERAS[i].current_pos_z = car.z - offset_distance;
            CAMERAS[i].current_target_x = car.x;
            CAMERAS[i].current_target_y = car.y + 1.0;
            CAMERAS[i].current_target_z = car.z + 5.0;
        }
    }
}

pub fn start_attract_mode() {
    unsafe {
        GAME_MODE = GameMode::AttractMode;
        IDLE_TIMER = 0.0;

        // Setup a demo race
        for i in 0..4 {
            CARS[i].car_type = match i {
                0 => CarType::Speedster,
                1 => CarType::Muscle,
                2 => CarType::Racer,
                _ => CarType::Drift,
            };
            CARS[i].init_stats();
        }

        init_race();
        ACTIVE_PLAYER_COUNT = 0; // All AI
    }
}

pub fn update_countdown(dt: f32) {
    unsafe {
        if COUNTDOWN_TIMER > 0 {
            COUNTDOWN_TIMER -= 1;
        } else {
            GAME_MODE = GameMode::Racing;
            RACE_TIME = 0.0;
            RACE_FINISHED = false;
        }

        for i in 0..ACTIVE_PLAYER_COUNT as usize {
            update_camera(&mut CAMERAS[i], &CARS[i], dt);
        }
    }
}

pub fn update_racing(dt: f32) {
    unsafe {
        RACE_TIME += dt;

        // Update human players
        for i in 0..ACTIVE_PLAYER_COUNT as usize {
            update_car_physics(&mut CARS[i], i as u32, dt);
            check_track_collision_with_effects(&mut CARS[i], i);
            check_checkpoints(&mut CARS[i], i);
            update_camera(&mut CAMERAS[i], &CARS[i], dt);
            CAMERAS[i].update_shake(random());

            // Update visual effects
            let speed_ratio = CARS[i].velocity_forward / CARS[i].max_speed;
            SPEED_LINE_INTENSITY[i] = (speed_ratio - 0.7).max(0.0) * 3.0;
            BOOST_GLOW_INTENSITY[i] = if CARS[i].is_boosting { 0.5 } else { 0.0 };

            spawn_car_particles(&CARS[i]);
        }

        // Update AI cars
        for i in ACTIVE_PLAYER_COUNT as usize..4 {
            update_ai_car(&mut CARS[i], dt);
        }

        // Update particles
        update_particles(dt);

        // Calculate positions
        calculate_positions();

        // Check for race finish (3 laps)
        for i in 0..ACTIVE_PLAYER_COUNT as usize {
            if CARS[i].current_lap >= 3 && !RACE_FINISHED {
                RACE_FINISHED = true;
                GAME_MODE = GameMode::RaceFinished;
                play_sound(SND_FINISH, 1.0, 0.0);
            }
        }

        // Pause check
        if button_pressed(0, BUTTON_START) != 0 {
            GAME_MODE = GameMode::Paused;
        }
    }
}

pub fn update_attract_mode(dt: f32) {
    unsafe {
        // Check for any input to exit attract mode
        for p in 0..4 {
            if buttons_held(p) != 0 {
                GAME_MODE = GameMode::MainMenu;
                MENU_SELECTION = 0;
                IDLE_TIMER = 0.0;
                return;
            }
        }

        // Run AI for all cars
        for i in 0..4 {
            update_ai_car(&mut CARS[i], dt);
            update_camera(&mut CAMERAS[i], &CARS[i], dt);
        }

        update_particles(dt);
        calculate_positions();

        // Loop demo race
        if CARS[0].current_lap >= 2 {
            init_race();
        }
    }
}
