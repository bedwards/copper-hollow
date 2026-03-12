//! Integration tests for CLI compose and export-midi commands.
//! These run the actual binary and verify JSON output, exit codes, and MIDI files on disk.

use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_copper-hollow"))
}

#[test]
fn cli_compose_returns_valid_json_with_patterns() {
    let output = binary()
        .args(["--seed", "42", "compose"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success(), "compose should exit 0");

    let stdout = String::from_utf8(output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["ok"], true);

    let tracks = json["data"]["tracks"].as_array().expect("tracks array");
    assert_eq!(tracks.len(), 16);

    // At least some tracks should have patterns with note events
    let has_events = tracks.iter().any(|t| {
        t["patterns"]
            .as_object()
            .map(|p| {
                p.values().any(|pat| {
                    pat["events"]
                        .as_array()
                        .map(|e| !e.is_empty())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });
    assert!(has_events, "composed song should have note events");
}

#[test]
fn cli_compose_is_deterministic() {
    let run = || {
        let output = binary()
            .args(["--seed", "42", "compose"])
            .output()
            .expect("failed to execute");
        assert!(output.status.success());
        String::from_utf8(output.stdout).expect("valid utf8")
    };

    let first = run();
    let second = run();
    assert_eq!(first, second, "same seed must produce identical output");
}

#[test]
fn cli_export_midi_creates_valid_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("copper_hollow_integration_test.mid");
    let path_str = path.to_string_lossy().to_string();

    // Clean up from previous runs
    let _ = std::fs::remove_file(&path);

    let output = binary()
        .args(["--seed", "42", "export-midi", &path_str])
        .output()
        .expect("failed to execute");

    assert!(output.status.success(), "export-midi should exit 0");

    let stdout = String::from_utf8(output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["path"], path_str);
    assert_eq!(json["data"]["tracks"], 16);
    assert_eq!(json["data"]["bars"], 64);

    // Verify file exists and is valid MIDI
    assert!(path.exists(), "MIDI file should exist on disk");
    let midi_bytes = std::fs::read(&path).expect("read MIDI file");
    let smf = midly::Smf::parse(&midi_bytes).expect("valid MIDI");
    assert_eq!(smf.tracks.len(), 17); // 1 conductor + 16 instrument tracks

    // Verify it has actual note content
    let total_notes: usize = smf
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
    assert!(total_notes > 0, "MIDI file should contain notes");

    std::fs::remove_file(&path).ok();
}

#[test]
fn cli_export_midi_invalid_path_exits_nonzero() {
    let output = binary()
        .args(["--seed", "42", "export-midi", "/nonexistent/dir/test.mid"])
        .output()
        .expect("failed to execute");

    assert!(
        !output.status.success(),
        "export-midi to invalid path should exit non-zero"
    );

    let stdout = String::from_utf8(output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["ok"], false);
    assert!(json["error"]
        .as_str()
        .expect("error string")
        .contains("MIDI export failed"));
}
