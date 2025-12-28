//! Particle system for NEON DRIFT
//!
//! Handles boost flames, drift smoke, collision sparks, and speed lines.

use crate::ffi::*;
use crate::types::*;
use crate::state::*;

pub fn spawn_particle(
    x: f32, y: f32, z: f32,
    vel_x: f32, vel_y: f32, vel_z: f32,
    life: f32, size: f32, color: u32, ptype: ParticleType
) {
    unsafe {
        let p = &mut PARTICLES[NEXT_PARTICLE];
        p.active = true;
        p.x = x;
        p.y = y;
        p.z = z;
        p.vel_x = vel_x;
        p.vel_y = vel_y;
        p.vel_z = vel_z;
        p.life = life;
        p.max_life = life;
        p.size = size;
        p.color = color;
        p.particle_type = ptype;
        NEXT_PARTICLE = (NEXT_PARTICLE + 1) % MAX_PARTICLES;
    }
}

pub fn spawn_boost_flames(car: &Car) {
    let angle = car.rotation_y;
    let sin_a = libm::sinf(angle);
    let cos_a = libm::cosf(angle);

    let exhaust_x = car.x - sin_a * 0.6;
    let exhaust_z = car.z - cos_a * 0.6;

    unsafe {
        let rand = random();
        let spread = ((rand & 0xFF) as f32 / 255.0 - 0.5) * 0.2;
        let vel_back = -car.velocity_forward * 0.3;

        // Cyan core flame
        spawn_particle(
            exhaust_x + spread * cos_a,
            0.15,
            exhaust_z + spread * sin_a,
            -sin_a * vel_back + spread,
            0.5 + (rand >> 16) as f32 / 65536.0 * 0.3,
            -cos_a * vel_back,
            0.3, 0.15, 0x00FFFFFF, ParticleType::BoostFlame
        );

        // Orange outer flame
        spawn_particle(
            exhaust_x + spread * cos_a * 1.2,
            0.12,
            exhaust_z + spread * sin_a * 1.2,
            -sin_a * vel_back * 0.8,
            0.3,
            -cos_a * vel_back * 0.8,
            0.4, 0.2, 0xFF6600FF, ParticleType::BoostFlame
        );
    }
}

pub fn spawn_drift_smoke(car: &Car) {
    let angle = car.rotation_y;
    let sin_a = libm::sinf(angle);
    let cos_a = libm::cosf(angle);

    unsafe {
        let rand = random();

        // Left wheel smoke
        let left_x = car.x - sin_a * 0.3 + cos_a * 0.3;
        let left_z = car.z - cos_a * 0.3 - sin_a * 0.3;
        spawn_particle(
            left_x, 0.05, left_z,
            ((rand & 0xFF) as f32 / 255.0 - 0.5) * 0.5,
            0.2,
            ((rand >> 8 & 0xFF) as f32 / 255.0 - 0.5) * 0.5,
            0.6, 0.25, 0x888888AA, ParticleType::DriftSmoke
        );

        // Right wheel smoke
        let right_x = car.x - sin_a * 0.3 - cos_a * 0.3;
        let right_z = car.z - cos_a * 0.3 + sin_a * 0.3;
        spawn_particle(
            right_x, 0.05, right_z,
            ((rand >> 16 & 0xFF) as f32 / 255.0 - 0.5) * 0.5,
            0.2,
            ((rand >> 24 & 0xFF) as f32 / 255.0 - 0.5) * 0.5,
            0.6, 0.25, 0x888888AA, ParticleType::DriftSmoke
        );
    }
}

pub fn spawn_collision_sparks(x: f32, z: f32) {
    unsafe {
        for _ in 0..8 {
            let rand = random();
            let vx = ((rand & 0xFF) as f32 / 128.0 - 1.0) * 3.0;
            let vy = ((rand >> 8 & 0xFF) as f32 / 255.0) * 2.0 + 1.0;
            let vz = ((rand >> 16 & 0xFF) as f32 / 128.0 - 1.0) * 3.0;

            spawn_particle(
                x, 0.2, z,
                vx, vy, vz,
                0.3, 0.05, 0xFFFF00FF, ParticleType::Spark
            );
        }
    }
}

pub fn update_particles(dt: f32) {
    unsafe {
        for i in 0..MAX_PARTICLES {
            let p = &mut PARTICLES[i];
            if !p.active { continue; }

            p.x += p.vel_x * dt;
            p.y += p.vel_y * dt;
            p.z += p.vel_z * dt;

            match p.particle_type {
                ParticleType::BoostFlame => {
                    p.vel_y -= 2.0 * dt;
                    p.size *= 0.95;
                }
                ParticleType::DriftSmoke => {
                    p.vel_y += 0.5 * dt;
                    p.size *= 1.02;
                    p.vel_x *= 0.95;
                    p.vel_z *= 0.95;
                }
                ParticleType::Spark => {
                    p.vel_y -= 15.0 * dt;
                }
                ParticleType::SpeedLine => {
                    // Just streak backward
                }
            }

            p.life -= dt;
            if p.life <= 0.0 || p.y < -0.5 {
                p.active = false;
            }
        }
    }
}

pub fn render_particles() {
    unsafe {
        for i in 0..MAX_PARTICLES {
            let p = &PARTICLES[i];
            if !p.active { continue; }

            let alpha = ((p.life / p.max_life) * 255.0) as u32;
            let color = (p.color & 0xFFFFFF00) | alpha;

            push_identity();
            push_translate(p.x, p.y, p.z);
            draw_billboard(p.size, p.size, 1, color);  // mode 1 = spherical billboard
        }
    }
}

pub fn render_speed_lines(player_id: usize, vp_width: u32, vp_height: u32) {
    unsafe {
        let intensity = SPEED_LINE_INTENSITY[player_id];
        if intensity < 0.1 { return; }

        let cx = vp_width as f32 / 2.0;
        let cy = vp_height as f32 / 2.0;
        let alpha = (intensity * 180.0) as u32;
        let color = 0xFFFFFF00 | alpha;

        for i in 0..12 {
            let angle = (i as f32 / 12.0) * 6.28318;
            let sin_a = libm::sinf(angle);
            let cos_a = libm::cosf(angle);

            let inner_r = 50.0 + (1.0 - intensity) * 100.0;
            let outer_r = inner_r + intensity * 200.0;

            let x1 = cx + cos_a * inner_r;
            let y1 = cy + sin_a * inner_r;
            let x2 = cx + cos_a * outer_r;
            let y2 = cy + sin_a * outer_r;

            draw_line(x1, y1, x2, y2, 2.0 + intensity * 3.0, color);
        }
    }
}

pub fn render_vignette(intensity: f32) {
    unsafe {
        if intensity < 0.01 { return; }

        let alpha = (intensity * 100.0) as u32;
        let color = alpha;

        draw_rect(0.0, 0.0, 200.0, 100.0, color);
        draw_rect(SCREEN_WIDTH as f32 - 200.0, 0.0, 200.0, 100.0, color);
        draw_rect(0.0, SCREEN_HEIGHT as f32 - 100.0, 200.0, 100.0, color);
        draw_rect(SCREEN_WIDTH as f32 - 200.0, SCREEN_HEIGHT as f32 - 100.0, 200.0, 100.0, color);
    }
}
