//! Lumina Depths sound definitions

/// Sound ID and description
pub type SoundDef = (&'static str, &'static str);

/// All Lumina Depths sounds
pub const SOUNDS: &[SoundDef] = &[
    // === SUBMERSIBLE ===
    ("sonar", "Sonar ping with echo"),
    ("propeller", "Propeller hum loop"),
    ("surface", "Surfacing sound"),
    ("hull_creak", "Hull stress creaking"),
    ("pressure_warning", "Depth pressure warning"),
    ("headlight_on", "Headlight power on"),
    ("headlight_off", "Headlight power off"),

    // === ZONE AMBIENTS ===
    ("ambient_sunlit", "Sunlit zone ambient - bright, active"),
    ("ambient_twilight", "Twilight zone ambient - mysterious, distant"),
    ("ambient_midnight", "Midnight zone ambient - deep, ominous"),
    ("ambient_vents", "Vent zone ambient - rumbling, hissing"),

    // === CREATURE SOUNDS ===
    ("whale", "Whale song call"),
    ("whale_echo", "Distant whale echo"),
    ("fish", "Fish school movement"),
    ("jellyfish", "Jellyfish pulse"),
    ("squid", "Squid jet propulsion"),
    ("anglerfish_lure", "Anglerfish bioluminescent pulse"),
    ("crab_click", "Crab claw clicking"),
    ("shrimp_snap", "Shrimp snap sound"),
    ("octopus_move", "Octopus movement swoosh"),
    ("eel_hiss", "Gulper eel hiss"),
    ("isopod_scuttle", "Isopod leg scuttling"),

    // === ENVIRONMENT ===
    ("bubbles", "Air bubbles rising"),
    ("bubbles_small", "Small bubble trail"),
    ("vent", "Thermal vent rumble"),
    ("vent_hiss", "Vent steam hiss"),
    ("cave", "Cave water drip"),
    ("current", "Ocean current flow"),
    ("sediment", "Seafloor sediment disturbance"),

    // === DISCOVERY & UI ===
    ("artifact", "Artifact discovered"),
    ("scan", "Object scan complete"),
    ("log", "Journal entry logged"),
    ("discovery", "New species discovered fanfare"),
    ("zone_enter", "Zone transition chime"),
    ("depth_milestone", "Depth milestone reached"),

    // === ENCOUNTERS ===
    ("encounter_start", "Epic encounter begins"),
    ("encounter_end", "Epic encounter ends"),
    ("danger_near", "Danger proximity warning"),
];
