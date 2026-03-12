// Pad/sustain chord generation: voice-led sustained chords with open, close,
// and drop-2 voicings per docs/engine/THEORY.md.
// Fully implemented — awaiting GUI integration (v0.4.0).
#![allow(dead_code)]

use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::song::{NoteEvent, Pattern, SongPart};
use super::theory::{Chord, PitchClass};
use super::{TICKS_PER_BAR, TICKS_PER_BEAT};

// ---------------------------------------------------------------------------
// PadVoicingType
// ---------------------------------------------------------------------------

/// Voicing spread for pad/sustain chords.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PadVoicingType {
    /// Notes within one octave. Compact sound.
    Close,
    /// Notes spread across 1.5-2 octaves. Fuller, more spacious.
    Open,
    /// Second note from top dropped an octave. Jazz-folk voicing.
    Drop2,
}

impl PadVoicingType {
    /// Choose voicing type based on song part.
    pub fn for_part(part: SongPart) -> Self {
        match part {
            SongPart::Intro | SongPart::Outro => PadVoicingType::Open,
            SongPart::Verse | SongPart::Bridge => PadVoicingType::Close,
            SongPart::PreChorus | SongPart::Chorus => PadVoicingType::Drop2,
        }
    }
}

// ---------------------------------------------------------------------------
// PadConfig
// ---------------------------------------------------------------------------

/// Configuration for pad/sustain chord generation.
#[derive(Clone, Debug)]
pub struct PadConfig<'a> {
    /// One chord per bar.
    pub chords_per_bar: &'a [Chord],
    pub part: SongPart,
    pub channel: u8,
    /// MIDI note range (low, high) for the instrument.
    pub range: (u8, u8),
    /// Optional voicing type override (auto-selected from part if None).
    pub voicing: Option<PadVoicingType>,
}

// ---------------------------------------------------------------------------
// PadEngine
// ---------------------------------------------------------------------------

/// Pad engine for generating voice-led sustained chord patterns.
/// Supports close, open, and drop-2 voicings with minimal voice motion
/// between chord changes. Fully deterministic given the same seed.
pub struct PadEngine {
    rng: ChaCha8Rng,
}

impl PadEngine {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    // -- Voicing construction -----------------------------------------------

    /// Build a close voicing: all chord tones within one octave, centered near `center`.
    fn close_voicing(chord: &Chord, center: u8, range: (u8, u8)) -> Vec<u8> {
        let intervals = chord.quality.intervals();
        let root_semi = chord.root.to_semitone();

        // Find the best octave for the root that keeps all notes in range and near center
        let mut best_root: Option<u8> = None;
        let mut best_dist = u8::MAX;

        for octave in 0..=10u8 {
            let root_midi = octave * 12 + root_semi;
            if root_midi < range.0 {
                continue;
            }
            let top = root_midi + intervals.last().copied().unwrap_or(0);
            if top > range.1 {
                continue;
            }
            let dist = root_midi.abs_diff(center);
            if dist < best_dist {
                best_dist = dist;
                best_root = Some(root_midi);
            }
        }

        // Fallback: find closest root in range even if upper chord tones exceed it.
        // The filter below will trim out-of-range notes.
        if best_root.is_none() {
            best_dist = u8::MAX;
            for octave in 0..=10u8 {
                let root_midi = octave * 12 + root_semi;
                if root_midi < range.0 || root_midi > range.1 {
                    continue;
                }
                let dist = root_midi.abs_diff(center);
                if dist < best_dist {
                    best_dist = dist;
                    best_root = Some(root_midi);
                }
            }
        }

        let root = match best_root {
            Some(r) => r,
            None => return Vec::new(),
        };

        intervals
            .iter()
            .map(|&i| root + i)
            .filter(|&n| n >= range.0 && n <= range.1)
            .collect()
    }

    /// Build an open voicing: root in bass, other notes an octave higher.
    /// Spreads across ~1.5-2 octaves for a fuller sound.
    fn open_voicing(chord: &Chord, center: u8, range: (u8, u8)) -> Vec<u8> {
        let intervals = chord.quality.intervals();
        let root_semi = chord.root.to_semitone();

        // Place root in lower register
        let lower_center = center.saturating_sub(6);
        let mut best_root: Option<u8> = None;
        let mut best_dist = u8::MAX;

        for octave in 0..=10u8 {
            let candidate = octave * 12 + root_semi;
            if candidate < range.0 || candidate > range.1 {
                continue;
            }
            let dist = candidate.abs_diff(lower_center);
            if dist < best_dist {
                best_dist = dist;
                best_root = Some(candidate);
            }
        }

        let root_midi = match best_root {
            Some(r) => r,
            None => return Vec::new(),
        };

        let mut notes = vec![root_midi];
        for &interval in intervals.iter().skip(1) {
            let note = root_midi + interval + 12;
            if note <= range.1 {
                notes.push(note);
            } else {
                let fallback = root_midi + interval;
                if fallback <= range.1 && fallback >= range.0 {
                    notes.push(fallback);
                }
            }
        }

        notes
    }

    /// Build a drop-2 voicing: close voicing with second-from-top dropped an octave.
    fn drop2_voicing(chord: &Chord, center: u8, range: (u8, u8)) -> Vec<u8> {
        let mut close = Self::close_voicing(chord, center, range);
        if close.len() >= 3 {
            close.sort();
            let drop_idx = close.len() - 2;
            let dropped = close[drop_idx].saturating_sub(12);
            if dropped >= range.0 {
                close[drop_idx] = dropped;
            }
            close.sort();
        }
        close
    }

    /// Build a voicing based on type.
    fn build_voicing(
        voicing_type: PadVoicingType,
        chord: &Chord,
        center: u8,
        range: (u8, u8),
    ) -> Vec<u8> {
        match voicing_type {
            PadVoicingType::Close => Self::close_voicing(chord, center, range),
            PadVoicingType::Open => Self::open_voicing(chord, center, range),
            PadVoicingType::Drop2 => Self::drop2_voicing(chord, center, range),
        }
    }

    // -- Voice leading ------------------------------------------------------

    /// Voice-lead from `prev_voicing` to `next_chord`, minimizing total voice motion.
    /// Keeps common tones stationary and moves remaining voices by smallest interval.
    pub fn voice_lead(
        prev_voicing: &[u8],
        next_chord: &Chord,
        voicing_type: PadVoicingType,
        range: (u8, u8),
    ) -> Vec<u8> {
        let next_pcs = next_chord.notes();

        if prev_voicing.is_empty() || next_pcs.is_empty() {
            return Vec::new();
        }

        // Build all possible target notes within range for each pitch class
        let target_options: Vec<Vec<u8>> = next_pcs
            .iter()
            .map(|pc| {
                let semi = pc.to_semitone();
                (0..=10u8)
                    .map(|oct| oct * 12 + semi)
                    .filter(|&n| n >= range.0 && n <= range.1)
                    .collect()
            })
            .collect();

        let num_voices = prev_voicing.len();
        let mut result = Vec::with_capacity(num_voices);
        let mut used_targets: Vec<bool> = vec![false; next_pcs.len()];
        let mut voice_assigned = vec![false; num_voices];

        // First pass: keep common tones stationary
        for (vi, &prev_note) in prev_voicing.iter().enumerate() {
            let prev_pc = PitchClass::from_midi(prev_note);
            for (ti, pc) in next_pcs.iter().enumerate() {
                if !used_targets[ti] && prev_pc == *pc {
                    result.push(prev_note);
                    voice_assigned[vi] = true;
                    used_targets[ti] = true;
                    break;
                }
            }
        }

        // Second pass: assign remaining voices to nearest available target
        for (vi, &prev_note) in prev_voicing.iter().enumerate() {
            if voice_assigned[vi] {
                continue;
            }

            let mut best_note = prev_note;
            let mut best_dist = u8::MAX;
            let mut best_ti: Option<usize> = None;

            for (ti, options) in target_options.iter().enumerate() {
                if used_targets[ti] {
                    continue;
                }
                for &candidate in options {
                    let dist = candidate.abs_diff(prev_note);
                    if dist < best_dist {
                        best_dist = dist;
                        best_note = candidate;
                        best_ti = Some(ti);
                    }
                }
            }

            result.push(best_note);
            voice_assigned[vi] = true;
            if let Some(ti) = best_ti {
                used_targets[ti] = true;
            }
        }

        // Handle case where next chord has more notes than previous voicing
        if result.len() < next_pcs.len() {
            for (ti, options) in target_options.iter().enumerate() {
                if used_targets[ti] || options.is_empty() {
                    continue;
                }
                let avg = if result.is_empty() {
                    (range.0 / 2).saturating_add(range.1 / 2)
                } else {
                    let sum: u16 = result.iter().map(|&n| n as u16).sum();
                    (sum / result.len() as u16) as u8
                };
                if let Some(&best) = options.iter().min_by_key(|&&n| n.abs_diff(avg)) {
                    result.push(best);
                }
            }
        }

        result.sort();

        // For open voicing, ensure sufficient spread
        if voicing_type == PadVoicingType::Open && result.len() >= 3 {
            let spread = result.last().unwrap_or(&0) - result.first().unwrap_or(&0);
            if spread < 12 {
                let last_idx = result.len() - 1;
                if result[last_idx] + 12 <= range.1 {
                    result[last_idx] += 12;
                }
            }
        }

        result
    }

    // -- Velocity helpers ---------------------------------------------------

    /// Compute base velocity for pad events. Pads are generally softer.
    fn base_velocity(&mut self, part: SongPart) -> u8 {
        let base: i16 = match part {
            SongPart::Intro | SongPart::Outro => 55,
            SongPart::Verse => 65,
            SongPart::PreChorus => 70,
            SongPart::Chorus => 80,
            SongPart::Bridge => 60,
        };
        let variation = self.rng.random_range(-5i16..=5);
        (base + variation).clamp(1, 127) as u8
    }

    // -- Main generation ----------------------------------------------------

    /// Generate voice-led pad/sustain chord patterns across the given bars.
    pub fn generate_pads(&mut self, config: &PadConfig<'_>) -> Pattern {
        let bars = config.chords_per_bar.len() as u32;
        if bars == 0 {
            return Pattern::empty(0);
        }

        let voicing_type = config
            .voicing
            .unwrap_or_else(|| PadVoicingType::for_part(config.part));
        let total_ticks = bars * TICKS_PER_BAR;

        let center = (config.range.0 / 2).saturating_add(config.range.1 / 2);

        let mut current_voicing = Self::build_voicing(
            voicing_type,
            &config.chords_per_bar[0],
            center,
            config.range,
        );

        let mut all_events = Vec::new();

        for bar_idx in 0..bars {
            let bar_offset = bar_idx * TICKS_PER_BAR;
            let chord = &config.chords_per_bar[bar_idx as usize];

            // Voice-lead to new chord (except first bar which uses initial voicing)
            if bar_idx > 0 {
                current_voicing =
                    Self::voice_lead(&current_voicing, chord, voicing_type, config.range);
            }

            let velocity = self.base_velocity(config.part);

            // Sustained chord: one note per voice lasting the whole bar
            // Slight stagger on attack for organic feel
            for (voice_idx, &note) in current_voicing.iter().enumerate() {
                let stagger = voice_idx as u32 * self.rng.random_range(2u32..=6);
                let tick = bar_offset + stagger;

                let voice_vel_offset = self.rng.random_range(-3i16..=3);
                let vel = (velocity as i16 + voice_vel_offset).clamp(1, 127) as u8;

                let legato_variation = self.rng.random_range(-10i32..=20);
                let duration = (TICKS_PER_BAR as i32 + legato_variation).max(1) as u32;

                all_events.push(NoteEvent {
                    tick,
                    note,
                    velocity: vel,
                    duration,
                    channel: config.channel,
                });
            }

            // 30% chance of re-articulation at beat 3 for rhythmic interest
            if self.rng.random::<f64>() < 0.30 {
                let reartic_tick = bar_offset + 2 * TICKS_PER_BEAT;
                let reartic_vel = ((velocity as f32) * 0.7).round().max(1.0) as u8;

                for &note in &current_voicing {
                    let stagger = self.rng.random_range(0u32..=4);
                    all_events.push(NoteEvent {
                        tick: reartic_tick + stagger,
                        note,
                        velocity: reartic_vel,
                        duration: 2 * TICKS_PER_BEAT,
                        channel: config.channel,
                    });
                }
            }
        }

        all_events.sort_by_key(|e| e.tick);

        Pattern {
            events: all_events,
            cc_events: Vec::new(),
            length_ticks: total_ticks,
            bars,
        }
    }

    /// Compute total voice motion (sum of absolute semitone distances) between two voicings.
    pub fn total_voice_motion(prev: &[u8], next: &[u8]) -> u32 {
        let len = prev.len().min(next.len());
        let mut prev_sorted = prev.to_vec();
        let mut next_sorted = next.to_vec();
        prev_sorted.sort();
        next_sorted.sort();

        prev_sorted
            .iter()
            .zip(next_sorted.iter())
            .take(len)
            .map(|(&a, &b)| a.abs_diff(b) as u32)
            .sum()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::theory::{ChordDegree, ChordQuality};

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

    fn test_config(chords: &[Chord]) -> PadConfig<'_> {
        PadConfig {
            chords_per_bar: chords,
            part: SongPart::Verse,
            channel: 11,
            range: (36, 84),
            voicing: None,
        }
    }

    // -- PadVoicingType -----------------------------------------------------

    #[test]
    fn voicing_type_for_part() {
        assert_eq!(
            PadVoicingType::for_part(SongPart::Intro),
            PadVoicingType::Open
        );
        assert_eq!(
            PadVoicingType::for_part(SongPart::Verse),
            PadVoicingType::Close
        );
        assert_eq!(
            PadVoicingType::for_part(SongPart::PreChorus),
            PadVoicingType::Drop2
        );
        assert_eq!(
            PadVoicingType::for_part(SongPart::Chorus),
            PadVoicingType::Drop2
        );
        assert_eq!(
            PadVoicingType::for_part(SongPart::Bridge),
            PadVoicingType::Close
        );
        assert_eq!(
            PadVoicingType::for_part(SongPart::Outro),
            PadVoicingType::Open
        );
    }

    #[test]
    fn voicing_type_serde_roundtrip() {
        for voicing in [
            PadVoicingType::Close,
            PadVoicingType::Open,
            PadVoicingType::Drop2,
        ] {
            let json = serde_json::to_string(&voicing).expect("serialize");
            let parsed: PadVoicingType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(parsed, voicing);
        }
    }

    // -- Close voicing ------------------------------------------------------

    #[test]
    fn close_voicing_within_octave() {
        let chord = c_major_chord();
        let voicing = PadEngine::close_voicing(&chord, 60, (36, 84));
        assert!(!voicing.is_empty());
        let spread = voicing.last().unwrap_or(&0) - voicing.first().unwrap_or(&0);
        assert!(spread <= 12, "close voicing spread {} exceeds octave", spread);
    }

    #[test]
    fn close_voicing_contains_chord_tones() {
        let chord = c_major_chord();
        let voicing = PadEngine::close_voicing(&chord, 60, (36, 84));
        let chord_pcs = chord.notes();
        for &note in &voicing {
            let pc = PitchClass::from_midi(note);
            assert!(
                chord_pcs.contains(&pc),
                "note {} ({:?}) not a chord tone",
                note,
                pc
            );
        }
    }

    // -- Open voicing -------------------------------------------------------

    #[test]
    fn open_voicing_spans_more_than_octave() {
        let chord = c_major_chord();
        let voicing = PadEngine::open_voicing(&chord, 60, (36, 84));
        assert!(voicing.len() >= 3, "open voicing should have >= 3 notes");
        let spread = voicing.last().unwrap_or(&0) - voicing.first().unwrap_or(&0);
        assert!(
            spread > 12,
            "open voicing spread {} should exceed octave",
            spread
        );
    }

    #[test]
    fn open_voicing_contains_chord_tones() {
        let chord = c_major_chord();
        let voicing = PadEngine::open_voicing(&chord, 60, (36, 84));
        let chord_pcs = chord.notes();
        for &note in &voicing {
            let pc = PitchClass::from_midi(note);
            assert!(
                chord_pcs.contains(&pc),
                "note {} ({:?}) not a chord tone",
                note,
                pc
            );
        }
    }

    // -- Drop-2 voicing -----------------------------------------------------

    #[test]
    fn drop2_voicing_wider_than_close() {
        let chord = c_major_chord();
        let close = PadEngine::close_voicing(&chord, 60, (36, 84));
        let drop2 = PadEngine::drop2_voicing(&chord, 60, (36, 84));

        let close_spread = close.last().unwrap_or(&0) - close.first().unwrap_or(&0);
        let drop2_spread = drop2.last().unwrap_or(&0) - drop2.first().unwrap_or(&0);
        assert!(
            drop2_spread > close_spread,
            "drop-2 spread {} should be wider than close {}",
            drop2_spread,
            close_spread
        );
    }

    #[test]
    fn drop2_voicing_contains_chord_tones() {
        let chord = c_major_chord();
        let voicing = PadEngine::drop2_voicing(&chord, 60, (36, 84));
        let chord_pcs = chord.notes();
        for &note in &voicing {
            let pc = PitchClass::from_midi(note);
            assert!(
                chord_pcs.contains(&pc),
                "note {} ({:?}) not a chord tone",
                note,
                pc
            );
        }
    }

    // -- Voice leading ------------------------------------------------------

    #[test]
    fn voice_leading_keeps_common_tones() {
        // C major (C E G) -> A minor (A C E): C and E are common
        let c_chord = c_major_chord();
        let am_chord = make_chord(PitchClass::A, ChordQuality::Minor, ChordDegree::VI);

        let initial = PadEngine::close_voicing(&c_chord, 60, (36, 84));
        let led = PadEngine::voice_lead(&initial, &am_chord, PadVoicingType::Close, (36, 84));

        // Common pitch classes (C, E) should appear at the same MIDI notes
        let initial_c: Vec<u8> = initial
            .iter()
            .filter(|&&n| PitchClass::from_midi(n) == PitchClass::C)
            .copied()
            .collect();
        let led_c: Vec<u8> = led
            .iter()
            .filter(|&&n| PitchClass::from_midi(n) == PitchClass::C)
            .copied()
            .collect();
        let initial_e: Vec<u8> = initial
            .iter()
            .filter(|&&n| PitchClass::from_midi(n) == PitchClass::E)
            .copied()
            .collect();
        let led_e: Vec<u8> = led
            .iter()
            .filter(|&&n| PitchClass::from_midi(n) == PitchClass::E)
            .copied()
            .collect();

        assert!(
            !initial_c.is_empty() && !led_c.is_empty(),
            "C should appear in both"
        );
        assert!(
            !initial_e.is_empty() && !led_e.is_empty(),
            "E should appear in both"
        );
        assert_eq!(initial_c[0], led_c[0], "common tone C should stay");
        assert_eq!(initial_e[0], led_e[0], "common tone E should stay");
    }

    #[test]
    fn voice_leading_minimizes_motion() {
        // Compare voice-led motion vs naive voicing from scratch
        let c_chord = c_major_chord();
        let f_chord = make_chord(PitchClass::F, ChordQuality::Major, ChordDegree::IV);

        let initial = PadEngine::close_voicing(&c_chord, 60, (36, 84));
        let led = PadEngine::voice_lead(&initial, &f_chord, PadVoicingType::Close, (36, 84));
        let naive = PadEngine::close_voicing(&f_chord, 60, (36, 84));

        let led_motion = PadEngine::total_voice_motion(&initial, &led);
        let naive_motion = PadEngine::total_voice_motion(&initial, &naive);

        assert!(
            led_motion <= naive_motion,
            "voice-led motion {} should be <= naive {}",
            led_motion,
            naive_motion
        );
    }

    #[test]
    fn voice_leading_stepwise_for_related_chords() {
        // C major -> Am: only G moves to A (2 semitones)
        let c_chord = c_major_chord();
        let am_chord = make_chord(PitchClass::A, ChordQuality::Minor, ChordDegree::VI);

        let initial = PadEngine::close_voicing(&c_chord, 60, (36, 84));
        let led = PadEngine::voice_lead(&initial, &am_chord, PadVoicingType::Close, (36, 84));

        let motion = PadEngine::total_voice_motion(&initial, &led);
        assert!(
            motion <= 4,
            "voice motion {} too large for C->Am (expected <= 4)",
            motion
        );
    }

    // -- Determinism --------------------------------------------------------

    #[test]
    fn same_seed_produces_identical_pads() {
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&chords);

        let mut e1 = PadEngine::new(42);
        let r1 = e1.generate_pads(&config);

        let mut e2 = PadEngine::new(42);
        let r2 = e2.generate_pads(&config);

        assert_eq!(r1, r2, "same seed must produce identical pad pattern");
    }

    #[test]
    fn different_seeds_produce_different_pads() {
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&chords);

        let mut e1 = PadEngine::new(42);
        let r1 = e1.generate_pads(&config);

        let mut e2 = PadEngine::new(99);
        let r2 = e2.generate_pads(&config);

        assert_ne!(r1, r2, "different seeds should produce different pad patterns");
    }

    // -- Note range bounds --------------------------------------------------

    #[test]
    fn pad_notes_within_range() {
        let chords = vec![
            c_major_chord(),
            make_chord(PitchClass::F, ChordQuality::Major, ChordDegree::IV),
            make_chord(PitchClass::G, ChordQuality::Major, ChordDegree::V),
            make_chord(PitchClass::A, ChordQuality::Minor, ChordDegree::VI),
        ];

        for voicing in [
            PadVoicingType::Close,
            PadVoicingType::Open,
            PadVoicingType::Drop2,
        ] {
            let config = PadConfig {
                chords_per_bar: &chords,
                part: SongPart::Verse,
                channel: 11,
                range: (36, 84),
                voicing: Some(voicing),
            };

            let mut engine = PadEngine::new(42);
            let pattern = engine.generate_pads(&config);

            for event in &pattern.events {
                assert!(
                    event.note >= 36 && event.note <= 84,
                    "note {} outside range [36, 84] in voicing {:?}",
                    event.note,
                    voicing,
                );
            }
        }
    }

    // -- Output structure ---------------------------------------------------

    #[test]
    fn pad_produces_events() {
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&chords);

        let mut engine = PadEngine::new(42);
        let pattern = engine.generate_pads(&config);

        assert!(!pattern.events.is_empty(), "pads should produce events");
        assert_eq!(pattern.bars, 4);
        assert_eq!(pattern.length_ticks, 4 * TICKS_PER_BAR);
    }

    #[test]
    fn pad_events_sorted_by_tick() {
        let chords = vec![c_major_chord(); 4];
        let config = test_config(&chords);

        let mut engine = PadEngine::new(42);
        let pattern = engine.generate_pads(&config);

        for i in 1..pattern.events.len() {
            assert!(
                pattern.events[i].tick >= pattern.events[i - 1].tick,
                "events should be sorted by tick",
            );
        }
    }

    #[test]
    fn pad_sustained_durations() {
        let chords = vec![c_major_chord(); 2];
        let config = PadConfig {
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 11,
            range: (36, 84),
            voicing: Some(PadVoicingType::Close),
        };

        let mut engine = PadEngine::new(42);
        let pattern = engine.generate_pads(&config);

        // Main chord hits (first beat) should have long durations near TICKS_PER_BAR
        let first_bar_events: Vec<_> = pattern
            .events
            .iter()
            .filter(|e| e.tick < TICKS_PER_BAR / 2)
            .collect();

        for event in &first_bar_events {
            assert!(
                event.duration >= TICKS_PER_BAR - 50,
                "pad duration {} too short for sustained chord",
                event.duration,
            );
        }
    }

    // -- Voice leading across progression -----------------------------------

    #[test]
    fn voice_leading_across_progression() {
        let chords = vec![
            c_major_chord(),
            make_chord(PitchClass::A, ChordQuality::Minor, ChordDegree::VI),
            make_chord(PitchClass::F, ChordQuality::Major, ChordDegree::IV),
            make_chord(PitchClass::G, ChordQuality::Major, ChordDegree::V),
        ];
        let config = PadConfig {
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 11,
            range: (36, 84),
            voicing: Some(PadVoicingType::Close),
        };

        let mut engine = PadEngine::new(42);
        let pattern = engine.generate_pads(&config);

        // Each bar should contain chord tones on the first beat
        for (bar_idx, chord) in chords.iter().enumerate() {
            let bar_start = bar_idx as u32 * TICKS_PER_BAR;
            let bar_end = bar_start + TICKS_PER_BEAT;
            let chord_pcs = chord.notes();

            let bar_notes: Vec<_> = pattern
                .events
                .iter()
                .filter(|e| e.tick >= bar_start && e.tick < bar_end)
                .collect();

            assert!(!bar_notes.is_empty(), "bar {} should have notes", bar_idx);

            for event in &bar_notes {
                let pc = PitchClass::from_midi(event.note);
                assert!(
                    chord_pcs.contains(&pc),
                    "bar {} note {} ({:?}) is not a chord tone of {:?}",
                    bar_idx,
                    event.note,
                    pc,
                    chord.root,
                );
            }
        }
    }

    // -- Empty / edge cases -------------------------------------------------

    #[test]
    fn empty_chords_produces_empty_pattern() {
        let chords: Vec<Chord> = vec![];
        let config = PadConfig {
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 11,
            range: (36, 84),
            voicing: None,
        };

        let mut engine = PadEngine::new(42);
        let pattern = engine.generate_pads(&config);
        assert!(pattern.events.is_empty());
        assert_eq!(pattern.bars, 0);
    }

    #[test]
    fn single_bar_produces_events() {
        let chords = vec![c_major_chord()];
        let config = test_config(&chords);

        let mut engine = PadEngine::new(42);
        let pattern = engine.generate_pads(&config);

        assert!(!pattern.events.is_empty());
        assert_eq!(pattern.bars, 1);
    }

    // -- Voicing type affects spread ----------------------------------------

    #[test]
    fn open_voicing_produces_wider_spread_than_close() {
        let chords = vec![c_major_chord(); 4];

        let close_config = PadConfig {
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 11,
            range: (36, 84),
            voicing: Some(PadVoicingType::Close),
        };
        let open_config = PadConfig {
            chords_per_bar: &chords,
            part: SongPart::Verse,
            channel: 11,
            range: (36, 84),
            voicing: Some(PadVoicingType::Open),
        };

        let mut engine = PadEngine::new(42);
        let close_pattern = engine.generate_pads(&close_config);

        let mut engine = PadEngine::new(42);
        let open_pattern = engine.generate_pads(&open_config);

        // Check first bar spread
        let close_first: Vec<u8> = close_pattern
            .events
            .iter()
            .filter(|e| e.tick < TICKS_PER_BEAT)
            .map(|e| e.note)
            .collect();
        let open_first: Vec<u8> = open_pattern
            .events
            .iter()
            .filter(|e| e.tick < TICKS_PER_BEAT)
            .map(|e| e.note)
            .collect();

        let close_spread =
            close_first.iter().max().unwrap_or(&0) - close_first.iter().min().unwrap_or(&0);
        let open_spread =
            open_first.iter().max().unwrap_or(&0) - open_first.iter().min().unwrap_or(&0);

        assert!(
            open_spread > close_spread,
            "open spread {} should be wider than close {}",
            open_spread,
            close_spread,
        );
    }

    // -- Out-of-range root fallback (issue #105) --------------------------------

    #[test]
    fn close_voicing_out_of_range_root_returns_valid_notes() {
        // Use a very narrow range where the full chord can't fit in a single octave
        // but the root itself still exists in range
        let chord = c_major_chord(); // C E G — intervals [0, 4, 7]
        // Range 60-64: root C=60 fits, but G=67 exceeds range.
        // The strict loop (root+top <= range.1) would fail, but fallback should find root=60.
        let voicing = PadEngine::close_voicing(&chord, 62, (60, 64));
        // Should contain at least the root, not default to range.0 blindly
        assert!(!voicing.is_empty(), "should produce notes even with narrow range");
        for &note in &voicing {
            let pc = PitchClass::from_midi(note);
            let chord_pcs = chord.notes();
            assert!(
                chord_pcs.contains(&pc),
                "note {} ({:?}) should be a chord tone",
                note,
                pc,
            );
        }
    }

    #[test]
    fn close_voicing_impossible_range_returns_empty() {
        // Range where no valid root pitch class exists at all (range too small and misaligned)
        let chord = make_chord(PitchClass::Cs, ChordQuality::Major, ChordDegree::I);
        // Range 60-60: only MIDI 60 = C, but root is C#. No C# exists in this range.
        let voicing = PadEngine::close_voicing(&chord, 60, (60, 60));
        assert!(voicing.is_empty(), "impossible range should return empty voicing");
    }

    #[test]
    fn open_voicing_out_of_range_root_returns_valid_notes() {
        let chord = c_major_chord();
        // Narrow range but root C exists within it
        let voicing = PadEngine::open_voicing(&chord, 62, (60, 67));
        assert!(!voicing.is_empty(), "should produce notes with narrow range");
        for &note in &voicing {
            assert!(note >= 60 && note <= 67, "note {} outside range", note);
        }
    }

    #[test]
    fn open_voicing_impossible_range_returns_empty() {
        let chord = make_chord(PitchClass::Cs, ChordQuality::Major, ChordDegree::I);
        let voicing = PadEngine::open_voicing(&chord, 60, (60, 60));
        assert!(voicing.is_empty(), "impossible range should return empty voicing");
    }

    #[test]
    fn voice_leading_no_valid_target_does_not_corrupt_state() {
        // Set up a scenario where all targets are used (more voices than targets)
        // 4 previous voices but only 3 target pitch classes
        let prev_voicing = vec![60, 64, 67, 72]; // 4 voices
        let next_chord = c_major_chord(); // C E G = 3 pitch classes

        let result = PadEngine::voice_lead(
            &prev_voicing,
            &next_chord,
            PadVoicingType::Close,
            (36, 84),
        );

        // Should have 4 notes (one per previous voice)
        assert_eq!(result.len(), 4, "should have one note per previous voice");

        // All notes should be in range
        for &note in &result {
            assert!(note >= 36 && note <= 84, "note {} outside range", note);
        }

        // All notes should be chord tones
        let chord_pcs = next_chord.notes();
        for &note in &result {
            let pc = PitchClass::from_midi(note);
            assert!(
                chord_pcs.contains(&pc),
                "note {} ({:?}) should be a chord tone after voice leading",
                note,
                pc,
            );
        }
    }
}
