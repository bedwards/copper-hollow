# Arrangement Engine

## Song Structure

Default structure: Intro → Verse → PreChorus → Chorus → Verse → PreChorus → Chorus → Bridge → Chorus → Outro

This is editable. The structure is an ordered Vec of SongPart values. Parts can repeat.

## Chord Progression Assignment

Each SongPart gets its own chord progression from the rhythm scale's diatonic chords. Progressions should create harmonic contrast between sections.

### Progression Selection Rules

**Verse and Chorus should differ.** If verse is I-V-vi-IV, chorus should NOT also be I-V-vi-IV. Pick a progression that starts on a different degree or has different motion.

**PreChorus should build toward Chorus.** End on V or IV — dominant-function chords that resolve to I at the chorus.

**Bridge should go somewhere new.** Start on vi or ii or IV. Bridge is harmonic adventure.

**Intro can be simple.** I alone, or I-IV, or the first half of the verse progression.

**Outro should resolve.** End on I. Plagal cadence (IV-I) or authentic (V-I) for final resolution.

### Transition Harmony

When two adjacent parts share a chord boundary, the last chord of one part should connect smoothly to the first chord of the next. Good transitions:

```
V → I (authentic cadence into new part)
IV → I (plagal cadence)
vi → I (deceptive to tonic — nice surprise)
ii → V (pre-dominant to dominant, if leading into a V-starting part)
```

Bad transitions to avoid:
```
I → I (no motion, feels static)
vii° → anything (diminished chord is unstable — avoid ending on it)
```

## Part-Specific Track Activation

Not all tracks play in all parts. This is how professional arrangements breathe.

### Default Activation Matrix

| Track | Intro | Verse | PreCh | Chorus | Bridge | Outro |
|-------|-------|-------|-------|--------|--------|-------|
| Kick | ○ | ● | ● | ● | ● | ○ |
| Snare | ○ | ● | ● | ● | ● | ○ |
| Hi-Hat | ○ | ● | ● | ● | ● | ○ |
| Tambourine | ○ | ○ | ○ | ● | ○ | ○ |
| Acoustic Gtr | ● | ● | ● | ● | ● | ● |
| Electric Gtr | ○ | ○ | ● | ● | ○ | ○ |
| Bass | ○ | ● | ● | ● | ● | ○ |
| Piano | ● | ● | ● | ● | ● | ● |
| Pedal Steel | ○ | ○ | ○ | ● | ● | ○ |
| Mandolin | ○ | ● | ● | ● | ○ | ○ |
| Banjo | ○ | ○ | ○ | ● | ○ | ○ |
| Hammond | ○ | ○ | ● | ● | ● | ○ |
| Pad | ● | ○ | ● | ○ | ● | ● |
| Lead Melody | ○ | ● | ● | ● | ● | ○ |
| Counter Mel | ○ | ○ | ○ | ● | ○ | ○ |
| Shaker | ○ | ○ | ○ | ● | ○ | ○ |

● = active, ○ = silent

This matrix is the DEFAULT. Users can toggle any cell. The engine respects it.

## Dynamics Scaling

Every song part has a dynamics multiplier applied to all velocity values:

```
Intro:     0.55
Verse:     0.70
PreChorus: 0.82
Chorus:    1.00
Bridge:    0.65
Outro:     0.50
```

This is multiplicative with per-track and per-note velocity. It ensures the chorus is the loudest point and parts build/recede naturally.

## Variation Between Repeated Parts

When a part appears twice (e.g., Verse 1 and Verse 2), the second instance should differ slightly:

**Same progression, same patterns, but:**
- Different seed for melody — same contour shape, different specific notes
- Rhythm patterns: 80% same, 20% variation (e.g., extra ghost strum, slightly different arpeggio order)
- Drums: variation pool kicks in (ghost notes appear in different spots)
- Bass: might switch from root-fifth to walking on the second verse

Implementation: use `seed + part_occurrence_index` as the seed for each occurrence. First verse uses seed N, second verse uses seed N+1.

## Crash Cymbal Placement

Automatically insert a crash cymbal (on the crash track if it exists, or synthesize an event on hi-hat track) on beat 1 of:
- First bar of Chorus
- First bar of Bridge (if drums are active)
- First bar after a part that had no drums (re-entry)

## Transition Events

At the boundary between parts:

**Hi-hat open:** On the "and of 4" of the LAST bar of the preceding part. Duration: 240 ticks (one 8th note).

**Crash:** Beat 1 of first bar of new part. Duration: 1920 ticks (ring for a full bar).

**Velocity boost:** First bar of a new part gets +5 velocity to all events (capped at 127).

**Bass approach:** The bass's beat 4 note in the last bar of a part targets the root of the first chord of the next part.

## Global Song Parameters

**Tempo:** BPM, typically 90-140 for folk/indie. Default 120.

**Swing:** 0.0-1.0. Default 0.0 (straight). 0.3-0.5 typical for country shuffle.

**Key (rhythm scale):** Defines the harmonic center. Default Bb Major.

**Lead scale:** Can differ from rhythm scale. Default G minor pentatonic + C# passing tone. This creates a blues-inflected melody over major harmony — extremely common in folk/Americana.

**Time signature:** 4/4 default. 3/4 (waltz), 6/8 (jig), 2/4 (polka) are genre-appropriate alternates but not implemented in v1.
