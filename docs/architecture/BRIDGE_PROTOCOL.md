# Bridge Protocol

## Transport

TCP over localhost. Copper Hollow Rust app listens on `127.0.0.1:9876`. Bitwig Java extension connects as client.

## Wire Format

Newline-delimited JSON. Each message is a single JSON object followed by `\n`. No framing, no length prefix. UTF-8 encoding.

## Message Direction

**App → Bitwig (commands):** The Rust app sends these to control Bitwig.

**Bitwig → App (events):** The Java extension sends these to report state.

## App → Bitwig Messages

### NoteOn
```json
{"type": "NoteOn", "channel": 0, "note": 60, "velocity": 100}
```
channel: 0-15. note: 0-127. velocity: 1-127 (0 = note off).

### NoteOff
```json
{"type": "NoteOff", "channel": 0, "note": 60}
```

### CC
```json
{"type": "CC", "channel": 0, "cc": 1, "value": 64}
```
cc: 0-127. value: 0-127.

### PitchBend
```json
{"type": "PitchBend", "channel": 0, "value": 8192}
```
value: 0-16383. 8192 = center (no bend).

### Play
```json
{"type": "Play"}
```

### Stop
```json
{"type": "Stop"}
```

### SetTempo
```json
{"type": "SetTempo", "bpm": 120.0}
```

### Panic
```json
{"type": "Panic"}
```
All notes off + all sound off on all 16 channels.

### GetTransport
```json
{"type": "GetTransport"}
```
Request current transport state. Bitwig responds with Transport message.

## Bitwig → App Messages

### Transport
```json
{"type": "Transport", "playing": true, "tempo": 120.0, "position": 16.5}
```
position: in beats from song start. playing: boolean.

### Beat
```json
{"type": "Beat", "position": 16.75}
```
Sent on every position update (high frequency when playing). Use for playhead animation.

### Connected
```json
{"type": "Connected", "api_version": 18}
```
Sent once after TCP connection established.

## Rust Implementation

The bridge is a tokio TCP listener. It accepts one connection at a time (Bitwig is the only client).

```rust
// In tokio runtime:
let listener = TcpListener::bind("127.0.0.1:9876").await?;
loop {
    let (stream, _) = listener.accept().await?;
    // Handle connection: split into reader/writer
    // Reader: parse FromBitwig messages, update AppState
    // Writer: stored in Arc<Mutex<>> for sending ToBitwig messages
}
```

The writer handle is shared with the GUI and CLI so they can trigger MIDI output. When the user presses Play in the GUI, it sends a `Play` message through the bridge writer.

## Real-Time MIDI Playback

When the transport is playing and the Rust app wants to stream a pattern in real time:

1. App knows the current beat position from Bitwig's Transport/Beat messages
2. App has patterns loaded with events at specific ticks
3. App converts ticks to beats: `beat = tick / 480.0`
4. App sends NoteOn/NoteOff at the right time, synchronized to Bitwig's transport

This requires a playback scheduler in the Rust app:
- Track which events have been sent
- On each Beat update, check if any pending events fall between last_beat and current_beat
- Send those events immediately

For initial version: simpler approach — export MIDI file and drag into Bitwig. Real-time streaming is a future enhancement.

## Connection Resilience

Bitwig extension reconnects automatically if the Rust app restarts. The Rust app accepts new connections, replacing the previous writer handle. No manual intervention needed.
