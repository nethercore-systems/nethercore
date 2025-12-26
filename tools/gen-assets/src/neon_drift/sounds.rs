//! Neon Drift sound definitions

/// Sound ID and description
pub type SoundDef = (&'static str, &'static str);

/// All Neon Drift sounds
pub const SOUNDS: &[SoundDef] = &[
    // Engine
    ("engine_idle", "Engine idle loop"),
    ("engine_rev", "Engine revving"),
    ("boost", "Nitro boost"),

    // Driving
    ("drift", "Tire drift/screech"),
    ("brake", "Hard brake"),
    ("shift", "Gear shift"),

    // Collisions
    ("wall", "Wall collision"),
    ("barrier", "Barrier crash"),

    // Race
    ("countdown", "Race countdown beep"),
    ("checkpoint", "Checkpoint passed"),
    ("finish", "Race finish fanfare"),
];
