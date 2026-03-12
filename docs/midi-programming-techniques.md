# MIDI Programming Techniques for Folk & Related Genres

Comprehensive reference for programming realistic MIDI parts across folk, modern folk, new folk, folktronica, folk rock, alt country, and indie rock genres.

---

## 1. Folk Guitar MIDI Programming

### Strumming Simulation
- **Strum timing**: Place chord notes with 5-15ms gaps between each string, not simultaneously
- **Downstroke**: Low string to high string (E2 → E4), each note offset ~10ms after previous
- **Upstroke**: High string to low string (E4 → E2), same offset pattern
- **Full 6-string strum at 10ms/string = ~50ms total sweep**
- **Velocity on downstrokes**: 80-100; upstrokes: 40-60
- Keep chords to 3-6 notes (guitars have 6 strings)

### Chord Voicing
- Use guitar-appropriate voicings, not piano voicings (open strings, barre shapes)
- Session Guitarist libraries handle voicing automatically when fed chord input
- "As Played" voicing mode gives full control over chord tensions
- Add4 tensions reflect natural guitarist tendencies

### Fingerpicking
- Program individual string plucks with slight timing variation (±5-10ms)
- Thumb plays bass strings (E, A, D), fingers play treble (G, B, E)
- Travis picking: alternating bass with melody on top — offset bass notes slightly ahead of beat
- Velocity: bass notes 70-85, melody notes 80-100

### Humanization
- Quantize sparingly — work in musical phrases rather than note-by-note
- Nudge entire sections slightly ahead or behind the beat
- Lower and randomize velocity on non-accented notes
- Palm mutes: short note length + low velocity
- Hammer-on simulation: place second note 50-100ms after first without re-triggering attack
- Use pitch bend wheel for vibrato curves

### Session Guitarist (Kontakt) Specifics
- Keyswitches C1-G1 (up to 8 patterns assignable)
- Pitchbend wheel controls Impact/dynamics (not pitch)
- Play notes/pattern changes slightly ahead of the beat for natural fret noise transitions
- Enable "Chase Events" in Bitwig for proper sequencer behavior
- Latch mode: playback continues after key release
- CC#111: global transport offset (value × 80 ticks)
- Layer two instances panned L/R with inverted voicings for stereo width

---

## 2. Banjo MIDI Programming

### Scruggs-Style (3-Finger) Roll Patterns
All rolls are 8 eighth-notes per measure. T=Thumb, I=Index, M=Middle.

| Roll Name | Finger Sequence | String Order |
|-----------|----------------|--------------|
| Alternating Thumb | T-I-T-M-T-I-T-M | 3-2-5-1-4-2-5-1 |
| Forward | T-I-M-T-I-M-T-I | 3-2-1-5-2-1-5-2 |
| Backward | M-I-T-M-I-T-M-I | 1-2-3-1-2-5-1-2 |
| Forward-Reverse | T-I-M-T-M-I-T-M | 3-2-1-5-1-2-3-1 |
| Foggy Mountain | I-M-T-M-T-I-M-T | (varies by context) |

String mapping (standard G tuning):
- String 5 (drone): G3
- String 4: D2
- String 3: G2
- String 2: B2
- String 1: D3

### Clawhammer Patterns
- **Bum-Ditty**: downstroke melody note → strum → thumb 5th string
- Pattern notation: P (pick/downstroke), B (brush/strum), T (thumb)
- Basic frailing: P-B-T repeated
- Velocity: Pick notes loudest (90-110), brush medium (60-80), thumb moderate (70-85)

### MIDI Programming Tips
- Slides, pull-offs, and hammer-ons are essential embellishments
- Use pitch bend for slides (bend range: 2 semitones)
- Program rolls as continuous eighth-note patterns
- Keep velocity variance of ±10-15 within rolls for natural feel
- Thumb on string 5 should be slightly quieter than melody notes
- Evolution Clawhammer Banjo (Orange Tree Samples): dedicated clawhammer articulations and pattern engine

---

## 3. Mandolin MIDI Programming

### Tremolo Technique
- Tremolo = rapid alternating down-up picks on same note (paired strings)
- "Pennate" = 2 consecutive equal pick strokes (Down + Up)
- Program as rapid repeated notes: 32nd notes at slow tempos, 16th-note triplets at medium
- **Critical**: vary velocity slightly within tremolo (±5-8) to avoid machine-gun effect
- Crescendo/decrescendo within tremolo phrases by ramping velocity
- Free tremolo: vary note density (speed up/slow down) for expression

### Strumming
- Mandolin has 4 courses of doubled strings — strum timing between courses: 3-5ms
- Chop chord: very short note length (staccato), high velocity, all strings struck simultaneously
- Back-beat chop on beats 2 and 4 is signature folk/bluegrass mandolin rhythm

### Melody Programming
- Mandolin melodies use lots of ornaments: grace notes, slides, hammer-ons
- Grace notes: program as 64th notes just before main note, velocity 60-70% of main note
- Tremolo on sustained melody notes (any note longer than a quarter)
- Use pitch bend (range: 2 semitones) for slides between notes

---

## 4. Fiddle/Violin MIDI Programming

### Essential CC Assignments (Common Across Libraries)
| CC | Function | Typical Use |
|----|----------|-------------|
| CC1 | Dynamics/Modwheel | Crossfade between dynamic layers — **write this constantly** |
| CC11 | Expression | Secondary volume control / bow pressure simulation |
| CC3 | Vibrato style | Select between vibrato types (Orchestral Tools) |
| CC14 | Vibrato depth/XFade | Modulate vibrato intensity (Cremona Quartet, OT) |
| CC64 | Sustain | Triggers articulation variations |
| CC72 | Release | Control release tail length |
| CC73 | Attack | Control attack speed |

### Celtic Fiddle Characteristics
- **Minimal vibrato** compared to classical violin — keep CC3/CC14 low
- Heavy use of ornaments: cuts, rolls, grace notes on either side of melody note
- Rapid, rhythmic bow articulations are the defining characteristic
- Grace notes typically 1/32 to 1/64 duration, velocity 50-65% of main note
- Rolls: main note → upper grace → main → lower grace → main (all within one beat)

### Bow Articulation Tips
- CC1 controls dynamic intensity — keep in lower half (40-70) for subtle playing
- Momentary spikes at note starts and ends simulate natural bow pressure changes
- Avoid CC1=90-127 simultaneously with CC11=1-40 (physically impossible)
- Short notes: reduce CC11 quickly after attack for détaché feel
- Legato: overlap notes slightly (10-20ms) and keep CC1 smooth

### SWAM Strings CC Defaults (Physically Modeled)
- CC1: Vibrato depth
- CC2: Bow position / bow pressure
- CC5: Legato/portamento time
- CC11: Expression (dynamics)
- Keep expression and bow pressure in lower halves with momentary spikes at note starts/ends
- All CCs fully customizable in MIDI mapping page

### Traveler Series Celtic Fiddle (Red Room Audio)
- 20 standard articulations + traditional ornaments
- Phrases and FX mode with 4 performance modes
- Customizable double stops
- Adjustable bow attacks and rebowing options
- TACT articulation control system

---

## 5. Pedal Steel MIDI Programming

### The Challenge
Pedal steel is uniquely complex: the bar slides across strings, foot pedals/knee levers bend individual strings independently, and volume pedal shapes every note. Standard MIDI (one pitch bend for all notes) cannot capture this.

### Multi-Channel Approach
1. **Channel A** (pitch bend range = 2 semitones): strings that bend up
2. **Channel B** (pitch bend range = 1 semitone): strings that bend down
3. **Channel C** (no pitch bend): strings that remain static
- Play the chord across 3 Kontakt instances on separate MIDI channels
- Automate pitch bend independently per channel

### MPE Approach (Preferred)
- Each string on a separate MIDI channel (channels 2+)
- Per-note pitch bend, pressure, and slide
- Bitwig has native MPE support

### Volume Swell
- Map CC11 (Expression) to an expression pedal or automate
- Every note should start with expression at 0 and swell up
- Classic "crying" steel sound: slow attack swell (200-500ms rise time)
- Quick swell for rhythm parts (50-100ms)

### Pitch Bend Programming
- Bar slides: smooth pitch bend curves, typically ±2 semitones
- Pedal/lever bends: usually 1-2 semitones on specific strings
- Classic country bends: 3rd→ minor 3rd, or 6th→ flat 7th

### Recommended Libraries
- **Impact Soundworks Pedal Steel**: simplified playability, no steel guitar knowledge needed
- **Wavelore Pedal Steel**: independent pitch bending per string on single MIDI channel via KSP
- **Ink Steel**: 8 chord sets + 4 single note sets, pitch bend follows major scale

---

## 6. Acoustic/Upright Bass MIDI Programming

### Fingerstyle Technique
- Note lengths: slightly shorter than full duration (85-90% of beat length) for natural decay
- Velocity variation: ±10-15 from baseline per note
- Slight timing push (5-15ms ahead of beat) for driving feel
- Play in parts on MIDI keyboard rather than step-programming when possible

### Ghost Notes
- Very short notes (1/32 or shorter) at velocity 25-40
- Muted/deadened string sound — use staccatissimo articulation or keyswitch
- Place between main beats (16th-note subdivisions)
- Right hand ghost notes fill space between quarter-note walking lines

### Slides
- Use pitch bend (range: 2 semitones) for short slides into notes
- Slide up to target note: start pitch bend at -2, ramp to 0 over 50-150ms
- Slide between notes: end previous note with bend toward target pitch
- truSlide (SubMission Audio): algorithmic slides from any note to any note

### Walking Bass (Folk/Country)
- Quarter notes on root, 3rd, 5th of chord + chromatic approach notes
- Walk up: ascending scalar motion approaching next chord root
- Walk down: descending scalar motion
- Emphasize beat 1 (velocity 90-100), beats 2-4 softer (70-85)

### Articulation Keyswitches (Typical)
- Sustain/finger, mute/palm mute, harmonics, slap, hammer-on/pull-off, slide
- Switch articulations via keyswitches below playing range

### Recommended: Toontrack Upright EBX
- MIDI patterns tailored for Americana, bluegrass, folk, indie, pop, rock, acoustic

---

## 7. Folk Drum Patterns

### Fundamental Patterns

**Boom-Chick (2-beat feel)**:
- Beat 1: Kick (vel 90-100)
- Beat 2: Snare (vel 75-85) or cross-stick
- Hi-hat: quarter or eighth notes, velocity alternating 65-80
- Classic country/folk foundation

**Train Beat**:
- Snare: constant eighth or sixteenth notes with heavy accents on 2 and 4
- Kick: beats 1 and 3
- Simulates locomotive rhythm — velocity pattern is key

**6/8 Feel**:
- Kick on beat 1, snare on beat 4
- Hi-hat or shaker: eighth notes with accent pattern 1-2-3-4-5-6 (1 and 4 accented)
- Common in Celtic, folk ballads

**Brush Patterns (General MIDI)**:
- Note 38: Brush Tap (replaces Snare)
- Note 39: Brush Slap (replaces Hand Clap)
- Note 40: Brush Swirl (replaces Electric Snare)
- Brush swirls: continuous circular motion, lower velocity (40-60)
- Brush taps: accents on 2 and 4 (70-85)

### Shaker Programming
- Basic 16th-note pattern: full velocity (85-95) on every eighth, lower velocity (55-70) on off-sixteenths
- Visualize physical motion: away-from-body strokes louder, toward-body strokes softer
- Alternate sample selection if possible to avoid machine-gun effect
- Vary each hit ±5-10 velocity for human feel

### Cajon Programming
- Map to kick (low tone), snare (slap/high tone), and ghost (fingertip) zones
- Basic pattern mirrors standard kick-snare but with softer attack
- Ghost notes between main hits at velocity 25-35
- Slight swing (5-10%) adds organic feel

### Ghost Notes on Snare
- Velocity: 25-40 (well below main hits at 80-100)
- Place on 16th-note subdivisions, typically left-hand only
- Remove notes that conflict with hi-hat hand (right hand busy)
- EZdrummer/Superior Drummer: multi-velocity layers produce different timbres at ghost note velocities

### Velocity Guidelines for Drums
| Element | Ghost | Soft | Medium | Accent | Hard |
|---------|-------|------|--------|--------|------|
| Kick | — | 60-70 | 80-90 | 95-110 | 115-127 |
| Snare | 25-40 | 55-70 | 75-90 | 95-110 | 115-127 |
| Hi-hat | 40-55 | 60-70 | 75-85 | 90-100 | 105-115 |
| Cross-stick | — | 50-65 | 70-80 | 85-95 | — |

### Indie Folk Drum Philosophy
- "No law says drums require eighth notes on the hi-hat — or even a hi-hat at all"
- Simple heartbeat/backbeat feels work best
- Use smooth ebb and flow rather than traditional rock defaults
- Minimal kit: kick, cross-stick, shaker/tambourine
- Leave space — negative space is characteristic of folk drumming

### Available Time Signatures for Folk/Country
- 2/4, 3/4, 4/4, 6/8, 9/8, 12/8
- Swing feels: straight, medium-swing, hard-swing
- Double-time variations for energetic sections

---

## 8. Hammond Organ MIDI Programming

### Hammond B-3X CC Assignments (IK Multimedia)
| CC | Function |
|----|----------|
| CC1 | Leslie Speed (Fast/Slow) via Modwheel |
| CC7 | Master Volume |
| CC11 | Expression Pedal / Swell |
| CC12-20 | Upper Manual Drawbars 1-9 |
| CC21-29 | Lower Manual Drawbars 1-9 |
| CC33 | Pedal Drawbar 1 |
| CC35 | Pedal Drawbar 2 |
| CC36 | Percussion On/Off |
| CC37 | Percussion Volume |
| CC38 | Vibrato Type |
| CC39 | Vibrato Great |
| CC40 | Vibrato Swell |
| CC42 | Leslie Amplifier Gain |
| CC43 | Percussion Decay |

### Leslie Speaker Control
- Three speeds: Stop, Slow (chorale), Fast (tremolo)
- CC1 (modwheel): toggle between slow and fast
- Jazz players typically use Stop and Fast only
- Classic rock/folk: ride between Slow and Fast for swirling effect
- Leslie spin-up/spin-down transition is part of the sound — don't switch instantaneously

### Drawbar Basics
- 9 drawbars per manual, each controls a harmonic partial
- Drawbar positions: 0 (off) to 8 (full)
- Notation format: 9-digit number (e.g., "888000000" = full fundamental + 2nd/3rd harmonics)
- Classic registrations:
  - Full organ: 888888888
  - Gospel/blues: 888800000
  - Jazz ballad: 808000000
  - Jimmy Smith: 888000000
  - Bright lead: 868868068

### Performance Tips
- Percussion circuit: upper manual only, triggers on detached (non-legato) notes
- Percussion adds click/attack — essential for rhythmic playing
- Configure two drawbar registrations and switch between them via preset keys
- Expression pedal (CC11): critical for dynamic swells
- Chorus/Vibrato: C3 setting most common; pure vibrato settings are special effects

---

## 9. Folktronica MIDI Techniques

### Production Framework
- **BPM range**: 70-110 BPM
- **Time signatures**: 4/4, 6/8, 3/4 — borrow folky meters
- **Approach**: acoustic instruments as foundation, electronics as texture/atmosphere

### Harmonic Language
- Simple, modal folk harmony: Dorian, Mixolydian, Aeolian
- Singable, pentatonic or modal melodies
- Keep progressions sparse — let timbre and texture carry emotional weight
- Double melodies with bowed strings or soft synth pads

### Glitch and Stutter Techniques
- **Stutter effect**: rhythmic repetition of small audio fragments with DSP processing
- Granular synthesis: divide audio into "grains" to synthesize new sounds and rhythmic patterns
- Micro-edits on acoustic recordings: chop, reverse, pitch-shift fragments
- Off-grid timing for "handmade" feel

### Sugar Bytes Tools for Folktronica
- **Effectrix**: 14-effect sequencer, store 12 sequences per preset, flip via MIDI keyboard
  - Stutter effect provides gating + sequenced panning + enveloping
  - Map different effect sequences to MIDI notes
- **Looperator**: chops incoming audio into 16 steps, each with independent effects
  - Trigger patterns via MIDI notes from separate track
  - Effects: filters, stutters, slices, distortion, tape stops, volume modulation, looping
- **Turnado**: 8 real-time effects with single-knob control
- **Egoist**: slice-based beat creation from any audio

### Sound Design Approaches
- Record acoustic instruments closely to capture detail
- Layer soft synth pads, granular textures, gentle drones underneath
- Found sounds / field recordings: wind, creaking floors, birdsong as rhythmic or atmospheric elements
- Light tape saturation, vinyl crackle, room noise for warmth
- Granular slicing for evolving pads from acoustic sources
- Gentle filtering, convolution reverb (natural spaces), spectral effects

### Arrangement Strategy
- Alternate intimate verses (mostly acoustic) with textural refrains
- Introduce rhythmic programming and pads gradually
- Dynamic swells via gradual addition of percussive details, arpeggios, counter-melodies
- Preserve negative space — airy, pastoral character
- Sidechain pads subtly to acoustic transients for cohesion
- Preserve performance artifacts (finger noise, breaths) for intimacy

### DIY Folktronica (Rabbitology Approach)
- Embrace imperfection — slightly-off drums and detuned elements are features
- Layer extensively (100+ vocal layers in some tracks)
- Use found sounds: bowl hums, rock clicks, frog croaks
- Ozone Imager (free): ultra-wide imaging on atmospheric elements
- Vocal Doubler (free): dreamy wideness on layered elements
- Test mixes across multiple playback systems

---

## 10. Humanization Techniques

### Timing Offsets
- **Range**: ±5-20 milliseconds (±5-12 MIDI ticks at 480 PPQ)
- Shift notes slightly ahead of beat for "driving" feel
- Shift notes slightly behind beat for "laid-back" feel
- Up to ±10 ticks adds subtle life without audible mistakes
- For drums: up to 8 ticks random, 12 ticks at faster tempos
- Re-quantize kick and crash after humanizing other elements (keep foundation tight)

### Velocity Variation
- **General rule**: vary ±10-15 from baseline velocity per note
- Hi-hat: base velocity 75-80, vary each hit between 65-95
- Downbeats slightly harder, offbeats softer
- Ghost notes: velocity 25-40
- For melody doubling: shift duplicate's velocity down 10-15 units + offset 2-3 ticks forward
- Drum velocity variance: up to ±4% (approximately ±5 on 0-127 scale)

### Groove and Swing
- 16th-note swing: every second 16th slightly late
- Bitwig: Humanize function — select all notes, set value (e.g., 60%), Apply
- Bitwig: Shuffle parameter for swing feel
- Bitwig lacks groove template pools — use external MIDI groove files
- Triplet quantize available for shuffle feels

### Note Length Variation
- Slightly shorten or lengthen notes (85-100% of grid value)
- Staccato passages: vary note-off times
- Legato passages: overlap notes by 10-20ms

### Layering Techniques
- Double melodic lines with slight timing and velocity offset
- Layer slightly different MIDI variations that complement rhythmically
- Pan doubled parts for stereo width

### Per-Instrument Summary

| Instrument | Timing Feel | Velocity Base | Key Humanization |
|------------|------------|---------------|-----------------|
| Acoustic Guitar | Slightly ahead | 75-90 | Strum timing, chord voicing |
| Banjo | On the beat | 80-95 | Roll consistency, ornaments |
| Mandolin | Slightly ahead | 80-100 | Tremolo variation, chop accents |
| Fiddle | Flexible | 60-90 | CC1 dynamics, ornaments |
| Pedal Steel | Behind the beat | 70-85 | Volume swell, pitch bend curves |
| Upright Bass | Slightly ahead | 75-90 | Ghost notes, slide-ins |
| Drums (folk) | Loose/behind | 70-90 | Ghost notes, velocity variance |
| Hammond | On the beat | 80-100 | Expression pedal, Leslie timing |

---

## Bitwig Studio-Specific Tips

### MIDI Humanize
- Select MIDI notes → Functions → Humanize → set percentage → Apply
- Affects timing only; velocity must be varied separately
- Start with 40-60% humanize for folk genres

### Groove Tools
- Shuffle: applies swing to every second note
- Quantize: snap to grid with adjustable strength
- Triplet quantize for shuffle/swing feels
- No native groove template pool — import MIDI groove files as workaround

### MPE Support
- Native MPE: ideal for pedal steel, expressive strings
- Per-note pitch bend, pressure, and timbre
- Works with SWAM instruments, Roli, Sensel Morph, etc.

### Plugin Integration
- Sugar Bytes Effectrix/Looperator: route MIDI from separate track for effect pattern triggering
- Session Guitarist: enable Chase Events for pattern sequencing
- Kontakt instruments: use Note FX for keyswitch management

---

## Sources

### Guitar Programming
- [Making MIDI Guitar Sound Real - Producer Society](https://producersociety.com/making-midi-guitar-sound-real-tutorial/)
- [MIDI Guitar Strumming - VI-CONTROL](https://vi-control.net/community/threads/programming-midi-strumming-pattern.154651/)
- [5 Ways to Breathe Life into MIDI Guitar - Native Instruments](https://blog.native-instruments.com/midi-acoustic-guitar/)
- [Faking Guitar Strumming in MIDI - Loopy Pro Forum](https://forum.loopypro.com/discussion/52977/faking-guitar-strumming-on-midi-any-tips-and-is-it-worth-it)
- [Guitar Patterns with Free MIDI Files - Splice](https://splice.com/blog/make-guitar-patterns/)
- [Strum Roll MIDI Device - Isotonik Studios](https://isotonikstudios.com/product/strum-roll/)

### Banjo Programming
- [How to Program Realistic Banjo Parts - MusicRadar](https://www.musicradar.com/tuition/tech/how-to-program-realistic-banjo-parts-in-logic-588803)
- [Elfshot Banjo Basic Rolls](https://www.elfshot.com/banjo/rolls.htm)
- [Banjo Roll Patterns - Wikipedia](https://en.wikipedia.org/wiki/Banjo_roll)
- [Evolution Clawhammer Banjo - Orange Tree Samples](https://www.orangetreesamples.com/products/evolution-clawhammer-banjo)
- [Essential Banjo Rolls Guide - ArtistWorks](https://blog.artistworks.com/essential-banjo-rolls-the-complete-guide-you-need-to-know/)
- [4 Essential Banjo Rolls - Deering Banjos](https://blog.deeringbanjos.com/the-four-essential-5-string-banjo-rolls)

### Mandolin
- [Mandolin Tremolo Technique - Mandozine](https://www.mandozine.com/techniques/tremolo/tremolo.html)
- [Mandolin Tremolo - Mando Montreal](https://www.mandomontreal.com/en/blog/how-can-i-get-a-good-tremolo-mandolin-picking-technique)

### Fiddle/Strings
- [CC1 vs CC11 for Strings - VI-CONTROL](https://vi-control.net/community/threads/the-difference-between-dynamics-cc-1-and-expression-cc-11.58127/)
- [Improving Realism in Orchestral Mock-ups - Production Expert](https://www.production-expert.com/production-expert-1/how-to-improve-realism-in-orchestral-mock-ups)
- [Orchestral Tools Controller Table](https://orchestraltools.helpscoutdocs.com/article/199-controller-table-annotated-list)
- [SWAM Solo Strings Manual](https://static.audiomodeling.com/manuals/strings/SWAM%20Solo%20Strings%20v3.8.0%20-%20User%20Manual.pdf)
- [Traveler Series Celtic Fiddle - Red Room Audio](https://pulse.audio/product/traveler-series-celtic-fiddle-by-red-room-audio/)
- [CC1 Drawing Techniques - VI-CONTROL](https://vi-control.net/community/threads/cc1-drawing-techniques-for-realistic-strings-brass-woodwinds.56845/)

### Pedal Steel
- [Impact Soundworks Pedal Steel](https://impactsoundworks.com/product/pedal-steel/)
- [Ink Steel Virtual Pedal Steel](https://ink-audio.com/products/ink-steel)
- [Pedal Steel MIDI Discussion - Logic Users Group](https://logic-users-group.com/threads/i-want-to-use-midi-keyboard-for-steel-pedal-guitar-can-i.8766/)
- [Wavelore Pedal Steel - KVR](https://www.kvraudio.com/forum/viewtopic.php?t=255433)

### Bass
- [Create Realistic MIDI Bass Lines - Native Instruments](https://blog.native-instruments.com/midi-bass/)
- [Amazing MIDI Bass Lines in 7 Steps](https://thalesmatos.com/blog/midi-bass-lines/)
- [Ghost Notes on Bass - KVR](https://www.kvraudio.com/forum/viewtopic.php?t=591417)
- [Upright EBX - Toontrack](https://www.toontrack.com/product/upright-ebx/)

### Drums
- [How to Program Authentic Shaker Patterns - MusicRadar](https://www.musicradar.com/tuition/tech/how-to-program-authentic-shaker-patterns-638625)
- [5 Advanced Tips for Virtual Drummers - Loopmasters](https://www.loopmasters.com/articles/4436-How-to-Program-Realistic-MIDI-Drums-5-advanced-tips-for-virtual-drummers)
- [3 Ways to Humanize Drums - Splice](https://splice.com/blog/humanize-your-drums/)
- [Indie Folk MIDI Drums - Toontrack](https://www.toontrack.com/product/indie-folk-midi/)
- [Free Country & Folk MIDI Patterns - Prosonic Studios](https://www.prosonic-studios.com/midi-drum-beats/country-and-folk)
- [Velocity Curve and Ghost Notes - Toontrack](https://www.toontrack.com/news/velocity-curve-and-ghost-notes/)

### Hammond Organ
- [Hammond B-3X CC List - IK Multimedia Forum](https://cgi.ikmultimedia.com/ikforum/viewtopic.php?f=12&t=25675)
- [Hammond B-3X Product Page - IK Multimedia](https://www.ikmultimedia.com/products/hammondb3x/)
- [Hammond B3 Drawbars Explained - Piano Groove](https://www.pianogroove.com/blues-piano-lessons/hammond-b3-drawbars-presets-controls/)
- [Logic Pro Vintage B3 Tutorial - MusicTech](https://musictech.com/tutorials/logic-pro-tutorial-vintage-b3/)

### Folktronica
- [Folktronica Genre Guide - Melodigging](https://www.melodigging.com/genre/folktronica)
- [Rabbitology DIY Folktronica - Native Instruments](https://blog.native-instruments.com/rabbithology/)
- [Effectrix - Sugar Bytes](https://sugar-bytes.de/effectrix)
- [Looperator - Sugar Bytes](https://sugar-bytes.de/looperator)
- [10 Steps to Producing Perfect Glitch - MusicRadar](https://www.musicradar.com/tuition/tech/10-steps-to-producing-perfect-glitch-607389)

### Humanization
- [How to Humanize MIDI - 20+ Pro Tips - Unison](https://unison.audio/how-to-humanize-midi/)
- [6 Ways to Humanize Your Tracks - Production Music Live](https://www.productionmusiclive.com/blogs/news/6-ways-to-humanize-your-tracks)
- [Swing, Shuffle, and Humanization - Sample Focus](https://blog.samplefocus.com/blog/swing-shuffle-and-humanization-how-to-program-grooves/)
- [Humanizing MIDI Drums - Mix Elite](https://mixelite.com/blog/humanizing-midi-drums/)

### Folk Production
- [Indie Folk Music Production Guide - Eliott Glinn Audio](https://eliottglinnaudio.com/blog/a-beginner-s-guide-to-indie-folk-music-production)
- [9 Tips for Mixing Acoustic Music - Waves](https://www.waves.com/tips-for-mixing-acoustic-music)
- [Session Guitarist Strummed Acoustic Manual - Native Instruments](https://www.native-instruments.com/fileadmin/ni_media/downloads/manuals/Session_Guitarist_-_Strummed_Acoustic_1.1_Manual_English.pdf)

### Bitwig Studio
- [5 Ways to Alter Groove in Bitwig - Ask.Video](https://ask.video/article/bitwig/5-ways-to-alter-the-groove-in-bitwig-studio)
- [Bitwig Groove Template Discussion - KVR](https://www.kvraudio.com/forum/viewtopic.php?t=607670)
- [Bitwig Global Groove - User Guide](https://www.bitwig.com/userguide/latest/the_global_groove/)
