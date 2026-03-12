use anyhow::Result;

use crate::engine::song::{InstrumentType, Pattern, SongPart, StrumPattern};
use crate::engine::theory::ScaleType;
use crate::state::AppState;

use super::{Cli, Commands};

/// Build the JSON response for a CLI command. Shared by `execute` (production)
/// and test helpers to avoid duplicating the match logic.
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
        _ => serde_json::json!({"ok": false, "error": "command not yet implemented"}),
    }
}

/// Execute a CLI command and print JSON to stdout.
/// Returns an error (non-zero exit code) when the response has `ok: false`.
pub fn execute(command: &Commands, args: &Cli) -> Result<()> {
    let response = build_response(command, args);

    if args.json_pretty {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("{}", serde_json::to_string(&response)?);
    }

    if response.get("ok") == Some(&serde_json::Value::Bool(false)) {
        let msg = response["error"]
            .as_str()
            .unwrap_or("unknown error");
        anyhow::bail!("{}", msg);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: execute a command and capture the JSON response.
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
    fn unimplemented_command_returns_error() {
        let resp = exec_json(&Commands::Compose, &default_args());
        assert_eq!(resp["ok"], false);
        let err = resp["error"].as_str().expect("error should be string");
        assert!(err.contains("not yet implemented"));
    }
}
