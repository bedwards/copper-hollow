// Melodic contour generation, chord tone targeting, voice leading.
// Fully implemented — awaiting GUI integration (v0.4.0).
#![allow(dead_code)]

use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::song::{CcEvent, NoteEvent, Pattern, SongPart};
use super::theory::{Chord, PitchClass, Scale};
use super::{PITCH_BEND_CENTER, TICKS_PER_BAR, TICKS_PER_BEAT};

// ---------------------------------------------------------------------------
// ContourShape
// ---------------------------------------------------------------------------

/// Melodic contour shape for a phrase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContourShape {
    /// Rise to a peak around midpoint, then descend. Most common for verses.
    Arch,
    /// Start high, gradually descend. Creates resolution feel.
    Descending,
    /// Build upward. Creates tension. Good for pre-chorus.
    Ascending,
    /// Two smaller arches. Conversational feel for verses.
    Wave,
    /// Stay in a narrow range with occasional departures.
    Static,
}

impl ContourShape {
    pub const ALL: [ContourShape; 5] = [
        ContourShape::Arch,
        ContourShape::Descending,
        ContourShape::Ascending,
        ContourShape::Wave,
        ContourShape::Static,
    ];

    /// Contour offset in scale degrees for a normalized position in [0.0, 1.0].
    /// Positive = higher, negative = lower from center.
    pub fn offset_at(self, t: f64) -> f64 {
        match self {
            ContourShape::Arch => {
                let x = t * 2.0 - 1.0;
                3.0 * (1.0 - x * x)
            }
            ContourShape::Descending => 3.0 - 5.0 * t,
            ContourShape::Ascending => -2.0 + 5.0 * t,
            ContourShape::Wave => 2.0 * (2.0 * std::f64::consts::PI * t).sin(),
            ContourShape::Static => 0.5 * (4.0 * std::f64::consts::PI * t).sin(),
        }
    }
}

// ---------------------------------------------------------------------------
// MelodyDensity
// ---------------------------------------------------------------------------

/// Rhythmic density for melody: how many notes per bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MelodyDensity {
    /// 3-4 notes per bar.
    Sparse,
    /// 5-7 notes per bar.
    Medium,
    /// 8-12 notes per bar.
    Dense,
}

impl MelodyDensity {
    /// Choose density based on song part.
    pub fn for_part(part: SongPart) -> Self {
        match part {
            SongPart::Intro | SongPart::Outro | SongPart::Bridge => MelodyDensity::Sparse,
            SongPart::Verse => MelodyDensity::Medium,
            SongPart::PreChorus | SongPart::Chorus => MelodyDensity::Dense,
        }
    }

    /// Note count range (min, max) for this density.
    pub fn note_range(self) -> (u32, u32) {
        match self {
            MelodyDensity::Sparse => (3, 4),
            MelodyDensity::Medium => (5, 7),
            MelodyDensity::Dense => (8, 12),
        }
    }
}

// ---------------------------------------------------------------------------
// MotionType
// ---------------------------------------------------------------------------

/// Type of melodic motion between consecutive notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionType {
    /// Move by one scale degree. 70% probability.
    Step,
    /// Skip one scale degree (a third). 20% probability.
    SmallLeap,
    /// Fourth, fifth, or octave. 10% probability. Must resolve by step.
    LargeLeap,
    /// Same pitch repeated. Max 2-3 repetitions.
    Repeat,
}

// ---------------------------------------------------------------------------
// MelodyConfig
// ---------------------------------------------------------------------------

/// Configuration for melody generation.
#[derive(Clone, Debug)]
pub struct MelodyConfig<'a> {
    pub scale: &'a Scale,
    /// One chord per bar.
    pub chords_per_bar: &'a [Chord],
    pub part: SongPart,
    pub channel: u8,
    /// MIDI note range (low, high) for the instrument.
    pub range: (u8, u8),
    /// Optional contour shape override (random if None).
    pub contour: Option<ContourShape>,
}

// ---------------------------------------------------------------------------
// MelodyEngine
// ---------------------------------------------------------------------------

/// Melody engine for generating lead and counter-melody lines with contour,
/// chord tone targeting, and voice leading. Fully deterministic given the same seed.
pub struct MelodyEngine {
    rng: ChaCha8Rng,
}

impl MelodyEngine {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    // -- Contour selection --------------------------------------------------

    /// Choose a contour shape weighted by song part.
    fn choose_contour(&mut self, part: SongPart) -> ContourShape {
        let r: f64 = self.rng.random();
        match part {
            SongPart::Verse => {
                if r < 0.40 {
                    ContourShape::Arch
                } else if r < 0.70 {
                    ContourShape::Wave
                } else if r < 0.85 {
                    ContourShape::Descending
                } else {
                    ContourShape::Static
                }
            }
            SongPart::Chorus => {
                if r < 0.35 {
                    ContourShape::Arch
                } else if r < 0.55 {
                    ContourShape::Ascending
                } else if r < 0.75 {
                    ContourShape::Wave
                } else {
                    ContourShape::Descending
                }
            }
            SongPart::PreChorus => {
                if r < 0.50 {
                    ContourShape::Ascending
                } else if r < 0.75 {
                    ContourShape::Arch
                } else {
                    ContourShape::Wave
                }
            }
            SongPart::Bridge => {
                if r < 0.30 {
                    ContourShape::Descending
                } else if r < 0.60 {
                    ContourShape::Static
                } else {
                    ContourShape::Arch
                }
            }
            SongPart::Intro | SongPart::Outro => {
                if r < 0.40 {
                    ContourShape::Static
                } else if r < 0.70 {
                    ContourShape::Descending
                } else {
                    ContourShape::Arch
                }
            }
        }
    }

    // -- Motion selection ---------------------------------------------------

    /// Choose motion type using the spec's probability distribution.
    fn choose_motion(&mut self) -> MotionType {
        let r: f64 = self.rng.random();
        if r < 0.70 {
            MotionType::Step
        } else if r < 0.90 {
            MotionType::SmallLeap
        } else {
            MotionType::LargeLeap
        }
    }

    // -- Pitch helpers ------------------------------------------------------

    /// Build a sorted list of MIDI note numbers in the scale within the given range.
    fn build_pitch_table(scale: &Scale, range: (u8, u8)) -> Vec<u8> {
        let pcs = scale.pitch_classes();
        let mut notes = Vec::new();
        for midi in range.0..=range.1 {
            let pc = PitchClass::from_midi(midi);
            if pcs.contains(&pc) {
                notes.push(midi);
            }
        }
        notes
    }

    /// Find the index in pitch_table of the note closest to the given MIDI pitch.
    fn closest_index(pitch_table: &[u8], target: u8) -> usize {
        pitch_table
            .iter()
            .enumerate()
            .min_by_key(|(_, &n)| (n as i16 - target as i16).unsigned_abs())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Check if a MIDI note is a chord tone.
    fn is_chord_tone(note: u8, chord: &Chord) -> bool {
        let pc = PitchClass::from_midi(note);
        chord.notes().contains(&pc)
    }

    /// Find the index of the nearest chord tone in the pitch table.
    fn nearest_chord_tone_index(pitch_table: &[u8], from_index: usize, chord: &Chord) -> usize {
        let mut best = from_index;
        let mut best_dist = usize::MAX;
        for (i, &note) in pitch_table.iter().enumerate() {
            if Self::is_chord_tone(note, chord) {
                let dist = from_index.abs_diff(i);
                if dist < best_dist {
                    best_dist = dist;
                    best = i;
                }
            }
        }
        best
    }

    // -- Rhythm generation --------------------------------------------------

    /// Duration palette by density.
    fn duration_palette(density: MelodyDensity) -> &'static [u32] {
        match density {
            MelodyDensity::Sparse => &[480, 720, 960, 480],
            MelodyDensity::Medium => &[240, 480, 720, 480, 240],
            MelodyDensity::Dense => &[120, 240, 240, 120, 240, 480],
        }
    }

    /// Generate rhythmic placements (tick, duration) for a single bar.
    fn generate_bar_rhythm(
        &mut self,
        bar_offset: u32,
        density: MelodyDensity,
    ) -> Vec<(u32, u32)> {
        let (min_notes, max_notes) = density.note_range();
        let note_count = self.rng.random_range(min_notes..=max_notes);
        let palette = Self::duration_palette(density);
        let bar_end = bar_offset + TICKS_PER_BAR;

        let mut placements = Vec::new();
        let mut tick = bar_offset;
        let mut rest_inserted = false;

        for i in 0..note_count {
            if tick >= bar_end {
                break;
            }

            if i > 0 {
                // 20% chance of syncopation: start on the "and" before next beat
                if self.rng.random::<f64>() < 0.20 {
                    let pos_in_bar = tick - bar_offset;
                    let next_beat =
                        (pos_in_bar / TICKS_PER_BEAT + 1) * TICKS_PER_BEAT + bar_offset;
                    if next_beat >= tick + 240 && next_beat <= bar_end {
                        tick = next_beat - 240;
                    }
                }

                // Insert rest for breathing (~40% chance per bar)
                if !rest_inserted && self.rng.random::<f64>() < 0.40 && tick + 240 < bar_end {
                    tick += 240;
                    rest_inserted = true;
                }
            }

            if tick >= bar_end {
                break;
            }

            let dur_idx = self.rng.random_range(0..palette.len());
            let dur = palette[dur_idx].min(bar_end - tick);
            if dur > 0 {
                placements.push((tick, dur));
                tick += dur;
            }
        }

        placements
    }

    // -- Pitch motion -------------------------------------------------------

    /// Apply motion rules to get the next pitch index.
    /// Returns (new_index, motion_used, direction).
    fn apply_motion(
        &mut self,
        current: usize,
        table_len: usize,
        contour_offset: f64,
        last_motion: MotionType,
        last_direction: i32,
        repeat_count: u8,
    ) -> (usize, MotionType, i32) {
        if table_len <= 1 {
            return (0, MotionType::Repeat, 0);
        }
        let max_idx = table_len - 1;

        // Direction from contour
        let dir: i32 = if contour_offset.abs() < 0.3 {
            if self.rng.random::<bool>() {
                1
            } else {
                -1
            }
        } else if contour_offset > 0.0 {
            1
        } else {
            -1
        };

        // After a large leap, must resolve by step in the opposite direction
        if last_motion == MotionType::LargeLeap {
            let resolve_dir = -last_direction;
            let new = (current as i32 + resolve_dir).clamp(0, max_idx as i32) as usize;
            return (new, MotionType::Step, resolve_dir);
        }

        // Too many repeats: force a step
        if repeat_count >= 2 {
            let new = (current as i32 + dir).clamp(0, max_idx as i32) as usize;
            return (new, MotionType::Step, dir);
        }

        let motion = self.choose_motion();
        let step = match motion {
            MotionType::Step => 1,
            MotionType::SmallLeap => 2,
            MotionType::LargeLeap => {
                const LARGE_LEAP_INTERVALS: [i32; 3] = [3, 4, 7]; // 4th, 5th, octave
                LARGE_LEAP_INTERVALS[self.rng.random_range(0..3)]
            }
            MotionType::Repeat => 0,
        };

        if step == 0 {
            return (current, MotionType::Repeat, 0);
        }

        let new = (current as i32 + step * dir).clamp(0, max_idx as i32) as usize;
        (new, motion, dir)
    }

    // -- Lead melody generation ---------------------------------------------

    /// Generate a lead melody pattern across the given bars.
    /// Processes bars in 2-bar phrases, each with its own contour.
    pub fn generate_melody(&mut self, config: &MelodyConfig<'_>) -> Pattern {
        let bars = config.chords_per_bar.len() as u32;
        if bars == 0 {
            return Pattern::empty(0);
        }

        let pitch_table = Self::build_pitch_table(config.scale, config.range);
        if pitch_table.is_empty() {
            return Pattern::empty(bars);
        }

        let density = MelodyDensity::for_part(config.part);
        let total_ticks = bars * TICKS_PER_BAR;

        // Start near the middle of the range
        let center = (config.range.0 / 2).saturating_add(config.range.1 / 2);
        let mut idx = Self::closest_index(&pitch_table, center);
        let mut last_motion = MotionType::Step;
        let mut last_dir: i32 = 1;
        let mut repeat_count: u8 = 0;
        let mut is_first_note = true;

        let mut events = Vec::new();
        let mut cc_events = Vec::new();

        // Process in 2-bar phrases
        let phrase_len = 2u32;
        let num_phrases = bars.div_ceil(phrase_len);

        for phrase_idx in 0..num_phrases {
            let phrase_start_bar = phrase_idx * phrase_len;
            let phrase_end_bar = (phrase_start_bar + phrase_len).min(bars);
            let phrase_bars = phrase_end_bar - phrase_start_bar;
            let phrase_ticks = phrase_bars * TICKS_PER_BAR;
            let phrase_tick_offset = phrase_start_bar * TICKS_PER_BAR;

            let contour = config
                .contour
                .unwrap_or_else(|| self.choose_contour(config.part));

            // Consequent phrases (odd) resolve to root/third on last note
            let is_consequent = phrase_idx % 2 == 1;

            for bar_rel in 0..phrase_bars {
                let bar_idx = phrase_start_bar + bar_rel;
                let bar_offset = bar_idx * TICKS_PER_BAR;
                let chord = &config.chords_per_bar[bar_idx as usize];
                let placements = self.generate_bar_rhythm(bar_offset, density);
                let placement_count = placements.len();

                for (note_idx, &(tick, duration)) in placements.iter().enumerate() {
                    // Position within this phrase for contour
                    let phrase_tick = tick.saturating_sub(phrase_tick_offset);
                    let t = if phrase_ticks > 0 {
                        phrase_tick as f64 / phrase_ticks as f64
                    } else {
                        0.0
                    };
                    let contour_offset = contour.offset_at(t);
                    let beat_in_bar = tick.saturating_sub(bar_offset) / TICKS_PER_BEAT;
                    let is_strong_beat = beat_in_bar == 0 || beat_in_bar == 2;

                    if is_first_note {
                        // First note: target a chord tone near center
                        idx = Self::nearest_chord_tone_index(&pitch_table, idx, chord);
                        is_first_note = false;
                    } else {
                        let (new_idx, motion, dir) = self.apply_motion(
                            idx,
                            pitch_table.len(),
                            contour_offset,
                            last_motion,
                            last_dir,
                            repeat_count,
                        );
                        idx = new_idx;
                        last_motion = motion;
                        if dir != 0 {
                            last_dir = dir;
                        }

                        // On strong beats, snap to chord tone if within 1 scale step
                        if is_strong_beat
                            && !Self::is_chord_tone(pitch_table[idx], chord)
                        {
                            let ct =
                                Self::nearest_chord_tone_index(&pitch_table, idx, chord);
                            if idx.abs_diff(ct) <= 1 {
                                idx = ct;
                            }
                        }
                    }

                    // Consequent phrase: resolve last note to root or third
                    let is_last_in_phrase =
                        bar_rel == phrase_bars - 1 && note_idx == placement_count - 1;
                    if is_consequent && is_last_in_phrase {
                        idx = Self::nearest_chord_tone_index(&pitch_table, idx, chord);
                    }

                    if last_motion == MotionType::Repeat {
                        repeat_count = repeat_count.saturating_add(1);
                    } else {
                        repeat_count = 0;
                    }

                    let base_vel: u8 = if is_strong_beat {
                        self.rng.random_range(90u8..=110).min(127)
                    } else {
                        self.rng.random_range(70u8..=95)
                    };

                    events.push(NoteEvent {
                        tick,
                        note: pitch_table[idx],
                        velocity: base_vel,
                        duration,
                        channel: config.channel,
                    });

                    // Grace note ornament (5-10% in chorus/bridge per spec)
                    let grace_prob = match config.part {
                        SongPart::Chorus | SongPart::Bridge => 0.08,
                        SongPart::PreChorus => 0.05,
                        _ => 0.02,
                    };
                    if self.rng.random::<f64>() < grace_prob && idx > 0 && tick >= 60 {
                        events.push(NoteEvent {
                            tick: tick - 60,
                            note: pitch_table[idx - 1],
                            velocity: ((base_vel as f32) * 0.75).round() as u8,
                            duration: 60,
                            channel: config.channel,
                        });
                    }

                    // Pitch bend slide ornament (3-5% per spec)
                    let slide_prob = match config.part {
                        SongPart::Chorus | SongPart::Bridge => 0.05,
                        _ => 0.03,
                    };
                    if self.rng.random::<f64>() < slide_prob && tick >= 120 {
                        let bend_start = PITCH_BEND_CENTER - (PITCH_BEND_CENTER / 6);
                        cc_events.push(CcEvent {
                            tick: tick - 120,
                            cc: 255,
                            value: bend_start,
                            channel: config.channel,
                        });
                        cc_events.push(CcEvent {
                            tick,
                            cc: 255,
                            value: PITCH_BEND_CENTER,
                            channel: config.channel,
                        });
                    }
                }
            }
        }

        events.sort_by_key(|e| e.tick);
        cc_events.sort_by_key(|e| e.tick);

        Pattern {
            events,
            cc_events,
            length_ticks: total_ticks,
            bars,
        }
    }

    // -- Counter-melody generation ------------------------------------------

    /// Generate a counter-melody that is rhythmically independent from the lead.
    /// Sparser than lead, in a separated register, preferring 3rds/6ths when
    /// sounding simultaneously with the lead.
    pub fn generate_counter_melody(
        &mut self,
        config: &MelodyConfig<'_>,
        lead: &Pattern,
    ) -> Pattern {
        let bars = config.chords_per_bar.len() as u32;
        if bars == 0 {
            return Pattern::empty(0);
        }

        let pitch_table = Self::build_pitch_table(config.scale, config.range);
        if pitch_table.is_empty() {
            return Pattern::empty(bars);
        }

        let total_ticks = bars * TICKS_PER_BAR;

        // Counter-melody is sparser than lead
        let density = match MelodyDensity::for_part(config.part) {
            MelodyDensity::Dense => MelodyDensity::Medium,
            _ => MelodyDensity::Sparse,
        };

        // Register separation: counter goes to opposite end from lead average
        let lead_avg = if lead.events.is_empty() {
            (config.range.0 as f64 + config.range.1 as f64) / 2.0
        } else {
            let sum: u64 = lead.events.iter().map(|e| e.note as u64).sum();
            sum as f64 / lead.events.len() as f64
        };
        let range_mid = (config.range.0 as f64 + config.range.1 as f64) / 2.0;
        let counter_center: u8 = if lead_avg > range_mid {
            config.range.0.saturating_add(12).min(config.range.1)
        } else {
            config.range.1.saturating_sub(12).max(config.range.0)
        };

        let mut idx = Self::closest_index(&pitch_table, counter_center);
        let mut last_motion = MotionType::Step;
        let mut last_dir: i32 = 1;
        let mut repeat_count: u8 = 0;

        let lead_ticks: Vec<u32> = lead.events.iter().map(|e| e.tick).collect();
        let mut events = Vec::new();

        // Process in 2-bar phrases
        let phrase_len = 2u32;
        let num_phrases = bars.div_ceil(phrase_len);

        for phrase_idx in 0..num_phrases {
            let phrase_start_bar = phrase_idx * phrase_len;
            let phrase_end_bar = (phrase_start_bar + phrase_len).min(bars);
            let phrase_bars = phrase_end_bar - phrase_start_bar;
            let phrase_ticks = phrase_bars * TICKS_PER_BAR;
            let phrase_tick_offset = phrase_start_bar * TICKS_PER_BAR;

            let contour = config
                .contour
                .unwrap_or_else(|| self.choose_contour(config.part));

            for bar_rel in 0..phrase_bars {
                let bar_idx = phrase_start_bar + bar_rel;
                let bar_offset = bar_idx * TICKS_PER_BAR;
                let chord = &config.chords_per_bar[bar_idx as usize];
                let placements = self.generate_bar_rhythm(bar_offset, density);

                for &(tick, duration) in &placements {
                    // Rhythmic independence: 70% chance to skip when lead has a note nearby
                    let lead_nearby =
                        lead_ticks.iter().any(|&lt| lt.abs_diff(tick) < 120);
                    if lead_nearby && self.rng.random::<f64>() < 0.70 {
                        continue;
                    }

                    let phrase_tick = tick.saturating_sub(phrase_tick_offset);
                    let t = if phrase_ticks > 0 {
                        phrase_tick as f64 / phrase_ticks as f64
                    } else {
                        0.0
                    };
                    let contour_offset = contour.offset_at(t);
                    let beat_in_bar = tick.saturating_sub(bar_offset) / TICKS_PER_BEAT;
                    let is_strong_beat = beat_in_bar == 0 || beat_in_bar == 2;

                    let (new_idx, motion, dir) = self.apply_motion(
                        idx,
                        pitch_table.len(),
                        contour_offset,
                        last_motion,
                        last_dir,
                        repeat_count,
                    );
                    idx = new_idx;
                    last_motion = motion;
                    if dir != 0 {
                        last_dir = dir;
                    }

                    // On strong beats, snap to chord tone
                    if is_strong_beat && !Self::is_chord_tone(pitch_table[idx], chord) {
                        let ct =
                            Self::nearest_chord_tone_index(&pitch_table, idx, chord);
                        if idx.abs_diff(ct) <= 1 {
                            idx = ct;
                        }
                    }

                    // Harmonic interval check: avoid unisons and seconds with lead
                    if lead_nearby {
                        if let Some(lead_note) = lead
                            .events
                            .iter()
                            .find(|e| e.tick.abs_diff(tick) < 120)
                            .map(|e| e.note)
                        {
                            let interval = (pitch_table[idx] as i16 - lead_note as i16)
                                .unsigned_abs()
                                % 12;
                            if interval <= 2 {
                                let adj = if idx + 1 < pitch_table.len() {
                                    1i32
                                } else {
                                    -1
                                };
                                idx = (idx as i32 + adj)
                                    .clamp(0, (pitch_table.len() - 1) as i32)
                                    as usize;
                            }
                        }
                    }

                    if last_motion == MotionType::Repeat {
                        repeat_count = repeat_count.saturating_add(1);
                    } else {
                        repeat_count = 0;
                    }

                    let vel: u8 = if is_strong_beat {
                        self.rng.random_range(80u8..=100).min(127)
                    } else {
                        self.rng.random_range(65u8..=85)
                    };

                    events.push(NoteEvent {
                        tick,
                        note: pitch_table[idx],
                        velocity: vel,
                        duration,
                        channel: config.channel,
                    });
                }
            }
        }

        events.sort_by_key(|e| e.tick);

        Pattern {
            events,
            cc_events: Vec::new(),
            length_ticks: total_ticks,
            bars,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::theory::{ChordDegree, ChordQuality, ScaleType};

    fn c_major_scale() -> Scale {
        Scale::new(PitchClass::C, ScaleType::Major)
    }

    fn c_major_chord() -> Chord {
        Chord {
            root: PitchClass::C,
            quality: ChordQuality::Major,
            degree: ChordDegree::I,
            inversion: 0,
        }
    }

    fn test_config<'a>(scale: &'a Scale, chords: &'a [Chord]) -> MelodyConfig<'a> {
        MelodyConfig {
            scale,
            chords_per_bar: chords,
            part: SongPart::Verse,
            channel: 8,
            range: (48, 72),
            contour: None,
        }
    }

    // -- ContourShape -------------------------------------------------------

    #[test]
    fn contour_arch_peaks_at_midpoint() {
        let peak = ContourShape::Arch.offset_at(0.5);
        let start = ContourShape::Arch.offset_at(0.0);
        let end = ContourShape::Arch.offset_at(1.0);
        assert!(peak > start, "arch should peak above start");
        assert!(peak > end, "arch should peak above end");
        assert!((peak - 3.0).abs() < 0.01, "arch peak should be ~3.0");
    }

    #[test]
    fn contour_descending_goes_down() {
        let start = ContourShape::Descending.offset_at(0.0);
        let end = ContourShape::Descending.offset_at(1.0);
        assert!(start > end, "descending should go from high to low");
    }

    #[test]
    fn contour_ascending_goes_up() {
        let start = ContourShape::Ascending.offset_at(0.0);
        let end = ContourShape::Ascending.offset_at(1.0);
        assert!(end > start, "ascending should go from low to high");
    }

    #[test]
    fn contour_static_stays_narrow() {
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let offset = ContourShape::Static.offset_at(t);
            assert!(
                offset.abs() <= 1.0,
                "static contour should stay within ±1.0, got {}",
                offset
            );
        }
    }

    #[test]
    fn contour_all_has_five_variants() {
        assert_eq!(ContourShape::ALL.len(), 5);
    }

    // -- MelodyDensity ------------------------------------------------------

    #[test]
    fn density_for_part() {
        assert_eq!(MelodyDensity::for_part(SongPart::Intro), MelodyDensity::Sparse);
        assert_eq!(MelodyDensity::for_part(SongPart::Verse), MelodyDensity::Medium);
        assert_eq!(MelodyDensity::for_part(SongPart::Chorus), MelodyDensity::Dense);
        assert_eq!(MelodyDensity::for_part(SongPart::Bridge), MelodyDensity::Sparse);
        assert_eq!(MelodyDensity::for_part(SongPart::Outro), MelodyDensity::Sparse);
    }

    #[test]
    fn density_note_ranges() {
        assert_eq!(MelodyDensity::Sparse.note_range(), (3, 4));
        assert_eq!(MelodyDensity::Medium.note_range(), (5, 7));
        assert_eq!(MelodyDensity::Dense.note_range(), (8, 12));
    }

    // -- Determinism --------------------------------------------------------

    #[test]
    fn same_seed_produces_identical_melody() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut e1 = MelodyEngine::new(42);
        let r1 = e1.generate_melody(&config);

        let mut e2 = MelodyEngine::new(42);
        let r2 = e2.generate_melody(&config);

        assert_eq!(r1, r2, "same seed must produce identical melody");
    }

    #[test]
    fn different_seeds_produce_different_melodies() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut e1 = MelodyEngine::new(42);
        let r1 = e1.generate_melody(&config);

        let mut e2 = MelodyEngine::new(99);
        let r2 = e2.generate_melody(&config);

        assert_ne!(r1, r2, "different seeds should produce different melodies");
    }

    #[test]
    fn counter_melody_determinism() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut e1 = MelodyEngine::new(42);
        let lead1 = e1.generate_melody(&config);
        let counter1 = e1.generate_counter_melody(&config, &lead1);

        let mut e2 = MelodyEngine::new(42);
        let lead2 = e2.generate_melody(&config);
        let counter2 = e2.generate_counter_melody(&config, &lead2);

        assert_eq!(counter1, counter2, "counter-melody must be deterministic");
    }

    // -- Note range bounds --------------------------------------------------

    #[test]
    fn melody_notes_within_range() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = MelodyConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Chorus,
            channel: 8,
            range: (48, 72),
            contour: Some(ContourShape::Arch),
        };

        let mut engine = MelodyEngine::new(42);
        let pattern = engine.generate_melody(&config);

        for event in &pattern.events {
            assert!(
                event.note >= 48 && event.note <= 72,
                "note {} outside range [48, 72]",
                event.note,
            );
        }
    }

    // -- Chord tone targeting -----------------------------------------------

    #[test]
    fn strong_beats_prefer_chord_tones() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 8];
        let config = MelodyConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 8,
            range: (48, 72),
            contour: Some(ContourShape::Arch),
        };

        let mut engine = MelodyEngine::new(42);
        let pattern = engine.generate_melody(&config);
        let chord_pcs = c_major_chord().notes();

        let mut strong_total = 0;
        let mut strong_chord_tone = 0;

        for event in &pattern.events {
            let bar_tick = event.tick % TICKS_PER_BAR;
            let beat = bar_tick / TICKS_PER_BEAT;
            if beat == 0 || beat == 2 {
                strong_total += 1;
                let pc = PitchClass::from_midi(event.note);
                if chord_pcs.contains(&pc) {
                    strong_chord_tone += 1;
                }
            }
        }

        if strong_total > 0 {
            let ratio = strong_chord_tone as f64 / strong_total as f64;
            assert!(
                ratio > 0.50,
                "expected >50% chord tones on strong beats, got {:.0}% ({}/{})",
                ratio * 100.0,
                strong_chord_tone,
                strong_total,
            );
        }
    }

    // -- Counter-melody -----------------------------------------------------

    #[test]
    fn counter_melody_is_sparser_than_lead() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = MelodyConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Chorus,
            channel: 8,
            range: (48, 72),
            contour: Some(ContourShape::Arch),
        };

        let mut engine = MelodyEngine::new(42);
        let lead = engine.generate_melody(&config);
        let counter = engine.generate_counter_melody(&config, &lead);

        assert!(
            counter.events.len() < lead.events.len(),
            "counter ({}) should be sparser than lead ({})",
            counter.events.len(),
            lead.events.len(),
        );
    }

    // -- Pitch table --------------------------------------------------------

    #[test]
    fn pitch_table_c_major() {
        let scale = c_major_scale();
        let table = MelodyEngine::build_pitch_table(&scale, (60, 72));
        assert_eq!(table, vec![60, 62, 64, 65, 67, 69, 71, 72]);
    }

    #[test]
    fn pitch_table_empty_when_no_scale_tones_in_range() {
        let scale = c_major_scale();
        let table = MelodyEngine::build_pitch_table(&scale, (61, 61));
        assert!(table.is_empty());
    }

    // -- Melody output structure --------------------------------------------

    #[test]
    fn melody_produces_events() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut engine = MelodyEngine::new(42);
        let pattern = engine.generate_melody(&config);

        assert!(!pattern.events.is_empty(), "melody should produce events");
        assert_eq!(pattern.bars, 4);
        assert_eq!(pattern.length_ticks, 4 * TICKS_PER_BAR);
    }

    #[test]
    fn melody_events_sorted_by_tick() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut engine = MelodyEngine::new(42);
        let pattern = engine.generate_melody(&config);

        for i in 1..pattern.events.len() {
            assert!(
                pattern.events[i].tick >= pattern.events[i - 1].tick,
                "events should be sorted by tick",
            );
        }
    }

    #[test]
    fn empty_chords_produces_empty_pattern() {
        let scale = c_major_scale();
        let chords: Vec<Chord> = vec![];
        let config = MelodyConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 8,
            range: (48, 72),
            contour: None,
        };

        let mut engine = MelodyEngine::new(42);
        let pattern = engine.generate_melody(&config);
        assert!(pattern.events.is_empty());
        assert_eq!(pattern.bars, 0);
    }

    // -- Step motion dominance ----------------------------------------------

    #[test]
    fn step_motion_dominates() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 16];
        let config = MelodyConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 8,
            range: (48, 84),
            contour: Some(ContourShape::Arch),
        };

        let mut engine = MelodyEngine::new(42);
        let pattern = engine.generate_melody(&config);
        let table = MelodyEngine::build_pitch_table(&scale, (48, 84));

        let mut step_count = 0u32;
        let mut total_intervals = 0u32;

        for i in 1..pattern.events.len() {
            let prev_idx =
                MelodyEngine::closest_index(&table, pattern.events[i - 1].note);
            let curr_idx =
                MelodyEngine::closest_index(&table, pattern.events[i].note);
            let diff = prev_idx.abs_diff(curr_idx);
            total_intervals += 1;
            if diff <= 1 {
                step_count += 1;
            }
        }

        if total_intervals > 10 {
            let ratio = step_count as f64 / total_intervals as f64;
            assert!(
                ratio > 0.40,
                "step motion should dominate (>40%), got {:.0}% ({}/{})",
                ratio * 100.0,
                step_count,
                total_intervals,
            );
        }
    }

    // -- LargeLeap intervals -------------------------------------------------

    #[test]
    fn large_leap_produces_only_fourth_fifth_or_octave() {
        let scale = c_major_scale();
        let pitch_table = MelodyEngine::build_pitch_table(&scale, (36, 96));
        let allowed_steps: std::collections::HashSet<usize> =
            [3usize, 4, 7].iter().copied().collect();

        // Run many iterations to exercise the RNG
        for seed in 0..200u64 {
            let mut engine = MelodyEngine::new(seed);
            // Force LargeLeap by calling apply_motion with conditions that
            // won't trigger the resolution or repeat-override paths
            for _ in 0..50 {
                let current = pitch_table.len() / 2;
                let (new_idx, motion, _dir) = engine.apply_motion(
                    current,
                    pitch_table.len(),
                    2.0, // positive contour offset => direction = 1
                    MotionType::Step, // last motion != LargeLeap => no forced resolution
                    1,
                    0, // repeat_count = 0 => no forced step
                );
                if motion == MotionType::LargeLeap {
                    let step = new_idx.abs_diff(current);
                    assert!(
                        allowed_steps.contains(&step),
                        "LargeLeap produced step of {} scale degrees (seed={}) — \
                         only 3 (4th), 4 (5th), or 7 (octave) are allowed",
                        step,
                        seed,
                    );
                }
            }
        }
    }

    // -- Serde roundtrips ---------------------------------------------------

    #[test]
    fn contour_shape_serde_roundtrip() {
        let shape = ContourShape::Arch;
        let json = serde_json::to_string(&shape).expect("serialize");
        assert_eq!(json, r#""arch""#);
        let parsed: ContourShape = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, shape);
    }

    #[test]
    fn melody_density_serde_roundtrip() {
        let density = MelodyDensity::Medium;
        let json = serde_json::to_string(&density).expect("serialize");
        assert_eq!(json, r#""medium""#);
        let parsed: MelodyDensity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, density);
    }

    #[test]
    fn motion_type_serde_roundtrip() {
        let motion = MotionType::SmallLeap;
        let json = serde_json::to_string(&motion).expect("serialize");
        assert_eq!(json, r#""small_leap""#);
        let parsed: MotionType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, motion);
    }
}
