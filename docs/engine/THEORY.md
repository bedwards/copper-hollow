# Music Theory Reference

## Diatonic Chord Derivation

Given a 7-note scale, stack every other note to build triads on each degree:

| Degree | Major | Natural Minor |
|--------|-------|---------------|
| I | Major | Minor |
| II | Minor | Diminished |
| III | Minor | Major |
| IV | Major | Minor |
| V | Major | Major |
| VI | Minor | Major |
| VII | Diminished | Major |

For pentatonic/blues scales (fewer than 7 notes), derive chords from the parent diatonic scale (major pentatonic → major, minor pentatonic → natural minor).

## Voice Leading for Chords

When a chord changes, minimize the motion of individual voices:

1. Keep common tones. If two chords share a note, that voice stays.
2. Move remaining voices by the smallest interval possible (prefer stepwise).
3. Avoid parallel fifths and octaves between outer voices (bass and top note).

Example: C major → A minor
```
C chord: C E G
Am chord: A C E
→ C stays, E stays, G moves down to A (or up to A)
→ Result: A C E (smooth, only one voice moved)
```

## Chord Voicing for Guitar-Range Instruments

Don't just play root-position triads. Use voicings appropriate to the instrument range:

**Open voicing (guitar-like):** Spread notes across 1.5-2 octaves. Root in bass, other notes above.
```
C major open voicing: C3 E3 G3 C4 E4 (5 notes spanning C3-E4)
```

**Close voicing (piano/organ):** Notes within one octave.
```
C major close: C4 E4 G4
```

**Drop-2 voicing (jazz-folk):** Take the second note from the top and drop it an octave.
```
C major drop-2: G3 C4 E4 (from C4 E4 G4, dropped E down)
```

## Scale-Chord Compatibility

When generating melody over chords from a DIFFERENT scale than the melody scale:

The rhythm scale is Bb major, the lead scale is G minor pentatonic. Over a Bb major chord, the melody uses G Bb C D F. Over an F major chord (V), same notes. The passing tone C# creates tension over any chord containing C natural — use it on weak beats approaching D.

Avoid: holding a passing tone (C#) over a chord that contains C (like Cm or C). The clash is fine as a quick passing motion but not sustained.

## Common Folk Progressions with Analysis

**I-V-vi-IV** (Bb-F-Gm-Eb): The most common progression in modern pop/folk. Works because each chord shares notes with its neighbors. Motion is smooth.

**I-IV-V-I** (Bb-Eb-F-Bb): Classic country/folk cadence. Strong root motion. The V-I resolution at the end provides closure.

**vi-IV-I-V** (Gm-Eb-Bb-F): Same chords as I-V-vi-IV but starting on vi. Creates a darker, more introspective feel. Good for verses.

**I-iii-IV-V** (Bb-Dm-Eb-F): The iii chord adds a gentle passing harmony. Slightly more sophisticated.

**ii-V-I** (Cm-F-Bb): Jazz-derived. The ii-V motion is the strongest harmonic pull in Western music. Good for pre-chorus endings.

## Key Relationships

When rhythm and lead scales have different roots, their relationship matters:

**G minor over Bb major:** G is the relative minor of Bb. The notes overlap heavily (Bb major = G natural minor). This is the default and most consonant pairing.

**E minor over G major:** Same relationship. Relative minor over its major.

**A minor pentatonic over C major:** Again relative minor. Very safe.

**D dorian over C major:** Shares all notes with C major but emphasizes D as a tonal center. Creates a modal, folk-authentic sound.

**A mixolydian over D major:** A is the V of D. Mixolydian over the IV key creates a Grateful Dead / jam-band folk feel.
