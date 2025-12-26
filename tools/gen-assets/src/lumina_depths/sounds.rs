//! Lumina Depths sound definitions

/// Sound ID and description
pub type SoundDef = (&'static str, &'static str);

/// All Lumina Depths sounds
pub const SOUNDS: &[SoundDef] = &[
    // Submersible
    ("sonar", "Sonar ping"),
    ("propeller", "Propeller hum"),
    ("surface", "Surfacing sound"),

    // Creatures
    ("whale", "Whale call"),
    ("fish", "Fish school movement"),
    ("jellyfish", "Jellyfish pulse"),

    // Environment
    ("bubbles", "Air bubbles rising"),
    ("vent", "Thermal vent rumble"),
    ("cave", "Cave water drip"),

    // Discovery
    ("artifact", "Artifact discovered"),
    ("scan", "Object scan complete"),
    ("log", "Journal entry logged"),
];
