# GUI Specification

## Framework

Iced 0.14.0. Elm architecture (Model → Message → Update → View). GPU-rendered via wgpu. Custom canvas widgets for pattern visualization.

## Theme

Dark background. Music-production aesthetic.

```
Background:      #1a1c20
Surface:         #24272e
Surface Hover:   #2e323a
Accent:          #4a8fcc
Accent Hover:    #5ea3e0
Text Primary:    #e0e2e6
Text Secondary:  #8a8e96
Text Muted:      #5a5e66
Root Note:       #4a9e5a (green)
Major Step:      #3a5570 (blue-gray)
Passing Tone:    #c8a050 (amber)
Active Dot:      #50b850 (green)
Inactive Dot:    #505050 (gray)
Mute Red:        #cc5050
Solo Yellow:     #cccc50
Beat Marker:     #ffffff20
Bar Line:        #ffffff40
```

Font: system monospace for data, system sans-serif for labels. No custom fonts needed.

## Layout

```
┌──────────────────────────────────────────────────────────┐
│ HEADER BAR                                               │
│  ▶/⏸ ⏹ | BPM: [120] | Bar 3 Beat 2.3 | ⟲ ⟳ | 🎲 All  │
│  [● Bitwig] [○ CLI]                                     │
├──────────────────────────────────────────────────────────┤
│ SCALE BAR                                                │
│  Rhythm: [Bb▾] [Major▾]  Lead: [G▾] [Min Pent▾]         │
│  Rhythm: [Bb][C][D][Eb][F][G][A]    (C#) passing        │
│  Lead:   [G][Bb][C][D][F]           (C#) passing         │
├──────────────────────────────────────────────────────────┤
│ STRUCTURE BAR                                            │
│  [Intro] [Verse*] [PreCh] [Chorus] [Verse] ...          │
│  Chorus: Bb  F  Gm  Eb    Strum: [Folk Strum▾]          │
├──────────────────────────────────────────────────────────┤
│ TRACK LIST (scrollable)                                  │
│ # │ Name            │ Role   │ Instr    │ Voice│M S│🎲│●│
│ 1 │ Kick            │ Drum   │ Kick     │      │  │ │●│
│ 2 │ Snare           │ Drum   │ Snare    │      │  │ │●│
│ ...                                                      │
│ 5 │ Acoustic Guitar  │ Rhythm │ Ac.Gtr  │ Poly │  │ │●│
│ ...                                                      │
├──────────────────────────────────────────────────────────┤
│ PATTERN VIEW (canvas, for selected track)                │
│  [Piano roll-lite: horizontal bars showing notes]        │
│  Beat markers, bar lines, notes colored by velocity      │
├──────────────────────────────────────────────────────────┤
│ STATUS BAR                                               │
│  History 3/12 | Seed: 42 | Bb Major | 36 bars total      │
└──────────────────────────────────────────────────────────┘
```

## Header Bar

**Transport controls:**
- Play/Pause toggle button: sends play/stop to Bitwig via bridge
- Stop button: stops and resets position to 0
- BPM: DragValue widget, range 40-240, step 0.5

**Position display:** "Bar N | Beat X.X" — updates from Bitwig transport observer

**Undo/Redo:** Buttons. Disabled when at history boundary. Tooltip shows snapshot label.

**Randomize All:** Prominent button. Generates new seed, recomposes everything, saves snapshot.

**Connection indicators:** Right-aligned. Green dot + "Bitwig" when connected, red dot when not. Gray dot + "CLI" when CLI client is connected.

## Scale Bar

Two rows, one for rhythm scale, one for lead scale.

**Scale selector:** Two ComboBoxes per scale — root (12 options) and type (7+ options).

**Note grid:** Horizontal row of buttons, one per scale degree. Root button is green and non-toggleable (always on). Other degrees toggle on/off (enabled_degrees). Background color: major steps slightly brighter than non-major. Passing tones shown in parentheses with amber color.

Clicking a non-root button toggles that degree. This affects all composition that uses this scale.

## Structure Bar

Horizontal row of buttons, one per song part in the structure. Selected part has accent background. Clicking selects that part — the track list and pattern view update to show patterns for that part.

Below: current part's chord progression displayed as chord names (e.g., "Bb  F  Gm  Eb"). Strum pattern ComboBox selector.

## Track List

16 rows, one per track/channel. Each row contains:

| Element | Widget | Behavior |
|---------|--------|----------|
| # | Label | Channel number 1-16 |
| Name | TextEdit | Editable, 120px wide |
| Role | ComboBox | Rhythm/Lead/Counter/Bass/Drum/Pad |
| Instrument | ComboBox | Filtered by role (drum shows percussion, others show melodic) |
| Voicing | ComboBox | Poly/Mono. Hidden for drum tracks |
| M | Toggle button | Mute. Red when active |
| S | Toggle button | Solo. Yellow when active |
| 🎲 | Button | Randomize this track for current part |
| ● | Toggle button | Active in current part. Green=yes, gray=no |

Row background alternates. Selected track row has accent background. Clicking anywhere on the row (except interactive elements) selects that track.

When a track is selected, the Pattern View below shows its pattern for the currently selected part.

## Pattern View

A custom Iced canvas widget showing the selected track's pattern as a horizontal piano-roll-lite.

**X axis:** Time in beats. Bar lines at every 4 beats. Beat markers at every beat. Grid lines at 8th note resolution.

**Y axis:** MIDI note number. For drum tracks, collapse to single row. For melodic tracks, show the range of notes present ± 5 semitones.

**Notes:** Horizontal rectangles. Width = duration. Height = fixed (one semitone row). Color intensity maps to velocity (louder = more opaque/brighter).

**Playhead:** Vertical line at current beat position when playing.

This widget is read-only — no mouse interaction for editing notes. All editing is via randomize or CLI.

## Interactions

**Randomize All:** New seed → recompose → save snapshot → redraw
**Randomize Track:** New seed for that track only → recompose that track → save snapshot
**Undo:** Load previous snapshot. All GUI state updates.
**Redo:** Load next snapshot.
**Scale change:** Recompose all patterns that use the changed scale.
**Part selection:** Update track list active-part indicators, update pattern view.
**Track selection:** Update pattern view to show selected track.
**Role/instrument change:** Recompose that track for all parts.

## Iced Implementation Notes

Main app struct:
```rust
struct FolkKomposer {
    state: SharedState,  // Arc<Mutex<AppState>>
}

enum Message {
    Play, Stop, TempoChanged(f64),
    Undo, Redo,
    RandomizeAll, RandomizeTrack(usize),
    SelectTrack(usize), SelectPart(usize),
    TrackNameChanged(usize, String),
    TrackRoleChanged(usize, TrackRole),
    TrackInstrumentChanged(usize, InstrumentType),
    TrackVoicingChanged(usize, Voicing),
    TrackMuteToggle(usize), TrackSoloToggle(usize),
    TrackActiveToggle(usize),
    RhythmRootChanged(PitchClass), RhythmTypeChanged(ScaleType),
    LeadRootChanged(PitchClass), LeadTypeChanged(ScaleType),
    ScaleDegreeToggle { scale: WhichScale, degree: usize },
    StrumPatternChanged(String),
    TransportUpdate { playing: bool, tempo: f64, position: f64 },
    Tick,  // for animation/transport display refresh
}
```

Use `iced::time::every(Duration::from_millis(50))` subscription for transport display updates when playing.

Use `iced::widget::canvas::Canvas` for the pattern view with a custom `Program` impl.
