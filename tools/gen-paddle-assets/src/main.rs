//! Paddle Game Asset Generator
//!
//! Generates assets for the paddle (pong) example game:
//! - ball.png (white circle)
//! - paddle.png (white rectangle)
//! - hit.wav (ball hit sound)
//! - score.wav (score sound)
//! - win.wav (victory fanfare)

use std::f32::consts::PI;
use std::fs;
use std::path::Path;

const SAMPLE_RATE: f32 = 22050.0;

fn main() {
    // Output to paddle's local assets folder
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("7-games")
        .join("paddle")
        .join("assets");

    fs::create_dir_all(&output_dir).expect("Failed to create assets directory");

    println!("Generating paddle assets...");

    // Generate textures
    generate_ball_png(&output_dir.join("ball.png"));
    println!("  Generated ball.png");

    generate_paddle_png(&output_dir.join("paddle.png"));
    println!("  Generated paddle.png");

    // Generate sounds
    generate_hit_wav(&output_dir.join("hit.wav"));
    println!("  Generated hit.wav");

    generate_score_wav(&output_dir.join("score.wav"));
    println!("  Generated score.wav");

    generate_win_wav(&output_dir.join("win.wav"));
    println!("  Generated win.wav");

    println!("Done!");
}

/// Generate a white circle for the ball (16x16)
fn generate_ball_png(path: &Path) {
    let size = 16u32;
    let center = size as f32 / 2.0;
    let radius = center - 1.0;

    let mut pixels = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            let idx = ((y * size + x) * 4) as usize;

            if dist <= radius {
                // White with slight anti-aliasing at edges
                let alpha = if dist > radius - 1.0 {
                    ((radius - dist + 1.0) * 255.0) as u8
                } else {
                    255
                };
                pixels[idx] = 255; // R
                pixels[idx + 1] = 255; // G
                pixels[idx + 2] = 255; // B
                pixels[idx + 3] = alpha; // A
            } else {
                // Transparent
                pixels[idx] = 0;
                pixels[idx + 1] = 0;
                pixels[idx + 2] = 0;
                pixels[idx + 3] = 0;
            }
        }
    }

    image::save_buffer(path, &pixels, size, size, image::ColorType::Rgba8)
        .expect("Failed to save ball.png");
}

/// Generate a white rectangle for the paddle (8x32)
fn generate_paddle_png(path: &Path) {
    let width = 8u32;
    let height = 32u32;

    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;

            // Slight rounded corners
            let corner_radius = 2.0f32;
            let dx = if x < corner_radius as u32 {
                corner_radius - x as f32 - 0.5
            } else if x >= width - corner_radius as u32 {
                x as f32 + 0.5 - (width as f32 - corner_radius)
            } else {
                0.0
            };
            let dy = if y < corner_radius as u32 {
                corner_radius - y as f32 - 0.5
            } else if y >= height - corner_radius as u32 {
                y as f32 + 0.5 - (height as f32 - corner_radius)
            } else {
                0.0
            };

            let corner_dist = (dx * dx + dy * dy).sqrt();

            if corner_dist <= corner_radius || (dx == 0.0 || dy == 0.0) {
                pixels[idx] = 255; // R
                pixels[idx + 1] = 255; // G
                pixels[idx + 2] = 255; // B
                pixels[idx + 3] = 255; // A
            } else {
                pixels[idx] = 0;
                pixels[idx + 1] = 0;
                pixels[idx + 2] = 0;
                pixels[idx + 3] = 0;
            }
        }
    }

    image::save_buffer(path, &pixels, width, height, image::ColorType::Rgba8)
        .expect("Failed to save paddle.png");
}

/// Generate a short "plink" sound for ball hit
fn generate_hit_wav(path: &Path) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("Failed to create hit.wav");

    let duration = 0.08; // 80ms
    let samples = (SAMPLE_RATE * duration) as usize;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let progress = i as f32 / samples as f32;

        // High-pitched ping with fast decay
        let freq = 880.0; // A5
        let envelope = (-progress * 20.0).exp();
        let sample = (2.0 * PI * freq * t).sin() * envelope;

        // Add a bit of higher harmonic for brightness
        let harmonic = (2.0 * PI * freq * 2.0 * t).sin() * envelope * 0.3;

        let final_sample = ((sample + harmonic) * 20000.0) as i16;
        writer.write_sample(final_sample).unwrap();
    }
    writer.finalize().unwrap();
}

/// Generate a rising "blip" sound for scoring
fn generate_score_wav(path: &Path) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("Failed to create score.wav");

    let duration = 0.15; // 150ms
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut phase = 0.0f32;

    for i in 0..samples {
        let progress = i as f32 / samples as f32;

        // Rising pitch from 440Hz to 880Hz
        let freq = 440.0 + 440.0 * progress;
        let envelope = (1.0 - progress).powf(0.5); // Slow decay

        phase += freq / SAMPLE_RATE;
        let sample = (2.0 * PI * phase).sin() * envelope;

        let final_sample = (sample * 22000.0) as i16;
        writer.write_sample(final_sample).unwrap();
    }
    writer.finalize().unwrap();
}

/// Generate a short victory fanfare
fn generate_win_wav(path: &Path) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("Failed to create win.wav");

    // Three ascending notes: C5, E5, G5 (major chord arpeggio)
    let notes = [523.25, 659.25, 783.99]; // C5, E5, G5
    let note_duration = 0.12;
    let gap = 0.02;

    for (note_idx, &freq) in notes.iter().enumerate() {
        let samples = (SAMPLE_RATE * note_duration) as usize;

        for i in 0..samples {
            let t = i as f32 / SAMPLE_RATE;
            let progress = i as f32 / samples as f32;

            // Bell-like envelope
            let attack = (progress * 10.0).min(1.0);
            let decay = (-progress * 5.0).exp();
            let envelope = attack * decay;

            let sample = (2.0 * PI * freq * t).sin() * envelope;
            // Add octave for brightness
            let octave = (2.0 * PI * freq * 2.0 * t).sin() * envelope * 0.2;

            let final_sample = ((sample + octave) * 18000.0) as i16;
            writer.write_sample(final_sample).unwrap();
        }

        // Write gap silence
        let gap_samples = (SAMPLE_RATE * gap) as usize;
        for _ in 0..gap_samples {
            writer.write_sample(0i16).unwrap();
        }
    }

    // Final sustained chord
    let chord_duration = 0.3;
    let chord_samples = (SAMPLE_RATE * chord_duration) as usize;
    for i in 0..chord_samples {
        let t = i as f32 / SAMPLE_RATE;
        let progress = i as f32 / chord_samples as f32;

        let envelope = (1.0 - progress).powf(0.3);

        let mut sample = 0.0f32;
        for &freq in &notes {
            sample += (2.0 * PI * freq * t).sin();
        }
        sample /= 3.0;

        let final_sample = (sample * envelope * 20000.0) as i16;
        writer.write_sample(final_sample).unwrap();
    }

    writer.finalize().unwrap();
}
