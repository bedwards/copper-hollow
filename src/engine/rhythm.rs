// Groove patterns, strum generation, humanization, and swing.

use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::song::{
    NoteEvent, Pattern, SongPart, StrumDirection, StrumHit, StrumPattern, TrackRole, VoiceTarget,
};
use super::{TICKS_PER_BAR, TICKS_PER_BEAT};

// ---------------------------------------------------------------------------
// GrooveTemplate
// ---------------------------------------------------------------------------

/// Groove feel adjustments applied to tick positions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrooveTemplate {
    /// No groove offset.
    #[default]
    Straight,
    /// 5–10 ticks late — relaxed, behind-the-beat.
    LaidBack,
    /// 3–7 ticks early — urgent, forward push.
    Pushing,
    /// Per-instrument: kick early, snare late, hats straight.
    HipHopPocket,
}

// ---------------------------------------------------------------------------
// MonoMode
// ---------------------------------------------------------------------------

/// Voice selection mode for mono (single-note) rhythm patterns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonoMode {
    /// Cycle through chord tones in ascending order.
    Arpeggio,
    /// Alternate between root and fifth.
    BassNote,
}

// ---------------------------------------------------------------------------
// HumanizeParams
// ---------------------------------------------------------------------------

/// Per-role humanization parameters (from DEFAULTS.md).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HumanizeParams {
    pub timing_std_dev: f64,
    pub timing_max_offset: i32,
    pub velocity_std_dev: f64,
    pub legato_factor: f64,
}

impl HumanizeParams {
    /// Look up humanization parameters by track role.
    pub fn for_role(role: TrackRole) -> Self {
        match role {
            TrackRole::Drum => Self {
                timing_std_dev: 5.0,
                timing_max_offset: 10,
                velocity_std_dev: 4.0,
                legato_factor: 0.3,
            },
            TrackRole::Bass => Self {
                timing_std_dev: 6.0,
                timing_max_offset: 12,
                velocity_std_dev: 5.0,
                legato_factor: 0.90,
            },
            TrackRole::Rhythm => Self {
                timing_std_dev: 8.0,
                timing_max_offset: 15,
                velocity_std_dev: 6.0,
                legato_factor: 0.85,
            },
            TrackRole::LeadMelody | TrackRole::CounterMelody => Self {
                timing_std_dev: 10.0,
                timing_max_offset: 18,
                velocity_std_dev: 8.0,
                legato_factor: 0.88,
            },
            TrackRole::PadSustain => Self {
                timing_std_dev: 4.0,
                timing_max_offset: 8,
                velocity_std_dev: 3.0,
                legato_factor: 0.98,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Dynamics scaling
// ---------------------------------------------------------------------------

/// Part-specific dynamics scaling multiplier.
pub fn dynamics_scale(part: SongPart) -> f64 {
    match part {
        SongPart::Intro => 0.55,
        SongPart::Verse => 0.70,
        SongPart::PreChorus => 0.82,
        SongPart::Chorus => 1.0,
        SongPart::Bridge => 0.65,
        SongPart::Outro => 0.50,
    }
}

// ---------------------------------------------------------------------------
// Strum pattern presets
// ---------------------------------------------------------------------------

impl StrumPattern {
    /// Travis fingerpick: alternating bass (B) and treble (H/M) voices.
    pub fn travis_pick() -> Self {
        Self {
            name: "Travis Pick".to_string(),
            hits: vec![
                StrumHit { tick_offset: 0, direction: StrumDirection::Down, velocity_factor: 0.9, stagger_ms: 0.0, voice_target: VoiceTarget::Bass },
                StrumHit { tick_offset: 240, direction: StrumDirection::Up, velocity_factor: 0.5, stagger_ms: 0.0, voice_target: VoiceTarget::High },
                StrumHit { tick_offset: 480, direction: StrumDirection::Down, velocity_factor: 0.7, stagger_ms: 0.0, voice_target: VoiceTarget::Mid },
                StrumHit { tick_offset: 720, direction: StrumDirection::Up, velocity_factor: 0.5, stagger_ms: 0.0, voice_target: VoiceTarget::High },
                StrumHit { tick_offset: 960, direction: StrumDirection::Down, velocity_factor: 0.85, stagger_ms: 0.0, voice_target: VoiceTarget::Bass },
                StrumHit { tick_offset: 1200, direction: StrumDirection::Up, velocity_factor: 0.5, stagger_ms: 0.0, voice_target: VoiceTarget::High },
                StrumHit { tick_offset: 1440, direction: StrumDirection::Down, velocity_factor: 0.7, stagger_ms: 0.0, voice_target: VoiceTarget::Mid },
                StrumHit { tick_offset: 1680, direction: StrumDirection::Up, velocity_factor: 0.5, stagger_ms: 0.0, voice_target: VoiceTarget::High },
            ],
            beats: 4,
        }
    }

    /// Steady down-stroke eighth notes with alternating dynamics.
    pub fn driving_eighths() -> Self {
        let vel = [1.0, 0.65, 0.85, 0.65, 1.0, 0.65, 0.85, 0.65];
        let hits = (0u32..8)
            .map(|i| StrumHit {
                tick_offset: i * 240,
                direction: StrumDirection::Down,
                velocity_factor: vel[i as usize],
                stagger_ms: 10.0,
                voice_target: VoiceTarget::All,
            })
            .collect();
        Self {
            name: "Driving 8ths".to_string(),
            hits,
            beats: 4,
        }
    }

    /// Country two-step: bass note then upper-voice chord.
    pub fn boom_chick() -> Self {
        Self {
            name: "Boom-Chick".to_string(),
            hits: vec![
                StrumHit { tick_offset: 0, direction: StrumDirection::Down, velocity_factor: 1.0, stagger_ms: 0.0, voice_target: VoiceTarget::Bass },
                StrumHit { tick_offset: 480, direction: StrumDirection::Down, velocity_factor: 0.7, stagger_ms: 8.0, voice_target: VoiceTarget::Upper },
                StrumHit { tick_offset: 960, direction: StrumDirection::Down, velocity_factor: 0.85, stagger_ms: 0.0, voice_target: VoiceTarget::Bass },
                StrumHit { tick_offset: 1440, direction: StrumDirection::Down, velocity_factor: 0.7, stagger_ms: 8.0, voice_target: VoiceTarget::Upper },
            ],
            beats: 4,
        }
    }

    /// Fast 16th-note strumming for indie/folktronica.
    pub fn sixteenth_strum() -> Self {
        let accent = [1.0f32, 0.65, 0.85, 0.65, 1.0, 0.65, 0.5, 0.65];
        let hits = (0u32..16)
            .map(|i| StrumHit {
                tick_offset: i * 120,
                direction: if i % 2 == 0 { StrumDirection::Down } else { StrumDirection::Up },
                velocity_factor: accent[(i / 2) as usize % accent.len()],
                stagger_ms: 4.0,
                voice_target: VoiceTarget::All,
            })
            .collect();
        Self {
            name: "16th Strum".to_string(),
            hits,
            beats: 4,
        }
    }

    /// Folk strum with ghost mutes: D . D U . g D g
    pub fn muted_strum() -> Self {
        Self {
            name: "Muted Strum".to_string(),
            hits: vec![
                StrumHit { tick_offset: 0, direction: StrumDirection::Down, velocity_factor: 1.0, stagger_ms: 12.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 480, direction: StrumDirection::Down, velocity_factor: 0.8, stagger_ms: 10.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 720, direction: StrumDirection::Up, velocity_factor: 0.6, stagger_ms: 6.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 1200, direction: StrumDirection::Ghost, velocity_factor: 0.2, stagger_ms: 0.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 1440, direction: StrumDirection::Down, velocity_factor: 0.85, stagger_ms: 10.0, voice_target: VoiceTarget::All },
                StrumHit { tick_offset: 1680, direction: StrumDirection::Ghost, velocity_factor: 0.2, stagger_ms: 0.0, voice_target: VoiceTarget::All },
            ],
            beats: 4,
        }
    }
}

// ---------------------------------------------------------------------------
// RhythmGenConfig
// ---------------------------------------------------------------------------

/// Configuration for full multi-bar rhythm generation.
#[derive(Clone, Debug)]
pub struct RhythmGenConfig<'a> {
    pub pattern: &'a StrumPattern,
    pub part: SongPart,
    pub role: TrackRole,
    pub channel: u8,
    pub tempo: f64,
    pub swing: f32,
    pub groove: GrooveTemplate,
}

// ---------------------------------------------------------------------------
// RhythmEngine
// ---------------------------------------------------------------------------

/// Rhythm engine for generating groove patterns, strum voicing, and humanization.
/// Fully deterministic given the same seed.
pub struct RhythmEngine {
    rng: ChaCha8Rng,
}

impl RhythmEngine {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Box-Muller Gaussian random value.
    fn gauss(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1: f64 = self.rng.random::<f64>().max(1e-10);
        let u2: f64 = self.rng.random::<f64>();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + z * std_dev
    }

    /// Apply swing to a tick position. Even-numbered 8th notes shift forward.
    pub fn apply_swing(tick: u32, swing: f32) -> u32 {
        if swing.abs() < f32::EPSILON {
            return tick;
        }
        let eighth = TICKS_PER_BEAT / 2; // 240
        let bar_tick = tick % TICKS_PER_BAR;
        let eighth_index = bar_tick / eighth;
        // Odd indices (1,3,5,7) are the "and" beats — the even-numbered 8th notes
        if eighth_index % 2 == 1 {
            let shift = (swing * 80.0) as u32;
            tick + shift
        } else {
            tick
        }
    }

    /// Apply groove template offset.
    fn apply_groove(&mut self, tick: u32, groove: GrooveTemplate) -> u32 {
        match groove {
            GrooveTemplate::Straight | GrooveTemplate::HipHopPocket => tick,
            GrooveTemplate::LaidBack => {
                let offset = self.rng.random_range(5u32..=10);
                tick + offset
            }
            GrooveTemplate::Pushing => {
                let offset = self.rng.random_range(3u32..=7);
                tick.saturating_sub(offset)
            }
        }
    }

    /// Accent curve value based on beat position within a bar.
    fn accent_curve(tick: u32) -> i32 {
        let beat = (tick % TICKS_PER_BAR) / TICKS_PER_BEAT;
        match beat {
            0 => 10,  // beat 1
            2 => 5,   // beat 3
            _ => -5,  // offbeats
        }
    }

    /// Convert stagger milliseconds to ticks at the given tempo.
    fn stagger_to_ticks(stagger_ms: f32, tempo: f64) -> u32 {
        let ticks = stagger_ms * TICKS_PER_BEAT as f32 * tempo as f32 / 60_000.0;
        ticks.round().max(0.0) as u32
    }

    /// Select chord voices based on target.
    fn select_voices(chord_notes: &[u8], target: VoiceTarget) -> Vec<u8> {
        if chord_notes.is_empty() {
            return Vec::new();
        }
        let mut sorted = chord_notes.to_vec();
        sorted.sort();

        match target {
            VoiceTarget::All => sorted,
            VoiceTarget::Bass => vec![sorted[0]],
            VoiceTarget::High => {
                if sorted.len() <= 1 {
                    sorted
                } else {
                    let mid = sorted.len() / 2;
                    sorted[mid..].to_vec()
                }
            }
            VoiceTarget::Mid => {
                if sorted.len() <= 2 {
                    sorted
                } else {
                    sorted[1..sorted.len() - 1].to_vec()
                }
            }
            VoiceTarget::Upper => {
                if sorted.len() <= 1 {
                    sorted
                } else {
                    sorted[1..].to_vec()
                }
            }
        }
    }

    /// Compute nominal duration for each hit in a pattern (gap to next hit).
    fn compute_durations(hits: &[StrumHit], pattern_ticks: u32) -> Vec<u32> {
        hits.iter()
            .enumerate()
            .map(|(i, _)| {
                let next = if i + 1 < hits.len() {
                    hits[i + 1].tick_offset
                } else {
                    pattern_ticks
                };
                next.saturating_sub(hits[i].tick_offset).max(1)
            })
            .collect()
    }

    /// Expand a single strum hit into note events for the given chord.
    #[allow(clippy::too_many_arguments)]
    fn strum_hit_to_events(
        &mut self,
        chord_notes: &[u8],
        hit: &StrumHit,
        bar_offset: u32,
        base_velocity: u8,
        tempo: f64,
        channel: u8,
        nominal_duration: u32,
    ) -> Vec<NoteEvent> {
        let notes = Self::select_voices(chord_notes, hit.voice_target);
        if notes.is_empty() {
            return Vec::new();
        }

        let vel = ((base_velocity as f32) * hit.velocity_factor)
            .round()
            .clamp(1.0, 127.0) as u8;
        let stagger_ticks = Self::stagger_to_ticks(hit.stagger_ms, tempo);

        // Order notes by strum direction
        let ordered: Vec<u8> = match hit.direction {
            StrumDirection::Down => {
                let mut n = notes;
                n.sort();
                n
            }
            StrumDirection::Up => {
                let mut n = notes;
                n.sort();
                n.reverse();
                n
            }
            StrumDirection::Ghost | StrumDirection::Mute => notes,
        };

        let mut events = Vec::new();
        for (i, &note) in ordered.iter().enumerate() {
            let tick = bar_offset + hit.tick_offset + (i as u32) * stagger_ticks;

            let note_vel = match hit.direction {
                StrumDirection::Down => {
                    // First note full, subsequent decrease 2-4 per note
                    let decrease = (i as i32) * 3;
                    (vel as i32 - decrease).clamp(1, 127) as u8
                }
                StrumDirection::Up => {
                    // Last note gets emphasis
                    if i == ordered.len() - 1 {
                        vel
                    } else {
                        let decrease = ((ordered.len() - 1 - i) as i32) * 3;
                        (vel as i32 - decrease).clamp(1, 127) as u8
                    }
                }
                StrumDirection::Ghost => self.rng.random_range(15u8..=30),
                StrumDirection::Mute => self.rng.random_range(40u8..=60),
            };

            let dur = match hit.direction {
                StrumDirection::Ghost => self.rng.random_range(30u32..=60),
                StrumDirection::Mute => nominal_duration.min(60),
                _ => nominal_duration,
            };

            events.push(NoteEvent {
                tick,
                note,
                velocity: note_vel,
                duration: dur,
                channel,
            });
        }

        events
    }

    /// Generate rhythm events for one bar of a poly (chord-strumming) track.
    pub fn generate_poly_bar(
        &mut self,
        chord_notes: &[u8],
        pattern: &StrumPattern,
        bar_offset: u32,
        tempo: f64,
        channel: u8,
    ) -> Vec<NoteEvent> {
        let base_velocity: u8 = 100;
        let pattern_ticks = pattern.beats * TICKS_PER_BEAT;
        let durations = Self::compute_durations(&pattern.hits, pattern_ticks);

        let mut events = Vec::new();
        for (i, hit) in pattern.hits.iter().enumerate() {
            let mut hit_events = self.strum_hit_to_events(
                chord_notes,
                hit,
                bar_offset,
                base_velocity,
                tempo,
                channel,
                durations[i],
            );
            events.append(&mut hit_events);
        }
        events
    }

    /// Generate rhythm events for one bar of a mono track.
    pub fn generate_mono_bar(
        &mut self,
        chord_notes: &[u8],
        pattern: &StrumPattern,
        bar_offset: u32,
        channel: u8,
        mode: MonoMode,
    ) -> Vec<NoteEvent> {
        if chord_notes.is_empty() {
            return Vec::new();
        }

        let mut sorted = chord_notes.to_vec();
        sorted.sort();
        let base_velocity: u8 = 100;
        let pattern_ticks = pattern.beats * TICKS_PER_BEAT;
        let durations = Self::compute_durations(&pattern.hits, pattern_ticks);

        let mut events = Vec::new();
        let mut arp_index: usize = 0;

        for (hit_idx, hit) in pattern.hits.iter().enumerate() {
            let tick = bar_offset + hit.tick_offset;
            let velocity = ((base_velocity as f32) * hit.velocity_factor)
                .round()
                .clamp(1.0, 127.0) as u8;

            let note = match mode {
                MonoMode::Arpeggio => {
                    let n = sorted[arp_index % sorted.len()];
                    arp_index += 1;
                    n
                }
                MonoMode::BassNote => {
                    if hit_idx % 2 == 0 {
                        sorted[0] // root
                    } else if sorted.len() > 2 {
                        sorted[2] // fifth (3rd element of triad)
                    } else {
                        sorted[0]
                    }
                }
            };

            let duration = match hit.direction {
                StrumDirection::Ghost => self.rng.random_range(30u32..=60),
                StrumDirection::Mute => durations[hit_idx].min(60),
                _ => durations[hit_idx],
            };

            events.push(NoteEvent {
                tick,
                note,
                velocity,
                duration,
                channel,
            });
        }

        events
    }

    /// Apply humanization (timing, velocity, duration) to note events in place.
    pub fn humanize(&mut self, events: &mut [NoteEvent], role: TrackRole, part: SongPart) {
        let params = HumanizeParams::for_role(role);
        let dyn_scale = dynamics_scale(part);

        for event in events.iter_mut() {
            // Timing: Gaussian offset, downbeats get tighter (÷2)
            let is_downbeat = event.tick % TICKS_PER_BAR == 0;
            let std_dev = if is_downbeat {
                params.timing_std_dev / 2.0
            } else {
                params.timing_std_dev
            };
            let max_off = if is_downbeat {
                params.timing_max_offset / 2
            } else {
                params.timing_max_offset
            };

            let timing_offset = self.gauss(0.0, std_dev).round() as i32;
            let timing_offset = timing_offset.clamp(-max_off, max_off);
            // Use i64 to avoid overflow when tick > i32::MAX (~2.1B).
            let new_tick = event.tick as i64 + timing_offset as i64;
            event.tick = new_tick.clamp(0, u32::MAX as i64) as u32;

            // Velocity: variation + accent curve, then dynamics scaling
            let vel_variation = self.gauss(0.0, params.velocity_std_dev).round() as i32;
            let accent = Self::accent_curve(event.tick);
            let humanized_vel = event.velocity as i32 + vel_variation + accent;
            let scaled_vel = (humanized_vel as f64 * dyn_scale).round() as i32;
            event.velocity = scaled_vel.clamp(1, 127) as u8;

            // Duration: legato factor + random variation
            let legato_dur = (event.duration as f64 * params.legato_factor).round() as i32;
            let dur_variation = self.rng.random_range(-10i32..=10);
            event.duration = (legato_dur + dur_variation).max(1) as u32;
        }
    }

    /// Apply swing to all events.
    pub fn apply_swing_to_events(events: &mut [NoteEvent], swing: f32) {
        for event in events.iter_mut() {
            event.tick = Self::apply_swing(event.tick, swing);
        }
    }

    /// Apply groove template to all events.
    pub fn apply_groove_to_events(&mut self, events: &mut [NoteEvent], groove: GrooveTemplate) {
        for event in events.iter_mut() {
            event.tick = self.apply_groove(event.tick, groove);
        }
    }

    /// Generate a full rhythm pattern for a poly track across multiple bars.
    pub fn generate_rhythm_pattern(
        &mut self,
        chords_per_bar: &[&[u8]],
        config: &RhythmGenConfig<'_>,
    ) -> Pattern {
        let bars = chords_per_bar.len() as u32;
        let mut all_events = Vec::new();

        for (bar_idx, chord_notes) in chords_per_bar.iter().enumerate() {
            let bar_offset = bar_idx as u32 * TICKS_PER_BAR;
            let mut bar_events = self.generate_poly_bar(
                chord_notes,
                config.pattern,
                bar_offset,
                config.tempo,
                config.channel,
            );
            all_events.append(&mut bar_events);
        }

        Self::apply_swing_to_events(&mut all_events, config.swing);
        self.apply_groove_to_events(&mut all_events, config.groove);
        self.humanize(&mut all_events, config.role, config.part);
        all_events.sort_by_key(|e| e.tick);

        Pattern {
            events: all_events,
            cc_events: Vec::new(),
            length_ticks: bars * TICKS_PER_BAR,
            bars,
        }
    }

    /// Generate a full rhythm pattern for a mono track across multiple bars.
    pub fn generate_mono_rhythm_pattern(
        &mut self,
        chords_per_bar: &[&[u8]],
        config: &RhythmGenConfig<'_>,
        mode: MonoMode,
    ) -> Pattern {
        let bars = chords_per_bar.len() as u32;
        let mut all_events = Vec::new();

        for (bar_idx, chord_notes) in chords_per_bar.iter().enumerate() {
            let bar_offset = bar_idx as u32 * TICKS_PER_BAR;
            let mut bar_events = self.generate_mono_bar(
                chord_notes,
                config.pattern,
                bar_offset,
                config.channel,
                mode,
            );
            all_events.append(&mut bar_events);
        }

        Self::apply_swing_to_events(&mut all_events, config.swing);
        self.apply_groove_to_events(&mut all_events, config.groove);
        self.humanize(&mut all_events, config.role, config.part);
        all_events.sort_by_key(|e| e.tick);

        Pattern {
            events: all_events,
            cc_events: Vec::new(),
            length_ticks: bars * TICKS_PER_BAR,
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

    // C major triad voiced in guitar range: C3, E3, G3, C4, E4
    const C_CHORD: &[u8] = &[48, 52, 55, 60, 64];

    // -- Determinism -------------------------------------------------------

    #[test]
    fn same_seed_produces_identical_poly_pattern() {
        let pattern = StrumPattern::default_folk();
        let chords: Vec<&[u8]> = vec![C_CHORD; 4];
        let config = RhythmGenConfig {
            pattern: &pattern,
            part: SongPart::Verse,
            role: TrackRole::Rhythm,
            channel: 4,
            tempo: 120.0,
            swing: 0.0,
            groove: GrooveTemplate::Straight,
        };

        let mut engine1 = RhythmEngine::new(42);
        let result1 = engine1.generate_rhythm_pattern(&chords, &config);

        let mut engine2 = RhythmEngine::new(42);
        let result2 = engine2.generate_rhythm_pattern(&chords, &config);

        assert_eq!(result1, result2, "same seed must produce identical output");
    }

    #[test]
    fn different_seeds_produce_different_poly_patterns() {
        let pattern = StrumPattern::default_folk();
        let chords: Vec<&[u8]> = vec![C_CHORD; 4];
        let config = RhythmGenConfig {
            pattern: &pattern,
            part: SongPart::Verse,
            role: TrackRole::Rhythm,
            channel: 4,
            tempo: 120.0,
            swing: 0.0,
            groove: GrooveTemplate::Straight,
        };

        let mut engine1 = RhythmEngine::new(42);
        let result1 = engine1.generate_rhythm_pattern(&chords, &config);

        let mut engine2 = RhythmEngine::new(99);
        let result2 = engine2.generate_rhythm_pattern(&chords, &config);

        assert_ne!(result1, result2, "different seeds should differ");
    }

    #[test]
    fn same_seed_mono_determinism() {
        let pattern = StrumPattern::default_folk();
        let chords: Vec<&[u8]> = vec![C_CHORD; 2];
        let config = RhythmGenConfig {
            pattern: &pattern,
            part: SongPart::Chorus,
            role: TrackRole::Bass,
            channel: 6,
            tempo: 120.0,
            swing: 0.0,
            groove: GrooveTemplate::Straight,
        };

        let mut e1 = RhythmEngine::new(77);
        let r1 = e1.generate_mono_rhythm_pattern(&chords, &config, MonoMode::Arpeggio);

        let mut e2 = RhythmEngine::new(77);
        let r2 = e2.generate_mono_rhythm_pattern(&chords, &config, MonoMode::Arpeggio);

        assert_eq!(r1, r2, "mono pattern must be deterministic");
    }

    // -- Strum pattern presets ---------------------------------------------

    #[test]
    fn folk_strum_preset() {
        let p = StrumPattern::default_folk();
        assert_eq!(p.hits.len(), 6);
        assert_eq!(p.beats, 4);
        assert_eq!(p.hits[0].direction, StrumDirection::Down);
        assert!((p.hits[0].velocity_factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn travis_pick_preset() {
        let p = StrumPattern::travis_pick();
        assert_eq!(p.hits.len(), 8);
        assert_eq!(p.beats, 4);
        assert_eq!(p.hits[0].voice_target, VoiceTarget::Bass);
        assert_eq!(p.hits[1].voice_target, VoiceTarget::High);
        assert_eq!(p.hits[2].voice_target, VoiceTarget::Mid);
    }

    #[test]
    fn driving_eighths_preset() {
        let p = StrumPattern::driving_eighths();
        assert_eq!(p.hits.len(), 8);
        assert_eq!(p.beats, 4);
        for hit in &p.hits {
            assert_eq!(hit.direction, StrumDirection::Down);
        }
        // tick positions are every 240
        for (i, hit) in p.hits.iter().enumerate() {
            assert_eq!(hit.tick_offset, i as u32 * 240);
        }
    }

    #[test]
    fn boom_chick_preset() {
        let p = StrumPattern::boom_chick();
        assert_eq!(p.hits.len(), 4);
        assert_eq!(p.hits[0].voice_target, VoiceTarget::Bass);
        assert_eq!(p.hits[1].voice_target, VoiceTarget::Upper);
        assert_eq!(p.hits[2].voice_target, VoiceTarget::Bass);
        assert_eq!(p.hits[3].voice_target, VoiceTarget::Upper);
    }

    #[test]
    fn sixteenth_strum_preset() {
        let p = StrumPattern::sixteenth_strum();
        assert_eq!(p.hits.len(), 16);
        assert_eq!(p.beats, 4);
        // Alternating D/U
        assert_eq!(p.hits[0].direction, StrumDirection::Down);
        assert_eq!(p.hits[1].direction, StrumDirection::Up);
        assert_eq!(p.hits[2].direction, StrumDirection::Down);
    }

    #[test]
    fn muted_strum_preset() {
        let p = StrumPattern::muted_strum();
        assert_eq!(p.hits.len(), 6);
        assert_eq!(p.hits[3].direction, StrumDirection::Ghost);
        assert_eq!(p.hits[5].direction, StrumDirection::Ghost);
        assert!((p.hits[3].velocity_factor - 0.2).abs() < f32::EPSILON);
    }

    // -- Swing -------------------------------------------------------------

    #[test]
    fn swing_zero_is_straight() {
        assert_eq!(RhythmEngine::apply_swing(0, 0.0), 0);
        assert_eq!(RhythmEngine::apply_swing(240, 0.0), 240);
        assert_eq!(RhythmEngine::apply_swing(480, 0.0), 480);
    }

    #[test]
    fn swing_shifts_even_eighths() {
        // tick 240 is on 8th-note index 1 (an "and" beat) — should shift
        let shifted = RhythmEngine::apply_swing(240, 0.5);
        assert_eq!(shifted, 240 + 40); // 0.5 * 80 = 40

        // tick 0 is on 8th-note index 0 — should not shift
        assert_eq!(RhythmEngine::apply_swing(0, 0.5), 0);

        // tick 480 is 8th-note index 2 — should not shift
        assert_eq!(RhythmEngine::apply_swing(480, 0.5), 480);

        // tick 720 is 8th-note index 3 — should shift
        assert_eq!(RhythmEngine::apply_swing(720, 1.0), 720 + 80);
    }

    // -- Humanization bounds -----------------------------------------------

    #[test]
    fn humanization_timing_within_bounds() {
        let mut engine = RhythmEngine::new(42);
        let mut events: Vec<NoteEvent> = (0..100)
            .map(|i| NoteEvent {
                tick: 480 + i * 240, // avoid tick 0 (downbeat has tighter bounds)
                note: 60,
                velocity: 100,
                duration: 240,
                channel: 0,
            })
            .collect();

        let original_ticks: Vec<u32> = events.iter().map(|e| e.tick).collect();
        engine.humanize(&mut events, TrackRole::Rhythm, SongPart::Chorus);

        let params = HumanizeParams::for_role(TrackRole::Rhythm);
        for (orig, event) in original_ticks.iter().zip(events.iter()) {
            let diff = (event.tick as i32) - (*orig as i32);
            assert!(
                diff.abs() <= params.timing_max_offset,
                "timing offset {} exceeds max {}",
                diff,
                params.timing_max_offset,
            );
        }
    }

    #[test]
    fn humanization_velocity_scaled_by_dynamics() {
        let mut engine = RhythmEngine::new(42);
        let mut events = vec![NoteEvent {
            tick: 480,
            note: 60,
            velocity: 100,
            duration: 240,
            channel: 0,
        }];

        engine.humanize(&mut events, TrackRole::Rhythm, SongPart::Intro);
        // Intro dynamics = 0.55, so velocity should be significantly reduced
        assert!(
            events[0].velocity < 80,
            "intro dynamics should reduce velocity, got {}",
            events[0].velocity
        );
    }

    // -- Voice selection ---------------------------------------------------

    #[test]
    fn voice_select_all() {
        let notes = vec![48, 52, 55, 60, 64];
        let selected = RhythmEngine::select_voices(&notes, VoiceTarget::All);
        assert_eq!(selected, vec![48, 52, 55, 60, 64]);
    }

    #[test]
    fn voice_select_bass() {
        let notes = vec![64, 48, 55]; // unsorted
        let selected = RhythmEngine::select_voices(&notes, VoiceTarget::Bass);
        assert_eq!(selected, vec![48]);
    }

    #[test]
    fn voice_select_upper() {
        let notes = vec![48, 52, 55, 60];
        let selected = RhythmEngine::select_voices(&notes, VoiceTarget::Upper);
        assert_eq!(selected, vec![52, 55, 60]);
    }

    #[test]
    fn voice_select_empty() {
        let selected = RhythmEngine::select_voices(&[], VoiceTarget::All);
        assert!(selected.is_empty());
    }

    // -- Dynamics scaling --------------------------------------------------

    #[test]
    fn dynamics_scale_values() {
        assert!((dynamics_scale(SongPart::Intro) - 0.55).abs() < f64::EPSILON);
        assert!((dynamics_scale(SongPart::Verse) - 0.70).abs() < f64::EPSILON);
        assert!((dynamics_scale(SongPart::PreChorus) - 0.82).abs() < f64::EPSILON);
        assert!((dynamics_scale(SongPart::Chorus) - 1.0).abs() < f64::EPSILON);
        assert!((dynamics_scale(SongPart::Bridge) - 0.65).abs() < f64::EPSILON);
        assert!((dynamics_scale(SongPart::Outro) - 0.50).abs() < f64::EPSILON);
    }

    // -- HumanizeParams ----------------------------------------------------

    #[test]
    fn humanize_params_per_role() {
        let drums = HumanizeParams::for_role(TrackRole::Drum);
        assert!((drums.timing_std_dev - 5.0).abs() < f64::EPSILON);
        assert_eq!(drums.timing_max_offset, 10);

        let rhythm = HumanizeParams::for_role(TrackRole::Rhythm);
        assert!((rhythm.timing_std_dev - 8.0).abs() < f64::EPSILON);
        assert_eq!(rhythm.timing_max_offset, 15);

        let pads = HumanizeParams::for_role(TrackRole::PadSustain);
        assert!((pads.legato_factor - 0.98).abs() < f64::EPSILON);
    }

    // -- Overflow safety ---------------------------------------------------

    #[test]
    fn humanize_large_tick_no_overflow() {
        let mut engine = RhythmEngine::new(42);
        // Tick near u32::MAX would overflow if cast to i32
        let large_tick = u32::MAX - 100;
        let mut events = vec![NoteEvent {
            tick: large_tick,
            note: 60,
            velocity: 100,
            duration: 240,
            channel: 0,
        }];

        // Should not panic or wrap to a garbage value
        engine.humanize(&mut events, TrackRole::Rhythm, SongPart::Chorus);

        // Tick should remain close to the original (within timing_max_offset=15)
        let diff = if events[0].tick > large_tick {
            events[0].tick - large_tick
        } else {
            large_tick - events[0].tick
        };
        assert!(
            diff <= 15,
            "tick drifted {} from original, expected <= 15",
            diff,
        );
    }

    #[test]
    fn humanize_tick_zero_no_underflow() {
        let mut engine = RhythmEngine::new(42);
        let mut events = vec![NoteEvent {
            tick: 0,
            note: 60,
            velocity: 100,
            duration: 240,
            channel: 0,
        }];

        // A negative timing offset should clamp to 0, not wrap to u32::MAX
        engine.humanize(&mut events, TrackRole::Rhythm, SongPart::Chorus);
        assert!(
            events[0].tick <= 15,
            "tick {} too large for near-zero input",
            events[0].tick,
        );
    }

    // -- Groove template ---------------------------------------------------

    #[test]
    fn groove_template_serde_roundtrip() {
        let groove = GrooveTemplate::LaidBack;
        let json = serde_json::to_string(&groove).expect("serialize");
        assert_eq!(json, r#""laid_back""#);
        let parsed: GrooveTemplate = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, groove);
    }

    #[test]
    fn mono_mode_serde_roundtrip() {
        let mode = MonoMode::BassNote;
        let json = serde_json::to_string(&mode).expect("serialize");
        let parsed: MonoMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, mode);
    }

    // -- Poly bar generation -----------------------------------------------

    #[test]
    fn poly_bar_produces_events_for_each_hit() {
        let mut engine = RhythmEngine::new(42);
        let pattern = StrumPattern::default_folk();
        let events = engine.generate_poly_bar(C_CHORD, &pattern, 0, 120.0, 4);
        // Folk strum: 6 hits × 5 notes = 30 events
        assert_eq!(events.len(), 6 * C_CHORD.len());
    }

    #[test]
    fn mono_bar_produces_one_event_per_hit() {
        let mut engine = RhythmEngine::new(42);
        let pattern = StrumPattern::default_folk();
        let events = engine.generate_mono_bar(C_CHORD, &pattern, 0, 6, MonoMode::Arpeggio);
        assert_eq!(events.len(), 6); // one per hit
    }

    // -- Stagger / voicing spread -----------------------------------------

    #[test]
    fn down_strum_staggers_low_to_high() {
        let mut engine = RhythmEngine::new(42);
        let hit = StrumHit {
            tick_offset: 0,
            direction: StrumDirection::Down,
            velocity_factor: 1.0,
            stagger_ms: 12.0,
            voice_target: VoiceTarget::All,
        };
        let events = engine.strum_hit_to_events(C_CHORD, &hit, 0, 100, 120.0, 0, 240);
        // Notes should appear in ascending order with increasing tick
        for i in 1..events.len() {
            assert!(events[i].note > events[i - 1].note);
            assert!(events[i].tick >= events[i - 1].tick);
        }
    }

    #[test]
    fn up_strum_staggers_high_to_low() {
        let mut engine = RhythmEngine::new(42);
        let hit = StrumHit {
            tick_offset: 0,
            direction: StrumDirection::Up,
            velocity_factor: 1.0,
            stagger_ms: 12.0,
            voice_target: VoiceTarget::All,
        };
        let events = engine.strum_hit_to_events(C_CHORD, &hit, 0, 100, 120.0, 0, 240);
        // Notes should appear in descending order
        for i in 1..events.len() {
            assert!(events[i].note < events[i - 1].note);
        }
    }

    #[test]
    fn ghost_strum_short_duration() {
        let mut engine = RhythmEngine::new(42);
        let hit = StrumHit {
            tick_offset: 0,
            direction: StrumDirection::Ghost,
            velocity_factor: 0.2,
            stagger_ms: 0.0,
            voice_target: VoiceTarget::All,
        };
        let events = engine.strum_hit_to_events(C_CHORD, &hit, 0, 100, 120.0, 0, 480);
        for e in &events {
            assert!(e.duration <= 60, "ghost duration {} exceeds 60", e.duration);
            assert!(e.velocity <= 30, "ghost velocity {} exceeds 30", e.velocity);
        }
    }
}
