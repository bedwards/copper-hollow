use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use super::theory::{ChordDegree, Scale};

// ---------------------------------------------------------------------------
// SongPart
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SongPart {
    Intro,
    Verse,
    #[serde(rename = "prechorus")]
    PreChorus,
    Chorus,
    Bridge,
    Outro,
}

impl SongPart {
    /// Default bar count for this part.
    pub fn typical_bars(self) -> u32 {
        match self {
            SongPart::Intro => 4,
            SongPart::Verse => 8,
            SongPart::PreChorus => 4,
            SongPart::Chorus => 8,
            SongPart::Bridge => 8,
            SongPart::Outro => 4,
        }
    }
}

impl FromStr for SongPart {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "intro" => Ok(SongPart::Intro),
            "verse" => Ok(SongPart::Verse),
            "prechorus" | "pre_chorus" => Ok(SongPart::PreChorus),
            "chorus" => Ok(SongPart::Chorus),
            "bridge" => Ok(SongPart::Bridge),
            "outro" => Ok(SongPart::Outro),
            _ => Err(anyhow::anyhow!("unknown song part: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// InstrumentType
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentType {
    // Melodic
    AcousticGuitar,
    ElectricGuitar,
    ElectricBass,
    AcousticBass,
    PedalSteel,
    Mandolin,
    Banjo,
    HammondOrgan,
    Piano,
    Pad,
    // Percussion
    Kick,
    Snare,
    HiHat,
    OpenHiHat,
    Clap,
    Tambourine,
    Cowbell,
    Shaker,
    RideCymbal,
    CrashCymbal,
    Toms,
    Rimshot,
}

impl InstrumentType {
    /// Whether this is a percussion instrument.
    pub fn is_percussion(self) -> bool {
        matches!(
            self,
            InstrumentType::Kick
                | InstrumentType::Snare
                | InstrumentType::HiHat
                | InstrumentType::OpenHiHat
                | InstrumentType::Clap
                | InstrumentType::Tambourine
                | InstrumentType::Cowbell
                | InstrumentType::Shaker
                | InstrumentType::RideCymbal
                | InstrumentType::CrashCymbal
                | InstrumentType::Toms
                | InstrumentType::Rimshot
        )
    }

    /// Comfortable MIDI note range (low, high) for melodic instruments.
    /// Returns `(36, 36)` for percussion (fixed MIDI note C1).
    pub fn midi_range(self) -> (u8, u8) {
        match self {
            InstrumentType::AcousticGuitar => (40, 79),
            InstrumentType::ElectricGuitar => (40, 84),
            InstrumentType::ElectricBass => (28, 55),
            InstrumentType::AcousticBass => (28, 50),
            InstrumentType::PedalSteel => (40, 79),
            InstrumentType::Mandolin => (55, 86),
            InstrumentType::Banjo => (48, 79),
            InstrumentType::HammondOrgan => (36, 84),
            InstrumentType::Piano => (28, 96),
            InstrumentType::Pad => (36, 84),
            _ => (36, 36), // percussion: fixed MIDI note C1
        }
    }
}

// ---------------------------------------------------------------------------
// TrackRole
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackRole {
    Rhythm,
    #[serde(rename = "lead")]
    LeadMelody,
    #[serde(rename = "counter")]
    CounterMelody,
    Bass,
    Drum,
    #[serde(rename = "pad")]
    PadSustain,
}

// ---------------------------------------------------------------------------
// Voicing
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Voicing {
    Poly,
    Mono,
}

// ---------------------------------------------------------------------------
// Strum types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrumDirection {
    Down,
    Up,
    Mute,
    Ghost,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrumHit {
    /// Position within pattern in ticks.
    pub tick_offset: u32,
    pub direction: StrumDirection,
    /// Velocity multiplier 0.0–1.0.
    pub velocity_factor: f32,
    /// Chord spread time in ms (0 = simultaneous).
    pub stagger_ms: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrumPattern {
    pub name: String,
    pub hits: Vec<StrumHit>,
    /// Pattern length in beats.
    pub beats: u32,
}

// ---------------------------------------------------------------------------
// NoteEvent / CcEvent
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteEvent {
    /// Absolute tick position from pattern start (480 ticks/beat).
    pub tick: u32,
    /// MIDI note 0–127.
    pub note: u8,
    /// Velocity 0–127.
    pub velocity: u8,
    /// Duration in ticks.
    pub duration: u32,
    /// MIDI channel 0–15, matches track id.
    pub channel: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CcEvent {
    pub tick: u32,
    /// CC number, or 255 for pitch bend.
    pub cc: u8,
    /// 0–127 for CC, 0–16383 for pitch bend (8192 = center).
    pub value: u16,
    pub channel: u8,
}

// ---------------------------------------------------------------------------
// Pattern
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub events: Vec<NoteEvent>,
    pub cc_events: Vec<CcEvent>,
    /// Total length in ticks.
    pub length_ticks: u32,
    /// Bar count this pattern spans.
    pub bars: u32,
}

impl Pattern {
    pub fn empty(bars: u32) -> Self {
        Self {
            events: Vec::new(),
            cc_events: Vec::new(),
            length_ticks: bars * super::TICKS_PER_BAR,
            bars,
        }
    }
}

// ---------------------------------------------------------------------------
// Track
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    /// 0–15, doubles as MIDI channel.
    pub id: u8,
    pub name: String,
    pub role: TrackRole,
    pub instrument: InstrumentType,
    pub voicing: Voicing,
    pub muted: bool,
    pub solo: bool,
    pub patterns: HashMap<SongPart, Pattern>,
    pub automation: HashMap<SongPart, Vec<CcEvent>>,
    pub active_parts: HashMap<SongPart, bool>,
}

// ---------------------------------------------------------------------------
// Song
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    /// Beats per minute.
    pub tempo: f64,
    pub time_signature: (u8, u8),
    pub rhythm_scale: Scale,
    pub lead_scale: Scale,
    /// Exactly 16 tracks (channels 0–15).
    pub tracks: Vec<Track>,
    /// Ordered song parts (may repeat, e.g. Verse appears twice).
    pub structure: Vec<SongPart>,
    pub progressions: HashMap<SongPart, Vec<ChordDegree>>,
    pub strum_pattern: StrumPattern,
    /// 0.0 = straight, 1.0 = full triplet swing.
    pub swing: f32,
}
