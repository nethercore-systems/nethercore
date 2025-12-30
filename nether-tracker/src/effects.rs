//! Unified tracker effect system
//!
//! This module provides a normalized effect enum that abstracts differences
//! between XM and IT effect semantics. Both formats are converted to this
//! unified representation during parsing.

/// Unified tracker effect (normalized from XM/IT)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerEffect {
    /// No effect
    None,

    // =========================================================================
    // Speed and Tempo
    // =========================================================================
    /// Set speed (ticks per row)
    /// IT: Axx, XM: Fxx (when param < 0x20)
    SetSpeed(u8),

    /// Set tempo (BPM)
    /// IT: Txx, XM: Fxx (when param >= 0x20)
    SetTempo(u8),

    // =========================================================================
    // Pattern Flow Control
    // =========================================================================
    /// Jump to pattern (order position)
    /// IT: Bxx, XM: Bxx
    PositionJump(u8),

    /// Break to row in next pattern
    /// IT: Cxx, XM: Dxx
    PatternBreak(u8),

    /// Pattern delay (rows)
    /// IT: SEx, XM: EEx
    PatternDelay(u8),

    /// Pattern loop
    /// IT: SBx, XM: E6x
    PatternLoop(u8),

    // =========================================================================
    // Volume Effects
    // =========================================================================
    /// Set volume (0-64)
    /// IT: Mxx, XM: Cxx
    SetVolume(u8),

    /// Volume slide
    /// IT: Dxy, XM: Axy
    VolumeSlide { up: u8, down: u8 },

    /// Fine volume slide up
    FineVolumeUp(u8),

    /// Fine volume slide down
    FineVolumeDown(u8),

    /// Global volume (0-128)
    /// IT: Vxx, XM: Gxx
    SetGlobalVolume(u8),

    /// Global volume slide
    /// IT: Wxy, XM: Hxy
    GlobalVolumeSlide { up: u8, down: u8 },

    /// Channel volume (IT only)
    /// IT: Mxx (when in channel mode)
    SetChannelVolume(u8),

    /// Channel volume slide (IT only)
    /// IT: Nxy
    ChannelVolumeSlide { up: u8, down: u8 },

    // =========================================================================
    // Pitch Effects
    // =========================================================================
    /// Pitch slide down
    /// IT: Exx, XM: 1xx
    PortamentoDown(u16),

    /// Pitch slide up
    /// IT: Fxx, XM: 2xx
    PortamentoUp(u16),

    /// Fine pitch slide down
    FinePortaDown(u16),

    /// Fine pitch slide up
    FinePortaUp(u16),

    /// Extra fine pitch slide down
    ExtraFinePortaDown(u16),

    /// Extra fine pitch slide up
    ExtraFinePortaUp(u16),

    /// Tone portamento (slide to note)
    /// IT: Gxx, XM: 3xx
    TonePortamento(u16),

    /// Tone portamento + volume slide
    /// IT: Lxy, XM: 5xy
    TonePortaVolSlide { porta: u16, vol_up: u8, vol_down: u8 },

    // =========================================================================
    // Modulation Effects
    // =========================================================================
    /// Vibrato
    /// IT: Hxy, XM: 4xy
    Vibrato { speed: u8, depth: u8 },

    /// Vibrato + volume slide
    /// IT: Kxy, XM: 6xy
    VibratoVolSlide { vib_speed: u8, vib_depth: u8, vol_up: u8, vol_down: u8 },

    /// Fine vibrato (IT only)
    /// IT: Uxy
    FineVibrato { speed: u8, depth: u8 },

    /// Tremolo
    /// IT: Rxy, XM: 7xy
    Tremolo { speed: u8, depth: u8 },

    /// Tremor (IT only)
    /// IT: Ixy
    Tremor { ontime: u8, offtime: u8 },

    /// Arpeggio
    /// IT: Jxy, XM: 0xy
    Arpeggio { note1: u8, note2: u8 },

    // =========================================================================
    // Panning Effects
    // =========================================================================
    /// Set panning (0-64)
    /// IT: Xxx, XM: 8xx
    SetPanning(u8),

    /// Panning slide
    /// IT: Pxy, XM: Pxy
    PanningSlide { left: u8, right: u8 },

    /// Panbrello (IT only)
    /// IT: Yxy
    Panbrello { speed: u8, depth: u8 },

    // =========================================================================
    // Sample Effects
    // =========================================================================
    /// Sample offset (position to start playback)
    /// IT: Oxx, XM: 9xx
    SampleOffset(u32),

    /// Retrigger note
    /// IT: Qxy, XM: Rxy
    Retrigger { ticks: u8, volume_change: i8 },

    /// Note cut (cut after N ticks)
    /// IT: SCx, XM: ECx
    NoteCut(u8),

    /// Note delay (trigger note after N ticks)
    /// IT: SDx, XM: EDx
    NoteDelay(u8),

    /// Set finetune (XM only)
    /// XM: E5x
    SetFinetune(i8),

    // =========================================================================
    // Filter Effects (IT only)
    // =========================================================================
    /// Set filter cutoff (IT only)
    /// IT: Zxx or MIDI macro
    SetFilterCutoff(u8),

    /// Set filter resonance (IT only)
    SetFilterResonance(u8),

    // =========================================================================
    // Waveform Control
    // =========================================================================
    /// Set vibrato waveform
    /// IT: S3x, XM: E4x
    VibratoWaveform(u8),

    /// Set tremolo waveform
    /// IT: S4x, XM: E7x
    TremoloWaveform(u8),

    /// Set panbrello waveform (IT only)
    /// IT: S5x
    PanbrelloWaveform(u8),

    // =========================================================================
    // Other Effects
    // =========================================================================
    /// Set envelope position (XM only)
    /// XM: Lxx
    SetEnvelopePosition(u8),

    /// Key off (release envelopes)
    /// XM: Kxx
    KeyOff,

    /// Set glissando (IT only)
    /// IT: S1x
    SetGlissando(bool),

    /// Multi retrig note (XM only)
    /// XM: Rxy
    MultiRetrigNote { ticks: u8, volume: u8 },
}

impl Default for TrackerEffect {
    fn default() -> Self {
        Self::None
    }
}

impl TrackerEffect {
    /// Check if this effect modifies pitch
    pub fn affects_pitch(&self) -> bool {
        matches!(
            self,
            Self::PortamentoDown(_)
                | Self::PortamentoUp(_)
                | Self::FinePortaDown(_)
                | Self::FinePortaUp(_)
                | Self::ExtraFinePortaDown(_)
                | Self::ExtraFinePortaUp(_)
                | Self::TonePortamento(_)
                | Self::TonePortaVolSlide { .. }
                | Self::Vibrato { .. }
                | Self::VibratoVolSlide { .. }
                | Self::FineVibrato { .. }
                | Self::Arpeggio { .. }
        )
    }

    /// Check if this effect modifies volume
    pub fn affects_volume(&self) -> bool {
        matches!(
            self,
            Self::SetVolume(_)
                | Self::VolumeSlide { .. }
                | Self::FineVolumeUp(_)
                | Self::FineVolumeDown(_)
                | Self::SetGlobalVolume(_)
                | Self::GlobalVolumeSlide { .. }
                | Self::SetChannelVolume(_)
                | Self::ChannelVolumeSlide { .. }
                | Self::Tremolo { .. }
                | Self::Tremor { .. }
        )
    }

    /// Check if this effect modifies panning
    pub fn affects_panning(&self) -> bool {
        matches!(
            self,
            Self::SetPanning(_) | Self::PanningSlide { .. } | Self::Panbrello { .. }
        )
    }

    /// Check if this effect controls pattern flow
    pub fn affects_pattern_flow(&self) -> bool {
        matches!(
            self,
            Self::PositionJump(_)
                | Self::PatternBreak(_)
                | Self::PatternDelay(_)
                | Self::PatternLoop(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_categories() {
        let porta_down = TrackerEffect::PortamentoDown(100);
        assert!(porta_down.affects_pitch());
        assert!(!porta_down.affects_volume());

        let vol_slide = TrackerEffect::VolumeSlide { up: 5, down: 0 };
        assert!(vol_slide.affects_volume());
        assert!(!vol_slide.affects_pitch());

        let set_pan = TrackerEffect::SetPanning(32);
        assert!(set_pan.affects_panning());
        assert!(!set_pan.affects_volume());

        let pattern_break = TrackerEffect::PatternBreak(16);
        assert!(pattern_break.affects_pattern_flow());
        assert!(!pattern_break.affects_pitch());
    }

    #[test]
    fn test_default_effect() {
        let effect = TrackerEffect::default();
        assert_eq!(effect, TrackerEffect::None);
    }
}
