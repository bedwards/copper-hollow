# FolkKomposer — Project Kickoff

## What This Is

FolkKomposer is a Rust application that composes production-ready MIDI arrangements for folk, indie, alt-country, and folktronica music. It has three interfaces:

1. A native GUI (Iced 0.14) for high-level creative control
2. A CLI (single binary, clap subcommands) for full programmatic MIDI control
3. A compiled Java extension (.bwextension) for Bitwig Studio that bridges the app to the DAW via TCP

The user never inputs individual notes through the GUI. They press "next suggestion" and navigate a history of snapshots. The CLI has absolute power over every note, tick, velocity, CC, and pitch bend — designed for Claude Code to drive generatively.

## Critical Design Principles

**The composition engine is the heart.** This is not a random note generator. It must produce tight, musical, genre-appropriate arrangements that sound like a competent folk/indie session musician wrote them. Rhythm must groove. Melodies must have contour and resolve. Bass must walk intelligently. Drums must breathe.

**The GUI must be beautiful.** Iced 0.14, GPU-rendered, dark theme, music-production aesthetic. Think Bitwig/Ableton's UI sensibility — clean, information-dense, no wasted space. Custom canvas widgets for pattern visualization.

**The CLI must be complete.** Every state change possible through the GUI must also be possible through `folkkomposer <subcommand> [args]`. Single invocation, prints JSON to stdout, exits. No daemon. No TCP. Shared state via a lock file or Unix socket to the running GUI process.

**The Bitwig extension must be real Java.** Not JavaScript. Maven project, compiled to .bwextension. Type-safe. Connects to the Rust app over TCP, relays MIDI and transport.

## Technology Stack (Verified March 2026)

| Component | Technology | Version | License |
|-----------|-----------|---------|---------|
| Language | Rust | 1.88+ | — |
| GUI | iced | 0.14.0 | MIT |
| CLI parser | clap | 4.x (latest) | MIT/Apache |
| Async runtime | tokio | 1.50.0 | MIT |
| MIDI files | midly | 0.5.3 | MIT |
| Serialization | serde + serde_json | 1.x | MIT/Apache |
| RNG | rand + rand_chacha | 0.8 / 0.3 | MIT/Apache |
| Logging | tracing + tracing-subscriber | 0.1 / 0.3 | MIT |
| Bitwig ext | Java 12+ | API 18 | — |
| Bitwig build | Maven | 3.x | — |

## File Layout

```
folkkomposer/
├── Cargo.toml
├── CLAUDE.md                    # Instructions for Claude Code
├── src/
│   ├── main.rs                  # Entry: GUI mode or CLI mode via clap
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── theory.rs            # Pitch, scale, chord, interval
│   │   ├── rhythm.rs            # Groove patterns, strum, humanization
│   │   ├── melody.rs            # Contour, targeting, voice leading
│   │   ├── drums.rs             # Per-instrument pattern generation
│   │   ├── bass.rs              # Walking, root-fifth, approach notes
│   │   ├── arrangement.rs       # Song parts, transitions, dynamics
│   │   ├── song.rs              # Song, Track, Pattern, NoteEvent
│   │   └── composer.rs          # Top-level compose orchestrator
│   ├── gui/
│   │   ├── mod.rs
│   │   ├── app.rs               # Iced Application impl
│   │   ├── theme.rs             # Dark theme, colors, fonts
│   │   ├── header.rs            # Transport, scales, structure bar
│   │   ├── tracks.rs            # Track list panel
│   │   ├── pattern_view.rs      # Canvas widget: piano-roll-lite
│   │   └── widgets.rs           # Scale grid, chord display, etc.
│   ├── cli/
│   │   ├── mod.rs
│   │   └── commands.rs          # All clap subcommands
│   ├── bridge/
│   │   ├── mod.rs               # TCP server for Bitwig connection
│   │   └── protocol.rs          # JSON message types
│   ├── midi_export.rs           # .mid file writing via midly
│   └── state.rs                 # SharedState, Snapshot, undo/redo
├── bitwig-extension/
│   ├── pom.xml                  # Maven build
│   └── src/main/java/com/folkkomposer/
│       ├── FolkKomposerExtensionDefinition.java
│       └── FolkKomposerExtension.java
└── docs/                        # These spec documents
```

## Build & Run

```bash
# GUI mode (default)
cargo run --release

# CLI mode
cargo run --release -- get-state
cargo run --release -- randomize --track 4 --part chorus
cargo run --release -- export-midi /tmp/song.mid

# Bitwig extension
cd bitwig-extension && mvn install
cp target/*.bwextension ~/Documents/Bitwig\ Studio/Extensions/
```

## Document Index

Read these specs in order when building:

1. `architecture/DATA_MODEL.md` — Song, Track, Pattern, NoteEvent structs
2. `engine/THEORY.md` — Music theory: scales, chords, intervals
3. `engine/RHYTHM.md` — Groove, strum patterns, humanization, swing
4. `engine/MELODY.md` — Melodic contour, targeting, voice leading
5. `engine/DRUMS.md` — Per-instrument drum patterns by song part
6. `engine/BASS.md` — Bass line generation strategies
7. `engine/ARRANGEMENT.md` — Song structure, transitions, dynamics
8. `gui/GUI_SPEC.md` — Complete Iced GUI specification
9. `cli/CLI_SPEC.md` — Every CLI command with args and output
10. `bitwig/BITWIG_EXTENSION.md` — Java extension architecture
11. `architecture/BRIDGE_PROTOCOL.md` — TCP JSON protocol
12. `reference/DEFAULTS.md` — All default values and presets
