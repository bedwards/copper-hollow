pub mod arrangement;
pub mod bass;
pub mod composer;
pub mod drums;
pub mod melody;
pub mod pads;
pub mod rhythm;
pub mod song;
pub mod theory;

/// MIDI ticks per beat (quarter note).
#[allow(dead_code)]
pub const TICKS_PER_BEAT: u32 = 480;
/// MIDI ticks per bar in 4/4 time.
#[allow(dead_code)]
pub const TICKS_PER_BAR: u32 = 1920;
/// MIDI note number for Middle C (C3 in Bitwig).
#[allow(dead_code)]
pub const MIDDLE_C: u8 = 60;
/// Pitch bend center value.
#[allow(dead_code)]
pub const PITCH_BEND_CENTER: u16 = 8192;
