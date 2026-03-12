// Song structure, section transitions, and dynamics scaling.
//
// Builds an ArrangementPlan from a Song definition: resolves chord progressions
// to concrete Chord structs per bar, computes section tick ranges with occurrence-
// based seed offsets, generates transition events (crash cymbals, open hi-hat,
// velocity boosts) at section boundaries.
// Fully implemented — awaiting GUI integration (v0.4.0).
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use super::rhythm::dynamics_scale;
use super::song::{InstrumentType, NoteEvent, Song, SongPart};
use super::theory::{Chord, ChordDegree, ChordQuality, Scale};
use super::{TICKS_PER_BAR, TICKS_PER_BEAT};

// ---------------------------------------------------------------------------
// TransitionKind
// ---------------------------------------------------------------------------

/// Category of transition event at a section boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    /// Crash cymbal on beat 1 of the new section.
    Crash,
    /// Open hi-hat on the "and of 4" of the preceding section's last bar.
    HiHatOpen,
    /// Velocity boost (+5) applied to first-bar events of the new section.
    VelocityBoost,
}

// ---------------------------------------------------------------------------
// TransitionEvent
// ---------------------------------------------------------------------------

/// A single transition event generated at a section boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionEvent {
    pub event: NoteEvent,
    pub kind: TransitionKind,
}

// ---------------------------------------------------------------------------
// SectionInstance
// ---------------------------------------------------------------------------

/// A concrete instance of a song part within the arrangement.
/// Repeated parts (e.g. two Verses) each get a distinct `occurrence` for
/// seed variation per the spec.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionInstance {
    /// Which song part this section is.
    pub part: SongPart,
    /// Zero-based occurrence index (0 = first Verse, 1 = second Verse, etc.).
    pub occurrence: u32,
    /// Number of bars in this section.
    pub bars: u32,
    /// Absolute start bar index from the beginning of the song.
    pub start_bar: u32,
    /// Absolute tick offset from song start.
    pub start_tick: u32,
    /// Absolute tick end (exclusive).
    pub end_tick: u32,
    /// Chord progression resolved to concrete Chord structs, one per bar.
    pub chords: Vec<Chord>,
    /// Dynamics multiplier for this section.
    pub dynamics: f64,
    /// Seed offset for this section instance (base_seed + occurrence).
    pub seed_offset: u64,
}

// ---------------------------------------------------------------------------
// ArrangementPlan
// ---------------------------------------------------------------------------

/// Full arrangement plan derived from a Song. Pure data — no IO, no async.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrangementPlan {
    /// Ordered section instances matching the song structure.
    pub sections: Vec<SectionInstance>,
    /// Transition events at section boundaries (crashes, hi-hat opens, boosts).
    pub transitions: Vec<TransitionEvent>,
    /// Total bars in the arrangement.
    pub total_bars: u32,
    /// Total ticks in the arrangement.
    pub total_ticks: u32,
}

// ---------------------------------------------------------------------------
// ArrangementEngine
// ---------------------------------------------------------------------------

/// Arrangement engine that builds a concrete plan from a Song definition.
/// Pure engine: no IO, no async, no GUI types. Data in, data out.
pub struct ArrangementEngine;

impl ArrangementEngine {
    /// Build a complete arrangement plan from a song definition and base seed.
    ///
    /// This resolves the abstract song structure into concrete section instances
    /// with tick ranges, resolved chords per bar, dynamics multipliers, and
    /// transition events at every section boundary.
    pub fn build_plan(song: &Song, base_seed: u64) -> ArrangementPlan {
        let sections = Self::build_sections(song, base_seed);
        let transitions = Self::build_transitions(song, &sections);
        let total_bars: u32 = sections.iter().map(|s| s.bars).sum();
        let total_ticks = total_bars * TICKS_PER_BAR;

        ArrangementPlan {
            sections,
            transitions,
            total_bars,
            total_ticks,
        }
    }

    // -- Section building ---------------------------------------------------

    /// Build section instances from the song structure, tracking occurrence
    /// counts for seed variation.
    fn build_sections(song: &Song, base_seed: u64) -> Vec<SectionInstance> {
        let mut sections = Vec::new();
        let mut occurrence_counts = std::collections::HashMap::new();
        let mut current_bar: u32 = 0;

        for &part in &song.structure {
            let occurrence = occurrence_counts.entry(part).or_insert(0u32);
            let bars = part.typical_bars();
            let start_tick = current_bar * TICKS_PER_BAR;
            let end_tick = start_tick + bars * TICKS_PER_BAR;

            let chords = Self::resolve_chords_for_section(
                song,
                part,
                bars,
            );

            sections.push(SectionInstance {
                part,
                occurrence: *occurrence,
                bars,
                start_bar: current_bar,
                start_tick,
                end_tick,
                chords,
                dynamics: dynamics_scale(part),
                seed_offset: base_seed.wrapping_add(*occurrence as u64),
            });

            *occurrence += 1;
            current_bar += bars;
        }

        sections
    }

    /// Resolve a chord progression for a section into concrete Chord structs,
    /// one per bar. The progression cycles if the section has more bars than
    /// chords in the progression.
    fn resolve_chords_for_section(
        song: &Song,
        part: SongPart,
        bars: u32,
    ) -> Vec<Chord> {
        let progression = match song.progressions.get(&part) {
            Some(prog) => prog.as_slice(),
            None => &[ChordDegree::I],
        };

        let diatonic = song.rhythm_scale.diatonic_chords();

        (0..bars)
            .map(|bar_idx| {
                let degree = progression[bar_idx as usize % progression.len()];
                Self::resolve_chord(&song.rhythm_scale, degree, &diatonic)
            })
            .collect()
    }

    /// Resolve a single ChordDegree to a concrete Chord using the scale's
    /// diatonic chord table.
    fn resolve_chord(
        scale: &Scale,
        degree: ChordDegree,
        diatonic: &[(ChordDegree, ChordQuality)],
    ) -> Chord {
        let idx = degree.to_index();
        let quality = diatonic[idx].1;

        let intervals = scale.scale_type.parent_diatonic_intervals();
        let root_semitone = intervals
            .get(idx)
            .copied()
            .unwrap_or(0);
        let root = scale.root.transpose(root_semitone as i8);

        Chord {
            root,
            quality,
            degree,
            inversion: 0,
        }
    }

    // -- Transition events --------------------------------------------------

    /// Build all transition events at section boundaries.
    fn build_transitions(
        song: &Song,
        sections: &[SectionInstance],
    ) -> Vec<TransitionEvent> {
        let mut transitions = Vec::new();

        for (i, section) in sections.iter().enumerate() {
            let prev = if i > 0 { Some(&sections[i - 1]) } else { None };

            // Crash cymbal placement per spec:
            // - First bar of Chorus
            // - First bar of Bridge (if drums are active)
            // - First bar after a part that had no drums (re-entry)
            let needs_crash = Self::needs_crash(song, section, prev);
            if needs_crash {
                let crash_note = 49; // GM Crash Cymbal 1
                transitions.push(TransitionEvent {
                    event: NoteEvent {
                        tick: section.start_tick,
                        note: crash_note,
                        velocity: 110,
                        duration: TICKS_PER_BAR, // ring for a full bar
                        channel: Self::find_drum_channel(song),
                    },
                    kind: TransitionKind::Crash,
                });
            }

            // Hi-hat open on "and of 4" of the preceding section's last bar
            if let Some(prev_section) = prev {
                let prev_drums_active = Self::drums_active_in(song, prev_section.part);
                if prev_drums_active {
                    let and_of_4_tick = prev_section.end_tick
                        .saturating_sub(TICKS_PER_BEAT / 2); // 240 ticks before end
                    let open_hat_note = 46; // GM Open Hi-Hat
                    transitions.push(TransitionEvent {
                        event: NoteEvent {
                            tick: and_of_4_tick,
                            note: open_hat_note,
                            velocity: 90,
                            duration: TICKS_PER_BEAT / 2, // 240 ticks (one 8th note)
                            channel: Self::find_drum_channel(song),
                        },
                        kind: TransitionKind::HiHatOpen,
                    });
                }
            }

            // Velocity boost: +5 to all events in first bar of new section
            // We emit a marker event at the section start. The composer
            // orchestrator applies the boost when assembling final MIDI.
            transitions.push(TransitionEvent {
                event: NoteEvent {
                    tick: section.start_tick,
                    note: 0,
                    velocity: 5, // boost amount
                    duration: TICKS_PER_BAR,
                    channel: 255, // sentinel: applies to all channels
                },
                kind: TransitionKind::VelocityBoost,
            });
        }

        transitions.sort_by_key(|t| t.event.tick);
        transitions
    }

    /// Determine whether a crash cymbal should be placed at the start of a section.
    fn needs_crash(
        song: &Song,
        section: &SectionInstance,
        prev: Option<&SectionInstance>,
    ) -> bool {
        // First bar of Chorus
        if section.part == SongPart::Chorus {
            return true;
        }

        // First bar of Bridge (if drums are active in bridge)
        if section.part == SongPart::Bridge && Self::drums_active_in(song, SongPart::Bridge) {
            return true;
        }

        // Re-entry: first bar after a part that had no drums
        if let Some(prev_section) = prev {
            let prev_had_drums = Self::drums_active_in(song, prev_section.part);
            let curr_has_drums = Self::drums_active_in(song, section.part);
            if !prev_had_drums && curr_has_drums {
                return true;
            }
        }

        false
    }

    /// Check if any drum track is active in the given song part.
    fn drums_active_in(song: &Song, part: SongPart) -> bool {
        song.tracks.iter().any(|track| {
            track.instrument.is_percussion()
                && track
                    .active_parts
                    .get(&part)
                    .copied()
                    .unwrap_or(false)
        })
    }

    /// Find the MIDI channel used by a drum track (prefer hi-hat channel).
    fn find_drum_channel(song: &Song) -> u8 {
        song.tracks
            .iter()
            .find(|t| t.instrument == InstrumentType::HiHat)
            .or_else(|| song.tracks.iter().find(|t| t.instrument.is_percussion()))
            .map(|t| t.id)
            .unwrap_or(9) // GM drum channel fallback
    }

    // -- Velocity boost application ----------------------------------------

    /// Apply the velocity boost (+5) to events in the first bar of each section.
    /// Called by the composer orchestrator after generating all track patterns.
    pub fn apply_velocity_boosts(
        events: &mut [NoteEvent],
        sections: &[SectionInstance],
    ) {
        for section in sections {
            let bar_end = section.start_tick + TICKS_PER_BAR;
            for event in events.iter_mut() {
                if event.tick >= section.start_tick && event.tick < bar_end {
                    event.velocity = (event.velocity as u16 + 5).min(127) as u8;
                }
            }
        }
    }

    // -- Section seed computation ------------------------------------------

    /// Compute the seed for a specific section instance.
    /// Uses `base_seed + occurrence_index` per the spec for variation
    /// between repeated parts.
    pub fn section_seed(base_seed: u64, occurrence: u32) -> u64 {
        base_seed.wrapping_add(occurrence as u64)
    }

    // -- Chord query helpers -----------------------------------------------

    /// Get the chords for a specific bar index across the whole arrangement.
    pub fn chord_at_bar(plan: &ArrangementPlan, bar: u32) -> Option<&Chord> {
        for section in &plan.sections {
            if bar >= section.start_bar && bar < section.start_bar + section.bars {
                let local_bar = (bar - section.start_bar) as usize;
                return section.chords.get(local_bar);
            }
        }
        None
    }

    /// Get the section instance that contains a given tick position.
    pub fn section_at_tick(plan: &ArrangementPlan, tick: u32) -> Option<&SectionInstance> {
        plan.sections
            .iter()
            .find(|s| tick >= s.start_tick && tick < s.end_tick)
    }

    /// Get all transition events of a specific kind.
    pub fn transitions_of_kind(
        plan: &ArrangementPlan,
        kind: TransitionKind,
    ) -> Vec<&TransitionEvent> {
        plan.transitions
            .iter()
            .filter(|t| t.kind == kind)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::theory::PitchClass;

    fn default_song() -> Song {
        Song::default_song()
    }

    // -- Section building ---------------------------------------------------

    #[test]
    fn plan_has_correct_section_count() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);
        // Default structure: Intro, Verse, PreChorus, Chorus, Verse, PreChorus, Chorus, Bridge, Chorus, Outro
        assert_eq!(plan.sections.len(), 10);
    }

    #[test]
    fn plan_total_bars_matches_song() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);
        assert_eq!(plan.total_bars, song.total_bars());
        assert_eq!(plan.total_ticks, song.total_ticks());
    }

    #[test]
    fn sections_have_correct_tick_ranges() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        // Sections should be contiguous
        for i in 1..plan.sections.len() {
            assert_eq!(
                plan.sections[i].start_tick,
                plan.sections[i - 1].end_tick,
                "section {} start should equal section {} end",
                i,
                i - 1,
            );
        }

        // First section starts at 0
        assert_eq!(plan.sections[0].start_tick, 0);

        // Last section ends at total ticks
        let last = plan.sections.last().expect("should have sections");
        assert_eq!(last.end_tick, plan.total_ticks);
    }

    #[test]
    fn sections_have_correct_bar_counts() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        assert_eq!(plan.sections[0].bars, 4);  // Intro
        assert_eq!(plan.sections[1].bars, 8);  // Verse
        assert_eq!(plan.sections[2].bars, 4);  // PreChorus
        assert_eq!(plan.sections[3].bars, 8);  // Chorus
    }

    #[test]
    fn occurrence_indices_track_repeats() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        // Find all Verse sections
        let verses: Vec<_> = plan.sections.iter().filter(|s| s.part == SongPart::Verse).collect();
        assert_eq!(verses.len(), 2);
        assert_eq!(verses[0].occurrence, 0);
        assert_eq!(verses[1].occurrence, 1);

        // Choruses appear 3 times
        let choruses: Vec<_> = plan.sections.iter().filter(|s| s.part == SongPart::Chorus).collect();
        assert_eq!(choruses.len(), 3);
        assert_eq!(choruses[0].occurrence, 0);
        assert_eq!(choruses[1].occurrence, 1);
        assert_eq!(choruses[2].occurrence, 2);
    }

    #[test]
    fn seed_offsets_differ_for_repeated_parts() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let verses: Vec<_> = plan.sections.iter().filter(|s| s.part == SongPart::Verse).collect();
        assert_ne!(
            verses[0].seed_offset,
            verses[1].seed_offset,
            "repeated verse sections should have different seeds"
        );
    }

    // -- Chord resolution ---------------------------------------------------

    #[test]
    fn chords_resolve_correctly_for_bb_major() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        // Intro has progression [I] -> all bars should be Bb major
        let intro = &plan.sections[0];
        assert_eq!(intro.chords.len(), 4);
        for chord in &intro.chords {
            assert_eq!(chord.root, PitchClass::As); // Bb
            assert_eq!(chord.quality, ChordQuality::Major);
            assert_eq!(chord.degree, ChordDegree::I);
        }
    }

    #[test]
    fn verse_progression_resolves_i_v_vi_iv() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        // Verse has I-V-vi-IV progression over 8 bars (cycles twice)
        let verse = &plan.sections[1];
        assert_eq!(verse.chords.len(), 8);

        // First cycle
        assert_eq!(verse.chords[0].degree, ChordDegree::I);
        assert_eq!(verse.chords[1].degree, ChordDegree::V);
        assert_eq!(verse.chords[2].degree, ChordDegree::VI);
        assert_eq!(verse.chords[3].degree, ChordDegree::IV);

        // Second cycle
        assert_eq!(verse.chords[4].degree, ChordDegree::I);
        assert_eq!(verse.chords[5].degree, ChordDegree::V);
        assert_eq!(verse.chords[6].degree, ChordDegree::VI);
        assert_eq!(verse.chords[7].degree, ChordDegree::IV);
    }

    #[test]
    fn chord_qualities_match_diatonic_table() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        // In Bb major: I=Major, V=Major, vi=Minor, IV=Major
        let verse = &plan.sections[1];
        assert_eq!(verse.chords[0].quality, ChordQuality::Major); // I
        assert_eq!(verse.chords[1].quality, ChordQuality::Major); // V
        assert_eq!(verse.chords[2].quality, ChordQuality::Minor); // vi
        assert_eq!(verse.chords[3].quality, ChordQuality::Major); // IV
    }

    #[test]
    fn chord_roots_match_bb_major_scale() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        // Bb major scale degrees: Bb, C, D, Eb, F, G, A
        let verse = &plan.sections[1];
        assert_eq!(verse.chords[0].root, PitchClass::As); // I = Bb
        assert_eq!(verse.chords[1].root, PitchClass::F);  // V = F
        assert_eq!(verse.chords[2].root, PitchClass::G);  // vi = G
        assert_eq!(verse.chords[3].root, PitchClass::Ds); // IV = Eb
    }

    // -- Dynamics -----------------------------------------------------------

    #[test]
    fn dynamics_multipliers_match_spec() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let intro = plan.sections.iter().find(|s| s.part == SongPart::Intro).expect("intro");
        assert!((intro.dynamics - 0.55).abs() < f64::EPSILON);

        let chorus = plan.sections.iter().find(|s| s.part == SongPart::Chorus).expect("chorus");
        assert!((chorus.dynamics - 1.0).abs() < f64::EPSILON);

        let bridge = plan.sections.iter().find(|s| s.part == SongPart::Bridge).expect("bridge");
        assert!((bridge.dynamics - 0.65).abs() < f64::EPSILON);

        let outro = plan.sections.iter().find(|s| s.part == SongPart::Outro).expect("outro");
        assert!((outro.dynamics - 0.50).abs() < f64::EPSILON);
    }

    // -- Transition events --------------------------------------------------

    #[test]
    fn crash_on_every_chorus() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let crashes = ArrangementEngine::transitions_of_kind(&plan, TransitionKind::Crash);
        let chorus_sections: Vec<_> = plan.sections.iter().filter(|s| s.part == SongPart::Chorus).collect();

        // Every chorus gets a crash
        for chorus in &chorus_sections {
            let has_crash = crashes.iter().any(|c| c.event.tick == chorus.start_tick);
            assert!(
                has_crash,
                "chorus at tick {} should have a crash cymbal",
                chorus.start_tick,
            );
        }
    }

    #[test]
    fn crash_note_is_gm_crash() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let crashes = ArrangementEngine::transitions_of_kind(&plan, TransitionKind::Crash);
        assert!(!crashes.is_empty(), "should have crash events");

        for crash in &crashes {
            assert_eq!(crash.event.note, 49, "crash should use GM crash cymbal note 49");
            assert_eq!(crash.event.duration, TICKS_PER_BAR, "crash should ring for a full bar");
        }
    }

    #[test]
    fn hi_hat_open_before_transitions() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let opens = ArrangementEngine::transitions_of_kind(&plan, TransitionKind::HiHatOpen);

        for open in &opens {
            assert_eq!(open.event.note, 46, "open hi-hat should use GM note 46");
            assert_eq!(
                open.event.duration,
                TICKS_PER_BEAT / 2,
                "open hi-hat should last one 8th note (240 ticks)"
            );
        }
    }

    #[test]
    fn velocity_boost_at_every_section() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let boosts = ArrangementEngine::transitions_of_kind(&plan, TransitionKind::VelocityBoost);
        assert_eq!(
            boosts.len(),
            plan.sections.len(),
            "every section should have a velocity boost marker"
        );
    }

    #[test]
    fn transitions_sorted_by_tick() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        for i in 1..plan.transitions.len() {
            assert!(
                plan.transitions[i].event.tick >= plan.transitions[i - 1].event.tick,
                "transitions should be sorted by tick"
            );
        }
    }

    // -- Crash on bridge with drums -----------------------------------------

    #[test]
    fn crash_on_bridge_when_drums_active() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let bridge = plan.sections.iter().find(|s| s.part == SongPart::Bridge).expect("bridge");
        let crashes = ArrangementEngine::transitions_of_kind(&plan, TransitionKind::Crash);
        let bridge_crash = crashes.iter().any(|c| c.event.tick == bridge.start_tick);
        assert!(bridge_crash, "bridge should have a crash (drums are active in bridge)");
    }

    // -- Query helpers ------------------------------------------------------

    #[test]
    fn chord_at_bar_returns_correct_chord() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        // Bar 0 is Intro, should be I (Bb major)
        let chord = ArrangementEngine::chord_at_bar(&plan, 0).expect("chord at bar 0");
        assert_eq!(chord.degree, ChordDegree::I);
        assert_eq!(chord.root, PitchClass::As);

        // Bar 4 is first bar of Verse, should be I
        let chord = ArrangementEngine::chord_at_bar(&plan, 4).expect("chord at bar 4");
        assert_eq!(chord.degree, ChordDegree::I);

        // Bar 5 is second bar of Verse, should be V
        let chord = ArrangementEngine::chord_at_bar(&plan, 5).expect("chord at bar 5");
        assert_eq!(chord.degree, ChordDegree::V);
    }

    #[test]
    fn chord_at_bar_out_of_range_returns_none() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);
        assert!(ArrangementEngine::chord_at_bar(&plan, 9999).is_none());
    }

    #[test]
    fn section_at_tick_finds_correct_section() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        // Tick 0 should be Intro
        let section = ArrangementEngine::section_at_tick(&plan, 0).expect("section at tick 0");
        assert_eq!(section.part, SongPart::Intro);

        // Tick at start of second section (Verse, 4 bars in)
        let verse_start = 4 * TICKS_PER_BAR;
        let section = ArrangementEngine::section_at_tick(&plan, verse_start).expect("section at verse");
        assert_eq!(section.part, SongPart::Verse);
    }

    #[test]
    fn section_at_tick_past_end_returns_none() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);
        assert!(ArrangementEngine::section_at_tick(&plan, u32::MAX).is_none());
    }

    // -- Velocity boost application ----------------------------------------

    #[test]
    fn apply_velocity_boosts_adds_5() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let mut events = vec![
            NoteEvent { tick: 0, note: 60, velocity: 100, duration: 480, channel: 0 },
            NoteEvent { tick: 480, note: 62, velocity: 90, duration: 480, channel: 0 },
        ];

        ArrangementEngine::apply_velocity_boosts(&mut events, &plan.sections);

        // Both events are in the first bar of Intro (section 0)
        assert_eq!(events[0].velocity, 105);
        assert_eq!(events[1].velocity, 95);
    }

    #[test]
    fn velocity_boost_caps_at_127() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);

        let mut events = vec![
            NoteEvent { tick: 0, note: 60, velocity: 125, duration: 480, channel: 0 },
        ];

        ArrangementEngine::apply_velocity_boosts(&mut events, &plan.sections);
        assert_eq!(events[0].velocity, 127, "velocity should cap at 127");
    }

    // -- Section seed -------------------------------------------------------

    #[test]
    fn section_seed_deterministic() {
        assert_eq!(
            ArrangementEngine::section_seed(42, 0),
            ArrangementEngine::section_seed(42, 0),
        );
        assert_ne!(
            ArrangementEngine::section_seed(42, 0),
            ArrangementEngine::section_seed(42, 1),
        );
    }

    // -- Determinism --------------------------------------------------------

    #[test]
    fn same_seed_produces_identical_plan() {
        let song = default_song();
        let plan1 = ArrangementEngine::build_plan(&song, 42);
        let plan2 = ArrangementEngine::build_plan(&song, 42);
        assert_eq!(plan1, plan2, "same seed must produce identical plan");
    }

    // -- Serde roundtrips ---------------------------------------------------

    #[test]
    fn transition_kind_serde_roundtrip() {
        let kind = TransitionKind::Crash;
        let json = serde_json::to_string(&kind).expect("serialize");
        assert_eq!(json, r#""crash""#);
        let parsed: TransitionKind = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, kind);
    }

    #[test]
    fn arrangement_plan_serde_roundtrip() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);
        let json = serde_json::to_string(&plan).expect("serialize");
        let parsed: ArrangementPlan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, plan);
    }

    #[test]
    fn section_instance_serde_roundtrip() {
        let song = default_song();
        let plan = ArrangementEngine::build_plan(&song, 42);
        let section = &plan.sections[0];
        let json = serde_json::to_string(section).expect("serialize");
        let parsed: SectionInstance = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, *section);
    }

    // -- Empty structure edge case ------------------------------------------

    #[test]
    fn empty_structure_produces_empty_plan() {
        let mut song = default_song();
        song.structure.clear();
        let plan = ArrangementEngine::build_plan(&song, 42);
        assert!(plan.sections.is_empty());
        assert!(plan.transitions.is_empty());
        assert_eq!(plan.total_bars, 0);
        assert_eq!(plan.total_ticks, 0);
    }
}
