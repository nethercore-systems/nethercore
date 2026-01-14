//! Note, instrument, and channel constants for Nether Storm

// ============================================================================
// Note Constants - F minor/Phrygian (F Gb Ab Bb C Db Eb)
// ============================================================================

// Octave 1 (sub bass range)
pub const F1: u8 = 17;
pub const _GB1: u8 = 18;
pub const _AB1: u8 = 20;
pub const _BB1: u8 = 22;
pub const C2: u8 = 24;
pub const DB1: u8 = 13;
pub const EB1: u8 = 15;

// Octave 2 (main bass range)
pub const F2: u8 = 29;
pub const _GB2: u8 = 30;
pub const _AB2: u8 = 32;
pub const _BB2: u8 = 34;
pub const C3: u8 = 36;
pub const DB2: u8 = 25;
pub const EB2: u8 = 27;

// Octave 3 (upper bass / pad range)
pub const F3: u8 = 41;
pub const _GB3: u8 = 42;
pub const AB3: u8 = 44;
pub const _BB3: u8 = 46;
pub const _C4: u8 = 48;
pub const DB3: u8 = 37;
pub const _EB3: u8 = 39;

// Octave 4 (lead range)
pub const F4: u8 = 53;
pub const _GB4: u8 = 54;
pub const AB4: u8 = 56;
pub const _BB4: u8 = 58;
pub const C5: u8 = 60;
pub const _DB4: u8 = 49;
pub const EB4: u8 = 51;

// Octave 5 (high lead range)
pub const F5: u8 = 65;
pub const EB5: u8 = 63;

// ============================================================================
// Instrument and Channel Constants
// ============================================================================

// Instruments (1-indexed for IT format)
pub const INST_KICK: u8 = 1;
pub const INST_SNARE: u8 = 2;
pub const INST_HH_CLOSED: u8 = 3;
pub const INST_HH_OPEN: u8 = 4;
pub const INST_BREAK: u8 = 5;
pub const INST_CYMBAL: u8 = 6;
pub const INST_SUB: u8 = 7;
pub const INST_REESE: u8 = 8;
pub const INST_WOBBLE: u8 = 9;
pub const INST_PAD: u8 = 10;
pub const INST_STAB: u8 = 11;
pub const INST_LEAD: u8 = 12;
pub const INST_RISER: u8 = 13;
pub const INST_IMPACT: u8 = 14;
pub const INST_ATMOS: u8 = 15;

// Channels (0-indexed)
pub const CH_KICK: u8 = 0;
pub const CH_SNARE: u8 = 1;
pub const CH_HIHAT: u8 = 2;
pub const CH_HIHAT_OPEN: u8 = 3;
pub const CH_BREAK: u8 = 4;
pub const CH_CYMBAL: u8 = 5;
pub const CH_SUB: u8 = 6;
pub const CH_REESE: u8 = 7;
pub const CH_WOBBLE: u8 = 8;
pub const CH_PAD: u8 = 9;
pub const CH_STAB: u8 = 10;
pub const CH_LEAD: u8 = 11;
pub const CH_RISER: u8 = 12;
pub const CH_IMPACT: u8 = 13;
pub const CH_ATMOS: u8 = 14;
