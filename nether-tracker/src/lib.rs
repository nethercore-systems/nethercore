//! Nether-Tracker: Unified tracker module format for Nethercore
//!
//! This crate provides format-agnostic tracker types that both XM and IT modules
//! can be converted to. This allows the playback engine to handle multiple formats
//! with a single unified implementation.
//!
//! # Design
//!
//! The unified format normalizes differences between XM and IT:
//! - Effect semantics are standardized
//! - Channel counts are unified (64 max)
//! - Envelopes are normalized
//! - NNA (New Note Actions) are represented uniformly
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────┐     ┌──────────────────┐
//! │  IT File (.it)   │     │  XM File (.xm)   │
//! └────────┬─────────┘     └────────┬─────────┘
//!          │                        │
//!     parse_it()               parse_xm()
//!          │                        │
//!          ▼                        ▼
//!     ┌────────────────────────────────────────┐
//!     │         TrackerModule (unified)         │
//!     │  - patterns: Vec<TrackerPattern>       │
//!     │  - instruments: Vec<TrackerInstrument> │
//!     │  - samples: Vec<TrackerSample>         │
//!     │  - format_flags: FormatFlags           │
//!     └────────────────────────────────────────┘
//!                      │
//!                      ▼
//!              TrackerEngine
//!         (plays any TrackerModule)
//! ```

mod convert_it;
mod convert_xm;
mod converter;
mod effects;
mod instrument;
mod pattern;
mod sample;

#[cfg(test)]
mod tests;

pub use convert_it::from_it_module;
pub use convert_xm::{convert_loop_points, from_xm_module};
pub use converter::{ItConverter, ModuleConverter, XmConverter};
pub use effects::TrackerEffect;
pub use instrument::{
    DuplicateCheckAction, DuplicateCheckType, EnvelopeFlags, NewNoteAction, TrackerEnvelope,
    TrackerInstrument,
};
pub use pattern::{TrackerNote, TrackerPattern};
pub use sample::{LoopType, TrackerSample};

// =============================================================================
// Unified Tracker Module
// =============================================================================

/// Unified tracker module format (agnostic to XM/IT origin)
#[derive(Debug, Clone)]
pub struct TrackerModule {
    /// Module name
    pub name: String,
    /// Number of channels used (1-64)
    pub num_channels: u8,
    /// Initial speed (ticks per row)
    pub initial_speed: u8,
    /// Initial tempo (BPM)
    pub initial_tempo: u8,
    /// Global volume (0-128)
    pub global_volume: u8,
    /// Mix volume (0-128, IT only - scales master output)
    pub mix_volume: u8,
    /// Panning separation (0-128, IT only - 128 = full stereo, 0 = mono)
    pub panning_separation: u8,
    /// Pattern order table
    pub order_table: Vec<u8>,
    /// Pattern data
    pub patterns: Vec<TrackerPattern>,
    /// Instrument definitions
    pub instruments: Vec<TrackerInstrument>,
    /// Sample definitions
    pub samples: Vec<TrackerSample>,
    /// Format-specific flags
    pub format: FormatFlags,
    /// Optional song message
    pub message: Option<String>,
    /// Restart position for song looping (XM feature, IT uses 0)
    pub restart_position: u16,
}

impl TrackerModule {
    /// Get the pattern at the given order position
    pub fn pattern_at_order(&self, order: u16) -> Option<&TrackerPattern> {
        let pattern_idx = *self.order_table.get(order as usize)? as usize;
        if pattern_idx >= 254 {
            return None; // Skip or end marker
        }
        self.patterns.get(pattern_idx)
    }

    /// Check if linear frequency slides are used (vs Amiga)
    pub fn uses_linear_slides(&self) -> bool {
        self.format.contains(FormatFlags::LINEAR_SLIDES)
    }

    /// Check if this module uses instruments (vs samples-only)
    pub fn uses_instruments(&self) -> bool {
        self.format.contains(FormatFlags::INSTRUMENTS)
    }

    /// Check if this module uses old effects mode (S3M compatibility)
    pub fn uses_old_effects(&self) -> bool {
        self.format.contains(FormatFlags::OLD_EFFECTS)
    }

    /// Check if this module links G memory with E/F for portamento
    pub fn uses_link_g_memory(&self) -> bool {
        self.format.contains(FormatFlags::LINK_G_MEMORY)
    }
}

/// Format-specific flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormatFlags(u16);

impl FormatFlags {
    /// Use linear frequency slides (vs Amiga slides)
    pub const LINEAR_SLIDES: Self = Self(0x0001);
    /// Use instruments (vs samples-only mode)
    pub const INSTRUMENTS: Self = Self(0x0002);
    /// Original format was IT (vs XM)
    pub const IS_IT_FORMAT: Self = Self(0x0004);
    /// Original format was XM
    pub const IS_XM_FORMAT: Self = Self(0x0008);
    /// Use old effects (S3M compatibility - affects vibrato/tremolo depth)
    pub const OLD_EFFECTS: Self = Self(0x0010);
    /// Link G memory with E/F for portamento
    pub const LINK_G_MEMORY: Self = Self(0x0020);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> u16 {
        self.0
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for FormatFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
