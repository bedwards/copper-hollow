# CLAUDE.md — Instructions for Claude Code

## Project: FolkKomposer

You are building a Rust application that composes MIDI music in folk/indie/alt-country styles. Read the docs/ folder specs before writing code.

## Build

```bash
cargo build --release
cargo test
cargo clippy -- -D warnings
```

## Architecture Rules

- State is `Arc<Mutex<AppState>>` shared between GUI thread and tokio async tasks (bridge, IPC)
- GUI runs on main thread via iced. Tokio runtime runs in a spawned std::thread
- CLI mode: binary detects subcommand via clap, connects to running GUI process over Unix socket at `/tmp/folkkomposer.sock`, sends JSON command, prints JSON response, exits
- If no GUI process is running, CLI operates in headless mode on an ephemeral AppState
- The Bitwig bridge is a TCP server on 127.0.0.1:9876 inside the tokio runtime
- All composition is deterministic given a seed. Same seed + same settings = same output

## Code Style

- No unwrap() in production paths. Use anyhow::Result or explicit error handling
- Prefer strong typing over stringly-typed interfaces. Enums for roles, instruments, parts
- Keep engine/ pure — no IO, no async, no GUI types. Only data in, data out
- serde Serialize/Deserialize on all data types for CLI JSON interchange
- 480 ticks per beat. All rhythmic values in ticks. 4/4 time assumed unless stated

## Composition Quality Bar

The engine must produce MIDI that sounds musical when loaded into a DAW with appropriate instruments. Specifically:

- Rhythm guitar: strum patterns with proper voicing spread, ghost strums, dynamics
- Bass: walking lines that target chord tones on strong beats, approach notes, octave variation  
- Drums: genre-appropriate patterns (NOT programmatic 4-on-floor). Kick/snare interplay, hat dynamics, ghost notes
- Melody: contour with tension/release, targeting chord tones on downbeats, step motion predominant with occasional leaps, rests for breathing
- Counter-melody: harmonically related but rhythmically independent from lead
- Pads: voice-led sustained chords, not just block triads
- ALL tracks: humanized timing (±5-15 ticks), velocity variation, per-part dynamics scaling

## Testing

Write unit tests for:
- Scale construction and pitch class math
- Diatonic chord derivation
- Pattern generation determinism (same seed = same output)
- CLI command parsing
- MIDI file export round-trip

## File Naming

- Rust modules: snake_case
- Java classes: PascalCase
- MIDI exports: `{song_title}_{timestamp}.mid`
