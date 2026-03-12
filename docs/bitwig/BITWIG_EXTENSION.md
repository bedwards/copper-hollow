# Bitwig Java Extension

## Overview

A compiled Java extension (.bwextension) that runs inside Bitwig Studio. It connects to the Copper Hollow Rust app via TCP on localhost:9876, relays MIDI events to Bitwig instruments, and sends transport state back to the app.

## Why Java, Not JavaScript

- Type-safe: compile-time checking prevents runtime errors in a live performance context
- Debuggable: attach IntelliJ debugger with breakpoints while Bitwig runs
- Full Java standard library: proper TCP socket handling, threading, JSON parsing
- The extension is a .bwextension file (a renamed .jar) — Maven builds it

## Project Structure

```
bitwig-extension/
├── pom.xml
└── src/main/java/com/copperhollow/
    ├── CopperHollowExtensionDefinition.java
    └── CopperHollowExtension.java
```

## Maven pom.xml

Key dependencies:
- `com.bitwig:extension-api:18` (from Bitwig's Maven repo or local install)
- `com.google.code.gson:gson:2.11.0` (JSON parsing)

Build target: produce a .jar, rename to .bwextension, output to Bitwig Extensions directory.

The Bitwig extension API artifact is available at `~/.m2/repository/com/bitwig/extension-api/` after generating a project from within Bitwig (Help > Documentation > Developer Resources > Controller Scripting Guide > New Project).

## CopperHollowExtensionDefinition.java

```java
// Defines the extension metadata
// UUID: generate a unique one
// Name: "Copper Hollow"
// Vendor: "Copper Hollow"
// MIDI ports: 0 input, 1 output
// Auto-detect: none (virtual controller)
// Required API: 18
```

Key methods to implement:
- `getId()` → UUID string
- `getName()` → "Copper Hollow"
- `getAuthor()` → "Brian Edwards"
- `getVersion()` → "1.0"
- `getRequiredAPIVersion()` → 18
- `getNumMidiInPorts()` → 0
- `getNumMidiOutPorts()` → 1
- `createInstance(ControllerHost)` → new CopperHollowExtension(this, host)

## CopperHollowExtension.java

This is the main extension class. It:

1. Gets the MIDI out port
2. Gets the transport object and registers observers
3. Opens a TCP connection to localhost:9876
4. Reads JSON messages from the socket and executes MIDI commands
5. Sends transport state updates back over the socket

### TCP Connection

Use `host.createRemoteConnection("127.0.0.1", 9876)` which returns a RemoteConnection. This is Bitwig's built-in TCP client API.

Alternatively, since this is Java, use `java.net.Socket` directly in a background thread. The advantage: full control over reconnection logic, buffering, and error handling.

**Recommended approach: java.net.Socket** because:
- Bitwig's RemoteConnection API has limited documentation
- Standard Java sockets are well-understood
- Can implement proper reconnection with backoff
- Can use BufferedReader for line-based JSON protocol

### Connection Lifecycle

```
init():
  Start connection thread
  → Attempt connect to localhost:9876
  → On success: start read loop
  → On failure: schedule retry in 3 seconds

Read loop:
  → Read line from socket (newline-delimited JSON)
  → Parse JSON
  → Handle message on Bitwig's controller thread via host.scheduleTask()

Reconnection:
  → On disconnect, wait 3 seconds, retry
  → Continue retrying indefinitely (app might not be running yet)
```

### Message Handling

Messages from Copper Hollow app (received by extension):

```json
{"type": "NoteOn", "channel": 0, "note": 60, "velocity": 100}
{"type": "NoteOff", "channel": 0, "note": 60}
{"type": "CC", "channel": 0, "cc": 1, "value": 64}
{"type": "PitchBend", "channel": 0, "value": 8192}
{"type": "Play"}
{"type": "Stop"}
{"type": "SetTempo", "bpm": 120.0}
{"type": "Panic"}
{"type": "GetTransport"}
```

Messages to Copper Hollow app (sent by extension):

```json
{"type": "Transport", "playing": true, "tempo": 120.0, "position": 16.5}
{"type": "Beat", "position": 16.5}
{"type": "Connected", "api_version": 18}
```

### MIDI Output

```java
MidiOut midiOut = host.getMidiOutPort(0);

// Note On: status = 0x90 | channel, data1 = note, data2 = velocity
midiOut.sendMidi(0x90 | channel, note, velocity);

// Note Off: status = 0x80 | channel
midiOut.sendMidi(0x80 | channel, note, 0);

// CC: status = 0xB0 | channel
midiOut.sendMidi(0xB0 | channel, ccNumber, value);

// Pitch Bend: status = 0xE0 | channel, LSB, MSB
midiOut.sendMidi(0xE0 | channel, value & 0x7F, (value >> 7) & 0x7F);

// All Notes Off (panic)
for (int ch = 0; ch < 16; ch++) {
    midiOut.sendMidi(0xB0 | ch, 123, 0);  // All Notes Off
    midiOut.sendMidi(0xB0 | ch, 120, 0);  // All Sound Off
}
```

### Transport Observers

```java
Transport transport = host.createTransport();

transport.isPlaying().addValueObserver(playing -> {
    sendToApp(new TransportMessage(playing, currentTempo, currentPosition));
});

transport.tempo().addRawValueObserver(tempo -> {
    currentTempo = tempo;
});

transport.getPosition().addValueObserver(position -> {
    currentPosition = position;
    sendToApp(new BeatMessage(position));
});
```

### Thread Safety

Bitwig's controller thread is single-threaded. All Bitwig API calls (sendMidi, transport control) must happen on this thread. The TCP read loop runs on its own thread. Bridge between them using `host.scheduleTask(Runnable, delay)` with delay=0 for immediate execution on the controller thread.

```java
// In TCP read thread:
Message msg = parseJson(line);

// Schedule execution on Bitwig's controller thread:
host.scheduleTask(() -> {
    handleMessage(msg);
}, 0);
```

## Installation

1. Generate project scaffold from Bitwig: Help > Documentation > Developer Resources > New Project
2. Replace generated Java files with Copper Hollow classes
3. Update pom.xml with correct dependencies and output path
4. `mvn install`
5. Copy .bwextension to `~/Documents/Bitwig Studio/Extensions/`
6. In Bitwig: Settings > Controllers > Add > Copper Hollow
7. Set MIDI Output to the virtual port or instrument bus you want to target

## Debugging

Set environment variable `BITWIG_DEBUG_PORT=5005` (or any free port).
In IntelliJ: create Remote JVM Debug configuration on that port.
Build and load extension, then attach debugger. Breakpoints work — Bitwig freezes on hit (expected).
