# Bass Engine

## Role

The bass locks with the kick drum and outlines the harmony. In folk and indie, the bass is understated but essential — it defines the root motion and gives the rhythm section its weight.

## Bass Patterns by Style

### Root-Fifth (default for verse)

The bread and butter. Root on beat 1, fifth on beat 3.

```
Bar in C major (C chord):
Beat: 1       2       3       4
Note: C2      -       G2      -
Vel:  100     -       80      -
Dur:  480     -       480     -
```

Variation: add an 8th note pickup into beat 3:
```
Beat: 1       2   .   3       4
Note: C2      -   E2  G2      -
Vel:  100     -   65  80      -
```

### Walking Bass (verse/chorus, higher energy)

Scalar motion connecting chord roots. Every beat has a note.

```
Bar: C chord → next bar Am chord
Beat: 1     2     3     4
Note: C2    D2    E2    G#2    (G# = chromatic approach to A)
Vel:  100   75    80    70
```

Rules:
- Beat 1: chord root (mandatory)
- Beat 2: scale tone moving toward beat 3 target
- Beat 3: chord tone (root, 3rd, or 5th)  
- Beat 4: approach note — one semitone or one scale step below/above the NEXT bar's root

### Pedal Bass (intro/bridge)

Stay on one note (usually the tonic) regardless of chord changes above.

```
Bars 1-4 over I-V-vi-IV:
Note: Bb1 throughout, whole notes or half notes
```

Creates tension when chord harmony moves above a static bass. Very effective for bridge.

### Octave Pattern (chorus energy boost)

Root and its octave alternating, 8th notes:

```
Beat: 1   .   2   .   3   .   4   .
Note: C2  C3  C2  C3  C2  C3  C2  C3
Vel:  100 70  85  65  90  70  85  65
```

Or with a rest pattern:
```
Beat: 1   .   2   .   3   .   4   .
Note: C2  -   C3  -   C2  -   C3  C2
```

### Dead Notes / Ghost Notes

In funkier or folktronica tracks, the bass can include muted ghost notes:
- Duration: 30-60 ticks
- Velocity: 20-40
- Placed on 16th note positions between real notes
- 10-15% probability per available slot

## Approach Note Logic

The approach note on beat 4 (or the last 8th note of a bar) targets the next chord's root:

```
Next root is ABOVE current note by a step or more:
  → approach from one scale step below (diatonic approach)
  → OR one semitone below (chromatic approach, 30% chance)

Next root is BELOW:
  → approach from one scale step above
  → OR one semitone above (30% chance)

Next root is the SAME:
  → approach from the fifth below, or a scale step above/below
```

Chromatic approach notes should have slightly lower velocity (70-80% of normal) and shorter duration.

## Voice Leading Between Chords

When chords change, the bass shouldn't jump randomly to the new root. Choose the voicing (octave) of the new root that is closest to the previous note:

```
Previous note: G2 (MIDI 43)
New chord root: C
Options: C2 (36), C3 (48)
→ Choose C3 (48) because |48-43| < |36-43|
UNLESS that puts us outside the instrument range, then pick the other.
```

## Register Management

Bass should generally stay in a 1.5 octave range within any 4-bar phrase. Avoid jumping across 2+ octaves bar to bar — it sounds erratic.

```
Preferred range per instrument:
  Electric Bass: E1 (28) to G3 (55), sweet spot B1-D3 (35-50)
  Acoustic Bass: E1 (28) to D3 (50), sweet spot A1-B2 (33-47)
```

## Interaction with Kick

Bass and kick should generally align on beat 1. When kick has a ghost note, bass should NOT play there (leave space). The rhythmic lock between bass and kick is what makes the low end feel solid.

```
If kick is on beats 1 and 3:
  Bass plays on beats 1 and 3 (or 1, 2, 3, 4 for walking)
  Bass does NOT play unexpected notes on the "and of 2" if kick has a ghost there

If kick has a syncopated hit on "and of 3":
  Bass can match it (lock) or stay on grid (counterpoint)
  Locking is preferred 70% of the time
```

## Part-Specific Behavior

| Part | Style | Notes/bar | Character |
|------|-------|-----------|-----------|
| Intro | Pedal or root only | 1-2 | Establishing |
| Verse | Root-fifth | 2-4 | Steady |
| PreChorus | Walking, ascending | 4 | Building motion |
| Chorus | Octave or walking | 4-8 | Driving |
| Bridge | Pedal or half-time root | 1-2 | Contrast |
| Outro | Root, fading | 1-2 | Resolving |
