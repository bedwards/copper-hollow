# Drum Engine

## Philosophy

Folk/indie drums are NOT EDM. They're organic, dynamic, and restrained. A good folk drummer plays FOR the song, not over it. No fills (per user spec). Ghost notes matter more than power. The hi-hat tells the story.

## Per-Instrument Patterns

Every drum instrument has patterns defined per song part. Intensity scales 0.0–1.0 by part:

| Part | Intensity | Character |
|------|-----------|-----------|
| Intro | 0.4 | Sparse, establishing |
| Verse | 0.65 | Steady, supportive |
| PreChorus | 0.8 | Building |
| Chorus | 1.0 | Full, driving |
| Bridge | 0.55 | Pulled back, different |
| Outro | 0.45 | Fading, sparse |

### Kick

**Verse pattern (root):**
```
Beat: 1 . . . 2 . . . 3 . . . 4 . . .
Hit:  X               X
Vel:  100             90
```

**Chorus pattern (add pickup):**
```
Beat: 1 . . . 2 . . . 3 . . . 4 . . .
Hit:  X               X           (x)
Vel:  110             95          65
```
The (x) on the "and of 4" is a ghost kick — 40% probability, adds forward motion.

**Bridge (half-time feel):**
```
Beat: 1 . . . 2 . . . 3 . . . 4 . . .
Hit:  X
Vel:  95
```
Only beat 1. Creates space.

**Variation pool (per bar, 15-25% chance):**
- Add ghost kick on "and of 2" (vel 50-65)
- Displace beat 3 kick to "and of 3" (syncopation)
- Add double kick: two quick hits before beat 1 of next bar (vel 60, 80)

### Snare

**Verse:**
```
Beat: 1 . . . 2 . . . 3 . . . 4 . . .
Hit:          X               X
Vel:          95              100
```
Standard backbeat. The backbone.

**Chorus (add ghost notes):**
```
Beat: 1 . . . 2 . . . 3 . . . 4 . . .
Hit:    (x)   X   (x)   (x)  X
Vel:    35    100  40    35   105
```
Ghost notes (x) at vel 30-45 on 8th note positions. They create shuffle feel.

**Bridge (cross-stick or rimshot):**
Same backbeat pattern but at vel 70 (lighter touch). Or move to beats 3 only for half-time feel.

**Variation pool:**
- Ghost note before backbeat (flam-like, 30 ticks before, vel 35)
- Snare on beat 4 slightly late (+10-15 ticks, creates drag)
- Ghost note run: 3 quick ghost notes (16ths) leading into beat 2 or 4

### Hi-Hat

The hi-hat is the timekeeper and the most expressive drum element.

**Verse (8th notes):**
```
Beat: 1 . 2 . 3 . 4 .
Hit:  X x X x X x X x
Vel:  80 50 70 50 75 50 70 50
```
Downbeats louder, upbeats softer. This velocity pattern is the groove.

**Chorus (push to 16ths or open on 2 & 4):**
Option A — 16th notes:
```
Every 16th position, vel pattern: 80 35 55 35 | 70 35 55 35 | ...
```
Option B — 8ths with open hat on backbeats:
```
Beat: 1 . 2 . 3 . 4 .
Type: C c O c C c O c
Vel:  80 50 90 50 75 50 90 50
```
O = open hi-hat (different instrument or higher velocity + longer duration)

**Bridge (quarter notes or ride switch):**
```
Beat: 1   2   3   4
Hit:  X   X   X   X
Vel:  65  60  65  60
```
Sparse. Creates contrast.

**Variation pool:**
- Open hat on "and of 4" before chorus (classic build signal)
- Occasional 16th note hat fill on beat 4 (vel 40-55, 2-3 quick hits)
- Skip a hit: 15% chance of dropping an upbeat hit (creates breathing)

### Tambourine

Offbeat pattern, lighter than hat. Used in chorus and bridge for texture.

```
Beat: 1 . 2 . 3 . 4 .
Hit:    x   x   x   x
Vel:    55  60  55  60
```
Only upbeats. Velocity moderate. Not present in verse typically.

### Shaker

Steady subdivision, very consistent velocity. Background texture.

```
16th notes at vel 40-50 with ±5 velocity variation.
Active in: chorus, sometimes bridge.
```

### Cowbell / Ride / Crash

**Cowbell:** Sparse accent, beat 1 only, low velocity (60). Maybe every other bar.

**Ride:** Alternative to hi-hat for bridge. Quarter notes, vel 70, with bell hits on beat 1 (vel 85).

**Crash:** Beat 1 of first bar of a new song part only. Vel 100-110. Never mid-section.

## No Fills

Per project spec, drums do NOT include fills. The pattern stays consistent within a part. Variation comes from the variation pool (ghost notes, micro-shifts, velocity changes) — not from tom runs or snare rolls.

## Part Transitions

When moving between song parts, the only drum "transition" allowed is:

- Open hi-hat on the last "and of 4" before the new part
- Crash on beat 1 of the new part
- Slight velocity boost on the first bar of the new part

This is handled at the arrangement level, not in the drum pattern generator.

## Humanization (Drum-Specific)

Drums get tighter humanization than melodic instruments:

```
Kick: ±5 ticks
Snare backbeat: ±4 ticks (very tight)
Snare ghost: ±10 ticks (looser)
Hi-hat: ±6 ticks
Tambourine/shaker: ±8 ticks
```

Velocity humanization: ±4 for kick/snare, ±6 for hats, ±8 for percussion.
