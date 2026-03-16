# Copper Hollow

A Rust GUI that composes deterministic MIDI music in folk/indie/alt-country styles and streams it to Bitwig Studio via a custom controller bridge.

## Vision

GUI (iced) on the main thread — tweak scales, chord progressions, song structure, strum patterns, mix 16 tracks. Hit "next" for a new seed-based variation. A TCP bridge on `127.0.0.1:9876` pushes MIDI to a custom Bitwig controller in real-time. CLI connects to the running GUI over a Unix socket for scripting and automation. Same seed + same settings = same output, always.

## Current Status (v0.3.0)

**Working now:**
- Full CLI with 40+ commands (compose, export-midi, randomize, set scales/tempos/progressions, mute/solo/undo/redo)
- Deterministic composition engine: 16 tracks, 64 bars, 10-part song structure, ~14k note events
- MIDI export (Standard MIDI format 1, 480 ticks/beat) — drop into any DAW
- 316 passing tests, CI green on GitHub Actions
- RALPH autonomous dev loop running continuously (research → plan → orchestrate → work → review → monitor)
- Automated PR review via custom Gemini 3.1 Pro GitHub Action

**Tracks:** kick, snare, hi-hat, tambourine, shaker, acoustic guitar, electric guitar, piano, banjo, electric bass, pedal steel, mandolin, hammond organ, pad, lead melody, counter melody

## Next Steps

- **v0.4.0** — iced GUI
- **v0.5.0** — Bitwig TCP bridge + Unix socket IPC between CLI and GUI
