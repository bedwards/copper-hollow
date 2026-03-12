// MIDI file export via midly.
//
// Converts a `Song` into a Standard MIDI File (Type 1, multi-track)
// with 480 ticks per beat, tempo meta events, track names, and
// note/CC/pitch-bend events.
// Fully implemented — awaiting GUI integration (v0.4.0).
#![allow(dead_code)]

use anyhow::Result;
use midly::num::{u14, u15, u24, u28, u4, u7};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use std::path::Path;

use crate::engine::song::Song;
use crate::engine::TICKS_PER_BAR;

/// Convert BPM to MIDI tempo (microseconds per quarter note).
fn bpm_to_microseconds(bpm: f64) -> u32 {
    (60_000_000.0 / bpm) as u32
}

/// Generate a MIDI filename: `{sanitized_title}_{unix_timestamp}.mid`.
pub fn generate_filename(title: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{sanitized}_{timestamp}.mid")
}

/// Absolute-time event for sorting before delta-time conversion.
enum AbsEvent {
    NoteOn {
        tick: u32,
        channel: u8,
        key: u8,
        vel: u8,
    },
    NoteOff {
        tick: u32,
        channel: u8,
        key: u8,
    },
    Controller {
        tick: u32,
        channel: u8,
        cc: u8,
        value: u8,
    },
    PitchBend {
        tick: u32,
        channel: u8,
        value: u16,
    },
}

impl AbsEvent {
    fn tick(&self) -> u32 {
        match self {
            Self::NoteOn { tick, .. }
            | Self::NoteOff { tick, .. }
            | Self::Controller { tick, .. }
            | Self::PitchBend { tick, .. } => *tick,
        }
    }

    /// Sort key: tick first, then event type (NoteOff < CC/PitchBend < NoteOn)
    /// so notes at the same tick close before new ones open.
    fn sort_key(&self) -> (u32, u8) {
        let priority = match self {
            Self::NoteOff { .. } => 0,
            Self::Controller { .. } | Self::PitchBend { .. } => 1,
            Self::NoteOn { .. } => 2,
        };
        (self.tick(), priority)
    }
}

/// Flatten a track's patterns across the song structure into absolute-tick events.
fn flatten_track(track: &crate::engine::song::Track, song: &Song) -> Vec<AbsEvent> {
    let mut events = Vec::new();
    let mut tick_offset: u32 = 0;

    for part in &song.structure {
        let part_ticks = part.typical_bars() * TICKS_PER_BAR;
        let active = track.active_parts.get(part).copied().unwrap_or(false);

        if active {
            if let Some(pattern) = track.patterns.get(part) {
                for note in &pattern.events {
                    events.push(AbsEvent::NoteOn {
                        tick: tick_offset + note.tick,
                        channel: note.channel,
                        key: note.note,
                        vel: note.velocity,
                    });
                    events.push(AbsEvent::NoteOff {
                        tick: tick_offset + note.tick + note.duration,
                        channel: note.channel,
                        key: note.note,
                    });
                }
                for cc in &pattern.cc_events {
                    let abs_tick = tick_offset + cc.tick;
                    if cc.cc == 255 {
                        events.push(AbsEvent::PitchBend {
                            tick: abs_tick,
                            channel: cc.channel,
                            value: cc.value,
                        });
                    } else {
                        events.push(AbsEvent::Controller {
                            tick: abs_tick,
                            channel: cc.channel,
                            cc: cc.cc,
                            value: cc.value.min(127) as u8,
                        });
                    }
                }
            }
        }

        tick_offset += part_ticks;
    }

    events.sort_by_key(|e| e.sort_key());
    events
}

/// Export a `Song` to Standard MIDI File bytes (Type 1, multi-track).
///
/// Layout:
/// - Track 0: conductor (tempo, time signature, song title)
/// - Tracks 1..=N: one per song track with name and note/CC events
pub fn export_to_bytes(song: &Song) -> Result<Vec<u8>> {
    // Pre-allocate strings so borrows outlive the Smf.
    let song_title = song.title.clone();
    let track_names: Vec<String> = song.tracks.iter().map(|t| t.name.clone()).collect();

    let mut smf = Smf::new(Header::new(
        Format::Parallel,
        Timing::Metrical(u15::new(480)),
    ));

    // --- Conductor track ---
    let tempo_us = bpm_to_microseconds(song.tempo);
    let denom_power = (f64::from(song.time_signature.1)).log2() as u8;
    smf.tracks.push(vec![
        TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::TrackName(song_title.as_bytes())),
        },
        TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(tempo_us))),
        },
        TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                song.time_signature.0,
                denom_power,
                24,
                8,
            )),
        },
        TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        },
    ]);

    // --- Instrument tracks ---
    for (i, track) in song.tracks.iter().enumerate() {
        let abs_events = flatten_track(track, song);
        let mut midi_track: Vec<TrackEvent<'_>> = Vec::new();

        midi_track.push(TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::TrackName(track_names[i].as_bytes())),
        });

        let mut last_tick: u32 = 0;
        for ev in &abs_events {
            let delta = ev.tick() - last_tick;
            last_tick = ev.tick();

            let kind = match ev {
                AbsEvent::NoteOn {
                    channel, key, vel, ..
                } => TrackEventKind::Midi {
                    channel: u4::new((*channel).min(15)),
                    message: MidiMessage::NoteOn {
                        key: u7::new((*key).min(127)),
                        vel: u7::new((*vel).min(127)),
                    },
                },
                AbsEvent::NoteOff { channel, key, .. } => TrackEventKind::Midi {
                    channel: u4::new((*channel).min(15)),
                    message: MidiMessage::NoteOff {
                        key: u7::new((*key).min(127)),
                        vel: u7::new(0),
                    },
                },
                AbsEvent::Controller {
                    channel, cc, value, ..
                } => TrackEventKind::Midi {
                    channel: u4::new((*channel).min(15)),
                    message: MidiMessage::Controller {
                        controller: u7::new((*cc).min(127)),
                        value: u7::new((*value).min(127)),
                    },
                },
                AbsEvent::PitchBend {
                    channel, value, ..
                } => TrackEventKind::Midi {
                    channel: u4::new((*channel).min(15)),
                    message: MidiMessage::PitchBend {
                        bend: midly::PitchBend(u14::new((*value).min(16383))),
                    },
                },
            };

            midi_track.push(TrackEvent {
                delta: u28::new(delta),
                kind,
            });
        }

        midi_track.push(TrackEvent {
            delta: u28::new(0),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        smf.tracks.push(midi_track);
    }

    let mut buf = Vec::new();
    smf.write_std(&mut buf)?;
    Ok(buf)
}

/// Write a `Song` as a MIDI file to the given path.
pub fn export_to_file(song: &Song, path: &Path) -> Result<()> {
    let bytes = export_to_bytes(song)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::song::{CcEvent, NoteEvent, Pattern, SongPart};
    use midly::num::{u15, u28, u4, u7};
    use midly::{Format, Header, Smf, Timing, TrackEvent, TrackEventKind};

    #[test]
    fn midi_write_read_roundtrip() {
        let mut smf = Smf::new(Header::new(
            Format::Parallel,
            Timing::Metrical(u15::new(480)),
        ));

        let track = vec![
            TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOn {
                        key: u7::new(60),
                        vel: u7::new(100),
                    },
                },
            },
            TrackEvent {
                delta: u28::new(480),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOff {
                        key: u7::new(60),
                        vel: u7::new(0),
                    },
                },
            },
            TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            },
        ];
        smf.tracks.push(track);

        let mut buf = Vec::new();
        smf.write_std(&mut buf).unwrap();

        let parsed = Smf::parse(&buf).unwrap();
        assert_eq!(parsed.tracks.len(), 1);
        assert_eq!(parsed.tracks[0].len(), 3);
    }

    #[test]
    fn bpm_to_microseconds_conversion() {
        assert_eq!(bpm_to_microseconds(120.0), 500_000);
        assert_eq!(bpm_to_microseconds(60.0), 1_000_000);
        assert_eq!(bpm_to_microseconds(140.0), 428571);
    }

    #[test]
    fn generate_filename_format() {
        let name = generate_filename("My Folk Song");
        assert!(name.starts_with("My_Folk_Song_"));
        assert!(name.ends_with(".mid"));
    }

    #[test]
    fn export_empty_song_produces_valid_midi() {
        let song = Song::default_song();
        let bytes = export_to_bytes(&song).unwrap();
        let parsed = Smf::parse(&bytes).unwrap();

        // 1 conductor + 16 instrument tracks
        assert_eq!(parsed.tracks.len(), 17);

        // Verify tick resolution
        match parsed.header.timing {
            Timing::Metrical(tpb) => assert_eq!(tpb.as_int(), 480),
            _ => panic!("expected metrical timing"),
        }
    }

    #[test]
    fn export_preserves_tempo() {
        let mut song = Song::default_song();
        song.tempo = 140.0;
        let bytes = export_to_bytes(&song).unwrap();
        let parsed = Smf::parse(&bytes).unwrap();

        let conductor = &parsed.tracks[0];
        let tempo_event = conductor
            .iter()
            .find(|ev| matches!(ev.kind, TrackEventKind::Meta(MetaMessage::Tempo(_))));
        assert!(tempo_event.is_some());

        if let TrackEventKind::Meta(MetaMessage::Tempo(us)) = tempo_event.unwrap().kind {
            assert_eq!(us.as_int(), 428571);
        }
    }

    #[test]
    fn export_includes_track_names() {
        let song = Song::default_song();
        let bytes = export_to_bytes(&song).unwrap();
        let parsed = Smf::parse(&bytes).unwrap();

        // Conductor track has song title
        let conductor = &parsed.tracks[0];
        let name_event = conductor.iter().find(|ev| {
            matches!(ev.kind, TrackEventKind::Meta(MetaMessage::TrackName(_)))
        });
        assert!(name_event.is_some());
        if let TrackEventKind::Meta(MetaMessage::TrackName(name)) = name_event.unwrap().kind {
            assert_eq!(std::str::from_utf8(name).unwrap(), "Untitled Folk Song");
        }

        // First instrument track (index 1) = "Kick"
        let kick_track = &parsed.tracks[1];
        let name_event = kick_track.iter().find(|ev| {
            matches!(ev.kind, TrackEventKind::Meta(MetaMessage::TrackName(_)))
        });
        assert!(name_event.is_some());
        if let TrackEventKind::Meta(MetaMessage::TrackName(name)) = name_event.unwrap().kind {
            assert_eq!(std::str::from_utf8(name).unwrap(), "Kick");
        }
    }

    #[test]
    fn export_roundtrip_with_notes() {
        let mut song = Song::default_song();

        // Add 32 kick hits (one per beat) to the Verse pattern
        let mut pattern = Pattern::empty(8);
        for beat in 0..32 {
            pattern.events.push(NoteEvent {
                tick: beat * 480,
                note: 36,
                velocity: 100,
                duration: 120,
                channel: 0,
            });
        }
        song.tracks[0].patterns.insert(SongPart::Verse, pattern);

        let bytes = export_to_bytes(&song).unwrap();
        let parsed = Smf::parse(&bytes).unwrap();

        // Kick = MIDI track index 1 (0 is conductor)
        let kick_track = &parsed.tracks[1];

        let note_on_count = kick_track
            .iter()
            .filter(|ev| {
                matches!(
                    ev.kind,
                    TrackEventKind::Midi {
                        message: MidiMessage::NoteOn { .. },
                        ..
                    }
                )
            })
            .count();

        // Verse appears twice in default structure → 32 * 2 = 64
        assert_eq!(note_on_count, 64);

        let note_off_count = kick_track
            .iter()
            .filter(|ev| {
                matches!(
                    ev.kind,
                    TrackEventKind::Midi {
                        message: MidiMessage::NoteOff { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(note_off_count, 64);
    }

    #[test]
    fn export_roundtrip_cc_and_pitch_bend() {
        let mut song = Song::default_song();

        let mut pattern = Pattern::empty(4);
        pattern.cc_events.push(CcEvent {
            tick: 0,
            cc: 1,
            value: 64,
            channel: 4,
        });
        pattern.cc_events.push(CcEvent {
            tick: 480,
            cc: 255,
            value: 8192,
            channel: 4,
        });
        song.tracks[4]
            .patterns
            .insert(SongPart::Intro, pattern);

        let bytes = export_to_bytes(&song).unwrap();
        let parsed = Smf::parse(&bytes).unwrap();

        // Acoustic Guitar = song track 4 → MIDI track index 5
        let guitar_track = &parsed.tracks[5];

        let cc_count = guitar_track
            .iter()
            .filter(|ev| {
                matches!(
                    ev.kind,
                    TrackEventKind::Midi {
                        message: MidiMessage::Controller { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(cc_count, 1);

        let pb_count = guitar_track
            .iter()
            .filter(|ev| {
                matches!(
                    ev.kind,
                    TrackEventKind::Midi {
                        message: MidiMessage::PitchBend { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(pb_count, 1);
    }

    #[test]
    fn export_file_roundtrip() {
        let mut song = Song::default_song();
        song.title = "Test Song".to_string();
        song.tempo = 130.0;

        // Kick: 32 notes per Verse
        let mut kick_pattern = Pattern::empty(8);
        for beat in 0..32 {
            kick_pattern.events.push(NoteEvent {
                tick: beat * 480,
                note: 36,
                velocity: 100,
                duration: 120,
                channel: 0,
            });
        }
        song.tracks[0]
            .patterns
            .insert(SongPart::Verse, kick_pattern);

        // Bass: 8 notes per Verse
        let mut bass_pattern = Pattern::empty(8);
        for bar in 0..8 {
            bass_pattern.events.push(NoteEvent {
                tick: bar * 1920,
                note: 40,
                velocity: 90,
                duration: 960,
                channel: 6,
            });
        }
        song.tracks[6]
            .patterns
            .insert(SongPart::Verse, bass_pattern);

        // Write to temp file
        let dir = std::env::temp_dir();
        let path = dir.join("copper_hollow_test.mid");
        export_to_file(&song, &path).unwrap();

        // Read back and verify
        let data = std::fs::read(&path).unwrap();
        let parsed = Smf::parse(&data).unwrap();

        assert_eq!(parsed.tracks.len(), 17);
        match parsed.header.timing {
            Timing::Metrical(tpb) => assert_eq!(tpb.as_int(), 480),
            _ => panic!("expected metrical timing"),
        }

        // Verify kick notes (Verse x2 = 64)
        let kick_notes = parsed.tracks[1]
            .iter()
            .filter(|ev| {
                matches!(
                    ev.kind,
                    TrackEventKind::Midi {
                        message: MidiMessage::NoteOn { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(kick_notes, 64);

        // Verify bass notes (Verse x2 = 16)
        let bass_notes = parsed.tracks[7]
            .iter()
            .filter(|ev| {
                matches!(
                    ev.kind,
                    TrackEventKind::Midi {
                        message: MidiMessage::NoteOn { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(bass_notes, 16);

        std::fs::remove_file(&path).ok();
    }
}
