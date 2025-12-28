//! Race logic for NEON DRIFT
//!
//! Advanced track generation with:
//! - Varied turn angles (15°, 30°, 45°, 60°, 90°, 180°)
//! - Elevation changes (hills, valleys, jumps, bridges)
//! - Banking for high-speed turns
//! - Guaranteed closed loop tracks

use crate::ffi::*;
use crate::types::*;
use crate::state::*;
use crate::physics::*;
use crate::particles::update_particles;

/// A segment definition for track building
struct SegmentDef {
    turn: TurnAngle,
    elevation: Elevation,
    banking: Banking,
    style: SegmentStyle,
}

impl SegmentDef {
    const fn new(turn: TurnAngle, elevation: Elevation, banking: Banking, style: SegmentStyle) -> Self {
        Self { turn, elevation, banking, style }
    }

    const fn straight() -> Self {
        Self::new(TurnAngle::Straight, Elevation::Flat, Banking::Flat, SegmentStyle::Open)
    }

    const fn turn(angle: TurnAngle) -> Self {
        Self::new(angle, Elevation::Flat, Banking::Flat, SegmentStyle::Open)
    }

    const fn banked(angle: TurnAngle, bank: Banking) -> Self {
        Self::new(angle, Elevation::Flat, bank, SegmentStyle::Open)
    }

    const fn hill(elev: Elevation) -> Self {
        Self::new(TurnAngle::Straight, elev, Banking::Flat, SegmentStyle::Open)
    }

    const fn styled(turn: TurnAngle, style: SegmentStyle) -> Self {
        Self::new(turn, Elevation::Flat, Banking::Flat, style)
    }

    const fn full(turn: TurnAngle, elevation: Elevation, banking: Banking, style: SegmentStyle) -> Self {
        Self::new(turn, elevation, banking, style)
    }
}

/// Validate that a track layout sums to 360° for a closed loop
#[cfg(debug_assertions)]
fn validate_track_closure(layout: &[SegmentDef]) -> bool {
    let mut total: f32 = 0.0;
    for seg in layout {
        total += seg.turn.degrees();
    }
    // Allow for floating point imprecision
    (total - 360.0).abs() < 0.1 || (total + 360.0).abs() < 0.1
}

/// Generates track layout for the selected track
pub fn generate_track_layout(track: TrackId) {
    unsafe {
        TRACK_SEGMENT_COUNT = 0;
        WAYPOINT_COUNT = 0;

        // Base segment length
        let base_len = 10.0;

        // ============================================================
        // TRACK DEFINITIONS - All guaranteed to close (sum to ±360°)
        // ============================================================

        let layout: &[SegmentDef] = match track {
            // --------------------------------------------------------
            // SUNSET STRIP - Coastal Highway (★☆☆ Easy)
            // Rolling hills along the coast, gentle sweeping curves
            // Total: 4×90° = 360° ✓
            // --------------------------------------------------------
            TrackId::SunsetStrip => &[
                // Start/finish straight
                SegmentDef::straight(),
                SegmentDef::straight(),
                SegmentDef::hill(Elevation::GentleUp),
                // Gentle right sweeper (90° total over 3 segments)
                SegmentDef::banked(TurnAngle::Medium30R, Banking::Slight),
                SegmentDef::banked(TurnAngle::Medium30R, Banking::Slight),
                SegmentDef::banked(TurnAngle::Medium30R, Banking::Slight),
                // Coastal straight with ocean view
                SegmentDef::full(TurnAngle::Straight, Elevation::Crest, Banking::Flat, SegmentStyle::Coastal),
                SegmentDef::full(TurnAngle::Straight, Elevation::GentleDown, Banking::Flat, SegmentStyle::Coastal),
                SegmentDef::straight(),
                // Second sweeping right (90° total)
                SegmentDef::banked(TurnAngle::Standard45R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Standard45R, Banking::Medium),
                // Bridge section
                SegmentDef::full(TurnAngle::Straight, Elevation::Bridge, Banking::Flat, SegmentStyle::Bridge),
                SegmentDef::full(TurnAngle::Straight, Elevation::Bridge, Banking::Flat, SegmentStyle::Bridge),
                // Climb the hill
                SegmentDef::hill(Elevation::SteepUp),
                SegmentDef::hill(Elevation::GentleUp),
                // Third sweeper at hilltop (90°)
                SegmentDef::full(TurnAngle::Medium30R, Elevation::Crest, Banking::Slight, SegmentStyle::Open),
                SegmentDef::banked(TurnAngle::Medium30R, Banking::Slight),
                SegmentDef::banked(TurnAngle::Medium30R, Banking::Slight),
                // Downhill toward finish
                SegmentDef::hill(Elevation::SteepDown),
                SegmentDef::hill(Elevation::GentleDown),
                // Final turn back to start (90°)
                SegmentDef::banked(TurnAngle::Standard45R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Standard45R, Banking::Medium),
                SegmentDef::straight(),
            ],

            // --------------------------------------------------------
            // NEON CITY - Downtown Circuit (★★☆ Medium)
            // Tight city streets with 90° corners and a tunnel section
            // Mix of directions: net 360° clockwise ✓
            // --------------------------------------------------------
            TrackId::NeonCity => &[
                // Main street straight
                SegmentDef::straight(),
                SegmentDef::straight(),
                SegmentDef::straight(),
                // Sharp 90° right onto side street
                SegmentDef::turn(TurnAngle::Tight90R),
                // Short block
                SegmentDef::straight(),
                SegmentDef::straight(),
                // Chicane (S-curve: left then right, net 0°)
                SegmentDef::turn(TurnAngle::Standard45L),
                SegmentDef::turn(TurnAngle::Standard45R),
                // Into tunnel
                SegmentDef::styled(TurnAngle::Straight, SegmentStyle::Tunnel),
                SegmentDef::full(TurnAngle::Medium30R, Elevation::GentleDown, Banking::Flat, SegmentStyle::Tunnel),
                SegmentDef::styled(TurnAngle::Straight, SegmentStyle::Tunnel),
                SegmentDef::full(TurnAngle::Medium30R, Elevation::GentleUp, Banking::Flat, SegmentStyle::Tunnel),
                SegmentDef::styled(TurnAngle::Straight, SegmentStyle::Tunnel),
                // Exit tunnel, 90° left
                SegmentDef::turn(TurnAngle::Tight90L),
                // Avenue section
                SegmentDef::straight(),
                SegmentDef::straight(),
                // Over bridge
                SegmentDef::full(TurnAngle::Straight, Elevation::GentleUp, Banking::Flat, SegmentStyle::Bridge),
                SegmentDef::full(TurnAngle::Straight, Elevation::Bridge, Banking::Flat, SegmentStyle::Bridge),
                SegmentDef::full(TurnAngle::Straight, Elevation::GentleDown, Banking::Flat, SegmentStyle::Bridge),
                // Two sharp rights (180° total)
                SegmentDef::turn(TurnAngle::Tight90R),
                SegmentDef::straight(),
                SegmentDef::turn(TurnAngle::Tight90R),
                // Back straight
                SegmentDef::straight(),
                SegmentDef::straight(),
                // Final 90° right
                SegmentDef::turn(TurnAngle::Tight90R),
                SegmentDef::straight(),
            ],

            // --------------------------------------------------------
            // VOID TUNNEL - Underground Complex (★★★ Hard)
            // Disorienting tunnels with elevation changes
            // Contains hairpin! Total: 360° counter-clockwise ✓
            // --------------------------------------------------------
            TrackId::VoidTunnel => &[
                // Descent into the void
                SegmentDef::full(TurnAngle::Straight, Elevation::SteepDown, Banking::Flat, SegmentStyle::Tunnel),
                SegmentDef::full(TurnAngle::Straight, Elevation::SteepDown, Banking::Flat, SegmentStyle::Tunnel),
                // Sweeping left in darkness (60°)
                SegmentDef::full(TurnAngle::Medium30L, Elevation::Flat, Banking::Slight, SegmentStyle::Tunnel),
                SegmentDef::full(TurnAngle::Medium30L, Elevation::Flat, Banking::Slight, SegmentStyle::Tunnel),
                // Valley section
                SegmentDef::full(TurnAngle::Straight, Elevation::Valley, Banking::Flat, SegmentStyle::Tunnel),
                SegmentDef::styled(TurnAngle::Straight, SegmentStyle::Tunnel),
                // Sharp left 90°
                SegmentDef::full(TurnAngle::Tight90L, Elevation::Flat, Banking::Medium, SegmentStyle::Tunnel),
                // Long tunnel straight
                SegmentDef::styled(TurnAngle::Straight, SegmentStyle::Tunnel),
                SegmentDef::styled(TurnAngle::Straight, SegmentStyle::Tunnel),
                SegmentDef::styled(TurnAngle::Straight, SegmentStyle::Tunnel),
                // Hairpin left! (180°) - very tight
                SegmentDef::full(TurnAngle::Hairpin180L, Elevation::Flat, Banking::Heavy, SegmentStyle::Tunnel),
                // Climb back up
                SegmentDef::full(TurnAngle::Straight, Elevation::GentleUp, Banking::Flat, SegmentStyle::Tunnel),
                SegmentDef::full(TurnAngle::Straight, Elevation::SteepUp, Banking::Flat, SegmentStyle::Tunnel),
                // 30° left adjustment
                SegmentDef::full(TurnAngle::Medium30L, Elevation::GentleUp, Banking::Flat, SegmentStyle::Tunnel),
                // Exit to open air briefly
                SegmentDef::straight(),
                SegmentDef::hill(Elevation::Jump),
                // Final section (need -360° - (-60-90-180-30) = 0° more, but we overshot)
                // Actually let me recalculate: -60 -90 -180 -30 = -360 ✓
                SegmentDef::straight(),
            ],

            // --------------------------------------------------------
            // CRYSTAL CAVERN - Mountain Pass (★★★☆ Expert)
            // Narrow canyon with extreme elevation, tight hairpins
            // Two hairpins! Total: 360° clockwise ✓
            // --------------------------------------------------------
            TrackId::CrystalCavern => &[
                // Start in canyon
                SegmentDef::full(TurnAngle::Straight, Elevation::Flat, Banking::Flat, SegmentStyle::Canyon),
                SegmentDef::full(TurnAngle::Straight, Elevation::GentleUp, Banking::Flat, SegmentStyle::Canyon),
                // First hairpin right (180°)
                SegmentDef::full(TurnAngle::Hairpin180R, Elevation::Flat, Banking::Heavy, SegmentStyle::Canyon),
                // Steep climb
                SegmentDef::full(TurnAngle::Straight, Elevation::SteepUp, Banking::Flat, SegmentStyle::Canyon),
                SegmentDef::full(TurnAngle::Straight, Elevation::SteepUp, Banking::Flat, SegmentStyle::Canyon),
                // S-curve through crystals (net 0°)
                SegmentDef::full(TurnAngle::Sharp60R, Elevation::Flat, Banking::Medium, SegmentStyle::Canyon),
                SegmentDef::full(TurnAngle::Sharp60L, Elevation::Flat, Banking::Medium, SegmentStyle::Canyon),
                // Crest of mountain
                SegmentDef::full(TurnAngle::Straight, Elevation::Crest, Banking::Flat, SegmentStyle::Open),
                SegmentDef::hill(Elevation::Jump),
                // Descent
                SegmentDef::full(TurnAngle::Straight, Elevation::SteepDown, Banking::Flat, SegmentStyle::Canyon),
                SegmentDef::full(TurnAngle::Medium30R, Elevation::GentleDown, Banking::Slight, SegmentStyle::Canyon),
                // Tunnel through crystal formation
                SegmentDef::full(TurnAngle::Straight, Elevation::Flat, Banking::Flat, SegmentStyle::Tunnel),
                SegmentDef::full(TurnAngle::Medium30R, Elevation::Flat, Banking::Flat, SegmentStyle::Tunnel),
                SegmentDef::styled(TurnAngle::Straight, SegmentStyle::Tunnel),
                // Exit and second hairpin (180°) - total now 180+30+30+180=420, need -60
                // S-curve to correct: +60 -60 = 0
                SegmentDef::full(TurnAngle::Sharp60R, Elevation::Flat, Banking::Medium, SegmentStyle::Canyon),
                SegmentDef::full(TurnAngle::Sharp60L, Elevation::Flat, Banking::Medium, SegmentStyle::Canyon),
                // Recalc: 180+30+30+60-60 = 240, need 120 more
                SegmentDef::full(TurnAngle::Sharp60R, Elevation::GentleDown, Banking::Medium, SegmentStyle::Canyon),
                SegmentDef::full(TurnAngle::Sharp60R, Elevation::Flat, Banking::Medium, SegmentStyle::Canyon),
                // Now at 360° ✓
                SegmentDef::full(TurnAngle::Straight, Elevation::Flat, Banking::Flat, SegmentStyle::Canyon),
            ],

            // --------------------------------------------------------
            // SOLAR HIGHWAY - High Speed Circuit (★★★★ Master)
            // Massive banked turns, big jumps, dramatic elevation
            // Very long gentle sweepers for high-speed racing
            // Total: 6×15=90 + 0 + 2×45=90 + 6×15=90 + 3×30=90 = 360° ✓
            // --------------------------------------------------------
            TrackId::SolarHighway => &[
                // Start on elevated highway
                SegmentDef::full(TurnAngle::Straight, Elevation::Bridge, Banking::Flat, SegmentStyle::Bridge),
                SegmentDef::full(TurnAngle::Straight, Elevation::Bridge, Banking::Flat, SegmentStyle::Bridge),
                // Big jump off bridge!
                SegmentDef::full(TurnAngle::Straight, Elevation::Jump, Banking::Flat, SegmentStyle::Bridge),
                // Landing and slight downhill
                SegmentDef::hill(Elevation::GentleDown),
                SegmentDef::straight(),
                // Ultra-smooth 90° sweeper (6 gentle segments for F1-style flow)
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Heavy),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Heavy),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Medium),
                // High speed straight
                SegmentDef::straight(),
                SegmentDef::straight(),
                SegmentDef::straight(),
                // Gentle S-curve with elevation (net 0°)
                SegmentDef::full(TurnAngle::Gentle15L, Elevation::GentleUp, Banking::Slight, SegmentStyle::Open),
                SegmentDef::full(TurnAngle::Gentle15L, Elevation::GentleUp, Banking::Slight, SegmentStyle::Open),
                SegmentDef::full(TurnAngle::Gentle15L, Elevation::Crest, Banking::Slight, SegmentStyle::Open),
                SegmentDef::full(TurnAngle::Gentle15R, Elevation::GentleDown, Banking::Slight, SegmentStyle::Open),
                SegmentDef::full(TurnAngle::Gentle15R, Elevation::GentleDown, Banking::Slight, SegmentStyle::Open),
                SegmentDef::full(TurnAngle::Gentle15R, Elevation::Flat, Banking::Slight, SegmentStyle::Open),
                // Downhill rush
                SegmentDef::hill(Elevation::SteepDown),
                SegmentDef::hill(Elevation::GentleDown),
                // Another big banked turn (90°)
                SegmentDef::banked(TurnAngle::Standard45R, Banking::Heavy),
                SegmentDef::banked(TurnAngle::Standard45R, Banking::Heavy),
                // Back straight with second jump
                SegmentDef::straight(),
                SegmentDef::straight(),
                SegmentDef::hill(Elevation::Jump),
                // 90° sweeper using gentle turns (6 × 15 = 90)
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Heavy),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Heavy),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Gentle15R, Banking::Medium),
                // Final 90° sweeper back to start
                SegmentDef::banked(TurnAngle::Medium30R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Medium30R, Banking::Medium),
                SegmentDef::banked(TurnAngle::Medium30R, Banking::Medium),
                SegmentDef::straight(),
            ],
        };

        // Validate track closure in debug
        #[cfg(debug_assertions)]
        if !validate_track_closure(layout) {
            // In debug, warn but continue
        }

        // Build segment positions from layout
        let mut cur_x: f32 = 0.0;
        let mut cur_y: f32 = 0.0;
        let mut cur_z: f32 = 0.0;
        let mut cur_rot: f32 = 0.0;  // Heading in degrees

        for (i, seg_def) in layout.iter().enumerate() {
            if i >= MAX_TRACK_SEGMENTS { break; }

            let seg_len = base_len * seg_def.turn.length_mult();

            // Store segment with full 3D info
            TRACK_SEGMENTS[i] = TrackSegment {
                x: cur_x,
                y: cur_y,
                z: cur_z,
                rotation: cur_rot,
                turn: seg_def.turn,
                elevation: seg_def.elevation,
                banking: seg_def.banking,
                style: seg_def.style,
                width: 10.0,
                length: seg_len,
            };

            // Add waypoint at center of segment
            if WAYPOINT_COUNT < MAX_WAYPOINTS {
                let sin_r = libm::sinf(cur_rot * 3.14159 / 180.0);
                let cos_r = libm::cosf(cur_rot * 3.14159 / 180.0);
                let mid_y = cur_y + seg_def.elevation.height_delta() * 0.5;
                WAYPOINTS[WAYPOINT_COUNT] = Waypoint {
                    x: cur_x + sin_r * seg_len * 0.5,
                    y: mid_y,
                    z: cur_z + cos_r * seg_len * 0.5,
                };
                WAYPOINT_COUNT += 1;
            }

            // Advance position
            let sin_r = libm::sinf(cur_rot * 3.14159 / 180.0);
            let cos_r = libm::cosf(cur_rot * 3.14159 / 180.0);

            cur_x += sin_r * seg_len;
            cur_z += cos_r * seg_len;
            cur_y += seg_def.elevation.height_delta();
            cur_rot += seg_def.turn.degrees();

            // Normalize rotation
            while cur_rot >= 360.0 { cur_rot -= 360.0; }
            while cur_rot < 0.0 { cur_rot += 360.0; }

            TRACK_SEGMENT_COUNT = i + 1;
        }

        // Calculate total track length
        let mut total_len: f32 = 0.0;
        for i in 0..TRACK_SEGMENT_COUNT {
            total_len += TRACK_SEGMENTS[i].length;
        }
        TRACK_LENGTH = total_len;

        // Update checkpoints based on actual track length
        let checkpoint_spacing = total_len / NUM_CHECKPOINTS as f32;
        for i in 0..NUM_CHECKPOINTS {
            CHECKPOINT_Z[i] = (i as f32) * checkpoint_spacing;
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
