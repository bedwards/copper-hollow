# CLI Specification

## Architecture

Single binary. `folkkomposer` with no args launches the GUI. With a subcommand, it operates as CLI.

CLI communicates with the running GUI process over a Unix domain socket at `/tmp/folkkomposer.sock`. If no GUI process is running, CLI operates headless — creates an ephemeral AppState, performs the command, outputs result, exits. Headless mode is useful for batch MIDI generation.

All CLI output is JSON to stdout. Errors go to stderr. Exit code 0 on success, 1 on error.

## Global Options

```
folkkomposer [OPTIONS] <COMMAND>

Options:
  --headless        Force headless mode (don't connect to GUI)
  --seed <N>        Set RNG seed (default: random)
  --json-pretty     Pretty-print JSON output
  -q, --quiet       Suppress non-essential output
```

## Commands

### State Inspection

```bash
# Full state dump
folkkomposer get-state

# Specific track
folkkomposer get-track <INDEX>
folkkomposer get-track 4

# Pattern for a track/part
folkkomposer get-pattern <TRACK_INDEX> <PART>
folkkomposer get-pattern 6 chorus

# Current song info
folkkomposer get-song

# Available options
folkkomposer list-scales
folkkomposer list-instruments
folkkomposer list-progressions <PART>
folkkomposer list-strum-patterns
folkkomposer list-parts
```

### Composition

```bash
# Randomize everything with new seed
folkkomposer randomize
folkkomposer randomize --seed 12345

# Randomize specific track
folkkomposer randomize --track 4
folkkomposer randomize --track 4 --part chorus

# Recompose all (same settings, same seed = same output)
folkkomposer compose

# Next suggestion (increment seed, recompose, save snapshot)
folkkomposer next
folkkomposer next --track 4
folkkomposer next --track 4 --part verse
```

### Song Settings

```bash
# Tempo
folkkomposer set-tempo 105

# Rhythm scale
folkkomposer set-rhythm-scale Bb major
folkkomposer set-rhythm-scale G dorian

# Lead scale
folkkomposer set-lead-scale G minor-pentatonic
folkkomposer set-lead-scale G minor-pentatonic --passing-tones 6

# Swing
folkkomposer set-swing 0.3

# Song title
folkkomposer set-title "Dusty Roads"

# Song structure
folkkomposer set-structure intro verse prechorus chorus verse prechorus chorus bridge chorus outro

# Strum pattern
folkkomposer set-strum-pattern "folk strum"
folkkomposer set-strum-pattern "travis pick"
```

### Chord Progressions

```bash
# Set progression for a part
folkkomposer set-progression chorus I V vi IV
folkkomposer set-progression verse I IV V I
folkkomposer set-progression bridge vi V IV

# Degrees are case-insensitive roman numerals: I II III IV V VI VII
```

### Track Settings

```bash
# Set track properties
folkkomposer set-track 4 --name "Acoustic Rhythm" --role rhythm --instrument acoustic-guitar --voicing poly

# Individual properties
folkkomposer set-track 4 --name "Lead Pedal Steel"
folkkomposer set-track 4 --role lead
folkkomposer set-track 4 --instrument pedal-steel
folkkomposer set-track 4 --voicing mono

# Mute/solo
folkkomposer mute 4
folkkomposer unmute 4
folkkomposer solo 4
folkkomposer unsolo 4

# Active in part
folkkomposer activate 4 chorus
folkkomposer deactivate 4 intro
```

### Direct MIDI Editing

The most powerful CLI capability — direct note manipulation.

```bash
# Set entire pattern (replaces all notes)
folkkomposer set-pattern 6 chorus --events '[
  {"tick": 0, "note": 43, "velocity": 100, "duration": 480},
  {"tick": 480, "note": 45, "velocity": 85, "duration": 480},
  {"tick": 960, "note": 47, "velocity": 90, "duration": 480},
  {"tick": 1440, "note": 43, "velocity": 80, "duration": 480}
]'

# Add a single note to existing pattern
folkkomposer add-note 6 chorus --tick 960 --note 48 --velocity 90 --duration 240

# Remove notes matching criteria
folkkomposer remove-notes 6 chorus --tick-range 0 960
folkkomposer remove-notes 6 chorus --note-range 36 48
folkkomposer remove-notes 6 chorus --tick 480

# Modify notes matching criteria
folkkomposer modify-notes 6 chorus --tick-range 0 1920 --velocity-add 10
folkkomposer modify-notes 6 chorus --tick-range 0 1920 --transpose 2
folkkomposer modify-notes 6 chorus --tick-range 0 1920 --shift-ticks -20
folkkomposer modify-notes 6 chorus --note 60 --set-velocity 100

# Set CC automation
folkkomposer set-cc 8 chorus --cc 1 --events '[
  {"tick": 0, "value": 0},
  {"tick": 960, "value": 64},
  {"tick": 1920, "value": 127}
]'

# Set pitch bend
folkkomposer set-pitchbend 8 chorus --events '[
  {"tick": 0, "value": 8192},
  {"tick": 240, "value": 7000},
  {"tick": 480, "value": 8192}
]'

# Copy pattern from one part to another
folkkomposer copy-pattern 4 verse chorus

# Clear a pattern
folkkomposer clear-pattern 4 bridge
```

### Scale Degree Toggling

```bash
# Toggle a scale degree on/off (0-indexed, 0=root always on)
folkkomposer toggle-degree rhythm 3    # toggle 4th degree of rhythm scale
folkkomposer toggle-degree lead 2      # toggle 3rd degree of lead scale

# Add/remove passing tones (semitone offset from root)
folkkomposer add-passing-tone lead 6   # add tritone passing tone
folkkomposer remove-passing-tone lead 6
```

### History

```bash
folkkomposer undo
folkkomposer redo
folkkomposer history              # list all snapshots
folkkomposer goto-snapshot 5      # jump to snapshot index
```

### Export

```bash
# Full song
folkkomposer export-midi output.mid

# Single track, single part
folkkomposer export-midi output.mid --track 4 --part chorus

# All tracks, single part
folkkomposer export-midi output.mid --part chorus
```

### Transport (requires Bitwig connection)

```bash
folkkomposer play
folkkomposer stop
folkkomposer transport-status
```

## Output Format

All commands return JSON:

```json
// Success
{"ok": true, "data": { ... }}

// Success with no data
{"ok": true}

// Error
{"ok": false, "error": "Track index out of range"}
```

### get-state output
```json
{
  "ok": true,
  "data": {
    "song": {
      "title": "Untitled Folk Song",
      "tempo": 120.0,
      "swing": 0.0,
      "rhythm_scale": {"root": "Bb", "type": "major"},
      "lead_scale": {"root": "G", "type": "minor_pentatonic", "passing_tones": [6]},
      "structure": ["intro", "verse", "prechorus", "chorus", ...],
      "progressions": {
        "verse": ["I", "V", "vi", "IV"],
        "chorus": ["I", "V", "vi", "IV"]
      },
      "strum_pattern": "folk strum",
      "tracks": [
        {"id": 0, "name": "Kick", "role": "drum", "instrument": "kick", "voicing": "mono", "muted": false, "solo": false},
        ...
      ]
    },
    "transport": {"playing": false, "tempo": 120.0, "beat": 0.0, "bar": 0},
    "history_index": 2,
    "history_length": 3,
    "bitwig_connected": false,
    "seed": 42
  }
}
```

### get-pattern output
```json
{
  "ok": true,
  "data": {
    "bars": 8,
    "length_ticks": 15360,
    "events": [
      {"tick": 0, "note": 36, "velocity": 100, "duration": 120, "channel": 0},
      {"tick": 960, "note": 36, "velocity": 90, "duration": 120, "channel": 0}
    ],
    "cc_events": []
  }
}
```

## Clap Implementation

Use `clap` derive API with subcommands:

```rust
#[derive(Parser)]
#[command(name = "folkkomposer", about = "Folk/indie composition engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    #[arg(long)]
    headless: bool,
    
    #[arg(long)]
    seed: Option<u64>,
    
    #[arg(long)]
    json_pretty: bool,
}

#[derive(Subcommand)]
enum Commands {
    GetState,
    GetTrack { index: usize },
    GetPattern { track: usize, part: String },
    Randomize { #[arg(long)] track: Option<usize>, #[arg(long)] part: Option<String>, #[arg(long)] seed: Option<u64> },
    // ... etc
}
```

When command is `None`, launch GUI. Otherwise, execute CLI.
