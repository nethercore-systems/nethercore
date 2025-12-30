//! IT module data structures

/// Parsed IT module (patterns and metadata - samples loaded from ROM)
#[derive(Debug, Clone)]
pub struct ItModule {
    /// Module name (max 26 chars)
    pub name: String,
    /// Number of channels used (1-64)
    pub num_channels: u8,
    /// Number of orders
    pub num_orders: u16,
    /// Number of instruments
    pub num_instruments: u16,
    /// Number of samples
    pub num_samples: u16,
    /// Number of patterns
    pub num_patterns: u16,
    /// Created with tracker version (Cwt/v)
    pub created_with: u16,
    /// Compatible with version (Cmwt)
    pub compatible_with: u16,
    /// Module flags
    pub flags: ItFlags,
    /// Special flags
    pub special: u16,
    /// Global volume (0-128)
    pub global_volume: u8,
    /// Mix volume (0-128)
    pub mix_volume: u8,
    /// Initial speed (ticks per row)
    pub initial_speed: u8,
    /// Initial tempo (BPM)
    pub initial_tempo: u8,
    /// Panning separation (0-128, 128 = max separation)
    pub panning_separation: u8,
    /// Pitch wheel depth for MIDI
    pub pitch_wheel_depth: u8,
    /// Per-channel default panning (64 channels)
    /// 0-64 = pan position, +128 = channel disabled
    pub channel_pan: [u8; 64],
    /// Per-channel default volume (64 channels, 0-64)
    pub channel_vol: [u8; 64],
    /// Pattern order table
    pub order_table: Vec<u8>,
    /// Pattern data
    pub patterns: Vec<ItPattern>,
    /// Instrument definitions
    pub instruments: Vec<ItInstrument>,
    /// Sample definitions
    pub samples: Vec<ItSample>,
    /// Song message (optional)
    pub message: Option<String>,
}

impl ItModule {
    /// Get the pattern at the given order position
    pub fn pattern_at_order(&self, order: u16) -> Option<&ItPattern> {
        let pattern_idx = *self.order_table.get(order as usize)? as usize;
        if pattern_idx >= 254 {
            return None; // Skip or end marker
        }
        self.patterns.get(pattern_idx)
    }

    /// Get total number of valid orders in the song
    pub fn total_orders(&self) -> u16 {
        self.order_table
            .iter()
            .take_while(|&&o| o != crate::ORDER_END)
            .filter(|&&o| o != crate::ORDER_SKIP)
            .count() as u16
    }

    /// Check if the module uses instruments (vs samples-only mode)
    pub fn uses_instruments(&self) -> bool {
        self.flags.contains(ItFlags::INSTRUMENTS)
    }

    /// Check if the module uses linear slides
    pub fn uses_linear_slides(&self) -> bool {
        self.flags.contains(ItFlags::LINEAR_SLIDES)
    }
}

impl Default for ItModule {
    fn default() -> Self {
        Self {
            name: String::new(),
            num_channels: 4,
            num_orders: 0,
            num_instruments: 0,
            num_samples: 0,
            num_patterns: 0,
            created_with: 0x0214, // IT 2.14
            compatible_with: 0x0200,
            flags: ItFlags::STEREO | ItFlags::INSTRUMENTS | ItFlags::LINEAR_SLIDES,
            special: 0,
            global_volume: 128,
            mix_volume: 48,
            initial_speed: 6,
            initial_tempo: 125,
            panning_separation: 128,
            pitch_wheel_depth: 0,
            channel_pan: [32; 64], // Center pan
            channel_vol: [64; 64], // Full volume
            order_table: Vec::new(),
            patterns: Vec::new(),
            instruments: Vec::new(),
            samples: Vec::new(),
            message: None,
        }
    }
}

/// IT module flags (from header byte 0x002C)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItFlags(u16);

impl ItFlags {
    /// Stereo output
    pub const STEREO: Self = Self(0x0001);
    /// Vol0MixOptimizations - skip mixing silent channels
    pub const VOL0_MIX_OPT: Self = Self(0x0002);
    /// Use instruments (vs samples-only mode)
    pub const INSTRUMENTS: Self = Self(0x0004);
    /// Use linear slides (vs Amiga slides)
    pub const LINEAR_SLIDES: Self = Self(0x0008);
    /// Use old effects (S3M compatibility)
    pub const OLD_EFFECTS: Self = Self(0x0010);
    /// Link G memory with E/F for portamento
    pub const LINK_G_MEMORY: Self = Self(0x0020);
    /// Use MIDI pitch controller
    pub const MIDI_PITCH_CTRL: Self = Self(0x0040);
    /// Request embedded MIDI configuration
    pub const EMBEDDED_MIDI: Self = Self(0x0080);

    /// Create flags from raw u16
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    /// Get raw bits
    pub const fn bits(&self) -> u16 {
        self.0
    }

    /// Check if flag is set
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combine flags
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for ItFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for ItFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

/// IT pattern containing rows of note data
#[derive(Debug, Clone)]
pub struct ItPattern {
    /// Number of rows in this pattern (1-200)
    pub num_rows: u16,
    /// Unpacked note data: [row][channel]
    pub notes: Vec<Vec<ItNote>>,
}

impl ItPattern {
    /// Get note at specific row and channel
    pub fn get_note(&self, row: u16, channel: u8) -> Option<&ItNote> {
        self.notes.get(row as usize)?.get(channel as usize)
    }

    /// Create an empty pattern with the given dimensions
    pub fn empty(num_rows: u16, num_channels: u8) -> Self {
        let mut notes = Vec::with_capacity(num_rows as usize);
        for _ in 0..num_rows {
            notes.push(vec![ItNote::default(); num_channels as usize]);
        }
        Self { num_rows, notes }
    }
}

/// Single note/command in a pattern
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ItNote {
    /// Note value: 0-119 = C-0 to B-9, 254 = note cut, 255 = note off, 253 = note fade
    pub note: u8,
    /// Instrument number (1-99, 0 = none)
    pub instrument: u8,
    /// Volume column (complex encoding, see IT spec)
    pub volume: u8,
    /// Effect command (A-Z = 1-26)
    pub effect: u8,
    /// Effect parameter
    pub effect_param: u8,
}

impl ItNote {
    /// Check if this is a note-cut (===)
    #[inline]
    pub fn is_note_cut(&self) -> bool {
        self.note == crate::NOTE_CUT
    }

    /// Check if this is a note-off (^^^)
    #[inline]
    pub fn is_note_off(&self) -> bool {
        self.note == crate::NOTE_OFF
    }

    /// Check if this note triggers a new note
    #[inline]
    pub fn has_note(&self) -> bool {
        self.note <= crate::NOTE_MAX
    }

    /// Check if this sets an instrument
    #[inline]
    pub fn has_instrument(&self) -> bool {
        self.instrument > 0
    }

    /// Check if there's an effect
    #[inline]
    pub fn has_effect(&self) -> bool {
        self.effect != 0 || self.effect_param != 0
    }

    /// Check if there's a volume column value
    #[inline]
    pub fn has_volume(&self) -> bool {
        self.volume != 0
    }

    /// Parse volume column value
    /// Returns (effect_type, value) where effect_type is:
    /// - 0-64: Set volume
    /// - 65-74 (a-j): Fine volume up
    /// - 75-84 (k-t): Fine volume down
    /// etc. (see IT spec)
    pub fn parse_volume(&self) -> Option<(VolumeEffect, u8)> {
        if self.volume == 0 {
            return None;
        }

        match self.volume {
            0..=64 => Some((VolumeEffect::SetVolume, self.volume)),
            65..=74 => Some((VolumeEffect::FineVolumeUp, self.volume - 65)),
            75..=84 => Some((VolumeEffect::FineVolumeDown, self.volume - 75)),
            85..=94 => Some((VolumeEffect::VolumeSlideUp, self.volume - 85)),
            95..=104 => Some((VolumeEffect::VolumeSlideDown, self.volume - 95)),
            105..=114 => Some((VolumeEffect::PitchSlideDown, self.volume - 105)),
            115..=124 => Some((VolumeEffect::PitchSlideUp, self.volume - 115)),
            128..=192 => Some((VolumeEffect::SetPanning, self.volume - 128)),
            193..=202 => Some((VolumeEffect::TonePortamento, self.volume - 193)),
            203..=212 => Some((VolumeEffect::Vibrato, self.volume - 203)),
            _ => None,
        }
    }

    /// Convert note number to octave and semitone
    /// Returns (octave 0-9, semitone 0-11) where semitone 0=C, 11=B
    pub fn note_to_octave_semitone(&self) -> Option<(u8, u8)> {
        if self.has_note() {
            Some((self.note / 12, self.note % 12))
        } else {
            None
        }
    }

    // ========== Builder Methods ==========

    /// Create a note with pitch, instrument, and volume
    ///
    /// # Arguments
    /// * `note` - Note name like "C-4" or note number (0-119)
    /// * `instrument` - Instrument number (1-99)
    /// * `volume` - Volume (0-64)
    ///
    /// # Examples
    /// ```
    /// use nether_it::ItNote;
    /// let note = ItNote::play("C-4", 1, 64);
    /// let note2 = ItNote::play_note(60, 1, 64); // Same as C-5
    /// ```
    pub fn play(note_name: &str, instrument: u8, volume: u8) -> Self {
        let note = note_from_name(note_name).unwrap_or(0);
        Self {
            note,
            instrument,
            volume: volume.min(64),
            effect: 0,
            effect_param: 0,
        }
    }

    /// Create a note with note number instead of name
    pub fn play_note(note: u8, instrument: u8, volume: u8) -> Self {
        Self {
            note: note.min(119),
            instrument,
            volume: volume.min(64),
            effect: 0,
            effect_param: 0,
        }
    }

    /// Create a note-off (^^^)
    pub fn off() -> Self {
        Self {
            note: crate::NOTE_OFF,
            instrument: 0,
            volume: 0,
            effect: 0,
            effect_param: 0,
        }
    }

    /// Create a note-cut (===)
    pub fn cut() -> Self {
        Self {
            note: crate::NOTE_CUT,
            instrument: 0,
            volume: 0,
            effect: 0,
            effect_param: 0,
        }
    }

    /// Create a note-fade
    pub fn fade() -> Self {
        Self {
            note: crate::NOTE_FADE,
            instrument: 0,
            volume: 0,
            effect: 0,
            effect_param: 0,
        }
    }

    /// Create a note with an effect
    pub fn with_effect(mut self, effect: u8, effect_param: u8) -> Self {
        self.effect = effect;
        self.effect_param = effect_param;
        self
    }

    /// Create a note with volume column
    pub fn with_volume_column(mut self, volume: u8) -> Self {
        self.volume = volume;
        self
    }
}

/// Convert note name to note number
///
/// Supports formats:
/// - "C-4" = Middle C (note 48)
/// - "C#4" or "Db4" = C# (note 49)
/// - "---" = No note (0)
///
/// Returns None for invalid note names
pub fn note_from_name(name: &str) -> Option<u8> {
    let name = name.trim();

    if name == "---" || name.is_empty() {
        return Some(0);
    }

    let name = name.replace('-', "");
    if name.len() < 2 {
        return None;
    }

    let semitone = match &name[0..1] {
        "C" => 0,
        "D" => 2,
        "E" => 4,
        "F" => 5,
        "G" => 7,
        "A" => 9,
        "B" => 11,
        _ => return None,
    };

    let mut offset = 1;
    let sharp = if name.len() > offset && &name[offset..offset+1] == "#" {
        offset += 1;
        1
    } else if name.len() > offset && &name[offset..offset+1] == "b" {
        offset += 1;
        -1
    } else {
        0
    };

    let octave: i32 = name[offset..].parse().ok()?;
    if !(0..=9).contains(&octave) {
        return None;
    }

    let note = (octave * 12 + semitone as i32 + sharp).clamp(0, 119) as u8;
    Some(note)
}

/// Volume column effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeEffect {
    /// Set volume (0-64)
    SetVolume,
    /// Fine volume slide up
    FineVolumeUp,
    /// Fine volume slide down
    FineVolumeDown,
    /// Volume slide up
    VolumeSlideUp,
    /// Volume slide down
    VolumeSlideDown,
    /// Pitch slide down
    PitchSlideDown,
    /// Pitch slide up
    PitchSlideUp,
    /// Set panning (0-64)
    SetPanning,
    /// Tone portamento
    TonePortamento,
    /// Vibrato depth
    Vibrato,
}

/// New Note Action - what happens when a new note is played
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NewNoteAction {
    /// Cut the previous note immediately
    #[default]
    Cut = 0,
    /// Continue playing the previous note in background
    Continue = 1,
    /// Release the previous note (key-off)
    NoteOff = 2,
    /// Fade out the previous note
    NoteFade = 3,
}

impl NewNoteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Cut,
            1 => Self::Continue,
            2 => Self::NoteOff,
            3 => Self::NoteFade,
            _ => Self::Cut,
        }
    }
}

/// Duplicate Check Type - when to check for duplicate notes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DuplicateCheckType {
    /// No duplicate checking
    #[default]
    Off = 0,
    /// Check for same note
    Note = 1,
    /// Check for same sample
    Sample = 2,
    /// Check for same instrument
    Instrument = 3,
}

impl DuplicateCheckType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Off,
            1 => Self::Note,
            2 => Self::Sample,
            3 => Self::Instrument,
            _ => Self::Off,
        }
    }
}

/// Duplicate Check Action - what to do with duplicate notes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DuplicateCheckAction {
    /// Cut the duplicate note
    #[default]
    Cut = 0,
    /// Release the duplicate note (key-off)
    NoteOff = 1,
    /// Fade out the duplicate note
    NoteFade = 2,
}

impl DuplicateCheckAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Cut,
            1 => Self::NoteOff,
            2 => Self::NoteFade,
            _ => Self::Cut,
        }
    }
}

/// IT instrument metadata
#[derive(Debug, Clone)]
pub struct ItInstrument {
    /// Instrument name (max 26 chars)
    pub name: String,
    /// DOS filename (max 12 chars)
    pub filename: String,
    /// New Note Action
    pub nna: NewNoteAction,
    /// Duplicate Check Type
    pub dct: DuplicateCheckType,
    /// Duplicate Check Action
    pub dca: DuplicateCheckAction,
    /// Fadeout value (0-128, multiply by 8 for internal 0-1024)
    pub fadeout: u16,
    /// Pitch-Pan Separation (-32 to +32)
    pub pitch_pan_separation: i8,
    /// Pitch-Pan Center note (0-119)
    pub pitch_pan_center: u8,
    /// Global volume (0-128)
    pub global_volume: u8,
    /// Default panning (0-64), None if not enabled
    pub default_pan: Option<u8>,
    /// Random volume variation (0-100%)
    pub random_volume: u8,
    /// Random panning variation (0-64)
    pub random_pan: u8,
    /// Note-Sample-Keyboard table (120 entries)
    /// Each entry: (note_to_play, sample_number)
    pub note_sample_table: [(u8, u8); 120],
    /// Volume envelope
    pub volume_envelope: Option<ItEnvelope>,
    /// Panning envelope
    pub panning_envelope: Option<ItEnvelope>,
    /// Pitch/Filter envelope
    pub pitch_envelope: Option<ItEnvelope>,
    /// Initial filter cutoff (0-127), None if not set
    pub filter_cutoff: Option<u8>,
    /// Initial filter resonance (0-127), None if not set
    pub filter_resonance: Option<u8>,
    /// MIDI channel (0-16, 0 = disabled)
    pub midi_channel: u8,
    /// MIDI program (0-127)
    pub midi_program: u8,
    /// MIDI bank (0-16383)
    pub midi_bank: u16,
}

impl Default for ItInstrument {
    fn default() -> Self {
        // Default note-sample table: each note maps to itself with sample 1
        let mut note_sample_table = [(0u8, 0u8); 120];
        for (i, entry) in note_sample_table.iter_mut().enumerate() {
            entry.0 = i as u8; // Note plays as itself
            entry.1 = 1; // Use sample 1
        }

        Self {
            name: String::new(),
            filename: String::new(),
            nna: NewNoteAction::Cut,
            dct: DuplicateCheckType::Off,
            dca: DuplicateCheckAction::Cut,
            fadeout: 0,
            pitch_pan_separation: 0,
            pitch_pan_center: 60, // C-5
            global_volume: 128,
            default_pan: None,
            random_volume: 0,
            random_pan: 0,
            note_sample_table,
            volume_envelope: None,
            panning_envelope: None,
            pitch_envelope: None,
            filter_cutoff: None,
            filter_resonance: None,
            midi_channel: 0,
            midi_program: 0,
            midi_bank: 0,
        }
    }
}

impl ItInstrument {
    /// Get the sample number for a given note
    pub fn sample_for_note(&self, note: u8) -> Option<u8> {
        if note < 120 {
            let (_, sample) = self.note_sample_table[note as usize];
            if sample > 0 {
                Some(sample)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the transposed note for a given input note
    pub fn note_for_input(&self, note: u8) -> u8 {
        if note < 120 {
            self.note_sample_table[note as usize].0
        } else {
            note
        }
    }
}

/// IT sample metadata
#[derive(Debug, Clone)]
pub struct ItSample {
    /// Sample name (max 26 chars)
    pub name: String,
    /// DOS filename (max 12 chars)
    pub filename: String,
    /// Global volume (0-64)
    pub global_volume: u8,
    /// Sample flags
    pub flags: ItSampleFlags,
    /// Default volume (0-64)
    pub default_volume: u8,
    /// Default panning (0-64), None if not enabled
    pub default_pan: Option<u8>,
    /// Sample length in samples (not bytes)
    pub length: u32,
    /// Loop begin (in samples)
    pub loop_begin: u32,
    /// Loop end (in samples)
    pub loop_end: u32,
    /// C5 speed (sample rate for C-5)
    pub c5_speed: u32,
    /// Sustain loop begin
    pub sustain_loop_begin: u32,
    /// Sustain loop end
    pub sustain_loop_end: u32,
    /// Auto-vibrato speed (0-64)
    pub vibrato_speed: u8,
    /// Auto-vibrato depth (0-64)
    pub vibrato_depth: u8,
    /// Auto-vibrato rate (0-64)
    pub vibrato_rate: u8,
    /// Auto-vibrato waveform (0=sine, 1=ramp down, 2=square, 3=random)
    pub vibrato_type: u8,
}

impl Default for ItSample {
    fn default() -> Self {
        Self {
            name: String::new(),
            filename: String::new(),
            global_volume: 64,
            flags: ItSampleFlags::empty(),
            default_volume: 64,
            default_pan: None,
            length: 0,
            loop_begin: 0,
            loop_end: 0,
            c5_speed: 8363, // Default Amiga sample rate
            sustain_loop_begin: 0,
            sustain_loop_end: 0,
            vibrato_speed: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
            vibrato_type: 0,
        }
    }
}

impl ItSample {
    /// Check if sample has loop enabled
    pub fn has_loop(&self) -> bool {
        self.flags.contains(ItSampleFlags::LOOP)
    }

    /// Check if sample has sustain loop enabled
    pub fn has_sustain_loop(&self) -> bool {
        self.flags.contains(ItSampleFlags::SUSTAIN_LOOP)
    }

    /// Check if loop is ping-pong (bidirectional)
    pub fn is_pingpong_loop(&self) -> bool {
        self.flags.contains(ItSampleFlags::PINGPONG_LOOP)
    }

    /// Check if sustain loop is ping-pong
    pub fn is_pingpong_sustain(&self) -> bool {
        self.flags.contains(ItSampleFlags::PINGPONG_SUSTAIN)
    }

    /// Check if sample is 16-bit
    pub fn is_16bit(&self) -> bool {
        self.flags.contains(ItSampleFlags::SAMPLE_16BIT)
    }

    /// Check if sample is stereo
    pub fn is_stereo(&self) -> bool {
        self.flags.contains(ItSampleFlags::STEREO)
    }

    /// Check if sample data is compressed
    pub fn is_compressed(&self) -> bool {
        self.flags.contains(ItSampleFlags::COMPRESSED)
    }
}

/// IT sample flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItSampleFlags(u8);

impl ItSampleFlags {
    /// Sample data exists
    pub const HAS_DATA: Self = Self(0x01);
    /// 16-bit sample (vs 8-bit)
    pub const SAMPLE_16BIT: Self = Self(0x02);
    /// Stereo sample
    pub const STEREO: Self = Self(0x04);
    /// Compressed sample (IT215)
    pub const COMPRESSED: Self = Self(0x08);
    /// Loop enabled
    pub const LOOP: Self = Self(0x10);
    /// Sustain loop enabled
    pub const SUSTAIN_LOOP: Self = Self(0x20);
    /// Ping-pong loop
    pub const PINGPONG_LOOP: Self = Self(0x40);
    /// Ping-pong sustain loop
    pub const PINGPONG_SUSTAIN: Self = Self(0x80);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> u8 {
        self.0
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for ItSampleFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// IT envelope
#[derive(Debug, Clone)]
pub struct ItEnvelope {
    /// Envelope points: (tick, value)
    /// For volume: value 0-64
    /// For panning: value -32 to +32 (stored as 0-64, 32 = center)
    /// For pitch: value -32 to +32 (half-semitones)
    pub points: Vec<(u16, i8)>,
    /// Loop begin point index
    pub loop_begin: u8,
    /// Loop end point index
    pub loop_end: u8,
    /// Sustain loop begin point index
    pub sustain_begin: u8,
    /// Sustain loop end point index
    pub sustain_end: u8,
    /// Envelope flags
    pub flags: ItEnvelopeFlags,
}

impl Default for ItEnvelope {
    fn default() -> Self {
        Self {
            points: vec![(0, 64), (100, 64)], // Flat envelope at max
            loop_begin: 0,
            loop_end: 0,
            sustain_begin: 0,
            sustain_end: 0,
            flags: ItEnvelopeFlags::empty(),
        }
    }
}

impl ItEnvelope {
    /// Check if envelope is enabled
    pub fn is_enabled(&self) -> bool {
        self.flags.contains(ItEnvelopeFlags::ENABLED)
    }

    /// Check if envelope has loop
    pub fn has_loop(&self) -> bool {
        self.flags.contains(ItEnvelopeFlags::LOOP)
    }

    /// Check if envelope has sustain loop
    pub fn has_sustain(&self) -> bool {
        self.flags.contains(ItEnvelopeFlags::SUSTAIN_LOOP)
    }

    /// Check if this is a filter envelope (for pitch envelope type)
    pub fn is_filter(&self) -> bool {
        self.flags.contains(ItEnvelopeFlags::FILTER)
    }

    /// Get interpolated envelope value at a given tick
    pub fn value_at(&self, tick: u16) -> i8 {
        if self.points.is_empty() {
            return 64; // Default max value
        }

        // Find the two points surrounding this tick
        for i in 0..self.points.len().saturating_sub(1) {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[i + 1];

            if tick >= x1 && tick < x2 {
                // Linear interpolation
                if x2 == x1 {
                    return y1;
                }
                let dx = (x2 - x1) as f32;
                let dy = y2 as f32 - y1 as f32;
                let t = (tick - x1) as f32 / dx;
                return (y1 as f32 + dy * t) as i8;
            }
        }

        // Past the last point, use the last value
        self.points.last().map(|(_, y)| *y).unwrap_or(64)
    }
}

/// IT envelope flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItEnvelopeFlags(u8);

impl ItEnvelopeFlags {
    /// Envelope is enabled
    pub const ENABLED: Self = Self(0x01);
    /// Loop is enabled
    pub const LOOP: Self = Self(0x02);
    /// Sustain loop is enabled
    pub const SUSTAIN_LOOP: Self = Self(0x04);
    /// Carry envelope (continue from previous note)
    pub const CARRY: Self = Self(0x08);
    /// Filter envelope (for pitch envelope type only)
    pub const FILTER: Self = Self(0x80);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> u8 {
        self.0
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for ItEnvelopeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_note_methods() {
        let note_off = ItNote {
            note: 255,
            ..Default::default()
        };
        assert!(note_off.is_note_off());
        assert!(!note_off.has_note());

        let note_cut = ItNote {
            note: 254,
            ..Default::default()
        };
        assert!(note_cut.is_note_cut());
        assert!(!note_cut.has_note());

        let note_c4 = ItNote {
            note: 48, // C-4
            instrument: 1,
            ..Default::default()
        };
        assert!(!note_c4.is_note_off());
        assert!(note_c4.has_note());
        assert!(note_c4.has_instrument());
        assert_eq!(note_c4.note_to_octave_semitone(), Some((4, 0)));
    }

    #[test]
    fn test_it_pattern_empty() {
        let pattern = ItPattern::empty(64, 8);
        assert_eq!(pattern.num_rows, 64);
        assert_eq!(pattern.notes.len(), 64);
        assert_eq!(pattern.notes[0].len(), 8);
    }

    #[test]
    fn test_it_sample_flags() {
        let mut sample = ItSample::default();
        assert!(!sample.has_loop());

        sample.flags = ItSampleFlags::HAS_DATA | ItSampleFlags::LOOP | ItSampleFlags::SAMPLE_16BIT;
        assert!(sample.has_loop());
        assert!(sample.is_16bit());
        assert!(!sample.is_stereo());
        assert!(!sample.is_pingpong_loop());
    }

    #[test]
    fn test_it_envelope_interpolation() {
        let env = ItEnvelope {
            points: vec![(0, 64), (10, 32), (20, 0)],
            flags: ItEnvelopeFlags::ENABLED,
            ..Default::default()
        };

        assert_eq!(env.value_at(0), 64);
        assert_eq!(env.value_at(5), 48); // Midpoint between 64 and 32
        assert_eq!(env.value_at(10), 32);
        assert_eq!(env.value_at(15), 16); // Midpoint between 32 and 0
        assert_eq!(env.value_at(20), 0);
        assert_eq!(env.value_at(30), 0); // Past end
    }

    #[test]
    fn test_nna_from_u8() {
        assert_eq!(NewNoteAction::from_u8(0), NewNoteAction::Cut);
        assert_eq!(NewNoteAction::from_u8(1), NewNoteAction::Continue);
        assert_eq!(NewNoteAction::from_u8(2), NewNoteAction::NoteOff);
        assert_eq!(NewNoteAction::from_u8(3), NewNoteAction::NoteFade);
        assert_eq!(NewNoteAction::from_u8(99), NewNoteAction::Cut); // Invalid defaults to Cut
    }

    #[test]
    fn test_volume_column_parsing() {
        let note_vol = ItNote {
            volume: 32,
            ..Default::default()
        };
        assert_eq!(
            note_vol.parse_volume(),
            Some((VolumeEffect::SetVolume, 32))
        );

        let note_pan = ItNote {
            volume: 160, // 128 + 32 = center pan
            ..Default::default()
        };
        assert_eq!(
            note_pan.parse_volume(),
            Some((VolumeEffect::SetPanning, 32))
        );

        let note_porta = ItNote {
            volume: 198, // 193 + 5
            ..Default::default()
        };
        assert_eq!(
            note_porta.parse_volume(),
            Some((VolumeEffect::TonePortamento, 5))
        );
    }
}
