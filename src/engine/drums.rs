// Per-instrument drum pattern generation by song part.
// Generates kick, snare, hi-hat, tambourine, shaker, cowbell, ride, and crash
// patterns following genre-appropriate folk/indie grooves per docs/engine/DRUMS.md.
// Fully implemented — awaiting GUI integration (v0.4.0).
#![allow(dead_code)]

use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::song::{InstrumentType, NoteEvent, Pattern, SongPart};
use super::{TICKS_PER_BAR, TICKS_PER_BEAT};

// ---------------------------------------------------------------------------
// DrumIntensity
// ---------------------------------------------------------------------------

/// Part-specific intensity scaling for drums (0.0–1.0).
pub fn drum_intensity(part: SongPart) -> f64 {
    match part {
        SongPart::Intro => 0.4,
        SongPart::Verse => 0.65,
        SongPart::PreChorus => 0.8,
        SongPart::Chorus => 1.0,
        SongPart::Bridge => 0.55,
        SongPart::Outro => 0.45,
    }
}

// ---------------------------------------------------------------------------
// HiHatMode
// ---------------------------------------------------------------------------

/// Hi-hat pattern mode for chorus sections.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HiHatMode {
    /// 16th notes with velocity accents.
    Sixteenths,
    /// 8th notes with open hat on backbeats (2 & 4).
    OpenOnBackbeats,
}

// ---------------------------------------------------------------------------
// DrumHumanize
// ---------------------------------------------------------------------------

/// Velocity threshold below which a snare/rimshot hit is considered a ghost note.
/// Ghost notes in the spec are vel 30-45; regular backbeats are 70+.
const GHOST_NOTE_VELOCITY_THRESHOLD: u8 = 50;

/// Per-instrument drum humanization parameters from spec.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrumHumanize {
    pub timing_ticks: i32,
    pub velocity_range: i32,
}

impl DrumHumanize {
    pub fn for_instrument(instrument: InstrumentType, is_ghost: bool) -> Self {
        match instrument {
            InstrumentType::Kick => Self {
                timing_ticks: 5,
                velocity_range: 4,
            },
            InstrumentType::Snare | InstrumentType::Rimshot => {
                if is_ghost {
                    Self {
                        timing_ticks: 10,
                        velocity_range: 4,
                    }
                } else {
                    Self {
                        timing_ticks: 4,
                        velocity_range: 4,
                    }
                }
            }
            InstrumentType::HiHat | InstrumentType::OpenHiHat => Self {
                timing_ticks: 6,
                velocity_range: 6,
            },
            InstrumentType::Tambourine | InstrumentType::Shaker => Self {
                timing_ticks: 8,
                velocity_range: 8,
            },
            _ => Self {
                timing_ticks: 6,
                velocity_range: 6,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// DrumConfig
// ---------------------------------------------------------------------------

/// Configuration for drum pattern generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrumConfig {
    pub part: SongPart,
    pub bars: u32,
    pub channel: u8,
}

// ---------------------------------------------------------------------------
// DrumEngine
// ---------------------------------------------------------------------------

/// Drum pattern engine for generating genre-appropriate folk/indie grooves.
/// Fully deterministic given the same seed.
pub struct DrumEngine {
    rng: ChaCha8Rng,
}

impl DrumEngine {
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

    /// Apply drum-specific humanization to events in place.
    ///
    /// For snare/rimshot instruments, ghost notes are detected per-event by
    /// velocity threshold (`GHOST_NOTE_VELOCITY_THRESHOLD`). Ghost notes get
    /// looser timing (±10 ticks) while regular hits stay tight (±4 ticks).
    fn humanize_events(
        &mut self,
        events: &mut [NoteEvent],
        instrument: InstrumentType,
        part: SongPart,
    ) {
        let intensity = drum_intensity(part);

        for event in events.iter_mut() {
            // Detect ghost notes per-event for snare/rimshot
            let is_ghost = matches!(
                instrument,
                InstrumentType::Snare | InstrumentType::Rimshot
            ) && event.velocity < GHOST_NOTE_VELOCITY_THRESHOLD;

            let params = DrumHumanize::for_instrument(instrument, is_ghost);

            // Timing offset
            let timing_offset = self
                .gauss(0.0, params.timing_ticks as f64)
                .round() as i32;
            let timing_offset = timing_offset.clamp(-params.timing_ticks, params.timing_ticks);
            let new_tick = event.tick as i64 + timing_offset as i64;
            event.tick = new_tick.clamp(0, u32::MAX as i64) as u32;

            // Velocity with intensity scaling
            let vel_var = self
                .gauss(0.0, params.velocity_range as f64)
                .round() as i32;
            let vel_var = vel_var.clamp(-params.velocity_range, params.velocity_range);
            let scaled_vel = ((event.velocity as f64) * intensity).round() as i32 + vel_var;
            event.velocity = scaled_vel.clamp(1, 127) as u8;
        }
    }

    // -- Kick ---------------------------------------------------------------

    fn generate_kick_bar(
        &mut self,
        bar_offset: u32,
        part: SongPart,
        channel: u8,
    ) -> Vec<NoteEvent> {
        let note = InstrumentType::Kick.gm_drum_note().unwrap_or(36);
        let mut events = Vec::new();

        match part {
            SongPart::Chorus | SongPart::PreChorus => {
                // Beat 1
                events.push(NoteEvent {
                    tick: bar_offset,
                    note,
                    velocity: 110,
                    duration: 120,
                    channel,
                });
                // Beat 3
                events.push(NoteEvent {
                    tick: bar_offset + 2 * TICKS_PER_BEAT,
                    note,
                    velocity: 95,
                    duration: 120,
                    channel,
                });
                // Ghost kick on "and of 4" (40% probability)
                if self.rng.random::<f64>() < 0.40 {
                    events.push(NoteEvent {
                        tick: bar_offset + 3 * TICKS_PER_BEAT + TICKS_PER_BEAT / 2,
                        note,
                        velocity: 65,
                        duration: 60,
                        channel,
                    });
                }
            }
            SongPart::Bridge => {
                // Only beat 1 — creates space
                events.push(NoteEvent {
                    tick: bar_offset,
                    note,
                    velocity: 95,
                    duration: 120,
                    channel,
                });
            }
            _ => {
                // Verse/Intro/Outro: standard two-kick pattern
                events.push(NoteEvent {
                    tick: bar_offset,
                    note,
                    velocity: 100,
                    duration: 120,
                    channel,
                });
                events.push(NoteEvent {
                    tick: bar_offset + 2 * TICKS_PER_BEAT,
                    note,
                    velocity: 90,
                    duration: 120,
                    channel,
                });
            }
        }

        // Variation pool (15-25% chance per bar)
        if self.rng.random::<f64>() < 0.20 {
            let var_type = self.rng.random_range(0u32..3);
            match var_type {
                0 => {
                    // Ghost kick on "and of 2" (vel 50-65)
                    events.push(NoteEvent {
                        tick: bar_offset + TICKS_PER_BEAT + TICKS_PER_BEAT / 2,
                        note,
                        velocity: self.rng.random_range(50u8..=65),
                        duration: 60,
                        channel,
                    });
                }
                1 => {
                    // Displace beat 3 kick to "and of 3" (syncopation)
                    let beat3_tick = bar_offset + 2 * TICKS_PER_BEAT;
                    events.retain(|e| e.tick != beat3_tick);
                    events.push(NoteEvent {
                        tick: bar_offset + 2 * TICKS_PER_BEAT + TICKS_PER_BEAT / 2,
                        note,
                        velocity: 85,
                        duration: 120,
                        channel,
                    });
                }
                _ => {
                    // Double kick: two quick hits before beat 1 of next bar (vel 60, 80)
                    events.push(NoteEvent {
                        tick: bar_offset + TICKS_PER_BAR - 240,
                        note,
                        velocity: 60,
                        duration: 60,
                        channel,
                    });
                    events.push(NoteEvent {
                        tick: bar_offset + TICKS_PER_BAR - 120,
                        note,
                        velocity: 80,
                        duration: 60,
                        channel,
                    });
                }
            }
        }

        events
    }

    // -- Snare --------------------------------------------------------------

    fn generate_snare_bar(
        &mut self,
        bar_offset: u32,
        part: SongPart,
        channel: u8,
    ) -> Vec<NoteEvent> {
        let note = InstrumentType::Snare.gm_drum_note().unwrap_or(38);
        let mut events = Vec::new();

        match part {
            SongPart::Chorus | SongPart::PreChorus => {
                // Ghost notes on 8th note positions (vel 30-45)
                let ghost_positions = [
                    TICKS_PER_BEAT / 2,                          // "and of 1"
                    TICKS_PER_BEAT + TICKS_PER_BEAT / 2,         // "and of 2"
                    2 * TICKS_PER_BEAT + TICKS_PER_BEAT / 2,     // "and of 3"
                ];
                for &pos in &ghost_positions {
                    events.push(NoteEvent {
                        tick: bar_offset + pos,
                        note,
                        velocity: self.rng.random_range(30u8..=45),
                        duration: 30,
                        channel,
                    });
                }
                // Backbeat on 2 and 4
                events.push(NoteEvent {
                    tick: bar_offset + TICKS_PER_BEAT,
                    note,
                    velocity: 100,
                    duration: 120,
                    channel,
                });
                events.push(NoteEvent {
                    tick: bar_offset + 3 * TICKS_PER_BEAT,
                    note,
                    velocity: 105,
                    duration: 120,
                    channel,
                });
            }
            SongPart::Bridge => {
                // Cross-stick / rimshot at lighter velocity
                let rimshot = InstrumentType::Rimshot.gm_drum_note().unwrap_or(37);
                events.push(NoteEvent {
                    tick: bar_offset + TICKS_PER_BEAT,
                    note: rimshot,
                    velocity: 70,
                    duration: 120,
                    channel,
                });
                events.push(NoteEvent {
                    tick: bar_offset + 3 * TICKS_PER_BEAT,
                    note: rimshot,
                    velocity: 70,
                    duration: 120,
                    channel,
                });
            }
            _ => {
                // Standard backbeat on 2 and 4
                events.push(NoteEvent {
                    tick: bar_offset + TICKS_PER_BEAT,
                    note,
                    velocity: 95,
                    duration: 120,
                    channel,
                });
                events.push(NoteEvent {
                    tick: bar_offset + 3 * TICKS_PER_BEAT,
                    note,
                    velocity: 100,
                    duration: 120,
                    channel,
                });
            }
        }

        // Snare variation pool (15-25% chance)
        if self.rng.random::<f64>() < 0.20 {
            let var_type = self.rng.random_range(0u32..3);
            match var_type {
                0 => {
                    // Ghost note before backbeat (flam-like, 30 ticks before, vel 35)
                    let beat = if self.rng.random::<bool>() { 1 } else { 3 };
                    let target = bar_offset + beat * TICKS_PER_BEAT;
                    if target >= 30 {
                        events.push(NoteEvent {
                            tick: target - 30,
                            note,
                            velocity: 35,
                            duration: 30,
                            channel,
                        });
                    }
                }
                1 => {
                    // Snare on beat 4 slightly late (+10-15 ticks, creates drag)
                    let beat4_tick = bar_offset + 3 * TICKS_PER_BEAT;
                    if let Some(beat4) = events
                        .iter_mut()
                        .find(|e| e.tick == beat4_tick && e.note == note)
                    {
                        beat4.tick += self.rng.random_range(10u32..=15);
                    }
                }
                _ => {
                    // Ghost note run: 3 quick 16ths leading into beat 2 or 4
                    let target_beat = if self.rng.random::<bool>() { 1 } else { 3 };
                    let target_tick = bar_offset + target_beat * TICKS_PER_BEAT;
                    for i in 1..=3u32 {
                        let offset = i * 120; // 16th note spacing
                        if target_tick >= offset {
                            events.push(NoteEvent {
                                tick: target_tick - offset,
                                note,
                                velocity: self.rng.random_range(30u8..=45),
                                duration: 30,
                                channel,
                            });
                        }
                    }
                }
            }
        }

        events
    }

    // -- Hi-Hat -------------------------------------------------------------

    fn generate_hihat_bar(
        &mut self,
        bar_offset: u32,
        part: SongPart,
        channel: u8,
    ) -> Vec<NoteEvent> {
        let closed = InstrumentType::HiHat.gm_drum_note().unwrap_or(42);
        let open = InstrumentType::OpenHiHat.gm_drum_note().unwrap_or(46);
        let eighth = TICKS_PER_BEAT / 2; // 240
        let mut events = Vec::new();

        match part {
            SongPart::Chorus | SongPart::PreChorus => {
                let mode = if self.rng.random::<bool>() {
                    HiHatMode::Sixteenths
                } else {
                    HiHatMode::OpenOnBackbeats
                };

                match mode {
                    HiHatMode::Sixteenths => {
                        let vel: [u8; 8] = [80, 35, 55, 35, 70, 35, 55, 35];
                        let sixteenth = TICKS_PER_BEAT / 4; // 120
                        for i in 0..16u32 {
                            events.push(NoteEvent {
                                tick: bar_offset + i * sixteenth,
                                note: closed,
                                velocity: vel[(i % 8) as usize],
                                duration: sixteenth.min(60),
                                channel,
                            });
                        }
                    }
                    HiHatMode::OpenOnBackbeats => {
                        let vel: [u8; 8] = [80, 50, 90, 50, 75, 50, 90, 50];
                        for i in 0..8u32 {
                            let beat = i / 2;
                            let is_backbeat = (beat == 1 || beat == 3) && i % 2 == 0;
                            let note = if is_backbeat { open } else { closed };
                            events.push(NoteEvent {
                                tick: bar_offset + i * eighth,
                                note,
                                velocity: vel[i as usize],
                                duration: if is_backbeat { 240 } else { 120 },
                                channel,
                            });
                        }
                    }
                }
            }
            SongPart::Bridge => {
                // Quarter notes — sparse contrast
                let vel: [u8; 4] = [65, 60, 65, 60];
                for i in 0..4u32 {
                    events.push(NoteEvent {
                        tick: bar_offset + i * TICKS_PER_BEAT,
                        note: closed,
                        velocity: vel[i as usize],
                        duration: 120,
                        channel,
                    });
                }
            }
            _ => {
                // Verse/Intro/Outro: 8th notes with groove velocity
                let vel: [u8; 8] = [80, 50, 70, 50, 75, 50, 70, 50];
                for i in 0..8u32 {
                    // 15% chance of dropping an upbeat (breathing)
                    if i % 2 == 1 && self.rng.random::<f64>() < 0.15 {
                        continue;
                    }
                    events.push(NoteEvent {
                        tick: bar_offset + i * eighth,
                        note: closed,
                        velocity: vel[i as usize],
                        duration: 120,
                        channel,
                    });
                }
            }
        }

        // Hi-hat variation pool (~20%)
        if self.rng.random::<f64>() < 0.20 {
            if self.rng.random::<bool>() {
                // Open hat on "and of 4" (classic build signal)
                events.push(NoteEvent {
                    tick: bar_offset + 3 * TICKS_PER_BEAT + eighth,
                    note: open,
                    velocity: 85,
                    duration: 240,
                    channel,
                });
            } else {
                // 16th note hat fill on beat 4 (2-3 quick hits)
                let count = self.rng.random_range(2u32..=3);
                let sixteenth = TICKS_PER_BEAT / 4;
                for i in 0..count {
                    events.push(NoteEvent {
                        tick: bar_offset + 3 * TICKS_PER_BEAT + i * sixteenth,
                        note: closed,
                        velocity: self.rng.random_range(40u8..=55),
                        duration: 60,
                        channel,
                    });
                }
            }
        }

        events
    }

    // -- Tambourine ---------------------------------------------------------

    fn generate_tambourine_bar(
        &mut self,
        bar_offset: u32,
        channel: u8,
    ) -> Vec<NoteEvent> {
        let note = InstrumentType::Tambourine.gm_drum_note().unwrap_or(54);
        let eighth = TICKS_PER_BEAT / 2; // 240
        let vel: [u8; 4] = [55, 60, 55, 60];
        let mut events = Vec::new();

        // Upbeats only: "and" of each beat
        for i in 0..4u32 {
            events.push(NoteEvent {
                tick: bar_offset + i * TICKS_PER_BEAT + eighth,
                note,
                velocity: vel[i as usize],
                duration: 60,
                channel,
            });
        }

        events
    }

    // -- Shaker -------------------------------------------------------------

    fn generate_shaker_bar(&mut self, bar_offset: u32, channel: u8) -> Vec<NoteEvent> {
        let note = InstrumentType::Shaker.gm_drum_note().unwrap_or(70);
        let sixteenth = TICKS_PER_BEAT / 4; // 120
        let mut events = Vec::new();

        // Steady 16th notes at vel 40-50 with ±5 variation
        for i in 0..16u32 {
            events.push(NoteEvent {
                tick: bar_offset + i * sixteenth,
                note,
                velocity: self.rng.random_range(40u8..=50),
                duration: 60,
                channel,
            });
        }

        events
    }

    // -- Ride ---------------------------------------------------------------

    fn generate_ride_bar(&mut self, bar_offset: u32, channel: u8) -> Vec<NoteEvent> {
        let note = InstrumentType::RideCymbal.gm_drum_note().unwrap_or(51);
        let mut events = Vec::new();

        // Quarter notes, bell hit (vel 85) on beat 1, vel 70 otherwise
        for i in 0..4u32 {
            events.push(NoteEvent {
                tick: bar_offset + i * TICKS_PER_BEAT,
                note,
                velocity: if i == 0 { 85 } else { 70 },
                duration: 240,
                channel,
            });
        }

        events
    }

    // -- Main generator -----------------------------------------------------

    /// Generate a complete drum pattern for the given part across multiple bars.
    /// Combines kick, snare, hi-hat, tambourine, shaker, cowbell, ride, and crash.
    pub fn generate_drum_pattern(&mut self, config: &DrumConfig) -> Pattern {
        if config.bars == 0 {
            return Pattern::empty(0);
        }

        let mut all_events = Vec::new();

        // Per-part decisions (made once for consistency across bars)
        let bridge_uses_ride = self.rng.random::<bool>();
        let bridge_has_shaker = self.rng.random::<bool>();

        for bar_idx in 0..config.bars {
            let bar_offset = bar_idx * TICKS_PER_BAR;
            let is_first_bar = bar_idx == 0;

            // Kick
            let mut kick = self.generate_kick_bar(bar_offset, config.part, config.channel);
            self.humanize_events(&mut kick, InstrumentType::Kick, config.part);
            all_events.append(&mut kick);

            // Snare
            let mut snare = self.generate_snare_bar(bar_offset, config.part, config.channel);
            self.humanize_events(&mut snare, InstrumentType::Snare, config.part);
            all_events.append(&mut snare);

            // Hi-Hat vs Ride (bridge alternates based on per-part coin flip)
            if config.part == SongPart::Bridge && bridge_uses_ride {
                let mut ride = self.generate_ride_bar(bar_offset, config.channel);
                self.humanize_events(&mut ride, InstrumentType::RideCymbal, config.part);
                all_events.append(&mut ride);
            } else {
                let mut hihat =
                    self.generate_hihat_bar(bar_offset, config.part, config.channel);
                self.humanize_events(&mut hihat, InstrumentType::HiHat, config.part);
                all_events.append(&mut hihat);
            }

            // Tambourine (chorus, pre-chorus, bridge)
            if matches!(
                config.part,
                SongPart::Chorus | SongPart::PreChorus | SongPart::Bridge
            ) {
                let mut tamb = self.generate_tambourine_bar(bar_offset, config.channel);
                self.humanize_events(
                    &mut tamb,
                    InstrumentType::Tambourine,
                    config.part,
                );
                all_events.append(&mut tamb);
            }

            // Shaker (chorus always, bridge sometimes)
            let shaker_active = match config.part {
                SongPart::Chorus => true,
                SongPart::Bridge => bridge_has_shaker,
                _ => false,
            };
            if shaker_active {
                let mut shaker = self.generate_shaker_bar(bar_offset, config.channel);
                self.humanize_events(
                    &mut shaker,
                    InstrumentType::Shaker,
                    config.part,
                );
                all_events.append(&mut shaker);
            }

            // Cowbell (chorus only, every other bar)
            if config.part == SongPart::Chorus && bar_idx % 2 == 0 {
                let note = InstrumentType::Cowbell.gm_drum_note().unwrap_or(56);
                let mut cowbell = vec![NoteEvent {
                    tick: bar_offset,
                    note,
                    velocity: 60,
                    duration: 60,
                    channel: config.channel,
                }];
                self.humanize_events(
                    &mut cowbell,
                    InstrumentType::Cowbell,
                    config.part,
                );
                all_events.append(&mut cowbell);
            }

            // Crash (beat 1 of first bar of part only)
            if is_first_bar {
                let note = InstrumentType::CrashCymbal.gm_drum_note().unwrap_or(49);
                let mut crash = vec![NoteEvent {
                    tick: bar_offset,
                    note,
                    velocity: self.rng.random_range(100u8..=110),
                    duration: 480,
                    channel: config.channel,
                }];
                self.humanize_events(
                    &mut crash,
                    InstrumentType::CrashCymbal,
                    config.part,
                );
                all_events.append(&mut crash);
            }
        }

        all_events.sort_by_key(|e| e.tick);

        Pattern {
            events: all_events,
            cc_events: Vec::new(),
            length_ticks: config.bars * TICKS_PER_BAR,
            bars: config.bars,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::song::InstrumentType;

    fn verse_config(bars: u32) -> DrumConfig {
        DrumConfig {
            part: SongPart::Verse,
            bars,
            channel: 9,
        }
    }

    fn chorus_config(bars: u32) -> DrumConfig {
        DrumConfig {
            part: SongPart::Chorus,
            bars,
            channel: 9,
        }
    }

    fn bridge_config(bars: u32) -> DrumConfig {
        DrumConfig {
            part: SongPart::Bridge,
            bars,
            channel: 9,
        }
    }

    // -- Determinism --------------------------------------------------------

    #[test]
    fn same_seed_produces_identical_drum_pattern() {
        let config = verse_config(4);

        let mut e1 = DrumEngine::new(42);
        let r1 = e1.generate_drum_pattern(&config);

        let mut e2 = DrumEngine::new(42);
        let r2 = e2.generate_drum_pattern(&config);

        assert_eq!(r1, r2, "same seed must produce identical drum pattern");
    }

    #[test]
    fn different_seeds_produce_different_drum_patterns() {
        let config = verse_config(4);

        let mut e1 = DrumEngine::new(42);
        let r1 = e1.generate_drum_pattern(&config);

        let mut e2 = DrumEngine::new(99);
        let r2 = e2.generate_drum_pattern(&config);

        assert_ne!(r1, r2, "different seeds should produce different patterns");
    }

    #[test]
    fn chorus_determinism() {
        let config = chorus_config(8);

        let mut e1 = DrumEngine::new(77);
        let r1 = e1.generate_drum_pattern(&config);

        let mut e2 = DrumEngine::new(77);
        let r2 = e2.generate_drum_pattern(&config);

        assert_eq!(r1, r2, "chorus drum pattern must be deterministic");
    }

    #[test]
    fn bridge_determinism() {
        let config = bridge_config(8);

        let mut e1 = DrumEngine::new(55);
        let r1 = e1.generate_drum_pattern(&config);

        let mut e2 = DrumEngine::new(55);
        let r2 = e2.generate_drum_pattern(&config);

        assert_eq!(r1, r2, "bridge drum pattern must be deterministic");
    }

    // -- Structure ----------------------------------------------------------

    #[test]
    fn pattern_bar_count_and_ticks() {
        let config = verse_config(4);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        assert_eq!(pattern.bars, 4);
        assert_eq!(pattern.length_ticks, 4 * TICKS_PER_BAR);
    }

    #[test]
    fn empty_bars_produces_empty_pattern() {
        let config = DrumConfig {
            part: SongPart::Verse,
            bars: 0,
            channel: 9,
        };
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        assert!(pattern.events.is_empty());
        assert_eq!(pattern.bars, 0);
    }

    #[test]
    fn events_sorted_by_tick() {
        let config = chorus_config(8);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        for i in 1..pattern.events.len() {
            assert!(
                pattern.events[i].tick >= pattern.events[i - 1].tick,
                "events must be sorted by tick"
            );
        }
    }

    // -- Instrument presence per part ---------------------------------------

    #[test]
    fn verse_has_kick_snare_hihat() {
        let config = verse_config(4);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        let kick = InstrumentType::Kick.gm_drum_note().unwrap();
        let snare = InstrumentType::Snare.gm_drum_note().unwrap();
        let hihat = InstrumentType::HiHat.gm_drum_note().unwrap();

        assert!(pattern.events.iter().any(|e| e.note == kick), "verse must have kick");
        assert!(pattern.events.iter().any(|e| e.note == snare), "verse must have snare");
        assert!(pattern.events.iter().any(|e| e.note == hihat), "verse must have hi-hat");
    }

    #[test]
    fn verse_has_no_tambourine_or_shaker() {
        let config = verse_config(4);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        let tamb = InstrumentType::Tambourine.gm_drum_note().unwrap();
        let shaker = InstrumentType::Shaker.gm_drum_note().unwrap();

        assert!(
            !pattern.events.iter().any(|e| e.note == tamb),
            "verse should not have tambourine"
        );
        assert!(
            !pattern.events.iter().any(|e| e.note == shaker),
            "verse should not have shaker"
        );
    }

    #[test]
    fn chorus_has_crash_on_first_bar() {
        let config = chorus_config(4);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        let crash = InstrumentType::CrashCymbal.gm_drum_note().unwrap();
        assert!(
            pattern.events.iter().any(|e| e.note == crash),
            "chorus must have crash on first bar"
        );
    }

    #[test]
    fn chorus_has_cowbell_on_even_bars() {
        let config = chorus_config(4);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        let cowbell = InstrumentType::Cowbell.gm_drum_note().unwrap();
        assert!(
            pattern.events.iter().any(|e| e.note == cowbell),
            "chorus must have cowbell"
        );
    }

    #[test]
    fn bridge_uses_rimshot() {
        let config = bridge_config(4);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        let rimshot = InstrumentType::Rimshot.gm_drum_note().unwrap();
        assert!(
            pattern.events.iter().any(|e| e.note == rimshot),
            "bridge must use rimshot/cross-stick"
        );
    }

    // -- Kick pattern -------------------------------------------------------

    #[test]
    fn verse_kick_on_beats_1_and_3() {
        let config = verse_config(1);
        let mut engine = DrumEngine::new(1000); // high seed to reduce variation
        let pattern = engine.generate_drum_pattern(&config);

        let kick = InstrumentType::Kick.gm_drum_note().unwrap();
        let kick_events: Vec<&NoteEvent> =
            pattern.events.iter().filter(|e| e.note == kick).collect();

        // At minimum, beats 1 and 3 (humanization may shift by ±5)
        assert!(
            kick_events.len() >= 2,
            "verse should have at least 2 kick hits per bar, got {}",
            kick_events.len()
        );
    }

    #[test]
    fn bridge_kick_only_beat_1() {
        let config = bridge_config(1);
        // Use a seed where variation doesn't fire
        let mut engine = DrumEngine::new(1234);
        let pattern = engine.generate_drum_pattern(&config);

        let kick = InstrumentType::Kick.gm_drum_note().unwrap();
        let kick_events: Vec<&NoteEvent> =
            pattern.events.iter().filter(|e| e.note == kick).collect();

        // Bridge has only beat 1 kick (plus possible variation)
        assert!(
            !kick_events.is_empty(),
            "bridge must have at least 1 kick"
        );
        // Base pattern is 1 hit; variation adds at most 2 more
        assert!(
            kick_events.len() <= 3,
            "bridge kick should be sparse, got {} hits",
            kick_events.len()
        );
    }

    // -- Snare pattern ------------------------------------------------------

    #[test]
    fn verse_snare_backbeat() {
        let config = verse_config(1);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        let snare = InstrumentType::Snare.gm_drum_note().unwrap();
        let snare_events: Vec<&NoteEvent> =
            pattern.events.iter().filter(|e| e.note == snare).collect();

        assert!(
            snare_events.len() >= 2,
            "verse should have at least 2 snare backbeats, got {}",
            snare_events.len()
        );
    }

    #[test]
    fn chorus_snare_has_ghost_notes() {
        let config = chorus_config(4);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        let snare = InstrumentType::Snare.gm_drum_note().unwrap();
        let ghost_count = pattern
            .events
            .iter()
            .filter(|e| e.note == snare && e.velocity < 50)
            .count();

        assert!(
            ghost_count > 0,
            "chorus snare should have ghost notes"
        );
    }

    // -- Hi-hat pattern -----------------------------------------------------

    #[test]
    fn verse_hihat_eighth_notes() {
        let config = verse_config(1);
        let mut engine = DrumEngine::new(42);
        let pattern = engine.generate_drum_pattern(&config);

        let hihat = InstrumentType::HiHat.gm_drum_note().unwrap();
        let hat_count = pattern.events.iter().filter(|e| e.note == hihat).count();

        // 8th notes = 8 per bar, minus possible dropped upbeats (15% each)
        assert!(
            hat_count >= 4,
            "verse should have at least 4 hi-hat hits per bar, got {}",
            hat_count
        );
    }

    // -- Velocity ranges ----------------------------------------------------

    #[test]
    fn all_velocities_in_valid_range() {
        for &part in &[
            SongPart::Verse,
            SongPart::Chorus,
            SongPart::Bridge,
            SongPart::Intro,
            SongPart::Outro,
        ] {
            let config = DrumConfig {
                part,
                bars: 4,
                channel: 9,
            };
            let mut engine = DrumEngine::new(42);
            let pattern = engine.generate_drum_pattern(&config);

            for event in &pattern.events {
                assert!(
                    event.velocity >= 1 && event.velocity <= 127,
                    "velocity {} out of range for {:?}",
                    event.velocity,
                    part,
                );
            }
        }
    }

    // -- Intensity scaling --------------------------------------------------

    #[test]
    fn drum_intensity_values() {
        assert!((drum_intensity(SongPart::Intro) - 0.4).abs() < f64::EPSILON);
        assert!((drum_intensity(SongPart::Verse) - 0.65).abs() < f64::EPSILON);
        assert!((drum_intensity(SongPart::PreChorus) - 0.8).abs() < f64::EPSILON);
        assert!((drum_intensity(SongPart::Chorus) - 1.0).abs() < f64::EPSILON);
        assert!((drum_intensity(SongPart::Bridge) - 0.55).abs() < f64::EPSILON);
        assert!((drum_intensity(SongPart::Outro) - 0.45).abs() < f64::EPSILON);
    }

    #[test]
    fn intro_velocities_lower_than_chorus() {
        let intro_config = DrumConfig {
            part: SongPart::Intro,
            bars: 4,
            channel: 9,
        };
        let chorus_config = chorus_config(4);

        let mut e1 = DrumEngine::new(42);
        let intro_pattern = e1.generate_drum_pattern(&intro_config);

        let mut e2 = DrumEngine::new(42);
        let chorus_pattern = e2.generate_drum_pattern(&chorus_config);

        let intro_avg: f64 = if intro_pattern.events.is_empty() {
            0.0
        } else {
            intro_pattern
                .events
                .iter()
                .map(|e| e.velocity as f64)
                .sum::<f64>()
                / intro_pattern.events.len() as f64
        };

        let chorus_avg: f64 = if chorus_pattern.events.is_empty() {
            0.0
        } else {
            chorus_pattern
                .events
                .iter()
                .map(|e| e.velocity as f64)
                .sum::<f64>()
                / chorus_pattern.events.len() as f64
        };

        assert!(
            intro_avg < chorus_avg,
            "intro avg velocity ({:.1}) should be lower than chorus ({:.1})",
            intro_avg,
            chorus_avg,
        );
    }

    // -- DrumHumanize -------------------------------------------------------

    #[test]
    fn humanize_params_per_instrument() {
        let kick = DrumHumanize::for_instrument(InstrumentType::Kick, false);
        assert_eq!(kick.timing_ticks, 5);
        assert_eq!(kick.velocity_range, 4);

        let snare = DrumHumanize::for_instrument(InstrumentType::Snare, false);
        assert_eq!(snare.timing_ticks, 4);

        let snare_ghost = DrumHumanize::for_instrument(InstrumentType::Snare, true);
        assert_eq!(snare_ghost.timing_ticks, 10);

        let hihat = DrumHumanize::for_instrument(InstrumentType::HiHat, false);
        assert_eq!(hihat.timing_ticks, 6);
        assert_eq!(hihat.velocity_range, 6);

        let tamb = DrumHumanize::for_instrument(InstrumentType::Tambourine, false);
        assert_eq!(tamb.timing_ticks, 8);
        assert_eq!(tamb.velocity_range, 8);
    }

    // -- Serde roundtrips ---------------------------------------------------

    #[test]
    fn hihat_mode_serde_roundtrip() {
        let mode = HiHatMode::OpenOnBackbeats;
        let json = serde_json::to_string(&mode).expect("serialize");
        assert_eq!(json, r#""open_on_backbeats""#);
        let parsed: HiHatMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, mode);
    }

    #[test]
    fn drum_humanize_serde_roundtrip() {
        let h = DrumHumanize::for_instrument(InstrumentType::Kick, false);
        let json = serde_json::to_string(&h).expect("serialize");
        let parsed: DrumHumanize = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, h);
    }

    #[test]
    fn drum_config_serde_roundtrip() {
        let config = DrumConfig {
            part: SongPart::Chorus,
            bars: 8,
            channel: 9,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let parsed: DrumConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.part, config.part);
        assert_eq!(parsed.bars, config.bars);
        assert_eq!(parsed.channel, config.channel);
    }

    // -- Humanization overflow safety ---------------------------------------

    #[test]
    fn humanize_large_tick_no_overflow() {
        let mut engine = DrumEngine::new(42);
        let large_tick = u32::MAX - 100;
        let mut events = vec![NoteEvent {
            tick: large_tick,
            note: 36,
            velocity: 100,
            duration: 120,
            channel: 9,
        }];

        engine.humanize_events(&mut events, InstrumentType::Kick, SongPart::Chorus);

        let diff = events[0].tick.abs_diff(large_tick);
        assert!(
            diff <= 10,
            "tick drifted {} from original, expected <= 10",
            diff,
        );
    }

    #[test]
    fn humanize_tick_zero_no_underflow() {
        let mut engine = DrumEngine::new(42);
        let mut events = vec![NoteEvent {
            tick: 0,
            note: 36,
            velocity: 100,
            duration: 120,
            channel: 9,
        }];

        engine.humanize_events(&mut events, InstrumentType::Kick, SongPart::Chorus);

        assert!(
            events[0].tick <= 10,
            "tick {} too large for near-zero input",
            events[0].tick,
        );
    }

    // -- Ghost note humanization detection -----------------------------------

    #[test]
    fn ghost_snare_gets_looser_timing_than_regular() {
        // Run many iterations to verify statistical difference in timing spread.
        // Ghost notes (vel < 50) should use ±10 tick params; regular (vel >= 50) use ±4.
        let base_tick: u32 = 1000;
        let iterations = 200;

        let mut ghost_total_drift: f64 = 0.0;
        let mut regular_total_drift: f64 = 0.0;

        for i in 0..iterations {
            // Ghost note event (vel 35, well below threshold)
            let mut ghost_events = vec![NoteEvent {
                tick: base_tick,
                note: 38,
                velocity: 35,
                duration: 30,
                channel: 9,
            }];
            let mut engine = DrumEngine::new(i);
            engine.humanize_events(&mut ghost_events, InstrumentType::Snare, SongPart::Chorus);
            ghost_total_drift += (ghost_events[0].tick as f64 - base_tick as f64).abs();

            // Regular snare event (vel 100)
            let mut regular_events = vec![NoteEvent {
                tick: base_tick,
                note: 38,
                velocity: 100,
                duration: 120,
                channel: 9,
            }];
            let mut engine2 = DrumEngine::new(i);
            engine2.humanize_events(&mut regular_events, InstrumentType::Snare, SongPart::Chorus);
            regular_total_drift += (regular_events[0].tick as f64 - base_tick as f64).abs();
        }

        let ghost_avg = ghost_total_drift / iterations as f64;
        let regular_avg = regular_total_drift / iterations as f64;

        // Ghost notes (±10 tick std dev) should have noticeably wider average drift
        // than regular notes (±4 tick std dev). With 200 samples this is very reliable.
        assert!(
            ghost_avg > regular_avg,
            "ghost avg drift ({:.2}) should exceed regular avg drift ({:.2})",
            ghost_avg,
            regular_avg,
        );
    }

    #[test]
    fn ghost_detection_uses_velocity_threshold() {
        // Events at exactly the threshold boundary: vel 49 = ghost, vel 50 = regular
        let base_tick: u32 = 1000;

        let mut below_drifts = Vec::new();
        let mut at_drifts = Vec::new();

        for i in 0..100u64 {
            let mut below = vec![NoteEvent {
                tick: base_tick,
                note: 38,
                velocity: 49, // below threshold
                duration: 30,
                channel: 9,
            }];
            let mut engine = DrumEngine::new(i);
            engine.humanize_events(&mut below, InstrumentType::Snare, SongPart::Verse);
            below_drifts.push((below[0].tick as i64 - base_tick as i64).unsigned_abs());

            let mut at = vec![NoteEvent {
                tick: base_tick,
                note: 38,
                velocity: 50, // at threshold — not ghost
                duration: 120,
                channel: 9,
            }];
            let mut engine2 = DrumEngine::new(i);
            engine2.humanize_events(&mut at, InstrumentType::Snare, SongPart::Verse);
            at_drifts.push((at[0].tick as i64 - base_tick as i64).unsigned_abs());
        }

        // vel 49 events should have max possible drift of 10, vel 50 max of 4
        let below_max = *below_drifts.iter().max().expect("non-empty");
        let at_max = *at_drifts.iter().max().expect("non-empty");

        assert!(
            below_max <= 10,
            "ghost note drift {} exceeds ±10 limit",
            below_max,
        );
        assert!(
            at_max <= 4,
            "regular note drift {} exceeds ±4 limit",
            at_max,
        );
    }

    #[test]
    fn non_snare_instruments_ignore_ghost_detection() {
        // Kick with low velocity should NOT get ghost timing (kick is always ±5)
        let base_tick: u32 = 1000;

        for i in 0..100u64 {
            let mut events = vec![NoteEvent {
                tick: base_tick,
                note: 36,
                velocity: 35, // low vel but it's a kick
                duration: 60,
                channel: 9,
            }];
            let mut engine = DrumEngine::new(i);
            engine.humanize_events(&mut events, InstrumentType::Kick, SongPart::Chorus);

            let drift = (events[0].tick as i64 - base_tick as i64).unsigned_abs();
            assert!(
                drift <= 5,
                "kick drift {} exceeds ±5 limit even with low velocity",
                drift,
            );
        }
    }

    // -- All parts produce events -------------------------------------------

    #[test]
    fn all_parts_produce_non_empty_patterns() {
        for &part in &[
            SongPart::Intro,
            SongPart::Verse,
            SongPart::PreChorus,
            SongPart::Chorus,
            SongPart::Bridge,
            SongPart::Outro,
        ] {
            let config = DrumConfig {
                part,
                bars: 4,
                channel: 9,
            };
            let mut engine = DrumEngine::new(42);
            let pattern = engine.generate_drum_pattern(&config);

            assert!(
                !pattern.events.is_empty(),
                "{:?} should produce drum events",
                part,
            );
        }
    }
}
