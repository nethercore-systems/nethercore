//! Physics and collision for NEON DRIFT
//!
//! Handles car physics, camera following, and collision detection.

use crate::ffi::*;
use crate::types::*;
use crate::state::*;
use crate::particles::{spawn_collision_sparks, spawn_boost_flames, spawn_drift_smoke};

pub fn update_car_physics(car: &mut Car, player_idx: u32, dt: f32) {
    unsafe {
        let gas = trigger_right(player_idx);
        let brake = trigger_left(player_idx);
        let steer_x = left_stick_x(player_idx) * -1.0;
        let boost_pressed = button_pressed(player_idx, BUTTON_A);

        // Acceleration/braking
        let accel_input = gas - (brake * 0.7);

        if accel_input > 0.01 {
            car.velocity_forward += car.acceleration * accel_input * dt;
        } else if accel_input < -0.01 {
            car.velocity_forward += car.acceleration * accel_input * 2.0 * dt;
        } else {
            car.velocity_forward *= 0.98;
        }

        // Boost activation
        if boost_pressed != 0 && car.boost_meter >= BOOST_COST && !car.is_boosting {
            car.is_boosting = true;
            car.boost_timer = BOOST_DURATION;
            car.boost_meter -= BOOST_COST;
            play_sound(SND_BOOST, 1.0, 0.0);
        }

        // Update boost timer
        if car.boost_timer > 0 {
            car.boost_timer -= 1;
            if car.boost_timer == 0 {
                car.is_boosting = false;
            }
        }

        // Apply boost multiplier
        let boost_mult = if car.is_boosting { BOOST_MULTIPLIER } else { 1.0 };
        let max_speed = car.max_speed * boost_mult;
        car.velocity_forward = car.velocity_forward.clamp(-max_speed * 0.5, max_speed);

        // Drift detection and physics
        let speed_factor = (car.velocity_forward.abs() / car.max_speed).min(1.0);

        if brake > DRIFT_THRESHOLD && steer_x.abs() > DRIFT_THRESHOLD && speed_factor > 0.4 {
            if !car.is_drifting {
                car.is_drifting = true;
                play_sound(SND_DRIFT, 0.7, 0.0);
            }

            let drift_power = steer_x * car.drift_factor;
            car.velocity_lateral += drift_power * 15.0 * dt;
            car.angular_velocity = drift_power * 120.0;
            car.velocity_forward *= 0.97;
            car.boost_meter = (car.boost_meter + 0.015).min(1.0);
        } else {
            car.is_drifting = false;
            let turn_speed = car.handling * 90.0 * speed_factor;
            car.angular_velocity = steer_x * turn_speed;
            car.velocity_lateral *= 0.85;
        }

        // Update rotation
        car.rotation_y += car.angular_velocity * dt;

        // Update position
        // Forward is -Z when rotation_y = 0 (into screen)
        let sin_rot = libm::sinf(car.rotation_y * 3.14159 / 180.0);
        let cos_rot = libm::cosf(car.rotation_y * 3.14159 / 180.0);
        let forward_x = -sin_rot;
        let forward_z = -cos_rot;
        let right_x = cos_rot;
        let right_z = -sin_rot;

        car.x += (forward_x * car.velocity_forward + right_x * car.velocity_lateral) * dt;
        car.z += (forward_z * car.velocity_forward + right_z * car.velocity_lateral) * dt;

        // Apply collision pushback
        car.x += car.collision_pushback_x;
        car.z += car.collision_pushback_z;
        car.collision_pushback_x *= 0.5;
        car.collision_pushback_z *= 0.5;
    }
}

pub fn update_camera(camera: &mut Camera, car: &Car, _dt: f32) {
    let sin_rot = libm::sinf(car.rotation_y * 3.14159 / 180.0);
    let cos_rot = libm::cosf(car.rotation_y * 3.14159 / 180.0);

    let offset_distance = 8.0;
    let offset_height = 3.0;

    let desired_pos_x = car.x - sin_rot * offset_distance;
    let desired_pos_y = car.y + offset_height;
    let desired_pos_z = car.z - cos_rot * offset_distance;

    let desired_target_x = car.x + sin_rot * 5.0;
    let desired_target_y = car.y + 1.0;
    let desired_target_z = car.z + cos_rot * 5.0;

    let lerp = 0.1;
    camera.current_pos_x += (desired_pos_x - camera.current_pos_x) * lerp;
    camera.current_pos_y += (desired_pos_y - camera.current_pos_y) * lerp;
    camera.current_pos_z += (desired_pos_z - camera.current_pos_z) * lerp;

    camera.current_target_x += (desired_target_x - camera.current_target_x) * lerp;
    camera.current_target_y += (desired_target_y - camera.current_target_y) * lerp;
    camera.current_target_z += (desired_target_z - camera.current_target_z) * lerp;
}

pub fn check_track_collision_with_effects(car: &mut Car, player_idx: usize) {
    let track_half_width = 5.0;
    let had_collision = car.x.abs() > track_half_width;

    if had_collision {
        unsafe {
            CAMERAS[player_idx].add_shake(0.3);
            play_sound(SND_WALL, 0.8, 0.0);
            spawn_collision_sparks(car.x, car.z);

            let pushback = if car.x > 0.0 { -0.5 } else { 0.5 };
            car.collision_pushback_x += pushback;
            car.velocity_forward *= 0.7;
        }
    }

    check_track_collision(car);
}

pub fn check_track_collision(car: &mut Car) {
    unsafe {
        // Find the nearest track segment
        let mut min_dist = f32::MAX;
        let mut nearest_seg_idx = 0;

        for i in 0..TRACK_SEGMENT_COUNT {
            let seg = &TRACK_SEGMENTS[i];
            let sin_r = libm::sinf(seg.rotation * 3.14159 / 180.0);
            let cos_r = libm::cosf(seg.rotation * 3.14159 / 180.0);

            // Center of segment
            let seg_cx = seg.x + sin_r * 5.0;
            let seg_cz = seg.z + cos_r * 5.0;

            let dx = car.x - seg_cx;
            let dz = car.z - seg_cz;
            let dist = dx * dx + dz * dz;

            if dist < min_dist {
                min_dist = dist;
                nearest_seg_idx = i;
            }
        }

        // Get the nearest segment for local collision and elevation
        if TRACK_SEGMENT_COUNT > 0 {
            let seg = &TRACK_SEGMENTS[nearest_seg_idx];
            let sin_r = libm::sinf(seg.rotation * 3.14159 / 180.0);
            let cos_r = libm::cosf(seg.rotation * 3.14159 / 180.0);

            // Transform car position to segment-local coordinates
            let rel_x = car.x - seg.x;
            let rel_z = car.z - seg.z;

            // Rotate to segment-local space
            let local_x = rel_x * cos_r - rel_z * sin_r;
            let local_z = rel_x * sin_r + rel_z * cos_r;

            // Calculate progress through segment (0 to 1)
            let progress = (local_z / seg.length).clamp(0.0, 1.0);

            // Update car height based on segment elevation
            // Interpolate from segment start to end height
            let target_y = seg.y + seg.elevation.height_delta() * progress;
            car.y = car.y + (target_y - car.y) * 0.15; // Smooth transition

            // Clamp to track bounds in local space
            let track_half_width = seg.width * 0.5;
            let clamped = local_x.clamp(-track_half_width, track_half_width);

            if local_x != clamped {
                // Transform back to world space
                let new_local_x = clamped;
                let new_x = new_local_x * cos_r + local_z * sin_r + seg.x;
                let new_z = -new_local_x * sin_r + local_z * cos_r + seg.z;
                car.x = new_x;
                car.z = new_z;
                car.velocity_lateral *= -0.3;
            }
        }
    }
}

pub fn check_checkpoints(car: &mut Car, _car_idx: usize) {
    unsafe {
        let next_checkpoint = (car.last_checkpoint + 1) % NUM_CHECKPOINTS;
        let checkpoint_z = CHECKPOINT_Z[next_checkpoint];
        let threshold = 5.0;

        if (car.z - checkpoint_z).abs() < threshold {
            car.last_checkpoint = next_checkpoint;
            play_sound(SND_CHECKPOINT, 0.6, 0.0);
        }
    }
}

pub fn update_ai_car(car: &mut Car, dt: f32) {
    unsafe {
        // Get target waypoint
        if WAYPOINT_COUNT == 0 {
            // Fallback to simple forward movement
            car.velocity_forward += car.acceleration * 0.8 * dt;
            car.velocity_forward = car.velocity_forward.min(car.max_speed * 0.9);

            // Forward is -Z when rotation_y = 0
            let sin_rot = libm::sinf(car.rotation_y * 3.14159 / 180.0);
            let cos_rot = libm::cosf(car.rotation_y * 3.14159 / 180.0);
            car.x -= sin_rot * car.velocity_forward * dt;
            car.z -= cos_rot * car.velocity_forward * dt;
            check_track_collision(car);
            return;
        }

        let target = WAYPOINTS[car.current_waypoint];

        // Calculate direction to waypoint (using 3D distance)
        let dx = target.x - car.x;
        let dy = target.y - car.y;
        let dz = target.z - car.z;
        let dist = libm::sqrtf(dx * dx + dz * dz); // 2D distance for reach check
        let _ = dy; // Y difference used for camera, not steering

        // Check if we've reached the waypoint
        if dist < 8.0 {
            car.current_waypoint = (car.current_waypoint + 1) % WAYPOINT_COUNT;

            // Check for lap completion
            if car.current_waypoint == 0 {
                car.current_lap += 1;
            }
        }

        // Calculate target angle (angle to waypoint)
        // Use -dx, -dz because forward is -Z when rotation_y = 0
        let target_angle = libm::atan2f(-dx, -dz) * 180.0 / 3.14159;

        // Normalize angles
        let mut angle_diff = target_angle - car.rotation_y;
        while angle_diff > 180.0 { angle_diff -= 360.0; }
        while angle_diff < -180.0 { angle_diff += 360.0; }

        // Steer toward waypoint
        let steer = (angle_diff / 45.0).clamp(-1.0, 1.0);

        // Accelerate
        car.velocity_forward += car.acceleration * 0.85 * dt;

        // Slow down for sharp turns
        let turn_factor = 1.0 - (steer.abs() * 0.3);
        car.velocity_forward = car.velocity_forward.min(car.max_speed * 0.9 * turn_factor);

        // Apply steering
        let speed_factor = (car.velocity_forward.abs() / car.max_speed).min(1.0);
        let turn_speed = car.handling * 120.0 * speed_factor;
        car.angular_velocity = steer * turn_speed;

        car.rotation_y += car.angular_velocity * dt;

        // Normalize rotation
        while car.rotation_y > 360.0 { car.rotation_y -= 360.0; }
        while car.rotation_y < 0.0 { car.rotation_y += 360.0; }

        // Update position (forward is -Z when rotation_y = 0)
        let sin_rot = libm::sinf(car.rotation_y * 3.14159 / 180.0);
        let cos_rot = libm::cosf(car.rotation_y * 3.14159 / 180.0);
        car.x -= sin_rot * car.velocity_forward * dt;
        car.z -= cos_rot * car.velocity_forward * dt;

        check_track_collision(car);
    }
}

pub fn calculate_positions() {
    unsafe {
        let mut progress: [(f32, usize); 4] = [(0.0, 0), (0.0, 1), (0.0, 2), (0.0, 3)];

        for i in 0..4 {
            let car = &CARS[i];
            let cp_progress = (car.last_checkpoint as f32) * (TRACK_LENGTH / NUM_CHECKPOINTS as f32);
            progress[i] = (
                (car.current_lap as f32) * TRACK_LENGTH + cp_progress + car.z,
                i
            );
        }

        // Sort by progress (descending)
        for i in 0..4 {
            for j in i+1..4 {
                if progress[j].0 > progress[i].0 {
                    let tmp = progress[i];
                    progress[i] = progress[j];
                    progress[j] = tmp;
                }
            }
        }

        // Assign positions
        for pos in 0..4 {
            CARS[progress[pos].1].race_position = (pos + 1) as u32;
        }
    }
}

pub fn spawn_car_particles(car: &Car) {
    if car.is_boosting {
        spawn_boost_flames(car);
    }
    if car.is_drifting {
        spawn_drift_smoke(car);
    }
}
