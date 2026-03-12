# CLI Specification

## Architecture

Single binary. `copper-hollow` with no args launches the GUI. With a subcommand, it operates as CLI.

CLI communicates with the running GUI process over a Unix domain socket at `/tmp/copper-hollow.sock`. If no GUI process is running, CLI operates headless — creates an ephemeral AppState, performs the command, outputs result, exits. Headless mode is useful for batch MIDI generation.

All CLI output is JSON to stdout. Errors go to stderr. Exit code 0 on success, 1 on error.

## Global Options

```
copper-hollow [OPTIONS] <COMMAND>

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
copper-hollow get-state

# Specific track
copper-hollow get-track <INDEX>
copper-hollow get-track 4

# Pattern for a track/part
copper-hollow get-pattern <TRACK_INDEX> <PART>
copper-hollow get-pattern 6 chorus

# Current song info
copper-hollow get-song

# Available options
copper-hollow list-scales
copper-hollow list-instruments
copper-hollow list-progressions <PART>
copper-hollow list-strum-patterns
copper-hollow list-parts
```

### Composition

```bash
# Randomize everything with new seed
copper-hollow randomize
copper-hollow randomize --seed 12345

# Randomize specific track
copper-hollow randomize --track 4
copper-hollow randomize --track 4 --part chorus

# Recompose all (same settings, same seed = same output)
copper-hollow compose

# Next suggestion (increment seed, recompose, save snapshot)
copper-hollow next
copper-hollow next --track 4
copper-hollow next --track 4 --part verse
```

### Song Settings

```bash
# Tempo
copper-hollow set-tempo 105

# Rhythm scale
copper-hollow set-rhythm-scale Bb major
copper-hollow set-rhythm-scale G dorian

# Lead scale
copper-hollow set-lead-scale G minor-pentatonic
copper-hollow set-lead-scale G minor-pentatonic --passing-tones 6

# Swing
copper-hollow set-swing 0.3

# Song title
copper-hollow set-title "Dusty Roads"

# Song structure
copper-hollow set-structure intro verse prechorus chorus verse prechorus chorus bridge chorus outro

# Strum pattern
copper-hollow set-strum-pattern "folk strum"
copper-hollow set-strum-pattern "travis pick"
```

### Chord Progressions

```bash
# Set progression for a part
copper-hollow set-progression chorus I V vi IV
copper-hollow set-progression verse I IV V I
copper-hollow set-progression bridge vi V IV

# Degrees are case-insensitive roman numerals: I II III IV V VI VII
```

### Track Settings

```bash
# Set track properties
copper-hollow set-track 4 --name "Acoustic Rhythm" --role rhythm --instrument acoustic-guitar --voicing poly

# Individual properties
copper-hollow set-track 4 --name "Lead Pedal Steel"
copper-hollow set-track 4 --role lead
copper-hollow set-track 4 --instrument pedal-steel
copper-hollow set-track 4 --voicing mono

# Mute/solo
copper-hollow mute 4
copper-hollow unmute 4
copper-hollow solo 4
copper-hollow unsolo 4

# Active in part
copper-hollow activate 4 chorus
copper-hollow deactivate 4 intro
```

### Direct MIDI Editing

The most powerful CLI capability — direct note manipulation.

```bash
# Set entire pattern (replaces all notes)
copper-hollow set-pattern 6 chorus --events '[
  {"tick": 0, "note": 43, "velocity": 100, "duration": 480},
  {"tick": 480, "note": 45, "velocity": 85, "duration": 480},
  {"tick": 960, "note": 47, "velocity": 90, "duration": 480},
  {"tick": 1440, "note": 43, "velocity": 80, "duration": 480}
]'

# Add a single note to existing pattern
copper-hollow add-note 6 chorus --tick 960 --note 48 --velocity 90 --duration 240

# Remove notes matching criteria
copper-hollow remove-notes 6 chorus --tick-range 0 960
copper-hollow remove-notes 6 chorus --note-range 36 48
copper-hollow remove-notes 6 chorus --tick 480

# Modify notes matching criteria
copper-hollow modify-notes 6 chorus --tick-range 0 1920 --velocity-add 10
copper-hollow modify-notes 6 chorus --tick-range 0 1920 --transpose 2
copper-hollow modify-notes 6 chorus --tick-range 0 1920 --shift-ticks -20
copper-hollow modify-notes 6 chorus --note 60 --set-velocity 100

# Set CC automation
copper-hollow set-cc 8 chorus --cc 1 --events '[
  {"tick": 0, "value": 0},
  {"tick": 960, "value": 64},
  {"tick": 1920, "value": 127}
]'

# Set pitch bend
copper-hollow set-pitchbend 8 chorus --events '[
  {"tick": 0, "value": 8192},
  {"tick": 240, "value": 7000},
  {"tick": 480, "value": 8192}
]'

# Copy pattern from one part to another
copper-hollow copy-pattern 4 verse chorus

# Clear a pattern
copper-hollow clear-pattern 4 bridge
```

### Scale Degree Toggling

```bash
# Toggle a scale degree on/off (0-indexed, 0=root always on)
copper-hollow toggle-degree rhythm 3    # toggle 4th degree of rhythm scale
copper-hollow toggle-degree lead 2      # toggle 3rd degree of lead scale

# Add/remove passing tones (semitone offset from root)
copper-hollow add-passing-tone lead 6   # add tritone passing tone
copper-hollow remove-passing-tone lead 6
```

### History

```bash
copper-hollow undo
copper-hollow redo
copper-hollow history              # list all snapshots
copper-hollow goto-snapshot 5      # jump to snapshot index
```

### Export

```bash
# Full song
copper-hollow export-midi output.mid

# Single track, single part
copper-hollow export-midi output.mid --track 4 --part chorus

# All tracks, single part
copper-hollow export-midi output.mid --part chorus
```

### Transport (requires Bitwig connection)

```bash
copper-hollow play
copper-hollow stop
copper-hollow transport-status
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
#[command(name = "copper-hollow", about = "Folk/indie composition engine")]
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
