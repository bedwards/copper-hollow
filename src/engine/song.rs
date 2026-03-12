use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use super::theory::{ChordDegree, PitchClass, Scale, ScaleType};

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

    /// General MIDI drum note for percussion instruments.
    /// Returns `None` for melodic instruments.
    pub fn gm_drum_note(self) -> Option<u8> {
        match self {
            InstrumentType::Kick => Some(36),        // Acoustic Bass Drum
            InstrumentType::Snare => Some(38),       // Acoustic Snare
            InstrumentType::HiHat => Some(42),       // Closed Hi-Hat
            InstrumentType::OpenHiHat => Some(46),   // Open Hi-Hat
            InstrumentType::Clap => Some(39),        // Hand Clap
            InstrumentType::Tambourine => Some(54),  // Tambourine
            InstrumentType::Cowbell => Some(56),      // Cowbell
            InstrumentType::Shaker => Some(70),      // Maracas
            InstrumentType::RideCymbal => Some(51),  // Ride Cymbal 1
            InstrumentType::CrashCymbal => Some(49), // Crash Cymbal 1
            InstrumentType::Toms => Some(45),        // Low Tom
            InstrumentType::Rimshot => Some(37),     // Side Stick
            _ => None,
        }
    }

    /// Comfortable MIDI note range (low, high) for melodic instruments.
    /// For percussion, returns `(note, note)` with the GM drum note.
    pub fn midi_range(self) -> (u8, u8) {
        if let Some(note) = self.gm_drum_note() {
            return (note, note);
        }
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
            _ => unreachable!("all percussion handled by gm_drum_note"),
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

/// Which chord voices a strum hit targets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceTarget {
    /// All chord tones.
    #[default]
    All,
    /// Lowest chord tone only (bass note).
    Bass,
    /// Middle chord tones.
    Mid,
    /// Upper chord tones (top half).
    High,
    /// All except root.
    Upper,
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
    /// Which chord voices this hit targets.
    #[serde(default)]
    pub voice_target: VoiceTarget,
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

impl StrumPattern {
    /// Default "Folk Strum" pattern: D . D U . U D U over 4 beats.
    pub fn default_folk() -> Self {
        Self {
            name: "Folk Strum".to_string(),
            hits: vec![
                StrumHit { tick_offset: 0, direction: StrumDirection::Down, velocity_factor: 1.0, stagger_ms: 12.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 480, direction: StrumDirection::Down, velocity_factor: 0.8, stagger_ms: 10.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 720, direction: StrumDirection::Up, velocity_factor: 0.6, stagger_ms: 6.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 1200, direction: StrumDirection::Up, velocity_factor: 0.6, stagger_ms: 6.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 1440, direction: StrumDirection::Down, velocity_factor: 0.85, stagger_ms: 10.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 1680, direction: StrumDirection::Up, velocity_factor: 0.6, stagger_ms: 6.0, voice_target: VoiceTarget::All },
            ],
            beats: 4,
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

impl Track {
    /// Create a new track with empty patterns and no active parts.
    pub fn new(id: u8, name: &str, role: TrackRole, instrument: InstrumentType, voicing: Voicing) -> Self {
        Self {
            id,
            name: name.to_string(),
            role,
            instrument,
            voicing,
            muted: false,
            solo: false,
            patterns: HashMap::new(),
            automation: HashMap::new(),
            active_parts: HashMap::new(),
        }
    }
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

impl Song {
    /// Build the default song matching docs/reference/DEFAULTS.md.
    pub fn default_song() -> Self {
        use InstrumentType::*;
        use SongPart::*;
        use TrackRole::*;
        use Voicing::*;

        // Active-parts matrix from DEFAULTS.md  (I V P C B O)
        type TrackDef<'a> = (u8, &'a str, TrackRole, InstrumentType, Voicing, [bool; 6]);
        let layout: Vec<TrackDef<'_>> = vec![
            ( 0, "Kick",            Drum,         Kick,            Poly, [false, true,  true,  true,  true,  false]),
            ( 1, "Snare",           Drum,         Snare,           Poly, [false, true,  true,  true,  true,  false]),
            ( 2, "Hi-Hat",          Drum,         HiHat,           Poly, [false, true,  true,  true,  true,  false]),
            ( 3, "Tambourine",      Drum,         Tambourine,      Poly, [false, false, false, true,  false, false]),
            ( 4, "Acoustic Guitar", Rhythm,       AcousticGuitar,  Poly, [true,  true,  true,  true,  true,  true ]),
            ( 5, "Electric Guitar", Rhythm,       ElectricGuitar,  Poly, [false, false, true,  true,  false, false]),
            ( 6, "Electric Bass",   Bass,         ElectricBass,    Mono, [false, true,  true,  true,  true,  false]),
            ( 7, "Piano",           Rhythm,       Piano,           Poly, [true,  true,  true,  true,  true,  true ]),
            ( 8, "Pedal Steel",     LeadMelody,   PedalSteel,      Mono, [false, false, false, true,  true,  false]),
            ( 9, "Mandolin",        CounterMelody,Mandolin,        Mono, [false, true,  true,  true,  false, false]),
            (10, "Banjo",           Rhythm,       Banjo,           Poly, [false, false, false, true,  false, false]),
            (11, "Hammond Organ",   PadSustain,   HammondOrgan,    Poly, [false, false, true,  true,  true,  false]),
            (12, "Pad",             PadSustain,   Pad,             Poly, [true,  false, true,  false, true,  true ]),
            (13, "Lead Melody",     LeadMelody,   AcousticGuitar,  Mono, [false, true,  true,  true,  true,  false]),
            (14, "Counter Melody",  CounterMelody,Mandolin,        Mono, [false, false, false, true,  false, false]),
            (15, "Shaker",          Drum,         Shaker,          Poly, [false, false, false, true,  false, false]),
        ];

        let parts_order = [Intro, Verse, PreChorus, Chorus, Bridge, Outro];

        let tracks: Vec<Track> = layout
            .into_iter()
            .map(|(id, name, role, instrument, voicing, active)| {
                let mut t = Track::new(id, name, role, instrument, voicing);
                for (i, &part) in parts_order.iter().enumerate() {
                    t.active_parts.insert(part, active[i]);
                }
                t
            })
            .collect();

        let mut progressions = HashMap::new();
        progressions.insert(Intro, vec![ChordDegree::I]);
        progressions.insert(Verse, vec![ChordDegree::I, ChordDegree::V, ChordDegree::VI, ChordDegree::IV]);
        progressions.insert(PreChorus, vec![ChordDegree::IV, ChordDegree::V]);
        progressions.insert(Chorus, vec![ChordDegree::I, ChordDegree::V, ChordDegree::VI, ChordDegree::IV]);
        progressions.insert(Bridge, vec![ChordDegree::VI, ChordDegree::V, ChordDegree::IV]);
        progressions.insert(Outro, vec![ChordDegree::IV, ChordDegree::I]);

        let structure = vec![
            Intro, Verse, PreChorus, Chorus,
            Verse, PreChorus, Chorus,
            Bridge, Chorus, Outro,
        ];

        let rhythm_scale = Scale::new(PitchClass::As, ScaleType::Major); // Bb Major
        let mut lead_scale = Scale::new(PitchClass::G, ScaleType::MinorPentatonic);
        lead_scale.passing_tones.push(6); // C#/Db passing tone

        Self {
            title: "Untitled Folk Song".to_string(),
            tempo: 120.0,
            time_signature: (4, 4),
            rhythm_scale,
            lead_scale,
            tracks,
            structure,
            progressions,
            strum_pattern: StrumPattern::default_folk(),
            swing: 0.0,
        }
    }

    /// Total bar count from the song structure.
    pub fn total_bars(&self) -> u32 {
        self.structure.iter().map(|p| p.typical_bars()).sum()
    }

    /// Total length in ticks.
    pub fn total_ticks(&self) -> u32 {
        self.total_bars() * super::TICKS_PER_BAR
    }
}

impl std::fmt::Display for SongPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SongPart::Intro => write!(f, "Intro"),
            SongPart::Verse => write!(f, "Verse"),
            SongPart::PreChorus => write!(f, "PreChorus"),
            SongPart::Chorus => write!(f, "Chorus"),
            SongPart::Bridge => write!(f, "Bridge"),
            SongPart::Outro => write!(f, "Outro"),
        }
    }
}

impl std::fmt::Display for TrackRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackRole::Rhythm => write!(f, "Rhythm"),
            TrackRole::LeadMelody => write!(f, "Lead"),
            TrackRole::CounterMelody => write!(f, "Counter"),
            TrackRole::Bass => write!(f, "Bass"),
            TrackRole::Drum => write!(f, "Drum"),
            TrackRole::PadSustain => write!(f, "Pad"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn song_part_typical_bars() {
        assert_eq!(SongPart::Intro.typical_bars(), 4);
        assert_eq!(SongPart::Verse.typical_bars(), 8);
        assert_eq!(SongPart::PreChorus.typical_bars(), 4);
        assert_eq!(SongPart::Chorus.typical_bars(), 8);
        assert_eq!(SongPart::Bridge.typical_bars(), 8);
        assert_eq!(SongPart::Outro.typical_bars(), 4);
    }

    #[test]
    fn song_part_from_str() {
        assert_eq!("intro".parse::<SongPart>().unwrap(), SongPart::Intro);
        assert_eq!("prechorus".parse::<SongPart>().unwrap(), SongPart::PreChorus);
        assert_eq!("pre_chorus".parse::<SongPart>().unwrap(), SongPart::PreChorus);
        assert!("unknown".parse::<SongPart>().is_err());
    }

    #[test]
    fn song_part_display() {
        assert_eq!(SongPart::Intro.to_string(), "Intro");
        assert_eq!(SongPart::PreChorus.to_string(), "PreChorus");
    }

    #[test]
    fn song_part_serde_roundtrip() {
        let part = SongPart::PreChorus;
        let json = serde_json::to_string(&part).unwrap();
        assert_eq!(json, r#""prechorus""#);
        let parsed: SongPart = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, part);
    }

    #[test]
    fn instrument_type_percussion() {
        assert!(InstrumentType::Kick.is_percussion());
        assert!(InstrumentType::Snare.is_percussion());
        assert!(InstrumentType::Shaker.is_percussion());
        assert!(!InstrumentType::AcousticGuitar.is_percussion());
        assert!(!InstrumentType::Piano.is_percussion());
    }

    #[test]
    fn instrument_type_midi_range() {
        assert_eq!(InstrumentType::AcousticGuitar.midi_range(), (40, 79));
        assert_eq!(InstrumentType::ElectricBass.midi_range(), (28, 55));
        // Percussion instruments return their specific GM drum note
        assert_eq!(InstrumentType::Kick.midi_range(), (36, 36));
        assert_eq!(InstrumentType::Snare.midi_range(), (38, 38));
        assert_eq!(InstrumentType::HiHat.midi_range(), (42, 42));
    }

    #[test]
    fn percussion_gm_drum_note_mapping() {
        // Each percussion instrument maps to its correct GM drum note
        assert_eq!(InstrumentType::Kick.gm_drum_note(), Some(36));
        assert_eq!(InstrumentType::Snare.gm_drum_note(), Some(38));
        assert_eq!(InstrumentType::HiHat.gm_drum_note(), Some(42));
        assert_eq!(InstrumentType::OpenHiHat.gm_drum_note(), Some(46));
        assert_eq!(InstrumentType::Clap.gm_drum_note(), Some(39));
        assert_eq!(InstrumentType::Tambourine.gm_drum_note(), Some(54));
        assert_eq!(InstrumentType::Cowbell.gm_drum_note(), Some(56));
        assert_eq!(InstrumentType::Shaker.gm_drum_note(), Some(70));
        assert_eq!(InstrumentType::RideCymbal.gm_drum_note(), Some(51));
        assert_eq!(InstrumentType::CrashCymbal.gm_drum_note(), Some(49));
        assert_eq!(InstrumentType::Toms.gm_drum_note(), Some(45));
        assert_eq!(InstrumentType::Rimshot.gm_drum_note(), Some(37));
        // Melodic instruments return None
        assert_eq!(InstrumentType::AcousticGuitar.gm_drum_note(), None);
        assert_eq!(InstrumentType::Piano.gm_drum_note(), None);
    }

    #[test]
    fn percussion_instruments_have_unique_notes() {
        let percussion = [
            InstrumentType::Kick, InstrumentType::Snare, InstrumentType::HiHat,
            InstrumentType::OpenHiHat, InstrumentType::Clap, InstrumentType::Tambourine,
            InstrumentType::Cowbell, InstrumentType::Shaker, InstrumentType::RideCymbal,
            InstrumentType::CrashCymbal, InstrumentType::Toms, InstrumentType::Rimshot,
        ];
        let mut notes: Vec<u8> = percussion.iter().map(|i| i.gm_drum_note().unwrap()).collect();
        let len_before = notes.len();
        notes.sort();
        notes.dedup();
        assert_eq!(notes.len(), len_before, "all percussion instruments must have unique GM notes");
    }

    #[test]
    fn track_role_serde_roundtrip() {
        let role = TrackRole::LeadMelody;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, r#""lead""#);
        let parsed: TrackRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, role);

        let role2 = TrackRole::PadSustain;
        let json2 = serde_json::to_string(&role2).unwrap();
        assert_eq!(json2, r#""pad""#);
    }

    #[test]
    fn voicing_serde_roundtrip() {
        let v = Voicing::Mono;
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, r#""mono""#);
        let parsed: Voicing = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, v);
    }

    #[test]
    fn note_event_serde_roundtrip() {
        let event = NoteEvent {
            tick: 480,
            note: 60,
            velocity: 100,
            duration: 240,
            channel: 0,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: NoteEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, event);
    }

    #[test]
    fn cc_event_serde_roundtrip() {
        let event = CcEvent {
            tick: 0,
            cc: 255,
            value: 8192,
            channel: 5,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: CcEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, event);
    }

    #[test]
    fn pattern_empty_length() {
        let p = Pattern::empty(4);
        assert_eq!(p.bars, 4);
        assert_eq!(p.length_ticks, 4 * crate::engine::TICKS_PER_BAR);
        assert!(p.events.is_empty());
        assert!(p.cc_events.is_empty());
    }

    #[test]
    fn pattern_serde_roundtrip() {
        let mut p = Pattern::empty(2);
        p.events.push(NoteEvent {
            tick: 0,
            note: 60,
            velocity: 80,
            duration: 480,
            channel: 0,
        });
        let json = serde_json::to_string(&p).unwrap();
        let parsed: Pattern = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn strum_pattern_default_folk() {
        let sp = StrumPattern::default_folk();
        assert_eq!(sp.name, "Folk Strum");
        assert_eq!(sp.beats, 4);
        assert_eq!(sp.hits.len(), 6);
        assert_eq!(sp.hits[0].direction, StrumDirection::Down);
        assert!((sp.hits[0].velocity_factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn track_new() {
        let t = Track::new(4, "Acoustic Guitar", TrackRole::Rhythm, InstrumentType::AcousticGuitar, Voicing::Poly);
        assert_eq!(t.id, 4);
        assert_eq!(t.name, "Acoustic Guitar");
        assert_eq!(t.role, TrackRole::Rhythm);
        assert!(!t.muted);
        assert!(!t.solo);
        assert!(t.patterns.is_empty());
    }

    #[test]
    fn track_serde_roundtrip() {
        let t = Track::new(0, "Kick", TrackRole::Drum, InstrumentType::Kick, Voicing::Poly);
        let json = serde_json::to_string(&t).unwrap();
        let parsed: Track = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn default_song_has_16_tracks() {
        let song = Song::default_song();
        assert_eq!(song.tracks.len(), 16);
        for (i, track) in song.tracks.iter().enumerate() {
            assert_eq!(track.id, i as u8);
        }
    }

    #[test]
    fn default_song_structure() {
        let song = Song::default_song();
        assert_eq!(
            song.structure,
            vec![
                SongPart::Intro, SongPart::Verse, SongPart::PreChorus, SongPart::Chorus,
                SongPart::Verse, SongPart::PreChorus, SongPart::Chorus,
                SongPart::Bridge, SongPart::Chorus, SongPart::Outro,
            ]
        );
    }

    #[test]
    fn default_song_total_bars() {
        let song = Song::default_song();
        // 4+8+4+8+8+4+8+8+8+4 = 64 bars  (Verse/PreChorus/Chorus appear twice)
        // Wait: Intro=4, Verse=8, PreChorus=4, Chorus=8, Verse=8, PreChorus=4, Chorus=8, Bridge=8, Chorus=8, Outro=4 = 64
        assert_eq!(song.total_bars(), 64);
    }

    #[test]
    fn default_song_defaults() {
        let song = Song::default_song();
        assert_eq!(song.title, "Untitled Folk Song");
        assert!((song.tempo - 120.0).abs() < f64::EPSILON);
        assert_eq!(song.time_signature, (4, 4));
        assert!((song.swing - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_song_scales() {
        let song = Song::default_song();
        use super::super::theory::{PitchClass, ScaleType};
        assert_eq!(song.rhythm_scale.root, PitchClass::As); // Bb
        assert_eq!(song.rhythm_scale.scale_type, ScaleType::Major);
        assert_eq!(song.lead_scale.root, PitchClass::G);
        assert_eq!(song.lead_scale.scale_type, ScaleType::MinorPentatonic);
        assert_eq!(song.lead_scale.passing_tones, vec![6]);
    }

    #[test]
    fn default_song_progressions() {
        let song = Song::default_song();
        let verse_prog = song.progressions.get(&SongPart::Verse).unwrap();
        assert_eq!(
            verse_prog,
            &vec![ChordDegree::I, ChordDegree::V, ChordDegree::VI, ChordDegree::IV]
        );
    }

    #[test]
    fn default_song_active_parts() {
        let song = Song::default_song();
        // Acoustic Guitar (ch 4) is active in all parts
        let ag = &song.tracks[4];
        assert_eq!(ag.active_parts.get(&SongPart::Intro), Some(&true));
        assert_eq!(ag.active_parts.get(&SongPart::Outro), Some(&true));
        // Kick (ch 0) is silent in intro and outro
        let kick = &song.tracks[0];
        assert_eq!(kick.active_parts.get(&SongPart::Intro), Some(&false));
        assert_eq!(kick.active_parts.get(&SongPart::Verse), Some(&true));
    }

    #[test]
    fn song_serde_roundtrip() {
        let song = Song::default_song();
        let json = serde_json::to_string(&song).unwrap();
        let parsed: Song = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, song);
    }
}
