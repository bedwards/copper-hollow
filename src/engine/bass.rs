// Bass line generation: walking, root-fifth, pedal, octave patterns with
// approach notes, voice leading, and register management per docs/engine/BASS.md.

use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::song::{NoteEvent, Pattern, SongPart};
use super::theory::{Chord, PitchClass, Scale};
use super::{TICKS_PER_BAR, TICKS_PER_BEAT};

// ---------------------------------------------------------------------------
// BassStyle
// ---------------------------------------------------------------------------

/// Bass line style controlling rhythmic density and note selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BassStyle {
    /// Root on beat 1, fifth on beat 3. Steady, understated. 2-4 notes/bar.
    RootFifth,
    /// Scalar motion connecting chord roots. Every beat has a note.
    Walking,
    /// Stay on one note (usually tonic) regardless of chord changes.
    Pedal,
    /// Root and octave alternating in 8th notes. High energy.
    Octave,
}

impl BassStyle {
    /// Choose bass style based on song part per spec table.
    pub fn for_part(part: SongPart) -> Self {
        match part {
            SongPart::Intro | SongPart::Bridge => BassStyle::Pedal,
            SongPart::Verse => BassStyle::RootFifth,
            SongPart::PreChorus => BassStyle::Walking,
            SongPart::Chorus => BassStyle::Octave,
            SongPart::Outro => BassStyle::RootFifth,
        }
    }
}

// ---------------------------------------------------------------------------
// BassConfig
// ---------------------------------------------------------------------------

/// Configuration for bass line generation.
#[derive(Clone, Debug)]
pub struct BassConfig<'a> {
    pub scale: &'a Scale,
    /// One chord per bar.
    pub chords_per_bar: &'a [Chord],
    pub part: SongPart,
    pub channel: u8,
    /// MIDI note range (low, high) for the instrument.
    pub range: (u8, u8),
    /// Optional style override (auto-selected from part if None).
    pub style: Option<BassStyle>,
    /// Tonic pitch class for pedal bass.
    pub tonic: PitchClass,
}

// ---------------------------------------------------------------------------
// BassEngine
// ---------------------------------------------------------------------------

/// Bass engine for generating walking lines, root-fifth patterns, pedal bass,
/// and octave patterns with approach notes and voice leading.
/// Fully deterministic given the same seed.
pub struct BassEngine {
    rng: ChaCha8Rng,
}

impl BassEngine {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    // -- Pitch helpers ------------------------------------------------------

    /// Build sorted list of MIDI notes in scale within range.
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

    /// Find MIDI note for a pitch class closest to a reference note, within range.
    fn closest_note_for_pc(pc: PitchClass, reference: u8, range: (u8, u8)) -> u8 {
        let semitone = pc.to_semitone();
        let mut best = range.0;
        let mut best_dist = u8::MAX;
        // Check all octaves within range
        for octave in 0..=10u8 {
            let note = octave * 12 + semitone;
            if note < range.0 || note > range.1 {
                continue;
            }
            let dist = note.abs_diff(reference);
            if dist < best_dist {
                best_dist = dist;
                best = note;
            }
        }
        best
    }

    /// Find closest index in pitch table to given MIDI note.
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

    /// Get the fifth of a chord root in the scale.
    fn chord_fifth(chord: &Chord) -> PitchClass {
        chord.root.transpose(7) // perfect fifth
    }

    // -- Approach note logic ------------------------------------------------

    /// Compute an approach note targeting `next_root`, approaching from
    /// `current_note`, using the scale. Returns (midi_note, is_chromatic).
    fn approach_note(
        &mut self,
        current_note: u8,
        next_root: PitchClass,
        pitch_table: &[u8],
        range: (u8, u8),
    ) -> (u8, bool) {
        let target = Self::closest_note_for_pc(next_root, current_note, range);

        // Determine direction to target
        let diff = target as i16 - current_note as i16;

        if diff == 0 {
            // Same root: approach from fifth below or scale step
            let fifth_below = current_note.saturating_sub(7).max(range.0);
            let step_above = self.scale_step_from(current_note, 1, pitch_table, range);
            let step_below = self.scale_step_from(current_note, -1, pitch_table, range);
            let choices = [fifth_below, step_above, step_below];
            let idx = self.rng.random_range(0..choices.len());
            return (choices[idx], false);
        }

        // 30% chance of chromatic approach
        let chromatic: bool = self.rng.random::<f64>() < 0.30;

        if chromatic {
            // One semitone below/above the target
            let approach = if diff > 0 {
                target.saturating_sub(1).max(range.0)
            } else {
                (target + 1).min(range.1)
            };
            (approach, true)
        } else {
            // One scale step below/above the target
            let approach = if diff > 0 {
                self.scale_step_from(target, -1, pitch_table, range)
            } else {
                self.scale_step_from(target, 1, pitch_table, range)
            };
            (approach, false)
        }
    }

    /// Get the MIDI note one scale step above (dir=1) or below (dir=-1) from a reference.
    fn scale_step_from(
        &self,
        reference: u8,
        dir: i32,
        pitch_table: &[u8],
        range: (u8, u8),
    ) -> u8 {
        let idx = Self::closest_index(pitch_table, reference);
        let new_idx = (idx as i32 + dir).clamp(0, pitch_table.len().saturating_sub(1) as i32);
        pitch_table[new_idx as usize].clamp(range.0, range.1)
    }

    // -- Pattern generators -------------------------------------------------

    /// Generate root-fifth pattern for one bar.
    #[allow(clippy::too_many_arguments)]
    fn generate_root_fifth_bar(
        &mut self,
        chord: &Chord,
        bar_offset: u32,
        prev_note: u8,
        pitch_table: &[u8],
        range: (u8, u8),
        next_root: Option<PitchClass>,
        channel: u8,
    ) -> (Vec<NoteEvent>, u8) {
        let root = Self::closest_note_for_pc(chord.root, prev_note, range);
        let fifth = Self::closest_note_for_pc(Self::chord_fifth(chord), root, range);

        let mut events = Vec::new();

        // Beat 1: root
        events.push(NoteEvent {
            tick: bar_offset,
            note: root,
            velocity: 100,
            duration: TICKS_PER_BEAT,
            channel,
        });

        // Variation: 40% chance of pickup 8th note into beat 3
        let has_pickup = self.rng.random::<f64>() < 0.40;

        if has_pickup {
            // Pickup note on "and of 2" — a chord tone (third)
            let third_pc = chord.root.transpose(
                if chord.quality == super::theory::ChordQuality::Minor { 3 } else { 4 }
            );
            let pickup_note = Self::closest_note_for_pc(third_pc, root, range);
            events.push(NoteEvent {
                tick: bar_offset + TICKS_PER_BEAT + TICKS_PER_BEAT / 2,
                note: pickup_note,
                velocity: 65,
                duration: TICKS_PER_BEAT / 2,
                channel,
            });
        }

        // Beat 3: fifth
        events.push(NoteEvent {
            tick: bar_offset + 2 * TICKS_PER_BEAT,
            note: fifth,
            velocity: 80,
            duration: TICKS_PER_BEAT,
            channel,
        });
        let mut last_note = fifth;

        // Beat 4: approach note if there's a next bar
        if let Some(next_r) = next_root {
            let (approach, is_chromatic) =
                self.approach_note(last_note, next_r, pitch_table, range);
            let vel = if is_chromatic { 60 } else { 70 };
            let dur = if is_chromatic {
                TICKS_PER_BEAT * 3 / 4
            } else {
                TICKS_PER_BEAT
            };
            events.push(NoteEvent {
                tick: bar_offset + 3 * TICKS_PER_BEAT,
                note: approach,
                velocity: vel,
                duration: dur,
                channel,
            });
            last_note = approach;
        }

        (events, last_note)
    }

    /// Generate walking bass pattern for one bar.
    #[allow(clippy::too_many_arguments)]
    fn generate_walking_bar(
        &mut self,
        chord: &Chord,
        bar_offset: u32,
        prev_note: u8,
        pitch_table: &[u8],
        range: (u8, u8),
        next_root: Option<PitchClass>,
        channel: u8,
    ) -> (Vec<NoteEvent>, u8) {
        let root = Self::closest_note_for_pc(chord.root, prev_note, range);
        let mut events = Vec::new();

        // Beat 1: chord root (mandatory per spec)
        events.push(NoteEvent {
            tick: bar_offset,
            note: root,
            velocity: 100,
            duration: TICKS_PER_BEAT,
            channel,
        });

        // Beat 3 target: chord tone (root, 3rd, or 5th)
        let chord_tones = chord.notes();
        let beat3_pc = if chord_tones.len() >= 3 {
            let choices = [chord_tones[0], chord_tones[1], chord_tones[2]];
            let idx = self.rng.random_range(0..choices.len());
            choices[idx]
        } else {
            chord.root
        };
        let beat3_note = Self::closest_note_for_pc(beat3_pc, root, range);

        // Beat 2: scale tone moving toward beat 3 target
        let direction = if beat3_note >= root { 1 } else { -1 };
        let beat2_note = self.scale_step_from(root, direction, pitch_table, range);

        events.push(NoteEvent {
            tick: bar_offset + TICKS_PER_BEAT,
            note: beat2_note,
            velocity: 75,
            duration: TICKS_PER_BEAT,
            channel,
        });

        // Beat 3: chord tone
        events.push(NoteEvent {
            tick: bar_offset + 2 * TICKS_PER_BEAT,
            note: beat3_note,
            velocity: 80,
            duration: TICKS_PER_BEAT,
            channel,
        });
        let mut last_note = beat3_note;

        // Beat 4: approach note to next bar's root
        if let Some(next_r) = next_root {
            let (approach, is_chromatic) =
                self.approach_note(last_note, next_r, pitch_table, range);
            let vel = if is_chromatic { 56 } else { 70 };
            let dur = if is_chromatic {
                TICKS_PER_BEAT * 3 / 4
            } else {
                TICKS_PER_BEAT
            };
            events.push(NoteEvent {
                tick: bar_offset + 3 * TICKS_PER_BEAT,
                note: approach,
                velocity: vel,
                duration: dur,
                channel,
            });
            last_note = approach;
        } else {
            // No next bar: resolve with a scale step back toward root
            let resolve = self.scale_step_from(last_note, -direction, pitch_table, range);
            events.push(NoteEvent {
                tick: bar_offset + 3 * TICKS_PER_BEAT,
                note: resolve,
                velocity: 70,
                duration: TICKS_PER_BEAT,
                channel,
            });
            last_note = resolve;
        }

        (events, last_note)
    }

    /// Generate pedal bass pattern for one bar.
    fn generate_pedal_bar(
        &mut self,
        bar_offset: u32,
        pedal_note: u8,
        channel: u8,
    ) -> (Vec<NoteEvent>, u8) {
        // Whole note or half notes
        let use_halves: bool = self.rng.random::<f64>() < 0.40;

        let mut events = Vec::new();
        if use_halves {
            events.push(NoteEvent {
                tick: bar_offset,
                note: pedal_note,
                velocity: 90,
                duration: 2 * TICKS_PER_BEAT,
                channel,
            });
            events.push(NoteEvent {
                tick: bar_offset + 2 * TICKS_PER_BEAT,
                note: pedal_note,
                velocity: 75,
                duration: 2 * TICKS_PER_BEAT,
                channel,
            });
        } else {
            events.push(NoteEvent {
                tick: bar_offset,
                note: pedal_note,
                velocity: 90,
                duration: TICKS_PER_BAR,
                channel,
            });
        }

        (events, pedal_note)
    }

    /// Generate octave pattern for one bar.
    fn generate_octave_bar(
        &mut self,
        chord: &Chord,
        bar_offset: u32,
        prev_note: u8,
        range: (u8, u8),
        channel: u8,
    ) -> (Vec<NoteEvent>, u8) {
        let root_low = Self::closest_note_for_pc(chord.root, prev_note, range);
        let root_high = if root_low + 12 <= range.1 {
            root_low + 12
        } else {
            // Can't go up an octave; use root_low for both
            root_low
        };

        let eighth = TICKS_PER_BEAT / 2; // 240 ticks
        let velocities: [u8; 8] = [100, 70, 85, 65, 90, 70, 85, 65];

        // 30% chance of rest pattern variant
        let rest_pattern = self.rng.random::<f64>() < 0.30;

        let mut events = Vec::new();
        for i in 0u32..8 {
            // Rest pattern: skip indices 1, 3, 5
            if rest_pattern && (i == 1 || i == 3 || i == 5) {
                continue;
            }

            let note = if i % 2 == 0 { root_low } else { root_high };
            events.push(NoteEvent {
                tick: bar_offset + i * eighth,
                note,
                velocity: velocities[i as usize],
                duration: eighth,
                channel,
            });
        }

        let last_note = if events.is_empty() {
            root_low
        } else {
            events.last().map(|e| e.note).unwrap_or(root_low)
        };
        (events, last_note)
    }

    // -- Ghost notes --------------------------------------------------------

    /// Optionally insert ghost notes on 16th-note positions between real notes.
    /// 10-15% probability per available slot, velocity 20-40, duration 30-60 ticks.
    fn add_ghost_notes(
        &mut self,
        events: &mut Vec<NoteEvent>,
        bar_offset: u32,
        prev_note: u8,
        range: (u8, u8),
        channel: u8,
    ) {
        let sixteenth = TICKS_PER_BEAT / 4; // 120 ticks
        let existing_ticks: Vec<u32> = events.iter().map(|e| e.tick).collect();

        let mut ghosts = Vec::new();
        for slot in 0..16u32 {
            let tick = bar_offset + slot * sixteenth;
            // Skip if a real note is nearby (within half a 16th)
            if existing_ticks
                .iter()
                .any(|&t| t.abs_diff(tick) < sixteenth / 2)
            {
                continue;
            }

            // 10-15% probability
            let prob = self.rng.random::<f64>();
            if prob < 0.125 {
                let vel = self.rng.random_range(20u8..=40);
                let dur = self.rng.random_range(30u32..=60);
                ghosts.push(NoteEvent {
                    tick,
                    note: prev_note.clamp(range.0, range.1),
                    velocity: vel,
                    duration: dur,
                    channel,
                });
            }
        }
        events.extend(ghosts);
    }

    // -- Main generation ----------------------------------------------------

    /// Generate a bass line pattern across the given bars.
    pub fn generate_bass(&mut self, config: &BassConfig<'_>) -> Pattern {
        let bars = config.chords_per_bar.len() as u32;
        if bars == 0 {
            return Pattern::empty(0);
        }

        let pitch_table = Self::build_pitch_table(config.scale, config.range);
        if pitch_table.is_empty() {
            return Pattern::empty(bars);
        }

        let style = config.style.unwrap_or_else(|| BassStyle::for_part(config.part));
        let total_ticks = bars * TICKS_PER_BAR;

        // Start near the sweet spot center
        let center = (config.range.0 / 2).saturating_add(config.range.1 / 2);
        let mut prev_note = Self::closest_note_for_pc(
            config.chords_per_bar[0].root,
            center,
            config.range,
        );

        // Pedal note: tonic in low register
        let pedal_note = Self::closest_note_for_pc(config.tonic, config.range.0 + 7, config.range);

        // Ghost notes only in walking and octave styles
        let use_ghosts = matches!(style, BassStyle::Walking | BassStyle::Octave);

        let mut all_events = Vec::new();

        for bar_idx in 0..bars {
            let bar_offset = bar_idx * TICKS_PER_BAR;
            let chord = &config.chords_per_bar[bar_idx as usize];
            let next_root = config
                .chords_per_bar
                .get((bar_idx + 1) as usize)
                .map(|c| c.root);

            let (mut bar_events, last_note) = match style {
                BassStyle::RootFifth => self.generate_root_fifth_bar(
                    chord,
                    bar_offset,
                    prev_note,
                    &pitch_table,
                    config.range,
                    next_root,
                    config.channel,
                ),
                BassStyle::Walking => self.generate_walking_bar(
                    chord,
                    bar_offset,
                    prev_note,
                    &pitch_table,
                    config.range,
                    next_root,
                    config.channel,
                ),
                BassStyle::Pedal => {
                    self.generate_pedal_bar(bar_offset, pedal_note, config.channel)
                }
                BassStyle::Octave => self.generate_octave_bar(
                    chord,
                    bar_offset,
                    prev_note,
                    config.range,
                    config.channel,
                ),
            };

            if use_ghosts {
                self.add_ghost_notes(
                    &mut bar_events,
                    bar_offset,
                    last_note,
                    config.range,
                    config.channel,
                );
            }

            all_events.extend(bar_events);
            prev_note = last_note;
        }

        all_events.sort_by_key(|e| e.tick);

        Pattern {
            events: all_events,
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

    fn make_chord(root: PitchClass, quality: ChordQuality, degree: ChordDegree) -> Chord {
        Chord {
            root,
            quality,
            degree,
            inversion: 0,
        }
    }

    fn c_major_chord() -> Chord {
        make_chord(PitchClass::C, ChordQuality::Major, ChordDegree::I)
    }

    fn test_config<'a>(scale: &'a Scale, chords: &'a [Chord]) -> BassConfig<'a> {
        BassConfig {
            scale,
            chords_per_bar: chords,
            part: SongPart::Verse,
            channel: 6,
            range: (28, 55), // Electric bass range per spec
            style: None,
            tonic: PitchClass::C,
        }
    }

    // -- BassStyle ----------------------------------------------------------

    #[test]
    fn style_for_part() {
        assert_eq!(BassStyle::for_part(SongPart::Intro), BassStyle::Pedal);
        assert_eq!(BassStyle::for_part(SongPart::Verse), BassStyle::RootFifth);
        assert_eq!(BassStyle::for_part(SongPart::PreChorus), BassStyle::Walking);
        assert_eq!(BassStyle::for_part(SongPart::Chorus), BassStyle::Octave);
        assert_eq!(BassStyle::for_part(SongPart::Bridge), BassStyle::Pedal);
        assert_eq!(BassStyle::for_part(SongPart::Outro), BassStyle::RootFifth);
    }

    #[test]
    fn bass_style_serde_roundtrip() {
        let style = BassStyle::Walking;
        let json = serde_json::to_string(&style).expect("serialize");
        assert_eq!(json, r#""walking""#);
        let parsed: BassStyle = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, style);
    }

    // -- Determinism --------------------------------------------------------

    #[test]
    fn same_seed_produces_identical_bass() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut e1 = BassEngine::new(42);
        let r1 = e1.generate_bass(&config);

        let mut e2 = BassEngine::new(42);
        let r2 = e2.generate_bass(&config);

        assert_eq!(r1, r2, "same seed must produce identical bass line");
    }

    #[test]
    fn different_seeds_produce_different_bass() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut e1 = BassEngine::new(42);
        let r1 = e1.generate_bass(&config);

        let mut e2 = BassEngine::new(99);
        let r2 = e2.generate_bass(&config);

        assert_ne!(r1, r2, "different seeds should produce different bass lines");
    }

    // -- Note range bounds --------------------------------------------------

    #[test]
    fn bass_notes_within_range() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 8];

        for style in [BassStyle::RootFifth, BassStyle::Walking, BassStyle::Pedal, BassStyle::Octave] {
            let config = BassConfig {
                scale: &scale,
                chords_per_bar: &chords,
                part: SongPart::Verse,
                channel: 6,
                range: (28, 55),
                style: Some(style),
                tonic: PitchClass::C,
            };

            let mut engine = BassEngine::new(42);
            let pattern = engine.generate_bass(&config);

            for event in &pattern.events {
                assert!(
                    event.note >= 28 && event.note <= 55,
                    "note {} outside range [28, 55] in style {:?}",
                    event.note,
                    style,
                );
            }
        }
    }

    // -- Root-fifth pattern -------------------------------------------------

    #[test]
    fn root_fifth_starts_on_root() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = BassConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 6,
            range: (28, 55),
            style: Some(BassStyle::RootFifth),
            tonic: PitchClass::C,
        };

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        // First note of each bar should be the chord root
        for bar in 0..4u32 {
            let bar_start = bar * TICKS_PER_BAR;
            let first_in_bar = pattern
                .events
                .iter()
                .find(|e| e.tick >= bar_start && e.tick < bar_start + TICKS_PER_BEAT);
            if let Some(event) = first_in_bar {
                let pc = PitchClass::from_midi(event.note);
                assert_eq!(
                    pc,
                    PitchClass::C,
                    "bar {} beat 1 should be root C, got {:?}",
                    bar,
                    pc,
                );
            }
        }
    }

    // -- Walking bass -------------------------------------------------------

    #[test]
    fn walking_has_four_notes_per_bar() {
        let scale = c_major_scale();
        let chords = vec![
            c_major_chord(),
            make_chord(PitchClass::A, ChordQuality::Minor, ChordDegree::VI),
            make_chord(PitchClass::F, ChordQuality::Major, ChordDegree::IV),
            make_chord(PitchClass::G, ChordQuality::Major, ChordDegree::V),
        ];
        let config = BassConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::PreChorus,
            channel: 6,
            range: (28, 55),
            style: Some(BassStyle::Walking),
            tonic: PitchClass::C,
        };

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        // Each bar should have at least 4 non-ghost notes (vel > 40)
        for bar in 0..4u32 {
            let bar_start = bar * TICKS_PER_BAR;
            let bar_end = bar_start + TICKS_PER_BAR;
            let bar_notes: Vec<_> = pattern
                .events
                .iter()
                .filter(|e| e.tick >= bar_start && e.tick < bar_end && e.velocity > 40)
                .collect();
            assert!(
                bar_notes.len() >= 4,
                "walking bass bar {} has {} non-ghost notes, expected >= 4",
                bar,
                bar_notes.len(),
            );
        }
    }

    #[test]
    fn walking_beat1_is_chord_root() {
        let scale = c_major_scale();
        let am = make_chord(PitchClass::A, ChordQuality::Minor, ChordDegree::VI);
        let chords = vec![c_major_chord(), am];
        let config = BassConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::PreChorus,
            channel: 6,
            range: (28, 55),
            style: Some(BassStyle::Walking),
            tonic: PitchClass::C,
        };

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        // Bar 0, beat 1 = C
        let beat1_bar0 = pattern.events.iter().find(|e| e.tick == 0 && e.velocity > 40);
        assert!(beat1_bar0.is_some(), "should have note at bar 0 beat 1");
        assert_eq!(PitchClass::from_midi(beat1_bar0.map(|e| e.note).unwrap_or(0)), PitchClass::C);

        // Bar 1, beat 1 = A
        let bar1_start = TICKS_PER_BAR;
        let beat1_bar1 = pattern
            .events
            .iter()
            .find(|e| e.tick == bar1_start && e.velocity > 40);
        assert!(beat1_bar1.is_some(), "should have note at bar 1 beat 1");
        assert_eq!(PitchClass::from_midi(beat1_bar1.map(|e| e.note).unwrap_or(0)), PitchClass::A);
    }

    // -- Pedal bass ---------------------------------------------------------

    #[test]
    fn pedal_stays_on_one_note() {
        let scale = c_major_scale();
        let chords = vec![
            c_major_chord(),
            make_chord(PitchClass::G, ChordQuality::Major, ChordDegree::V),
            make_chord(PitchClass::A, ChordQuality::Minor, ChordDegree::VI),
            make_chord(PitchClass::F, ChordQuality::Major, ChordDegree::IV),
        ];
        let config = BassConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Intro,
            channel: 6,
            range: (28, 55),
            style: Some(BassStyle::Pedal),
            tonic: PitchClass::C,
        };

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        // All notes should be the same pitch (pedal on tonic)
        let first_note = pattern.events[0].note;
        for event in &pattern.events {
            assert_eq!(
                event.note, first_note,
                "pedal bass should stay on one note, got {} vs {}",
                event.note, first_note,
            );
        }
        // Pedal note should be tonic C
        assert_eq!(PitchClass::from_midi(first_note), PitchClass::C);
    }

    // -- Octave pattern -----------------------------------------------------

    #[test]
    fn octave_pattern_uses_root_and_octave() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 2];
        let config = BassConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Chorus,
            channel: 6,
            range: (28, 55),
            style: Some(BassStyle::Octave),
            tonic: PitchClass::C,
        };

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        // All notes should be C (root or octave)
        for event in &pattern.events {
            assert_eq!(
                PitchClass::from_midi(event.note),
                PitchClass::C,
                "octave pattern note {} is not C",
                event.note,
            );
        }
    }

    // -- Voice leading ------------------------------------------------------

    #[test]
    fn voice_leading_chooses_closest_voicing() {
        // G2 (43) -> next chord root C: should pick C3 (48) over C2 (36)
        let note = BassEngine::closest_note_for_pc(PitchClass::C, 43, (28, 55));
        assert_eq!(note, 48, "should choose C3 (48) as closest to G2 (43), got {}", note);

        // D3 (50) -> next chord root A: should pick A2 (45) over A3 (57)
        let note = BassEngine::closest_note_for_pc(PitchClass::A, 50, (28, 55));
        assert_eq!(note, 45, "should choose A2 (45) as closest to D3 (50), got {}", note);
    }

    // -- Ghost notes --------------------------------------------------------

    #[test]
    fn ghost_notes_are_quiet_and_short() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 8];
        let config = BassConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::PreChorus,
            channel: 6,
            range: (28, 55),
            style: Some(BassStyle::Walking),
            tonic: PitchClass::C,
        };

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        let ghost_notes: Vec<_> = pattern
            .events
            .iter()
            .filter(|e| e.velocity <= 40)
            .collect();

        for ghost in &ghost_notes {
            assert!(
                ghost.duration <= 60,
                "ghost note duration {} exceeds 60",
                ghost.duration,
            );
            assert!(
                ghost.velocity >= 20 && ghost.velocity <= 40,
                "ghost velocity {} outside [20, 40]",
                ghost.velocity,
            );
        }
    }

    // -- Empty / edge cases -------------------------------------------------

    #[test]
    fn empty_chords_produces_empty_pattern() {
        let scale = c_major_scale();
        let chords: Vec<Chord> = vec![];
        let config = BassConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 6,
            range: (28, 55),
            style: None,
            tonic: PitchClass::C,
        };

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);
        assert!(pattern.events.is_empty());
        assert_eq!(pattern.bars, 0);
    }

    // -- Output structure ---------------------------------------------------

    #[test]
    fn bass_produces_events() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        assert!(!pattern.events.is_empty(), "bass should produce events");
        assert_eq!(pattern.bars, 4);
        assert_eq!(pattern.length_ticks, 4 * TICKS_PER_BAR);
    }

    #[test]
    fn bass_events_sorted_by_tick() {
        let scale = c_major_scale();
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&scale, &chords);

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        for i in 1..pattern.events.len() {
            assert!(
                pattern.events[i].tick >= pattern.events[i - 1].tick,
                "events should be sorted by tick",
            );
        }
    }

    // -- Approach notes -----------------------------------------------------

    #[test]
    fn approach_note_is_near_target() {
        let scale = c_major_scale();
        let pitch_table = BassEngine::build_pitch_table(&scale, (28, 55));
        let range = (28, 55);

        let mut engine = BassEngine::new(42);

        // Approach to A from C (36): result should be within 2 semitones of A
        for _ in 0..20 {
            let (note, _) = engine.approach_note(36, PitchClass::A, &pitch_table, range);
            let target = BassEngine::closest_note_for_pc(PitchClass::A, 36, range);
            let diff = note.abs_diff(target);
            assert!(
                diff <= 3,
                "approach note {} is {} semitones from target A ({})",
                note,
                diff,
                target,
            );
        }
    }

    // -- Multi-chord walking ------------------------------------------------

    #[test]
    fn walking_across_progression() {
        let scale = c_major_scale();
        let chords = vec![
            c_major_chord(),
            make_chord(PitchClass::G, ChordQuality::Major, ChordDegree::V),
            make_chord(PitchClass::A, ChordQuality::Minor, ChordDegree::VI),
            make_chord(PitchClass::F, ChordQuality::Major, ChordDegree::IV),
        ];
        let config = BassConfig {
            scale: &scale,
            chords_per_bar: &chords,
            part: SongPart::PreChorus,
            channel: 6,
            range: (28, 55),
            style: Some(BassStyle::Walking),
            tonic: PitchClass::C,
        };

        let mut engine = BassEngine::new(42);
        let pattern = engine.generate_bass(&config);

        // Verify we have notes and they're well-formed
        assert!(pattern.events.len() >= 16, "4 bars walking should have >= 16 notes");

        // Check register management: no jumps > 18 semitones between consecutive notes
        for i in 1..pattern.events.len() {
            if pattern.events[i].velocity <= 40 {
                continue; // skip ghost notes
            }
            if pattern.events[i - 1].velocity <= 40 {
                continue;
            }
            let jump = pattern.events[i]
                .note
                .abs_diff(pattern.events[i - 1].note);
            assert!(
                jump <= 18,
                "jump of {} semitones between notes {} and {} is too large",
                jump,
                pattern.events[i - 1].note,
                pattern.events[i].note,
            );
        }
    }

    // -- Pitch table --------------------------------------------------------

    #[test]
    fn pitch_table_bass_range() {
        let scale = c_major_scale();
        let table = BassEngine::build_pitch_table(&scale, (36, 48));
        // C2=36, D2=38, E2=40, F2=41, G2=43, A2=45, B2=47, C3=48
        assert_eq!(table, vec![36, 38, 40, 41, 43, 45, 47, 48]);
    }
}
