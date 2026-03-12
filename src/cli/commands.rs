use anyhow::Result;
use std::path::Path;

use crate::engine::song::{InstrumentType, Pattern, SongPart, StrumPattern};
use crate::engine::theory::ScaleType;
use crate::midi_export;
use crate::state::AppState;

use super::{Cli, Commands};

/// Build the JSON response for a CLI command.
/// Extracted from `execute` for testability.
fn build_response(command: &Commands, args: &Cli) -> serde_json::Value {
    match command {
        Commands::ListScales => {
            let scales = vec![
                ScaleType::Major,
                ScaleType::NaturalMinor,
                ScaleType::HarmonicMinor,
                ScaleType::Dorian,
                ScaleType::Mixolydian,
                ScaleType::MinorPentatonic,
                ScaleType::Blues,
            ];
            serde_json::json!({"ok": true, "data": scales})
        }
        Commands::ListInstruments => {
            let instruments = vec![
                InstrumentType::AcousticGuitar,
                InstrumentType::ElectricGuitar,
                InstrumentType::ElectricBass,
                InstrumentType::AcousticBass,
                InstrumentType::PedalSteel,
                InstrumentType::Mandolin,
                InstrumentType::Banjo,
                InstrumentType::HammondOrgan,
                InstrumentType::Piano,
                InstrumentType::Pad,
                InstrumentType::Kick,
                InstrumentType::Snare,
                InstrumentType::HiHat,
                InstrumentType::OpenHiHat,
                InstrumentType::Clap,
                InstrumentType::Tambourine,
                InstrumentType::Cowbell,
                InstrumentType::Shaker,
                InstrumentType::RideCymbal,
                InstrumentType::CrashCymbal,
                InstrumentType::Toms,
                InstrumentType::Rimshot,
            ];
            serde_json::json!({"ok": true, "data": instruments})
        }
        Commands::ListParts => {
            let parts = vec![
                SongPart::Intro,
                SongPart::Verse,
                SongPart::PreChorus,
                SongPart::Chorus,
                SongPart::Bridge,
                SongPart::Outro,
            ];
            serde_json::json!({"ok": true, "data": parts})
        }
        Commands::ListStrumPatterns => {
            let patterns = vec![StrumPattern::default_folk()];
            serde_json::json!({"ok": true, "data": patterns})
        }
        Commands::GetState => {
            let seed = args.seed.unwrap_or(42);
            let state = AppState::new(seed);
            serde_json::json!({"ok": true, "data": state})
        }
        Commands::GetSong => {
            let seed = args.seed.unwrap_or(42);
            let state = AppState::new(seed);
            serde_json::json!({"ok": true, "data": state.song})
        }
        Commands::GetTrack { index } => {
            let seed = args.seed.unwrap_or(42);
            let state = AppState::new(seed);
            if *index >= state.song.tracks.len() {
                serde_json::json!({"ok": false, "error": format!("Track index {} out of range (0-{})", index, state.song.tracks.len() - 1)})
            } else {
                serde_json::json!({"ok": true, "data": state.song.tracks[*index]})
            }
        }
        Commands::GetPattern { track, part } => {
            let seed = args.seed.unwrap_or(42);
            let state = AppState::new(seed);
            match part.parse::<SongPart>() {
                Err(e) => serde_json::json!({"ok": false, "error": e.to_string()}),
                Ok(song_part) => {
                    if *track >= state.song.tracks.len() {
                        serde_json::json!({"ok": false, "error": format!("Track index {} out of range (0-{})", track, state.song.tracks.len() - 1)})
                    } else {
                        match state.song.tracks[*track].patterns.get(&song_part) {
                            Some(pattern) => serde_json::json!({"ok": true, "data": pattern}),
                            None => {
                                let empty = Pattern::empty(song_part.typical_bars());
                                serde_json::json!({"ok": true, "data": empty})
                            }
                        }
                    }
                }
            }
        }
        Commands::ListProgressions { part } => {
            let seed = args.seed.unwrap_or(42);
            let state = AppState::new(seed);
            match part.parse::<SongPart>() {
                Err(e) => serde_json::json!({"ok": false, "error": e.to_string()}),
                Ok(song_part) => {
                    match state.song.progressions.get(&song_part) {
                        Some(prog) => serde_json::json!({"ok": true, "data": prog}),
                        None => serde_json::json!({"ok": true, "data": []}),
                    }
                }
            }
        }
        Commands::Compose => {
            let seed = args.seed.unwrap_or(42);
            let mut state = AppState::new(seed);
            state.composer.compose(&mut state.song);
            serde_json::json!({"ok": true, "data": state.song})
        }
        Commands::ExportMidi { path, .. } => {
            let seed = args.seed.unwrap_or(42);
            let mut state = AppState::new(seed);
            state.composer.compose(&mut state.song);
            let export_path = Path::new(path);
            match midi_export::export_to_file(&state.song, export_path) {
                Ok(()) => serde_json::json!({
                    "ok": true,
                    "data": {
                        "path": path,
                        "tracks": state.song.tracks.len(),
                        "bars": state.song.total_bars()
                    }
                }),
                Err(e) => serde_json::json!({
                    "ok": false,
                    "error": format!("MIDI export failed: {e}")
                }),
            }
        }
        _ => serde_json::json!({"ok": true, "data": "not yet implemented"}),
    }
}

/// Execute a CLI command and print JSON to stdout.
pub fn execute(command: &Commands, args: &Cli) -> Result<()> {
    let response = build_response(command, args);

    if args.json_pretty {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("{}", serde_json::to_string(&response)?);
    }

    // Non-zero exit for failed compose/export-midi per production quality standards
    if matches!(command, Commands::Compose | Commands::ExportMidi { .. })
        && response.get("ok") == Some(&serde_json::Value::Bool(false))
    {
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a command response for testing.
    fn exec_json(command: &Commands, args: &Cli) -> serde_json::Value {
        build_response(command, args)
    }

    fn default_args() -> Cli {
        Cli {
            command: None,
            headless: false,
            seed: None,
            json_pretty: false,
            quiet: false,
        }
    }

    #[test]
    fn list_scales_returns_all_variants() {
        let resp = exec_json(&Commands::ListScales, &default_args());
        assert_eq!(resp["ok"], true);
        let data = resp["data"].as_array().expect("data should be array");
        assert_eq!(data.len(), 7);
        assert_eq!(data[0], "major");
        assert_eq!(data[1], "natural_minor");
        assert_eq!(data[2], "harmonic_minor");
        assert_eq!(data[3], "dorian");
        assert_eq!(data[4], "mixolydian");
        assert_eq!(data[5], "minor_pentatonic");
        assert_eq!(data[6], "blues");
    }

    #[test]
    fn list_instruments_returns_all_variants() {
        let resp = exec_json(&Commands::ListInstruments, &default_args());
        assert_eq!(resp["ok"], true);
        let data = resp["data"].as_array().expect("data should be array");
        assert_eq!(data.len(), 22);
        assert_eq!(data[0], "acoustic_guitar");
        assert_eq!(data[9], "pad");
        assert_eq!(data[10], "kick");
        assert_eq!(data[21], "rimshot");
    }

    #[test]
    fn list_parts_returns_all_variants() {
        let resp = exec_json(&Commands::ListParts, &default_args());
        assert_eq!(resp["ok"], true);
        let data = resp["data"].as_array().expect("data should be array");
        assert_eq!(data.len(), 6);
        assert_eq!(data[0], "intro");
        assert_eq!(data[1], "verse");
        assert_eq!(data[2], "prechorus");
        assert_eq!(data[3], "chorus");
        assert_eq!(data[4], "bridge");
        assert_eq!(data[5], "outro");
    }

    #[test]
    fn list_strum_patterns_returns_default() {
        let resp = exec_json(&Commands::ListStrumPatterns, &default_args());
        assert_eq!(resp["ok"], true);
        let data = resp["data"].as_array().expect("data should be array");
        assert_eq!(data.len(), 1);
        assert_eq!(data[0]["name"], "Folk Strum");
    }

    #[test]
    fn get_state_returns_valid_state() {
        let resp = exec_json(&Commands::GetState, &default_args());
        assert_eq!(resp["ok"], true);
        let data = &resp["data"];
        assert_eq!(data["seed_counter"], 42);
        assert_eq!(data["is_playing"], false);
        assert_eq!(data["bitwig_connected"], false);
        assert_eq!(data["song"]["title"], "Untitled Folk Song");
        assert_eq!(data["song"]["tempo"], 120.0);
    }

    #[test]
    fn get_state_respects_seed_arg() {
        let mut args = default_args();
        args.seed = Some(999);
        let resp = exec_json(&Commands::GetState, &args);
        assert_eq!(resp["data"]["seed_counter"], 999);
    }

    #[test]
    fn get_song_returns_song_data() {
        let resp = exec_json(&Commands::GetSong, &default_args());
        assert_eq!(resp["ok"], true);
        let data = &resp["data"];
        assert_eq!(data["title"], "Untitled Folk Song");
        assert_eq!(data["tempo"], 120.0);
        assert_eq!(data["time_signature"], serde_json::json!([4, 4]));
        let tracks = data["tracks"].as_array().expect("tracks should be array");
        assert_eq!(tracks.len(), 16);
    }

    #[test]
    fn get_song_structure_matches_default() {
        let resp = exec_json(&Commands::GetSong, &default_args());
        let structure = resp["data"]["structure"]
            .as_array()
            .expect("structure should be array");
        assert_eq!(structure.len(), 10);
        assert_eq!(structure[0], "intro");
        assert_eq!(structure[1], "verse");
        assert_eq!(structure[2], "prechorus");
        assert_eq!(structure[3], "chorus");
    }

    #[test]
    fn all_responses_are_valid_json() {
        let args = default_args();
        let commands: Vec<Commands> = vec![
            Commands::ListScales,
            Commands::ListInstruments,
            Commands::ListParts,
            Commands::ListStrumPatterns,
            Commands::GetState,
            Commands::GetSong,
            Commands::Compose,
        ];
        for cmd in &commands {
            let resp = exec_json(cmd, &args);
            // Verify roundtrip: serialize to string and parse back
            let json_str = serde_json::to_string(&resp).expect("should serialize");
            let _: serde_json::Value =
                serde_json::from_str(&json_str).expect("should parse back");
            assert_eq!(resp["ok"], true);
        }
    }

    #[test]
    fn get_track_returns_track_data() {
        let resp = exec_json(&Commands::GetTrack { index: 0 }, &default_args());
        assert_eq!(resp["ok"], true);
        let data = &resp["data"];
        assert_eq!(data["id"], 0);
        assert_eq!(data["name"], "Kick");
        assert_eq!(data["role"], "drum");
        assert_eq!(data["instrument"], "kick");
    }

    #[test]
    fn get_track_returns_correct_index() {
        let resp = exec_json(&Commands::GetTrack { index: 4 }, &default_args());
        assert_eq!(resp["ok"], true);
        assert_eq!(resp["data"]["id"], 4);
        assert_eq!(resp["data"]["name"], "Acoustic Guitar");
        assert_eq!(resp["data"]["role"], "rhythm");
        assert_eq!(resp["data"]["instrument"], "acoustic_guitar");
        assert_eq!(resp["data"]["voicing"], "poly");
    }

    #[test]
    fn get_track_out_of_range_returns_error() {
        let resp = exec_json(&Commands::GetTrack { index: 99 }, &default_args());
        assert_eq!(resp["ok"], false);
        let err = resp["error"].as_str().expect("error should be string");
        assert!(err.contains("out of range"));
    }

    #[test]
    fn get_pattern_returns_empty_pattern() {
        // Default song has no composed patterns yet, so we get an empty pattern
        let resp = exec_json(
            &Commands::GetPattern { track: 0, part: "verse".to_string() },
            &default_args(),
        );
        assert_eq!(resp["ok"], true);
        let data = &resp["data"];
        assert_eq!(data["bars"], 8); // verse = 8 bars
        assert_eq!(data["events"].as_array().expect("events array").len(), 0);
    }

    #[test]
    fn get_pattern_invalid_part_returns_error() {
        let resp = exec_json(
            &Commands::GetPattern { track: 0, part: "nonexistent".to_string() },
            &default_args(),
        );
        assert_eq!(resp["ok"], false);
        let err = resp["error"].as_str().expect("error should be string");
        assert!(err.contains("unknown song part"));
    }

    #[test]
    fn get_pattern_track_out_of_range_returns_error() {
        let resp = exec_json(
            &Commands::GetPattern { track: 99, part: "chorus".to_string() },
            &default_args(),
        );
        assert_eq!(resp["ok"], false);
        let err = resp["error"].as_str().expect("error should be string");
        assert!(err.contains("out of range"));
    }

    #[test]
    fn list_progressions_returns_verse_progression() {
        let resp = exec_json(
            &Commands::ListProgressions { part: "verse".to_string() },
            &default_args(),
        );
        assert_eq!(resp["ok"], true);
        let data = resp["data"].as_array().expect("data should be array");
        assert_eq!(data.len(), 4);
        assert_eq!(data[0], "I");
        assert_eq!(data[1], "V");
        assert_eq!(data[2], "VI");
        assert_eq!(data[3], "IV");
    }

    #[test]
    fn list_progressions_returns_bridge_progression() {
        let resp = exec_json(
            &Commands::ListProgressions { part: "bridge".to_string() },
            &default_args(),
        );
        assert_eq!(resp["ok"], true);
        let data = resp["data"].as_array().expect("data should be array");
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn list_progressions_invalid_part_returns_error() {
        let resp = exec_json(
            &Commands::ListProgressions { part: "nonexistent".to_string() },
            &default_args(),
        );
        assert_eq!(resp["ok"], false);
        let err = resp["error"].as_str().expect("error should be string");
        assert!(err.contains("unknown song part"));
    }

    #[test]
    fn unimplemented_command_returns_stub() {
        // Compose and ExportMidi are now wired; test with a still-unimplemented command
        let resp = exec_json(&Commands::Undo, &default_args());
        assert_eq!(resp["ok"], true);
        assert_eq!(resp["data"], "not yet implemented");
    }

    // -- Compose tests -------------------------------------------------------

    #[test]
    fn compose_returns_populated_song() {
        let resp = exec_json(&Commands::Compose, &default_args());
        assert_eq!(resp["ok"], true);
        let tracks = resp["data"]["tracks"].as_array().expect("tracks array");
        assert_eq!(tracks.len(), 16);

        // At least some tracks should have populated patterns
        let has_patterns = tracks.iter().any(|t| {
            t.get("patterns")
                .and_then(|p| p.as_object())
                .map(|p| !p.is_empty())
                .unwrap_or(false)
        });
        assert!(has_patterns, "composed song should have populated patterns");
    }

    #[test]
    fn compose_is_deterministic() {
        let mut args = default_args();
        args.seed = Some(42);
        let resp1 = exec_json(&Commands::Compose, &args);
        let resp2 = exec_json(&Commands::Compose, &args);
        assert_eq!(resp1, resp2, "same seed should produce identical output");
    }

    #[test]
    fn compose_different_seeds_differ() {
        let mut args1 = default_args();
        args1.seed = Some(42);
        let mut args2 = default_args();
        args2.seed = Some(99);
        let resp1 = exec_json(&Commands::Compose, &args1);
        let resp2 = exec_json(&Commands::Compose, &args2);
        assert_ne!(resp1, resp2, "different seeds should produce different output");
    }

    #[test]
    fn compose_tracks_have_note_events() {
        let mut args = default_args();
        args.seed = Some(42);
        let resp = exec_json(&Commands::Compose, &args);
        let tracks = resp["data"]["tracks"].as_array().expect("tracks array");

        // Acoustic Guitar (index 4) is active in all parts and should have events
        let guitar = &tracks[4];
        let patterns = guitar["patterns"].as_object().expect("patterns object");
        assert!(!patterns.is_empty(), "guitar should have patterns");

        let has_events = patterns.values().any(|p| {
            p["events"]
                .as_array()
                .map(|e| !e.is_empty())
                .unwrap_or(false)
        });
        assert!(has_events, "guitar patterns should have note events");
    }

    #[test]
    fn compose_respects_seed_arg() {
        let mut args = default_args();
        args.seed = Some(12345);
        let resp = exec_json(&Commands::Compose, &args);
        assert_eq!(resp["ok"], true);

        // Verify composition actually happened (not just default empty song)
        let tracks = resp["data"]["tracks"].as_array().expect("tracks array");
        let total_patterns: usize = tracks.iter().map(|t| {
            t["patterns"].as_object().map(|p| p.len()).unwrap_or(0)
        }).sum();
        assert!(total_patterns > 0, "composed song should have patterns");
    }

    // -- ExportMidi tests ----------------------------------------------------

    #[test]
    fn export_midi_creates_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("copper_hollow_cmd_test.mid");
        let path_str = path.to_string_lossy().to_string();

        let resp = exec_json(
            &Commands::ExportMidi {
                path: path_str.clone(),
                track: None,
                part: None,
            },
            &default_args(),
        );
        assert_eq!(resp["ok"], true);
        assert_eq!(resp["data"]["path"], path_str);
        assert_eq!(resp["data"]["tracks"], 16);
        assert_eq!(resp["data"]["bars"], 64);

        // Verify file exists on disk
        assert!(path.exists(), "MIDI file should exist on disk");

        // Verify file is valid MIDI that can be read back
        let data = std::fs::read(&path).expect("read MIDI file");
        let parsed = midly::Smf::parse(&data).expect("parse MIDI");
        assert_eq!(parsed.tracks.len(), 17); // 1 conductor + 16 instrument

        // Verify at least some tracks have note events
        let total_note_ons: usize = parsed
            .tracks
            .iter()
            .skip(1)
            .map(|t| {
                t.iter()
                    .filter(|ev| {
                        matches!(
                            ev.kind,
                            midly::TrackEventKind::Midi {
                                message: midly::MidiMessage::NoteOn { .. },
                                ..
                            }
                        )
                    })
                    .count()
            })
            .sum();
        assert!(total_note_ons > 0, "exported MIDI should contain notes");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn export_midi_invalid_path_returns_error() {
        let resp = exec_json(
            &Commands::ExportMidi {
                path: "/nonexistent/dir/test.mid".to_string(),
                track: None,
                part: None,
            },
            &default_args(),
        );
        assert_eq!(resp["ok"], false);
        let err = resp["error"].as_str().expect("error should be string");
        assert!(err.contains("MIDI export failed"));
    }

    #[test]
    fn export_midi_deterministic() {
        let dir = std::env::temp_dir();
        let path1 = dir.join("copper_hollow_det_test_1.mid");
        let path2 = dir.join("copper_hollow_det_test_2.mid");

        let mut args = default_args();
        args.seed = Some(42);

        exec_json(
            &Commands::ExportMidi {
                path: path1.to_string_lossy().to_string(),
                track: None,
                part: None,
            },
            &args,
        );
        exec_json(
            &Commands::ExportMidi {
                path: path2.to_string_lossy().to_string(),
                track: None,
                part: None,
            },
            &args,
        );

        let bytes1 = std::fs::read(&path1).expect("read file 1");
        let bytes2 = std::fs::read(&path2).expect("read file 2");
        assert_eq!(bytes1, bytes2, "same seed should produce identical MIDI files");

        std::fs::remove_file(&path1).ok();
        std::fs::remove_file(&path2).ok();
    }
}
