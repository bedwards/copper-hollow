pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "copper-hollow", about = "Folk/indie MIDI composition engine")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Force headless mode (don't connect to GUI).
    #[arg(long)]
    pub headless: bool,

    /// Set RNG seed.
    #[arg(long)]
    pub seed: Option<u64>,

    /// Pretty-print JSON output.
    #[arg(long)]
    pub json_pretty: bool,

    /// Suppress non-essential output.
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    // -- State Inspection --
    /// Full state dump.
    GetState,
    /// Get a specific track by index.
    GetTrack { index: usize },
    /// Get a pattern for a track and part.
    GetPattern { track: usize, part: String },
    /// Get current song info.
    GetSong,
    /// List available scales.
    ListScales,
    /// List available instruments.
    ListInstruments,
    /// List available progressions for a part.
    ListProgressions { part: String },
    /// List strum pattern presets.
    ListStrumPatterns,
    /// List song parts.
    ListParts,

    // -- Composition --
    /// Randomize with a new seed.
    Randomize {
        #[arg(long)]
        track: Option<usize>,
        #[arg(long)]
        part: Option<String>,
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Recompose all (same settings, same seed = same output).
    Compose,
    /// Next suggestion (increment seed, recompose).
    Next {
        #[arg(long)]
        track: Option<usize>,
        #[arg(long)]
        part: Option<String>,
    },

    // -- Song Settings --
    /// Set tempo in BPM.
    SetTempo { bpm: f64 },
    /// Set rhythm scale (root and type).
    SetRhythmScale { root: String, scale_type: String },
    /// Set lead scale.
    SetLeadScale {
        root: String,
        scale_type: String,
        #[arg(long)]
        passing_tones: Option<Vec<u8>>,
    },
    /// Set swing amount (0.0–1.0).
    SetSwing { amount: f32 },
    /// Set song title.
    SetTitle { title: String },
    /// Set song structure (ordered part names).
    SetStructure { parts: Vec<String> },
    /// Set strum pattern by name.
    SetStrumPattern { name: String },

    // -- Chord Progressions --
    /// Set chord progression for a part (Roman numerals).
    SetProgression { part: String, degrees: Vec<String> },

    // -- Track Settings --
    /// Set track properties.
    SetTrack {
        index: usize,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        role: Option<String>,
        #[arg(long)]
        instrument: Option<String>,
        #[arg(long)]
        voicing: Option<String>,
    },
    /// Mute a track.
    Mute { track: usize },
    /// Unmute a track.
    Unmute { track: usize },
    /// Solo a track.
    Solo { track: usize },
    /// Unsolo a track.
    Unsolo { track: usize },
    /// Activate a track in a part.
    Activate { track: usize, part: String },
    /// Deactivate a track in a part.
    Deactivate { track: usize, part: String },

    // -- Direct MIDI Editing --
    /// Set entire pattern (replaces all notes). Events as JSON array string.
    SetPattern {
        track: usize,
        part: String,
        #[arg(long)]
        events: String,
    },
    /// Add a single note to an existing pattern.
    AddNote {
        track: usize,
        part: String,
        #[arg(long)]
        tick: u32,
        #[arg(long)]
        note: u8,
        #[arg(long)]
        velocity: u8,
        #[arg(long)]
        duration: u32,
    },
    /// Remove notes matching criteria.
    RemoveNotes {
        track: usize,
        part: String,
        #[arg(long)]
        tick: Option<u32>,
        #[arg(long)]
        tick_range: Option<Vec<u32>>,
        #[arg(long)]
        note_range: Option<Vec<u8>>,
    },
    /// Modify notes matching criteria.
    ModifyNotes {
        track: usize,
        part: String,
        #[arg(long)]
        tick_range: Option<Vec<u32>>,
        #[arg(long)]
        note: Option<u8>,
        #[arg(long)]
        velocity_add: Option<i8>,
        #[arg(long)]
        transpose: Option<i8>,
        #[arg(long)]
        shift_ticks: Option<i32>,
        #[arg(long)]
        set_velocity: Option<u8>,
    },
    /// Set CC automation. Events as JSON array string.
    SetCc {
        track: usize,
        part: String,
        #[arg(long)]
        cc: u8,
        #[arg(long)]
        events: String,
    },
    /// Set pitch bend. Events as JSON array string.
    SetPitchbend {
        track: usize,
        part: String,
        #[arg(long)]
        events: String,
    },
    /// Copy a pattern from one part to another.
    CopyPattern {
        track: usize,
        from: String,
        to: String,
    },
    /// Clear a pattern.
    ClearPattern { track: usize, part: String },

    // -- Scale Degree Toggling --
    /// Toggle a scale degree on/off.
    ToggleDegree { scale: String, degree: usize },
    /// Add a passing tone (semitone offset from root).
    AddPassingTone { scale: String, semitone: u8 },
    /// Remove a passing tone.
    RemovePassingTone { scale: String, semitone: u8 },

    // -- History --
    /// Undo last change.
    Undo,
    /// Redo last undone change.
    Redo,
    /// List all snapshots.
    History,
    /// Jump to a snapshot index.
    GotoSnapshot { index: usize },

    // -- Export --
    /// Export song to MIDI file.
    ExportMidi {
        path: String,
        #[arg(long)]
        track: Option<usize>,
        #[arg(long)]
        part: Option<String>,
    },

    // -- Transport (requires Bitwig connection) --
    /// Start playback.
    Play,
    /// Stop playback.
    Stop,
    /// Get transport status.
    TransportStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_no_args_is_gui_mode() {
        let cli = Cli::try_parse_from(["copper-hollow"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn parse_get_state() {
        let cli = Cli::try_parse_from(["copper-hollow", "get-state"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::GetState)));
    }

    #[test]
    fn parse_set_tempo() {
        let cli = Cli::try_parse_from(["copper-hollow", "set-tempo", "105"]).unwrap();
        if let Some(Commands::SetTempo { bpm }) = cli.command {
            assert!((bpm - 105.0).abs() < f64::EPSILON);
        } else {
            panic!("expected SetTempo");
        }
    }

    #[test]
    fn parse_randomize_with_flags() {
        let cli = Cli::try_parse_from([
            "copper-hollow",
            "randomize",
            "--track",
            "4",
            "--part",
            "chorus",
            "--seed",
            "12345",
        ])
        .unwrap();
        if let Some(Commands::Randomize { track, part, seed }) = cli.command {
            assert_eq!(track, Some(4));
            assert_eq!(part.as_deref(), Some("chorus"));
            assert_eq!(seed, Some(12345));
        } else {
            panic!("expected Randomize");
        }
    }

    #[test]
    fn parse_export_midi() {
        let cli = Cli::try_parse_from([
            "copper-hollow",
            "export-midi",
            "/tmp/song.mid",
            "--track",
            "4",
            "--part",
            "chorus",
        ])
        .unwrap();
        if let Some(Commands::ExportMidi { path, track, part }) = cli.command {
            assert_eq!(path, "/tmp/song.mid");
            assert_eq!(track, Some(4));
            assert_eq!(part.as_deref(), Some("chorus"));
        } else {
            panic!("expected ExportMidi");
        }
    }

    #[test]
    fn parse_global_options() {
        let cli =
            Cli::try_parse_from(["copper-hollow", "--headless", "--seed", "42", "get-state"])
                .unwrap();
        assert!(cli.headless);
        assert_eq!(cli.seed, Some(42));
    }
}
