# Defaults & Presets

## Song Defaults

| Setting | Default |
|---------|---------|
| Title | "Untitled Folk Song" |
| Tempo | 120 BPM |
| Time Signature | 4/4 |
| Swing | 0.0 (straight) |
| Rhythm Scale | Bb Major |
| Lead Scale | G Minor Pentatonic, passing tone at 6 semitones (C#/Db) |

## Default Song Structure

Intro → Verse → PreChorus → Chorus → Verse → PreChorus → Chorus → Bridge → Chorus → Outro

Total: 60 bars (4+8+4+8+8+4+8+8+8+4, second verse/prechorus/chorus same bar counts)

## Default Chord Progressions

| Part | Progression | In Bb Major |
|------|-------------|-------------|
| Intro | I | Bb |
| Verse | I V vi IV | Bb F Gm Eb |
| PreChorus | IV V | Eb F |
| Chorus | I V vi IV | Bb F Gm Eb |
| Bridge | vi V IV | Gm F Eb |
| Outro | IV I | Eb Bb |

## Default Track Layout

| Ch | Name | Role | Instrument | Voicing | Active: I V P C B O |
|----|------|------|------------|---------|---------------------|
| 1 | Kick | Drum | Kick | - | ○●●●●○ |
| 2 | Snare | Drum | Snare | - | ○●●●●○ |
| 3 | Hi-Hat | Drum | HiHat | - | ○●●●●○ |
| 4 | Tambourine | Drum | Tambourine | - | ○○○●○○ |
| 5 | Acoustic Guitar | Rhythm | AcousticGuitar | Poly | ●●●●●● |
| 6 | Electric Guitar | Rhythm | ElectricGuitar | Poly | ○○●●○○ |
| 7 | Electric Bass | Bass | ElectricBass | Mono | ○●●●●○ |
| 8 | Piano | Rhythm | Piano | Poly | ●●●●●● |
| 9 | Pedal Steel | Lead | PedalSteel | Mono | ○○○●●○ |
| 10 | Mandolin | Counter | Mandolin | Mono | ○●●●○○ |
| 11 | Banjo | Rhythm | Banjo | Poly | ○○○●○○ |
| 12 | Hammond Organ | Pad | HammondOrgan | Poly | ○○●●●○ |
| 13 | Pad | Pad | Pad | Poly | ●○●○●● |
| 14 | Lead Melody | Lead | AcousticGuitar | Mono | ○●●●●○ |
| 15 | Counter Melody | Counter | Mandolin | Mono | ○○○●○○ |
| 16 | Shaker | Drum | Shaker | - | ○○○●○○ |

I=Intro, V=Verse, P=PreChorus, C=Chorus, B=Bridge, O=Outro. ●=active, ○=silent.

## Default Strum Pattern

"Folk Strum" — D . D U . U D U

```
Tick    Dir    Vel%    Stagger(ms)
0       Down   100%    12
480     Down   80%     10
720     Up     60%     6
1200    Up     60%     6
1440    Down   85%     10
1680    Up     60%     6
```

## Strum Pattern Presets

| Name | Pattern | Character |
|------|---------|-----------|
| Folk Strum | D.DU.UDU | Classic folk strumming |
| Travis Pick | BHMHBHMH | Fingerpick, alternating bass |
| Driving 8ths | DDDDDDDD | Steady, punk-folk energy |
| Boom-Chick | B.C.B.C. | Country two-step |
| 16th Strum | DUDUDUDUDUDUDUDUDU | Fast indie/folktronica |
| Muted Strum | D.DU.gDg | Folk strum with ghost mutes |

## Instrument MIDI Ranges

| Instrument | Low | High | Sweet Spot |
|-----------|------|------|------------|
| Acoustic Guitar | E2 (40) | G5 (79) | A2-E4 (45-64) |
| Electric Guitar | E2 (40) | C6 (84) | A2-G4 (45-67) |
| Electric Bass | E1 (28) | G3 (55) | A1-D3 (33-50) |
| Acoustic Bass | E1 (28) | D3 (50) | A1-B2 (33-47) |
| Pedal Steel | E2 (40) | G5 (79) | B2-D5 (47-74) |
| Mandolin | G3 (55) | D6 (86) | C4-A5 (60-81) |
| Banjo | C3 (48) | G5 (79) | D3-D5 (50-74) |
| Hammond Organ | C2 (36) | C6 (84) | F2-F5 (41-77) |
| Piano | E1 (28) | C7 (96) | C3-C6 (48-84) |
| Pad | C2 (36) | C6 (84) | F3-F5 (53-77) |

## Dynamics Scaling by Part

| Part | Multiplier |
|------|-----------|
| Intro | 0.55 |
| Verse | 0.70 |
| PreChorus | 0.82 |
| Chorus | 1.00 |
| Bridge | 0.65 |
| Outro | 0.50 |

## Typical Bar Counts

| Part | Bars |
|------|------|
| Intro | 4 |
| Verse | 8 |
| PreChorus | 4 |
| Chorus | 8 |
| Bridge | 8 |
| Outro | 4 |

## Humanization Parameters

| Parameter | Drums | Bass | Rhythm | Melody | Pads |
|-----------|-------|------|--------|--------|------|
| Timing std_dev (ticks) | 5 | 6 | 8 | 10 | 4 |
| Timing max offset | ±10 | ±12 | ±15 | ±18 | ±8 |
| Velocity std_dev | 4 | 5 | 6 | 8 | 3 |
| Legato factor | 0.3 | 0.90 | 0.85 | 0.88 | 0.98 |

## Network Ports

| Service | Port | Protocol |
|---------|------|----------|
| Bitwig Bridge | 9876 | TCP, JSON lines |
| CLI IPC | /tmp/copper-hollow.sock | Unix domain socket, JSON lines |
