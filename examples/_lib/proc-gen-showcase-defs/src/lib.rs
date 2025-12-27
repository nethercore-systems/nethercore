//! Showcase sound definitions
//!
//! This is the **SINGLE SOURCE OF TRUTH** for showcase sounds.
//! To add a new showcase sound, just add an entry to the SHOWCASE_SOUNDS array!
//!
//! Both the generator tool and the viewer automatically use these definitions.

#![no_std]

/// Showcase sound definition
#[derive(Clone, Copy)]
pub struct ShowcaseSound {
    /// ID used in ROM and filename
    pub id: &'static str,
    /// Display name in the viewer
    pub name: &'static str,
    /// Description shown in the viewer
    pub description: &'static str,
}

/// **ADD NEW SHOWCASE SOUNDS HERE** - This is the ONLY place to edit!
///
/// To add a new sound:
/// 1. Add an entry here with id, name, and description
/// 2. Add the generator function to `tools/proc-gen/src/audio/showcase.rs`
/// 3. Done! Everything else updates automatically.
pub const SHOWCASE_SOUNDS: &[ShowcaseSound] = &[
    ShowcaseSound {
        id: "coin",
        name: "Coin",
        description: "Quick pickup sound",
    },
    ShowcaseSound {
        id: "jump",
        name: "Jump",
        description: "Player jump",
    },
    ShowcaseSound {
        id: "laser",
        name: "Laser",
        description: "Shoot/zap effect",
    },
    ShowcaseSound {
        id: "explosion",
        name: "Explosion",
        description: "Big boom",
    },
    ShowcaseSound {
        id: "hit",
        name: "Hit",
        description: "Damage/impact",
    },
    ShowcaseSound {
        id: "click",
        name: "Click",
        description: "UI interaction",
    },
    ShowcaseSound {
        id: "powerup",
        name: "Power-Up",
        description: "Rising arpeggio",
    },
    ShowcaseSound {
        id: "death",
        name: "Death",
        description: "Descending arpeggio",
    },
];
