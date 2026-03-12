# Melody Engine

## Principles

A good folk melody is singable. It moves primarily by step, leaps resolve, phrases breathe, and chord tones land on strong beats. The engine generates melodies that follow these rules without exception.

## Melodic Contour

Each phrase (typically 2 or 4 bars) follows a contour shape chosen from:

**Arch:** Rise to a peak around bar 2-3, then descend. Most common. Good for verse melodies.

**Descending:** Start high, gradually come down. Creates resolution feel. Good for chorus endings.

**Ascending:** Build upward. Creates tension. Good for pre-chorus.

**Wave:** Two smaller arches. Creates conversational feel. Good for verses.

**Static with ornament:** Stay in a narrow range (3-4 notes) with occasional departures. Good for spoken/rhythmic sections.

The contour defines target pitch zones per beat. The melody moves through these zones using step motion, with occasional leaps to hit chord tones.

## Chord Tone Targeting

On every strong beat (beats 1 and 3 in 4/4), the melody should land on or near a chord tone of the current chord. "Near" means within one scale step.

```
Priority on downbeats:
1. Root of current chord (highest priority on beat 1 of new chord)
2. Third of current chord
3. Fifth of current chord
4. Scale tone adjacent to a chord tone
```

On weak beats (2, 4, and all offbeats), any scale tone is acceptable. Passing tones (chromatic) may appear on the weakest subdivisions only (8th note offbeats, 16th notes).

## Motion Rules

**Step motion: 70% of intervals.** Move by one scale degree up or down. This is the backbone of singable melody.

**Small leap: 20% of intervals.** Skip one scale degree (a third). Always resolve by step in the opposite direction.

**Large leap: 10% of intervals.** Fourth, fifth, or octave. Must be followed by step motion in the opposite direction (leap resolution). Never two leaps in the same direction.

**Repeated note: Allowed** but limited to 2-3 repetitions max. More than that becomes static.

## Rhythm of Melody

The melody rhythm is independent of the strum pattern but syncs to the beat grid.

**Note durations available:** 16th (120t), 8th (240t), quarter (480t), dotted quarter (720t), half (960t), dotted half (1440t), whole (1920t).

**Rhythm generation algorithm:**
1. For each bar, choose a rhythmic density: sparse (3-4 notes), medium (5-7 notes), dense (8-12 notes)
2. Density varies by song part: verse=medium, chorus=medium-dense, bridge=sparse, intro/outro=sparse
3. Place a note on beat 1 (mandatory)
4. Fill remaining bar with notes at chosen density
5. Insert rests: minimum one 8th-note rest per 2-bar phrase (melodies must breathe)
6. Occasional syncopation: ~20% chance of an 8th note anticipation (note starts on the "and" before a strong beat)

## Passing Tones

When the lead scale has defined passing tones (e.g., C# in G minor pentatonic), these may appear:
- Only on weak 8th-note positions
- Only as transitions between two adjacent scale tones
- Duration: never longer than an 8th note
- Marked in the data model so the GUI can display them distinctly

## Counter-Melody Rules

Counter-melodies are generated with the same algorithm but with constraints relative to the lead melody:

**Rhythmic independence:** Counter-melody should NOT move in rhythmic unison with the lead. When lead has a note, counter preferably has a rest (or vice versa). Some overlap is fine (~30% of beats can coincide).

**Harmonic intervals:** When both sound simultaneously, prefer thirds and sixths. Avoid unisons and octaves (too blended). Avoid seconds (too dissonant for folk). Fifths are acceptable.

**Register separation:** Counter-melody operates in a different octave or register zone than lead. If lead is in the middle of the instrument range, counter sits above or below.

**Simplicity:** Counter-melody is sparser than lead. Fewer notes, longer durations. It fills gaps, not competes.

## Mono vs Poly Melody

Lead and counter-melody default to Mono (one note at a time). They CAN be set to Poly, in which case:

- Dyads (two notes, typically a third or sixth apart) on strong beats
- Single notes on weak beats and fast passages
- Never more than 2 simultaneous notes for lead/counter roles
- The "extra" note is always a diatonic interval above or below the primary note

## Phrase Structure

Melodies organize into 2-bar and 4-bar phrases:

**2-bar phrase:** A musical sentence. Has a beginning, motion, and a small resolution or continuation.

**4-bar phrase (period):** Two 2-bar phrases. The first (antecedent) ends on a non-tonic tone (creates expectation). The second (consequent) resolves to root or third.

**8-bar section:** Two 4-bar periods. The second typically varies the first — same rhythm, different pitches (or vice versa).

The engine generates at this phrase level, not note-by-note. First decide the phrase structure, then fill in pitches that satisfy contour + chord tone rules.

## Ornaments and Flourishes

At higher intensity settings or in specific song parts:

**Grace notes:** A fast note (duration 60t, velocity 75%) a scale step below the target note, immediately before it. 5-10% probability per note in chorus/bridge.

**Slides (pitch bend):** CC event: pitch bend from -2 semitones to 0 over 120 ticks, leading into a note. Good for pedal steel, mandolin. 3-5% probability.

**Hammer-on/pull-off simulation:** Two rapid notes (60t each) on adjacent scale tones, first at higher velocity. Appropriate for guitar, mandolin. 5-8% probability.

**Trills:** Rapid alternation between two adjacent tones, 4-6 repetitions. Very rare — only in bridge or dramatic moments. <2% probability.
