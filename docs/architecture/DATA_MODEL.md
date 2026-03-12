# Data Model

## Core Types

### PitchClass
Enum: C, Cs, D, Ds, E, F, Fs, G, Gs, A, As, B

Methods: `from_midi(u8)`, `to_semitone() -> u8`, `transpose(semitones: i8) -> PitchClass`

Display: C, C#, D, Eb, E, F, F#, G, Ab, A, Bb, B (use flats for Bb, Eb, Ab, Db, Gb)

### ScaleType
Enum with intervals (semitones from root):

| ScaleType | Intervals |
|-----------|-----------|
| Major | 0 2 4 5 7 9 11 |
| NaturalMinor | 0 2 3 5 7 8 10 |
| HarmonicMinor | 0 2 3 5 7 8 11 |
| Dorian | 0 2 3 5 7 9 10 |
| Mixolydian | 0 2 4 5 7 9 10 |
| MinorPentatonic | 0 3 5 7 10 |
| Blues | 0 3 5 6 7 10 |

### Scale
```
root: PitchClass
scale_type: ScaleType
passing_tones: Vec<u8>          // semitone offsets from root, togglable
enabled_degrees: Vec<bool>      // per-interval toggle, index 0 (root) always true
```

### ChordQuality
Major, Minor, Diminished, Augmented, Sus2, Sus4, Major7, Minor7, Dominant7, Add9

Each has `intervals() -> &[u8]` returning semitone offsets.

### ChordDegree
I, II, III, IV, V, VI, VII — maps to index 0-6 into diatonic chord array.

### Chord
```
root: PitchClass
quality: ChordQuality
degree: ChordDegree
inversion: u8               // 0 = root, 1 = first, 2 = second
```

### SongPart
Enum: Intro, Verse, PreChorus, Chorus, Bridge, Outro

Each has `typical_bars() -> u32`: Intro=4, Verse=8, PreChorus=4, Chorus=8, Bridge=8, Outro=4

### InstrumentType
Two categories:

**Melodic:** AcousticGuitar, ElectricGuitar, ElectricBass, AcousticBass, PedalSteel, Mandolin, Banjo, HammondOrgan, Piano, Pad

Each has `midi_range() -> (u8, u8)` for comfortable playing range.

**Percussion:** Kick, Snare, HiHat, OpenHiHat, Clap, Tambourine, Cowbell, Shaker, RideCymbal, CrashCymbal, Toms, Rimshot

All percussion sends fixed MIDI note 36 (C1). One instrument per track.

### TrackRole
Enum: Rhythm, LeadMelody, CounterMelody, Bass, Drum, PadSustain

### Voicing
Enum: Poly, Mono

Mono means strictly one note sounding at any given tick. Rhythm tracks in Mono mode produce arpeggios or bass-note patterns. Lead/Counter in Mono is single-note melody.

### NoteEvent
```
tick: u32           // absolute position from pattern start, 480 ticks/beat
note: u8            // MIDI note 0-127
velocity: u8        // 0-127
duration: u32       // in ticks
channel: u8         // 0-15, matches track.id
```

### CcEvent
```
tick: u32
cc: u8              // CC number, or 255 for pitch bend
value: u16          // 0-127 for CC, 0-16383 for pitch bend (8192 = center)
channel: u8
```

### Pattern
```
events: Vec<NoteEvent>
cc_events: Vec<CcEvent>
length_ticks: u32           // total length
bars: u32                   // bar count this pattern spans
```

### Track
```
id: u8                      // 0-15, doubles as MIDI channel
name: String
role: TrackRole
instrument: InstrumentType
voicing: Voicing
muted: bool
solo: bool
patterns: HashMap<SongPart, Pattern>
automation: HashMap<SongPart, Vec<CcEvent>>
active_parts: HashMap<SongPart, bool>
```

### StrumPattern
```
name: String
hits: Vec<StrumHit>
beats: u32                  // pattern length in beats
```

### StrumHit
```
tick_offset: u32            // position within pattern
direction: StrumDirection   // Down, Up, Mute, Ghost
velocity_factor: f32        // 0.0–1.0 multiplier
stagger_ms: f32             // chord spread time (0 = simultaneous)
```

StrumDirection enum: Down, Up, Mute, Ghost

### Song
```
title: String
tempo: f64                  // BPM
time_signature: (u8, u8)
rhythm_scale: Scale
lead_scale: Scale
tracks: Vec<Track>          // exactly 16 (channels 0-15)
structure: Vec<SongPart>    // ordered, can repeat (e.g. Verse appears twice)
progressions: HashMap<SongPart, Vec<ChordDegree>>
strum_pattern: StrumPattern
swing: f32                  // 0.0 = straight, 1.0 = full triplet swing
```

### Snapshot
```
song: Song
seed: u64
label: String
timestamp: u64              // unix millis
```

### AppState
```
song: Song
composer: Composer
history: Vec<Snapshot>
history_index: usize
is_playing: bool
tempo: f64
beat_position: f64
bar_position: u32
bitwig_connected: bool
selected_track: usize
selected_part_index: usize
seed_counter: u64
```

## Serialization

All types derive `Serialize, Deserialize`. JSON is the interchange format for CLI and bridge protocol. Enums serialize as lowercase strings: `"verse"`, `"kick"`, `"rhythm"`, `"mono"`.

## MIDI Constants

- Ticks per beat: 480
- Ticks per bar (4/4): 1920
- Middle C (C3 in Bitwig): MIDI 60
- Drum note for single-instrument tracks: MIDI 36 (C1)
- Pitch bend center: 8192
