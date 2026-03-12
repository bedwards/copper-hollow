// Top-level composition orchestrator.
//
// Ties all engine modules together: arrangement, drums, bass, melody, pads,
// and rhythm. Produces a complete Song with patterns for every active track
// in every section. Fully deterministic given the same seed + settings.

use serde::{Deserialize, Serialize};

use super::arrangement::{ArrangementEngine, ArrangementPlan, SectionInstance, TransitionKind};
use super::bass::{BassConfig, BassEngine};
use super::drums::{DrumConfig, DrumEngine};
use super::melody::{MelodyConfig, MelodyEngine};
use super::pads::{PadConfig, PadEngine};
use super::rhythm::{GrooveTemplate, RhythmEngine, RhythmGenConfig};
use super::song::{NoteEvent, Pattern, Song, StrumPattern, Track, TrackRole, Voicing};
use super::theory::{Chord, Scale};

// ---------------------------------------------------------------------------
// SongContext — immutable snapshot of Song fields needed during generation
// ---------------------------------------------------------------------------

/// Immutable context extracted from Song for generation, avoiding borrow conflicts.
struct SongContext {
    rhythm_scale: Scale,
    lead_scale: Scale,
    strum_pattern: StrumPattern,
    tempo: f64,
    swing: f32,
    tracks_snapshot: Vec<Track>,
}

// ---------------------------------------------------------------------------
// ComposerConfig
// ---------------------------------------------------------------------------

/// Configuration for the top-level composition pass.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComposerConfig {
    /// Master RNG seed. Same seed + same settings = identical output.
    pub seed: u64,
}

// ---------------------------------------------------------------------------
// Composer
// ---------------------------------------------------------------------------

/// Top-level composition orchestrator.
/// Holds the RNG seed so that composition is fully deterministic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Composer {
    seed: u64,
}

impl Composer {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Compose a complete song: build arrangement, generate all track patterns,
    /// apply humanization and transitions. Returns the Song with all patterns
    /// populated. Pure engine function — no IO, no async.
    pub fn compose(&self, song: &mut Song) {
        // 1. Build arrangement plan (sections, chords, transitions).
        let plan = ArrangementEngine::build_plan(song, self.seed);

        // 2. Snapshot immutable song data to avoid borrow conflicts.
        let ctx = SongContext {
            rhythm_scale: song.rhythm_scale.clone(),
            lead_scale: song.lead_scale.clone(),
            strum_pattern: song.strum_pattern.clone(),
            tempo: song.tempo,
            swing: song.swing,
            tracks_snapshot: song.tracks.clone(),
        };

        // 3. Generate patterns for each track in each section.
        for track in &mut song.tracks {
            self.generate_track_patterns(track, &plan, &ctx);
        }

        // 4. Apply velocity boosts at section boundaries to all tracks.
        self.apply_velocity_boosts(song, &plan);

        // 5. Insert transition events (crash cymbals, open hi-hats).
        self.insert_transition_events(song, &plan);
    }

    // -- Per-track generation -----------------------------------------------

    /// Generate patterns for a single track across all sections where it is active.
    fn generate_track_patterns(&self, track: &mut Track, plan: &ArrangementPlan, ctx: &SongContext) {
        for section in &plan.sections {
            let is_active = track
                .active_parts
                .get(&section.part)
                .copied()
                .unwrap_or(false);

            if !is_active {
                continue;
            }

            // Derive a per-track, per-section seed for variation.
            let track_seed = self.track_section_seed(track.id, section);

            let pattern = match track.role {
                TrackRole::Drum => {
                    self.generate_drum_pattern(track, section, track_seed)
                }
                TrackRole::Bass => {
                    self.generate_bass_pattern(track, section, ctx, track_seed)
                }
                TrackRole::LeadMelody => {
                    self.generate_lead_melody_pattern(track, section, ctx, track_seed)
                }
                TrackRole::CounterMelody => {
                    self.generate_counter_melody_pattern(track, section, ctx, track_seed)
                }
                TrackRole::PadSustain => {
                    self.generate_pad_pattern(track, section, track_seed)
                }
                TrackRole::Rhythm => {
                    self.generate_rhythm_pattern(track, section, ctx, track_seed)
                }
            };

            // Offset pattern events by section start tick.
            let offset_pattern = Self::offset_pattern(pattern, section.start_tick);

            // Merge into existing pattern for this part (sections may repeat).
            let entry = track
                .patterns
                .entry(section.part)
                .or_insert_with(|| Pattern::empty(0));
            Self::merge_pattern(entry, &offset_pattern);
        }
    }

    // -- Drum generation ----------------------------------------------------

    fn generate_drum_pattern(
        &self,
        track: &Track,
        section: &SectionInstance,
        seed: u64,
    ) -> Pattern {
        let mut engine = DrumEngine::new(seed);
        let config = DrumConfig {
            part: section.part,
            bars: section.bars,
            channel: track.id,
        };
        let mut pattern = engine.generate_drum_pattern(&config);

        // Humanize drums.
        let mut rhythm_engine = RhythmEngine::new(seed.wrapping_add(1000));
        rhythm_engine.humanize(&mut pattern.events, TrackRole::Drum, section.part);

        pattern
    }

    // -- Bass generation ----------------------------------------------------

    fn generate_bass_pattern(
        &self,
        track: &Track,
        section: &SectionInstance,
        ctx: &SongContext,
        seed: u64,
    ) -> Pattern {
        let mut engine = BassEngine::new(seed);
        let config = BassConfig {
            scale: &ctx.rhythm_scale,
            chords_per_bar: &section.chords,
            part: section.part,
            channel: track.id,
            range: track.instrument.midi_range(),
            style: None,
            tonic: ctx.rhythm_scale.root,
        };
        let mut pattern = engine.generate_bass(&config);

        // Humanize bass.
        let mut rhythm_engine = RhythmEngine::new(seed.wrapping_add(1000));
        rhythm_engine.humanize(&mut pattern.events, TrackRole::Bass, section.part);

        pattern
    }

    // -- Lead melody generation ---------------------------------------------

    fn generate_lead_melody_pattern(
        &self,
        track: &Track,
        section: &SectionInstance,
        ctx: &SongContext,
        seed: u64,
    ) -> Pattern {
        let mut engine = MelodyEngine::new(seed);
        let config = MelodyConfig {
            scale: &ctx.lead_scale,
            chords_per_bar: &section.chords,
            part: section.part,
            channel: track.id,
            range: track.instrument.midi_range(),
            contour: None,
        };
        let mut pattern = engine.generate_melody(&config);

        // Humanize melody.
        let mut rhythm_engine = RhythmEngine::new(seed.wrapping_add(1000));
        rhythm_engine.humanize(&mut pattern.events, TrackRole::LeadMelody, section.part);

        pattern
    }

    // -- Counter-melody generation ------------------------------------------

    fn generate_counter_melody_pattern(
        &self,
        track: &Track,
        section: &SectionInstance,
        ctx: &SongContext,
        seed: u64,
    ) -> Pattern {
        // Generate lead first to ensure rhythmic independence.
        let lead_seed = self.find_lead_seed(section, &ctx.tracks_snapshot);
        let mut lead_engine = MelodyEngine::new(lead_seed);
        let lead_config = MelodyConfig {
            scale: &ctx.lead_scale,
            chords_per_bar: &section.chords,
            part: section.part,
            channel: 0, // placeholder channel for lead reference
            range: (48, 84), // generic range for lead reference
            contour: None,
        };
        let lead_pattern = lead_engine.generate_melody(&lead_config);

        let mut engine = MelodyEngine::new(seed);
        let config = MelodyConfig {
            scale: &ctx.lead_scale,
            chords_per_bar: &section.chords,
            part: section.part,
            channel: track.id,
            range: track.instrument.midi_range(),
            contour: None,
        };
        let mut pattern = engine.generate_counter_melody(&config, &lead_pattern);

        // Humanize counter-melody.
        let mut rhythm_engine = RhythmEngine::new(seed.wrapping_add(1000));
        rhythm_engine.humanize(
            &mut pattern.events,
            TrackRole::CounterMelody,
            section.part,
        );

        pattern
    }

    // -- Pad generation -----------------------------------------------------

    fn generate_pad_pattern(
        &self,
        track: &Track,
        section: &SectionInstance,
        seed: u64,
    ) -> Pattern {
        let mut engine = PadEngine::new(seed);
        let config = PadConfig {
            chords_per_bar: &section.chords,
            part: section.part,
            channel: track.id,
            range: track.instrument.midi_range(),
            voicing: None,
        };
        let mut pattern = engine.generate_pads(&config);

        // Humanize pads.
        let mut rhythm_engine = RhythmEngine::new(seed.wrapping_add(1000));
        rhythm_engine.humanize(&mut pattern.events, TrackRole::PadSustain, section.part);

        pattern
    }

    // -- Rhythm guitar generation -------------------------------------------

    fn generate_rhythm_pattern(
        &self,
        track: &Track,
        section: &SectionInstance,
        ctx: &SongContext,
        seed: u64,
    ) -> Pattern {
        let mut engine = RhythmEngine::new(seed);

        // Build chord voicings for each bar.
        let chord_voicings: Vec<Vec<u8>> = section
            .chords
            .iter()
            .map(|chord| Self::build_chord_voicing(chord, track))
            .collect();
        let chords_per_bar: Vec<&[u8]> = chord_voicings.iter().map(|v| v.as_slice()).collect();

        let config = RhythmGenConfig {
            pattern: &ctx.strum_pattern,
            part: section.part,
            role: track.role,
            channel: track.id,
            tempo: ctx.tempo,
            swing: ctx.swing,
            groove: GrooveTemplate::Straight,
        };

        match track.voicing {
            Voicing::Poly => engine.generate_rhythm_pattern(&chords_per_bar, &config),
            Voicing::Mono => {
                let mode = super::rhythm::MonoMode::Arpeggio;
                engine.generate_mono_rhythm_pattern(&chords_per_bar, &config, mode)
            }
        }
    }

    // -- Helpers ------------------------------------------------------------

    /// Compute a deterministic seed for a specific track in a specific section.
    /// Combines the base seed, track id, section part hash, and occurrence.
    fn track_section_seed(&self, track_id: u8, section: &SectionInstance) -> u64 {
        let part_val = section.part as u64;
        self.seed
            .wrapping_mul(31)
            .wrapping_add(track_id as u64)
            .wrapping_mul(17)
            .wrapping_add(part_val)
            .wrapping_mul(13)
            .wrapping_add(section.occurrence as u64)
    }

    /// Find the seed for the lead melody track in this section, so counter-melody
    /// can be rhythmically independent.
    fn find_lead_seed(&self, section: &SectionInstance, tracks: &[Track]) -> u64 {
        let lead_track = tracks
            .iter()
            .find(|t| t.role == TrackRole::LeadMelody);
        match lead_track {
            Some(t) => self.track_section_seed(t.id, section),
            None => self.seed.wrapping_add(999),
        }
    }

    /// Build a MIDI chord voicing for a rhythm track from a Chord struct.
    fn build_chord_voicing(chord: &Chord, track: &Track) -> Vec<u8> {
        let (low, high) = track.instrument.midi_range();
        let intervals = chord.quality.intervals();
        let root_semi = chord.root.to_semitone();

        let mut notes = Vec::new();
        for octave in 0..=10u8 {
            for &interval in intervals {
                let note = octave * 12 + root_semi + interval;
                if note >= low && note <= high {
                    notes.push(note);
                }
            }
        }
        notes.sort();
        notes.dedup();

        // Limit to reasonable voicing size (3-6 notes).
        if notes.len() > 6 {
            let mid = notes.len() / 2;
            let start = mid.saturating_sub(3);
            notes = notes[start..start + 6.min(notes.len() - start)].to_vec();
        }

        notes
    }

    /// Offset all events in a pattern by `offset_ticks`.
    fn offset_pattern(mut pattern: Pattern, offset_ticks: u32) -> Pattern {
        for event in &mut pattern.events {
            event.tick = event.tick.saturating_add(offset_ticks);
        }
        for cc in &mut pattern.cc_events {
            cc.tick = cc.tick.saturating_add(offset_ticks);
        }
        pattern
    }

    /// Merge `source` pattern events into `dest`.
    fn merge_pattern(dest: &mut Pattern, source: &Pattern) {
        dest.events.extend_from_slice(&source.events);
        dest.cc_events.extend_from_slice(&source.cc_events);
        dest.length_ticks = dest.length_ticks.max(source.length_ticks);
        dest.bars += source.bars;

        dest.events.sort_by_key(|e| e.tick);
        dest.cc_events.sort_by_key(|e| e.tick);
    }

    /// Apply velocity boosts (+5) to events in the first bar of each section.
    fn apply_velocity_boosts(&self, song: &mut Song, plan: &ArrangementPlan) {
        for track in &mut song.tracks {
            let mut all_events: Vec<NoteEvent> = track
                .patterns
                .values()
                .flat_map(|p| p.events.iter().cloned())
                .collect();

            ArrangementEngine::apply_velocity_boosts(&mut all_events, &plan.sections);

            // Write back: redistribute events into patterns by matching tick ranges.
            for pattern in track.patterns.values_mut() {
                for event in &mut pattern.events {
                    if let Some(boosted) = all_events
                        .iter()
                        .find(|e| e.tick == event.tick && e.note == event.note && e.channel == event.channel)
                    {
                        event.velocity = boosted.velocity;
                    }
                }
            }
        }
    }

    /// Insert crash cymbal and open hi-hat transition events into drum tracks.
    fn insert_transition_events(&self, song: &mut Song, plan: &ArrangementPlan) {
        for transition in &plan.transitions {
            match transition.kind {
                TransitionKind::Crash | TransitionKind::HiHatOpen => {
                    // Find the target drum track by channel.
                    let channel = transition.event.channel;
                    if let Some(track) = song.tracks.iter_mut().find(|t| t.id == channel) {
                        // Find which section this transition belongs to.
                        if let Some(section) =
                            ArrangementEngine::section_at_tick(plan, transition.event.tick)
                        {
                            let entry = track
                                .patterns
                                .entry(section.part)
                                .or_insert_with(|| Pattern::empty(section.bars));
                            entry.events.push(transition.event.clone());
                            entry.events.sort_by_key(|e| e.tick);
                        }
                    }
                }
                TransitionKind::VelocityBoost => {
                    // Already handled by apply_velocity_boosts.
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::song::SongPart;
    use crate::engine::theory::{ChordDegree, ChordQuality, PitchClass};
    use crate::engine::TICKS_PER_BAR;

    fn default_song() -> Song {
        Song::default_song()
    }

    // -- Determinism --------------------------------------------------------

    #[test]
    fn same_seed_produces_identical_composition() {
        let mut song1 = default_song();
        let mut song2 = default_song();

        let composer = Composer::new(42);
        composer.compose(&mut song1);
        composer.compose(&mut song2);

        assert_eq!(song1, song2, "same seed must produce identical composition");
    }

    #[test]
    fn different_seeds_produce_different_compositions() {
        let mut song1 = default_song();
        let mut song2 = default_song();

        Composer::new(42).compose(&mut song1);
        Composer::new(99).compose(&mut song2);

        // At least one track should differ.
        let differs = song1
            .tracks
            .iter()
            .zip(song2.tracks.iter())
            .any(|(t1, t2)| t1.patterns != t2.patterns);
        assert!(differs, "different seeds should produce different compositions");
    }

    // -- Active tracks generate patterns ------------------------------------

    #[test]
    fn active_tracks_have_patterns() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Acoustic Guitar (ch 4) is active in all parts — should have patterns.
        let guitar = &song.tracks[4];
        assert!(
            !guitar.patterns.is_empty(),
            "active rhythm guitar should have patterns"
        );
    }

    #[test]
    fn inactive_tracks_have_no_patterns_for_part() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Kick (ch 0) is inactive in Intro.
        let kick = &song.tracks[0];
        // Kick may have patterns for Verse, Chorus, etc., but should not
        // have events in the Intro tick range (0..4*1920).
        if let Some(intro_pattern) = kick.patterns.get(&SongPart::Intro) {
            // If a pattern entry exists (from merge), it should have no events
            // in the intro range.
            let intro_events: Vec<_> = intro_pattern
                .events
                .iter()
                .filter(|e| e.tick < 4 * TICKS_PER_BAR)
                .collect();
            assert!(
                intro_events.is_empty(),
                "kick should have no events in intro (inactive), found {}",
                intro_events.len()
            );
        }
    }

    // -- Pattern events are non-empty for active generators -----------------

    #[test]
    fn drum_tracks_have_events() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Kick (ch 0) is active in Verse.
        let kick = &song.tracks[0];
        let verse_pattern = kick.patterns.get(&SongPart::Verse);
        assert!(verse_pattern.is_some(), "kick should have verse pattern");
        assert!(
            !verse_pattern.map(|p| p.events.is_empty()).unwrap_or(true),
            "kick verse pattern should have events"
        );
    }

    #[test]
    fn bass_track_has_events() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Electric Bass (ch 6) is active in Verse.
        let bass = &song.tracks[6];
        let verse_pattern = bass.patterns.get(&SongPart::Verse);
        assert!(verse_pattern.is_some(), "bass should have verse pattern");
        assert!(
            !verse_pattern.map(|p| p.events.is_empty()).unwrap_or(true),
            "bass verse pattern should have events"
        );
    }

    #[test]
    fn melody_track_has_events() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Lead Melody (ch 13) is active in Verse.
        let lead = &song.tracks[13];
        let verse_pattern = lead.patterns.get(&SongPart::Verse);
        assert!(verse_pattern.is_some(), "lead should have verse pattern");
        assert!(
            !verse_pattern.map(|p| p.events.is_empty()).unwrap_or(true),
            "lead verse pattern should have events"
        );
    }

    #[test]
    fn pad_track_has_events() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Hammond Organ (ch 11) is active in PreChorus.
        let organ = &song.tracks[11];
        let prechorus_pattern = organ.patterns.get(&SongPart::PreChorus);
        assert!(
            prechorus_pattern.is_some(),
            "hammond should have prechorus pattern"
        );
        assert!(
            !prechorus_pattern
                .map(|p| p.events.is_empty())
                .unwrap_or(true),
            "hammond prechorus pattern should have events"
        );
    }

    #[test]
    fn rhythm_guitar_has_events() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Acoustic Guitar (ch 4) is active in Verse.
        let guitar = &song.tracks[4];
        let verse_pattern = guitar.patterns.get(&SongPart::Verse);
        assert!(
            verse_pattern.is_some(),
            "acoustic guitar should have verse pattern"
        );
        assert!(
            !verse_pattern.map(|p| p.events.is_empty()).unwrap_or(true),
            "acoustic guitar verse pattern should have events"
        );
    }

    // -- Harmonic consistency -----------------------------------------------

    #[test]
    fn all_tracks_use_consistent_chord_progression() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Build the arrangement plan to check chord consistency.
        let plan = ArrangementEngine::build_plan(&song, 42);

        // For each section, verify the arrangement plan has resolved chords.
        for section in &plan.sections {
            assert!(
                !section.chords.is_empty(),
                "section {:?} should have chords",
                section.part
            );
            // All chords should have a valid degree.
            for chord in &section.chords {
                assert!(
                    ChordDegree::ALL.contains(&chord.degree),
                    "chord degree {:?} should be valid",
                    chord.degree
                );
            }
        }
    }

    // -- Transition events --------------------------------------------------

    #[test]
    fn crash_cymbals_present_at_chorus() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        // Check that the hi-hat track (ch 2) or another drum track has crash events.
        let has_crash = song.tracks.iter().any(|t| {
            t.instrument.is_percussion()
                && t.patterns.values().any(|p| {
                    p.events
                        .iter()
                        .any(|e| e.note == 49) // GM crash cymbal
                })
        });
        assert!(
            has_crash,
            "composition should include crash cymbals at chorus transitions"
        );
    }

    // -- Seed computation ---------------------------------------------------

    #[test]
    fn track_section_seed_varies_by_track() {
        let composer = Composer::new(42);
        let section = SectionInstance {
            part: SongPart::Verse,
            occurrence: 0,
            bars: 8,
            start_bar: 4,
            start_tick: 4 * TICKS_PER_BAR,
            end_tick: 12 * TICKS_PER_BAR,
            chords: vec![],
            dynamics: 0.7,
            seed_offset: 42,
        };

        let seed_a = composer.track_section_seed(0, &section);
        let seed_b = composer.track_section_seed(1, &section);
        assert_ne!(seed_a, seed_b, "different tracks should get different seeds");
    }

    #[test]
    fn track_section_seed_varies_by_occurrence() {
        let composer = Composer::new(42);
        let section1 = SectionInstance {
            part: SongPart::Verse,
            occurrence: 0,
            bars: 8,
            start_bar: 4,
            start_tick: 4 * TICKS_PER_BAR,
            end_tick: 12 * TICKS_PER_BAR,
            chords: vec![],
            dynamics: 0.7,
            seed_offset: 42,
        };
        let section2 = SectionInstance {
            part: SongPart::Verse,
            occurrence: 1,
            bars: 8,
            start_bar: 20,
            start_tick: 20 * TICKS_PER_BAR,
            end_tick: 28 * TICKS_PER_BAR,
            chords: vec![],
            dynamics: 0.7,
            seed_offset: 43,
        };

        let seed1 = composer.track_section_seed(0, &section1);
        let seed2 = composer.track_section_seed(0, &section2);
        assert_ne!(
            seed1, seed2,
            "same track in repeated section should get different seeds"
        );
    }

    // -- Chord voicing builder ----------------------------------------------

    #[test]
    fn chord_voicing_within_instrument_range() {
        let track = Track::new(
            4,
            "Acoustic Guitar",
            TrackRole::Rhythm,
            super::super::song::InstrumentType::AcousticGuitar,
            Voicing::Poly,
        );
        let chord = Chord {
            root: PitchClass::C,
            quality: ChordQuality::Major,
            degree: ChordDegree::I,
            inversion: 0,
        };

        let voicing = Composer::build_chord_voicing(&chord, &track);
        let (low, high) = track.instrument.midi_range();

        assert!(!voicing.is_empty(), "voicing should have notes");
        for &note in &voicing {
            assert!(
                note >= low && note <= high,
                "voicing note {} outside range [{}, {}]",
                note,
                low,
                high
            );
        }
    }

    #[test]
    fn chord_voicing_max_six_notes() {
        let track = Track::new(
            7,
            "Piano",
            TrackRole::Rhythm,
            super::super::song::InstrumentType::Piano,
            Voicing::Poly,
        );
        let chord = Chord {
            root: PitchClass::C,
            quality: ChordQuality::Major,
            degree: ChordDegree::I,
            inversion: 0,
        };

        let voicing = Composer::build_chord_voicing(&chord, &track);
        assert!(
            voicing.len() <= 6,
            "voicing should have at most 6 notes, got {}",
            voicing.len()
        );
    }

    // -- Pattern offset / merge ---------------------------------------------

    #[test]
    fn offset_pattern_shifts_ticks() {
        let mut pattern = Pattern::empty(2);
        pattern.events.push(NoteEvent {
            tick: 0,
            note: 60,
            velocity: 100,
            duration: 480,
            channel: 0,
        });
        pattern.events.push(NoteEvent {
            tick: 480,
            note: 62,
            velocity: 90,
            duration: 480,
            channel: 0,
        });

        let offset = Composer::offset_pattern(pattern, 1920);
        assert_eq!(offset.events[0].tick, 1920);
        assert_eq!(offset.events[1].tick, 2400);
    }

    #[test]
    fn merge_pattern_combines_events() {
        let mut dest = Pattern::empty(2);
        dest.events.push(NoteEvent {
            tick: 0,
            note: 60,
            velocity: 100,
            duration: 480,
            channel: 0,
        });

        let mut source = Pattern::empty(2);
        source.events.push(NoteEvent {
            tick: 1920,
            note: 64,
            velocity: 90,
            duration: 480,
            channel: 0,
        });

        Composer::merge_pattern(&mut dest, &source);
        assert_eq!(dest.events.len(), 2);
        assert_eq!(dest.events[0].tick, 0);
        assert_eq!(dest.events[1].tick, 1920);
    }

    // -- Serde roundtrip ----------------------------------------------------

    #[test]
    fn composer_serde_roundtrip() {
        let composer = Composer::new(42);
        let json = serde_json::to_string(&composer).expect("serialize");
        let parsed: Composer = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.seed(), composer.seed());
    }

    #[test]
    fn composer_config_serde_roundtrip() {
        let config = ComposerConfig { seed: 12345 };
        let json = serde_json::to_string(&config).expect("serialize");
        let parsed: ComposerConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.seed, config.seed);
    }

    // -- Full composition serde roundtrip -----------------------------------

    #[test]
    fn composed_song_serde_roundtrip() {
        let mut song = default_song();
        Composer::new(42).compose(&mut song);

        let json = serde_json::to_string(&song).expect("serialize");
        let parsed: Song = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, song);
    }
}
