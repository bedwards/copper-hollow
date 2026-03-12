use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// PitchClass
// ---------------------------------------------------------------------------

/// The 12 pitch classes of Western music.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PitchClass {
    C = 0,
    Cs = 1,
    D = 2,
    Ds = 3,
    E = 4,
    F = 5,
    Fs = 6,
    G = 7,
    Gs = 8,
    A = 9,
    As = 10,
    B = 11,
}

impl PitchClass {
    /// All 12 pitch classes in chromatic order.
    pub const ALL: [PitchClass; 12] = [
        PitchClass::C,
        PitchClass::Cs,
        PitchClass::D,
        PitchClass::Ds,
        PitchClass::E,
        PitchClass::F,
        PitchClass::Fs,
        PitchClass::G,
        PitchClass::Gs,
        PitchClass::A,
        PitchClass::As,
        PitchClass::B,
    ];

    /// Convert a MIDI note number to its pitch class.
    pub fn from_midi(note: u8) -> Self {
        PitchClass::ALL[(note % 12) as usize]
    }

    /// Semitone offset from C (0–11).
    pub fn to_semitone(self) -> u8 {
        self as u8
    }

    /// Transpose by a signed number of semitones.
    pub fn transpose(self, semitones: i8) -> Self {
        let val = (self.to_semitone() as i16 + semitones as i16).rem_euclid(12) as u8;
        PitchClass::ALL[val as usize]
    }
}

/// Display uses standard naming: C, C#, D, Eb, E, F, F#, G, Ab, A, Bb, B.
impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PitchClass::C => "C",
            PitchClass::Cs => "C#",
            PitchClass::D => "D",
            PitchClass::Ds => "Eb",
            PitchClass::E => "E",
            PitchClass::F => "F",
            PitchClass::Fs => "F#",
            PitchClass::G => "G",
            PitchClass::Gs => "Ab",
            PitchClass::A => "A",
            PitchClass::As => "Bb",
            PitchClass::B => "B",
        };
        write!(f, "{name}")
    }
}

impl FromStr for PitchClass {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(PitchClass::C),
            "C#" | "Db" => Ok(PitchClass::Cs),
            "D" => Ok(PitchClass::D),
            "D#" | "Eb" => Ok(PitchClass::Ds),
            "E" => Ok(PitchClass::E),
            "F" => Ok(PitchClass::F),
            "F#" | "Gb" => Ok(PitchClass::Fs),
            "G" => Ok(PitchClass::G),
            "G#" | "Ab" => Ok(PitchClass::Gs),
            "A" => Ok(PitchClass::A),
            "A#" | "Bb" => Ok(PitchClass::As),
            "B" => Ok(PitchClass::B),
            _ => Err(anyhow::anyhow!("unknown pitch class: {s}")),
        }
    }
}

impl Serialize for PitchClass {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PitchClass {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// ScaleType
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleType {
    Major,
    NaturalMinor,
    HarmonicMinor,
    Dorian,
    Mixolydian,
    MinorPentatonic,
    Blues,
}

impl ScaleType {
    /// Semitone intervals from root for this scale.
    pub fn intervals(self) -> &'static [u8] {
        match self {
            ScaleType::Major => &[0, 2, 4, 5, 7, 9, 11],
            ScaleType::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            ScaleType::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            ScaleType::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            ScaleType::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            ScaleType::MinorPentatonic => &[0, 3, 5, 7, 10],
            ScaleType::Blues => &[0, 3, 5, 6, 7, 10],
        }
    }

    /// For scales with fewer than 7 notes, return the parent diatonic intervals
    /// used for chord derivation.
    pub fn parent_diatonic_intervals(self) -> &'static [u8] {
        match self {
            ScaleType::MinorPentatonic | ScaleType::Blues => ScaleType::NaturalMinor.intervals(),
            _ => self.intervals(),
        }
    }
}

impl FromStr for ScaleType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "major" => Ok(ScaleType::Major),
            "natural_minor" | "minor" => Ok(ScaleType::NaturalMinor),
            "harmonic_minor" => Ok(ScaleType::HarmonicMinor),
            "dorian" => Ok(ScaleType::Dorian),
            "mixolydian" => Ok(ScaleType::Mixolydian),
            "minor_pentatonic" => Ok(ScaleType::MinorPentatonic),
            "blues" => Ok(ScaleType::Blues),
            _ => Err(anyhow::anyhow!("unknown scale type: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Scale
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub root: PitchClass,
    pub scale_type: ScaleType,
    /// Extra semitone offsets from root that can be toggled on.
    pub passing_tones: Vec<u8>,
    /// Per-degree toggle; index 0 (root) is always treated as enabled.
    pub enabled_degrees: Vec<bool>,
}

impl Scale {
    pub fn new(root: PitchClass, scale_type: ScaleType) -> Self {
        let n = scale_type.intervals().len();
        Self {
            root,
            scale_type,
            passing_tones: Vec::new(),
            enabled_degrees: vec![true; n],
        }
    }

    /// Concrete pitch classes for the enabled degrees.
    pub fn pitch_classes(&self) -> Vec<PitchClass> {
        self.scale_type
            .intervals()
            .iter()
            .enumerate()
            .filter(|(i, _)| self.enabled_degrees.get(*i).copied().unwrap_or(true))
            .map(|(_, &interval)| self.root.transpose(interval as i8))
            .collect()
    }

    /// Derive diatonic triads for each of the 7 scale degrees.
    /// For pentatonic/blues, uses the parent diatonic scale.
    pub fn diatonic_chords(&self) -> Vec<(ChordDegree, ChordQuality)> {
        let intervals = self.scale_type.parent_diatonic_intervals();
        if intervals.len() < 7 {
            return Vec::new();
        }

        ChordDegree::ALL
            .iter()
            .enumerate()
            .map(|(i, &degree)| {
                let root = intervals[i] as i16;

                let third = {
                    let mut v = intervals[(i + 2) % 7] as i16;
                    if v <= root {
                        v += 12;
                    }
                    v
                };
                let fifth = {
                    let mut v = intervals[(i + 4) % 7] as i16;
                    if v <= root {
                        v += 12;
                    }
                    v
                };

                let quality = match ((third - root) as u8, (fifth - root) as u8) {
                    (4, 7) => ChordQuality::Major,
                    (3, 7) => ChordQuality::Minor,
                    (3, 6) => ChordQuality::Diminished,
                    (4, 8) => ChordQuality::Augmented,
                    _ => ChordQuality::Major,
                };

                (degree, quality)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// ChordQuality
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
    Major7,
    Minor7,
    Dominant7,
    Add9,
}

impl ChordQuality {
    /// Semitone offsets from root for this chord quality.
    pub fn intervals(self) -> &'static [u8] {
        match self {
            ChordQuality::Major => &[0, 4, 7],
            ChordQuality::Minor => &[0, 3, 7],
            ChordQuality::Diminished => &[0, 3, 6],
            ChordQuality::Augmented => &[0, 4, 8],
            ChordQuality::Sus2 => &[0, 2, 7],
            ChordQuality::Sus4 => &[0, 5, 7],
            ChordQuality::Major7 => &[0, 4, 7, 11],
            ChordQuality::Minor7 => &[0, 3, 7, 10],
            ChordQuality::Dominant7 => &[0, 4, 7, 10],
            ChordQuality::Add9 => &[0, 4, 7, 14],
        }
    }
}

// ---------------------------------------------------------------------------
// ChordDegree
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum ChordDegree {
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
}

impl ChordDegree {
    pub const ALL: [ChordDegree; 7] = [
        ChordDegree::I,
        ChordDegree::II,
        ChordDegree::III,
        ChordDegree::IV,
        ChordDegree::V,
        ChordDegree::VI,
        ChordDegree::VII,
    ];

    /// Zero-based index (I=0, VII=6).
    pub fn to_index(self) -> usize {
        self as usize
    }
}

impl fmt::Display for ChordDegree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ChordDegree::I => "I",
            ChordDegree::II => "II",
            ChordDegree::III => "III",
            ChordDegree::IV => "IV",
            ChordDegree::V => "V",
            ChordDegree::VI => "VI",
            ChordDegree::VII => "VII",
        };
        write!(f, "{s}")
    }
}

impl FromStr for ChordDegree {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "I" => Ok(ChordDegree::I),
            "II" => Ok(ChordDegree::II),
            "III" => Ok(ChordDegree::III),
            "IV" => Ok(ChordDegree::IV),
            "V" => Ok(ChordDegree::V),
            "VI" => Ok(ChordDegree::VI),
            "VII" => Ok(ChordDegree::VII),
            _ => Err(anyhow::anyhow!("unknown chord degree: {s}")),
        }
    }
}

impl Serialize for ChordDegree {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ChordDegree {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Chord
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chord {
    pub root: PitchClass,
    pub quality: ChordQuality,
    pub degree: ChordDegree,
    /// 0 = root position, 1 = first inversion, 2 = second inversion.
    pub inversion: u8,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- PitchClass --------------------------------------------------------

    #[test]
    fn pitch_class_from_midi() {
        assert_eq!(PitchClass::from_midi(60), PitchClass::C); // Middle C
        assert_eq!(PitchClass::from_midi(61), PitchClass::Cs);
        assert_eq!(PitchClass::from_midi(69), PitchClass::A); // A440
        assert_eq!(PitchClass::from_midi(0), PitchClass::C);
        assert_eq!(PitchClass::from_midi(127), PitchClass::G); // 127 % 12 = 7
    }

    #[test]
    fn pitch_class_to_semitone() {
        assert_eq!(PitchClass::C.to_semitone(), 0);
        assert_eq!(PitchClass::Fs.to_semitone(), 6);
        assert_eq!(PitchClass::B.to_semitone(), 11);
    }

    #[test]
    fn pitch_class_transpose() {
        assert_eq!(PitchClass::C.transpose(4), PitchClass::E);
        assert_eq!(PitchClass::C.transpose(7), PitchClass::G);
        assert_eq!(PitchClass::G.transpose(5), PitchClass::C);
        assert_eq!(PitchClass::A.transpose(-3), PitchClass::Fs);
        assert_eq!(PitchClass::C.transpose(-1), PitchClass::B);
        assert_eq!(PitchClass::C.transpose(12), PitchClass::C);
        assert_eq!(PitchClass::C.transpose(-12), PitchClass::C);
    }

    #[test]
    fn pitch_class_display() {
        assert_eq!(PitchClass::As.to_string(), "Bb");
        assert_eq!(PitchClass::Ds.to_string(), "Eb");
        assert_eq!(PitchClass::Gs.to_string(), "Ab");
        assert_eq!(PitchClass::Cs.to_string(), "C#");
        assert_eq!(PitchClass::Fs.to_string(), "F#");
    }

    #[test]
    fn pitch_class_from_str() {
        assert_eq!("Bb".parse::<PitchClass>().unwrap(), PitchClass::As);
        assert_eq!("A#".parse::<PitchClass>().unwrap(), PitchClass::As);
        assert_eq!("C#".parse::<PitchClass>().unwrap(), PitchClass::Cs);
        assert_eq!("Db".parse::<PitchClass>().unwrap(), PitchClass::Cs);
        assert!("X".parse::<PitchClass>().is_err());
    }

    #[test]
    fn pitch_class_serde_roundtrip() {
        let pc = PitchClass::As;
        let json = serde_json::to_string(&pc).unwrap();
        assert_eq!(json, r#""Bb""#);
        let parsed: PitchClass = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pc);
    }

    // -- Scale construction ------------------------------------------------

    #[test]
    fn c_major_scale_notes() {
        let scale = Scale::new(PitchClass::C, ScaleType::Major);
        let notes = scale.pitch_classes();
        assert_eq!(
            notes,
            vec![
                PitchClass::C,
                PitchClass::D,
                PitchClass::E,
                PitchClass::F,
                PitchClass::G,
                PitchClass::A,
                PitchClass::B,
            ]
        );
    }

    #[test]
    fn g_natural_minor_scale_notes() {
        let scale = Scale::new(PitchClass::G, ScaleType::NaturalMinor);
        let notes = scale.pitch_classes();
        assert_eq!(
            notes,
            vec![
                PitchClass::G,
                PitchClass::A,
                PitchClass::As, // Bb
                PitchClass::C,
                PitchClass::D,
                PitchClass::Ds, // Eb
                PitchClass::F,
            ]
        );
    }

    #[test]
    fn bb_major_scale_notes() {
        let scale = Scale::new(PitchClass::As, ScaleType::Major);
        let notes = scale.pitch_classes();
        assert_eq!(
            notes,
            vec![
                PitchClass::As, // Bb
                PitchClass::C,
                PitchClass::D,
                PitchClass::Ds, // Eb
                PitchClass::F,
                PitchClass::G,
                PitchClass::A,
            ]
        );
    }

    #[test]
    fn g_minor_pentatonic_notes() {
        let scale = Scale::new(PitchClass::G, ScaleType::MinorPentatonic);
        let notes = scale.pitch_classes();
        assert_eq!(
            notes,
            vec![
                PitchClass::G,
                PitchClass::As, // Bb
                PitchClass::C,
                PitchClass::D,
                PitchClass::F,
            ]
        );
    }

    #[test]
    fn scale_with_disabled_degrees() {
        let mut scale = Scale::new(PitchClass::C, ScaleType::Major);
        scale.enabled_degrees[3] = false; // disable 4th degree (F)
        let notes = scale.pitch_classes();
        assert_eq!(
            notes,
            vec![
                PitchClass::C,
                PitchClass::D,
                PitchClass::E,
                // F is disabled
                PitchClass::G,
                PitchClass::A,
                PitchClass::B,
            ]
        );
    }

    // -- Diatonic chord derivation -----------------------------------------

    #[test]
    fn c_major_diatonic_chords() {
        let scale = Scale::new(PitchClass::C, ScaleType::Major);
        let chords = scale.diatonic_chords();
        let qualities: Vec<ChordQuality> = chords.iter().map(|(_, q)| *q).collect();
        assert_eq!(
            qualities,
            vec![
                ChordQuality::Major,      // I
                ChordQuality::Minor,      // ii
                ChordQuality::Minor,      // iii
                ChordQuality::Major,      // IV
                ChordQuality::Major,      // V
                ChordQuality::Minor,      // vi
                ChordQuality::Diminished, // vii
            ]
        );
    }

    #[test]
    fn a_natural_minor_diatonic_chords() {
        let scale = Scale::new(PitchClass::A, ScaleType::NaturalMinor);
        let chords = scale.diatonic_chords();
        let qualities: Vec<ChordQuality> = chords.iter().map(|(_, q)| *q).collect();
        assert_eq!(
            qualities,
            vec![
                ChordQuality::Minor,      // i
                ChordQuality::Diminished, // ii
                ChordQuality::Major,      // III
                ChordQuality::Minor,      // iv
                ChordQuality::Minor,      // v
                ChordQuality::Major,      // VI
                ChordQuality::Major,      // VII
            ]
        );
    }

    #[test]
    fn minor_pentatonic_derives_from_natural_minor() {
        let pent = Scale::new(PitchClass::G, ScaleType::MinorPentatonic);
        let minor = Scale::new(PitchClass::G, ScaleType::NaturalMinor);
        assert_eq!(pent.diatonic_chords(), minor.diatonic_chords());
    }

    // -- ChordQuality intervals --------------------------------------------

    #[test]
    fn chord_quality_intervals() {
        assert_eq!(ChordQuality::Major.intervals(), &[0, 4, 7]);
        assert_eq!(ChordQuality::Minor.intervals(), &[0, 3, 7]);
        assert_eq!(ChordQuality::Diminished.intervals(), &[0, 3, 6]);
        assert_eq!(ChordQuality::Dominant7.intervals(), &[0, 4, 7, 10]);
    }

    // -- ChordDegree -------------------------------------------------------

    #[test]
    fn chord_degree_index() {
        assert_eq!(ChordDegree::I.to_index(), 0);
        assert_eq!(ChordDegree::IV.to_index(), 3);
        assert_eq!(ChordDegree::VII.to_index(), 6);
    }

    #[test]
    fn chord_degree_serde_roundtrip() {
        let deg = ChordDegree::IV;
        let json = serde_json::to_string(&deg).unwrap();
        assert_eq!(json, r#""IV""#);
        let parsed: ChordDegree = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, deg);
    }

    // -- Scale serde -------------------------------------------------------

    #[test]
    fn scale_serde_roundtrip() {
        let scale = Scale::new(PitchClass::As, ScaleType::Major);
        let json = serde_json::to_string(&scale).unwrap();
        let parsed: Scale = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, scale);
    }
}
