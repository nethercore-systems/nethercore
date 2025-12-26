//! Prism Survivors sound definitions

/// Sound ID and description
pub type SoundDef = (&'static str, &'static str);

/// All Prism Survivors sounds
pub const SOUNDS: &[SoundDef] = &[
    // Combat
    ("shoot", "Player weapon fire"),
    ("hit", "Enemy hit impact"),
    ("death", "Enemy death"),

    // Player
    ("dash", "Player dash/dodge"),
    ("level_up", "Player level up"),
    ("hurt", "Player takes damage"),

    // Pickups
    ("xp", "XP orb pickup"),
    ("coin", "Coin pickup"),
    ("powerup", "Powerup collected"),

    // UI
    ("menu", "Menu open"),
    ("select", "Menu selection"),
    ("back", "Menu back"),
];
