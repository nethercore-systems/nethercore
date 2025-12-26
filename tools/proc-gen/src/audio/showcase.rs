//! Showcase sound generation
//!
//! Re-exports showcase definitions and provides generation functions.
//! The definitions come from `proc-gen-showcase-defs` crate (single source of truth).

use super::Synth;

// Re-export showcase definitions
pub use proc_gen_showcase_defs::{ShowcaseSound, SHOWCASE_SOUNDS};

/// Generate a showcase sound by ID using the Synth API
///
/// **TO ADD A NEW SOUND:**
/// 1. Add the definition to `examples/proc-gen-showcase-defs/src/lib.rs`
/// 2. Add a match arm here with your Synth API call
pub fn generate_showcase_sound(synth: &Synth, id: &str) -> Vec<f32> {
    match id {
        "coin" => synth.coin(),
        "jump" => synth.jump(),
        "laser" => synth.laser(),
        "explosion" => synth.explosion(),
        "hit" => synth.hit(),
        "click" => synth.click(),
        "powerup" => synth.powerup(),
        "death" => synth.death(),
        _ => panic!("Unknown showcase sound ID: {}", id),
    }
}
