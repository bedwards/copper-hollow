use serde::{Deserialize, Serialize};

use crate::engine::composer::Composer;
use crate::engine::song::Song;

/// An immutable snapshot of the song state for undo/redo.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub song: Song,
    pub seed: u64,
    pub label: String,
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,
}

/// Shared application state.
/// Wrapped in `Arc<Mutex<AppState>>` for cross-thread access.
pub struct AppState {
    pub song: Song,
    pub composer: Composer,
    pub history: Vec<Snapshot>,
    pub history_index: usize,
    pub is_playing: bool,
    pub tempo: f64,
    pub beat_position: f64,
    pub bar_position: u32,
    pub bitwig_connected: bool,
    pub selected_track: usize,
    pub selected_part_index: usize,
    pub seed_counter: u64,
}
