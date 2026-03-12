# Rhythm Engine

## Philosophy

Rhythm is feel. A perfectly quantized pattern sounds dead. Every note in Copper Hollow passes through humanization. The engine thinks in grooves, not grids.

## Strum Patterns

Strum patterns define rhythm guitar articulation. Each is a sequence of StrumHit values over N beats.

### Preset Patterns

**Folk Strum** (default) — D . D U . U D U
```
Beat:    1   .   2   .   3   .   4   .
Hit:     D       D   U       U   D   U
Tick:    0       480 720     1200 1440 1680
Vel:     1.0     0.8 0.6     0.6 0.85 0.6
Stagg:   12      10  6       6   10   6
Dir:     Down    Down Up     Up  Down Up
```

**Travis Pick** — alternating bass + finger pattern
```
Beat:    1   .   2   .   3   .   4   .
Hit:     B   H   M   H   B   H   M   H
Tick:    0   240 480 720 960 1200 1440 1680
Vel:     0.9 0.5 0.7 0.5 0.85 0.5 0.7 0.5
```
B = bass note (lowest chord tone), M = mid voices, H = high voices. Travis pick uses Mono-style voice selection within a Poly context — each hit targets a specific register of the chord.

**Driving Eighths** — steady down strokes
```
All 8th note positions, Down, vel alternating 1.0/0.65/0.85/0.65...
```

**Boom-Chick** — country two-step
```
Beat:    1       2       3       4
Hit:     Bass    Chord   Bass    Chord
Tick:    0       480     960     1440
Vel:     1.0     0.7     0.85    0.7
```
Bass = root note only (mono). Chord = upper voices (poly, no root).

**16th Strum** — folktronica, faster indie
```
16th note positions, alternating D/U, accent pattern: strong . med . strong . weak .
```

**Muted Strum** — with ghost strums
```
Same as Folk Strum but Ghost hits on beats 3 and the "." of 3
Ghost hits: very low velocity (0.2), muted (short duration ~60 ticks)
```

### Strum Voicing

When a strum hit fires on a chord:

**Down strum:** Notes sound from lowest to highest with stagger delay (8-15ms between notes). First note gets full velocity, subsequent notes decrease slightly (2-4 per note).

**Up strum:** Notes sound from highest to lowest with stagger. Last note in sequence gets emphasis.

**Ghost:** Same as muted — very short duration (30-60 ticks), low velocity (15-30), creates percussive texture.

**Mute:** Duration capped at 60 ticks, velocity at 40-60. Pitched but choked.

## Swing

Swing is applied globally. It shifts every even-numbered 8th note forward in time.

```
swing = 0.0: straight (8th notes at 0, 240, 480, 720, ...)
swing = 0.5: medium swing (even 8ths shifted to ~300)  
swing = 1.0: full triplet swing (even 8ths at 320, creating 2:1 ratio)

Formula: if on even 8th, new_tick = original_tick + (swing * 80)
```

Swing applies to all tracks uniformly but drums can have independent swing per instrument (hi-hat swings, kick stays straight).

## Humanization

Every note event passes through humanization before final output.

### Timing

Each role has its own std_dev and max offset (see DEFAULTS.md for values):

| Role | std_dev (ticks) | max offset |
|------|-----------------|------------|
| Drums | 5 | ±10 |
| Bass | 6 | ±12 |
| Rhythm | 8 | ±15 |
| Melody | 10 | ±18 |
| Pads | 4 | ±8 |

```
offset = random_gauss(mean=0, std_dev=<role_std_dev>) ticks
final_offset = clamp(offset, -max_offset, +max_offset)
```

Downbeats (tick % 1920 == 0) get tighter humanization (÷2).

### Velocity
```
base_variation = random_gauss(mean=0, std_dev=6)
accent_curve: beat 1 = +10, beat 3 = +5, offbeats = -5
part_dynamics: intro=0.55, verse=0.70, prechorus=0.82, chorus=1.0, bridge=0.65, outro=0.50
final_velocity = clamp(base_vel + base_variation + accent_curve) × part_dynamics
```

### Duration
```
legato_factor: pads=0.98, rhythm=0.85, bass=0.90, melody=0.88, drums=0.3
final_duration = nominal_duration × legato_factor + random(-10, +10)
```

## Groove Templates

Beyond swing, grooves shift specific subdivisions to create pocket feel.

**Laid-back:** Everything 5-10 ticks late. Creates relaxed, behind-the-beat feel.

**Pushing:** Everything 3-7 ticks early. Creates urgency.

**Hip-hop pocket:** Kick slightly early, snare slightly late, hats straight. (Not primary genre but available for folktronica.)

## Rhythm Generation for Poly Tracks

For a rhythm guitar playing the chord progression:

1. Look up the current chord for this bar
2. Voice the chord in the instrument's range (see voicing rules in THEORY.md)
3. For each strum hit in the pattern:
   - Apply velocity factor
   - Apply stagger between notes
   - Apply humanization
   - Create NoteEvents

For bars where the chord changes mid-bar (if progression has more chords than bars), split the bar at the chord boundary.

## Rhythm Generation for Mono Tracks

Mono rhythm = arpeggio or bass-note pattern derived from the chord.

**Arpeggio mode:** Cycle through chord tones in order (ascending, descending, or alternating). Each strum hit picks the next tone.

**Bass-note mode:** Alternate between root and fifth, with occasional third. Think boom-chick pattern where every hit is a single note.
