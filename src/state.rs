use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::engine::composer::Composer;
use crate::engine::song::Song;

/// Unix socket path for CLI ↔ GUI IPC.
pub const IPC_SOCKET_PATH: &str = "/tmp/copper-hollow.sock";

/// Thread-safe shared state handle.
pub type SharedState = Arc<Mutex<AppState>>;

/// Create a new shared state with the given seed.
pub fn new_shared(seed: u64) -> SharedState {
    Arc::new(Mutex::new(AppState::new(seed)))
}

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
#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl AppState {
    /// Create a new AppState with the default song and given seed.
    /// Automatically takes an initial snapshot.
    pub fn new(seed: u64) -> Self {
        let song = Song::default_song();
        let tempo = song.tempo;
        let snapshot = Snapshot {
            song: song.clone(),
            seed,
            label: "Initial state".to_string(),
            timestamp: Self::now_millis(),
        };
        Self {
            song,
            composer: Composer::new(seed),
            history: vec![snapshot],
            history_index: 0,
            is_playing: false,
            tempo,
            beat_position: 0.0,
            bar_position: 0,
            bitwig_connected: false,
            selected_track: 0,
            selected_part_index: 0,
            seed_counter: seed,
        }
    }

    /// Get the current unix timestamp in milliseconds.
    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Capture the current state as a snapshot.
    /// Truncates any redo history beyond the current index.
    pub fn take_snapshot(&mut self, label: &str) {
        // Discard any snapshots after current position (redo history)
        self.history.truncate(self.history_index + 1);

        let snapshot = Snapshot {
            song: self.song.clone(),
            seed: self.composer.seed(),
            label: label.to_string(),
            timestamp: Self::now_millis(),
        };

        self.history.push(snapshot);
        self.history_index = self.history.len() - 1;
    }

    /// Undo: move back one snapshot. Returns `true` if successful.
    pub fn undo(&mut self) -> bool {
        if !self.can_undo() {
            return false;
        }
        self.history_index -= 1;
        self.restore_from_current();
        true
    }

    /// Redo: move forward one snapshot. Returns `true` if successful.
    pub fn redo(&mut self) -> bool {
        if !self.can_redo() {
            return false;
        }
        self.history_index += 1;
        self.restore_from_current();
        true
    }

    /// Jump to a specific snapshot index. Returns `true` if successful.
    pub fn goto_snapshot(&mut self, index: usize) -> bool {
        if index >= self.history.len() {
            return false;
        }
        self.history_index = index;
        self.restore_from_current();
        true
    }

    /// Restore song and composer from the snapshot at `history_index`.
    fn restore_from_current(&mut self) {
        if let Some(snapshot) = self.history.get(self.history_index) {
            self.song = snapshot.song.clone();
            self.tempo = self.song.tempo;
            self.composer = Composer::new(snapshot.seed);
            self.seed_counter = snapshot.seed;
        }
    }

    /// Whether undo is available.
    pub fn can_undo(&self) -> bool {
        self.history_index > 0
    }

    /// Whether redo is available.
    pub fn can_redo(&self) -> bool {
        self.history_index < self.history.len().saturating_sub(1)
    }

    /// Number of snapshots in history.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Get the current snapshot, if any.
    pub fn current_snapshot(&self) -> Option<&Snapshot> {
        self.history.get(self.history_index)
    }
}

// ---------------------------------------------------------------------------
// IPC protocol types for CLI ↔ GUI communication over Unix socket
// ---------------------------------------------------------------------------

/// Request from CLI to GUI process over Unix socket.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpcRequest {
    /// The CLI command name (e.g. "get-state", "undo").
    pub command: String,
    /// Command arguments as a JSON value.
    #[serde(default)]
    pub args: serde_json::Value,
}

/// Response from GUI process to CLI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IpcResponse {
    /// Successful response with data.
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    /// Successful response with no data.
    pub fn success_empty() -> Self {
        Self {
            ok: true,
            data: None,
            error: None,
        }
    }

    /// Error response.
    pub fn error(msg: &str) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_initial_snapshot() {
        let state = AppState::new(42);
        assert_eq!(state.history_len(), 1);
        assert_eq!(state.history_index, 0);
        assert_eq!(state.seed_counter, 42);
        assert_eq!(state.composer.seed(), 42);
        assert!(!state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn new_state_defaults() {
        let state = AppState::new(1);
        assert!(!state.is_playing);
        assert!((state.tempo - 120.0).abs() < f64::EPSILON);
        assert!((state.beat_position - 0.0).abs() < f64::EPSILON);
        assert_eq!(state.bar_position, 0);
        assert!(!state.bitwig_connected);
        assert_eq!(state.selected_track, 0);
        assert_eq!(state.selected_part_index, 0);
        assert_eq!(state.song.title, "Untitled Folk Song");
    }

    #[test]
    fn take_snapshot_adds_to_history() {
        let mut state = AppState::new(42);
        assert_eq!(state.history_len(), 1);

        state.song.title = "Changed".to_string();
        state.take_snapshot("Changed title");
        assert_eq!(state.history_len(), 2);
        assert_eq!(state.history_index, 1);
        assert_eq!(state.history[1].label, "Changed title");
        assert_eq!(state.history[1].song.title, "Changed");
    }

    #[test]
    fn undo_restores_previous_snapshot() {
        let mut state = AppState::new(42);
        let original_title = state.song.title.clone();

        state.song.title = "Modified".to_string();
        state.take_snapshot("modify");

        assert_eq!(state.song.title, "Modified");
        assert!(state.can_undo());

        assert!(state.undo());
        assert_eq!(state.song.title, original_title);
        assert_eq!(state.history_index, 0);
    }

    #[test]
    fn redo_restores_next_snapshot() {
        let mut state = AppState::new(42);

        state.song.title = "Modified".to_string();
        state.take_snapshot("modify");

        state.undo();
        assert!(state.can_redo());

        assert!(state.redo());
        assert_eq!(state.song.title, "Modified");
        assert_eq!(state.history_index, 1);
    }

    #[test]
    fn undo_at_start_returns_false() {
        let mut state = AppState::new(42);
        assert!(!state.undo());
    }

    #[test]
    fn redo_at_end_returns_false() {
        let mut state = AppState::new(42);
        assert!(!state.redo());

        state.take_snapshot("snap");
        assert!(!state.redo());
    }

    #[test]
    fn take_snapshot_after_undo_truncates_redo() {
        let mut state = AppState::new(42);

        state.song.title = "A".to_string();
        state.take_snapshot("snap A");

        state.song.title = "B".to_string();
        state.take_snapshot("snap B");

        state.song.title = "C".to_string();
        state.take_snapshot("snap C");

        // history: [Initial, A, B, C], index=3
        assert_eq!(state.history_len(), 4);

        // Undo twice -> index=1 (snap A)
        state.undo();
        state.undo();
        assert_eq!(state.history_index, 1);
        assert_eq!(state.song.title, "A");

        // New snapshot should truncate B and C
        state.song.title = "D".to_string();
        state.take_snapshot("snap D");

        assert_eq!(state.history_len(), 3); // [Initial, A, D]
        assert_eq!(state.history_index, 2);
        assert_eq!(state.history[2].song.title, "D");
        assert!(!state.can_redo());
    }

    #[test]
    fn goto_snapshot_works() {
        let mut state = AppState::new(42);

        state.song.title = "First".to_string();
        state.take_snapshot("first");

        state.song.title = "Second".to_string();
        state.take_snapshot("second");

        state.song.title = "Third".to_string();
        state.take_snapshot("third");

        assert!(state.goto_snapshot(1));
        assert_eq!(state.song.title, "First");
        assert_eq!(state.history_index, 1);

        assert!(state.goto_snapshot(3));
        assert_eq!(state.song.title, "Third");
        assert_eq!(state.history_index, 3);
    }

    #[test]
    fn goto_snapshot_out_of_bounds() {
        let mut state = AppState::new(42);
        assert!(!state.goto_snapshot(5));
        assert_eq!(state.history_index, 0);
    }

    #[test]
    fn current_snapshot_matches_index() {
        let mut state = AppState::new(42);

        state.song.title = "Modified".to_string();
        state.take_snapshot("modified");

        let snap = state.current_snapshot().expect("should have snapshot");
        assert_eq!(snap.song.title, "Modified");
        assert_eq!(snap.label, "modified");
    }

    #[test]
    fn snapshot_preserves_seed() {
        let mut state = AppState::new(42);

        state.seed_counter = 99;
        state.composer = Composer::new(99);
        state.take_snapshot("new seed");

        state.undo();
        assert_eq!(state.composer.seed(), 42);
        assert_eq!(state.seed_counter, 42);

        state.redo();
        assert_eq!(state.composer.seed(), 99);
        assert_eq!(state.seed_counter, 99);
    }

    #[test]
    fn shared_state_accessible_from_multiple_threads() {
        let shared = new_shared(42);

        let shared_clone = Arc::clone(&shared);
        let handle = std::thread::spawn(move || {
            let mut state = shared_clone.lock().expect("lock failed");
            state.song.title = "Thread modified".to_string();
            state.take_snapshot("thread edit");
        });

        handle.join().expect("thread panicked");

        let state = shared.lock().expect("lock failed");
        assert_eq!(state.song.title, "Thread modified");
        assert_eq!(state.history_len(), 2);
    }

    #[test]
    fn app_state_serde_roundtrip() {
        let mut state = AppState::new(42);
        state.song.title = "Serde Test".to_string();
        state.take_snapshot("serde");

        let json = serde_json::to_string(&state).expect("serialize");
        let parsed: AppState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.song.title, "Serde Test");
        assert_eq!(parsed.history_len(), 2);
        assert_eq!(parsed.history_index, 1);
        assert_eq!(parsed.composer.seed(), 42);
        assert_eq!(parsed.seed_counter, 42);
    }

    #[test]
    fn snapshot_serde_roundtrip() {
        let snapshot = Snapshot {
            song: Song::default_song(),
            seed: 123,
            label: "test".to_string(),
            timestamp: 1700000000000,
        };
        let json = serde_json::to_string(&snapshot).expect("serialize");
        let parsed: Snapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.seed, 123);
        assert_eq!(parsed.label, "test");
        assert_eq!(parsed.timestamp, 1700000000000);
        assert_eq!(parsed.song.title, snapshot.song.title);
    }

    #[test]
    fn ipc_request_serde_roundtrip() {
        let req = IpcRequest {
            command: "get-state".to_string(),
            args: serde_json::json!({}),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: IpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.command, "get-state");
    }

    #[test]
    fn ipc_request_default_args() {
        let json = r#"{"command":"undo"}"#;
        let parsed: IpcRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(parsed.command, "undo");
        assert!(parsed.args.is_null());
    }

    #[test]
    fn ipc_response_success() {
        let resp = IpcResponse::success(serde_json::json!({"tempo": 120.0}));
        assert!(resp.ok);
        assert!(resp.data.is_some());
        assert!(resp.error.is_none());

        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(!json.contains("error")); // skip_serializing_if
    }

    #[test]
    fn ipc_response_success_empty() {
        let resp = IpcResponse::success_empty();
        assert!(resp.ok);
        assert!(resp.data.is_none());
        assert!(resp.error.is_none());
    }

    #[test]
    fn ipc_response_error() {
        let resp = IpcResponse::error("Track index out of range");
        assert!(!resp.ok);
        assert!(resp.data.is_none());
        assert_eq!(resp.error.as_deref(), Some("Track index out of range"));
    }

    #[test]
    fn ipc_response_serde_roundtrip() {
        let resp = IpcResponse::error("bad");
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: IpcResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(!parsed.ok);
        assert_eq!(parsed.error.as_deref(), Some("bad"));
    }

    #[test]
    fn ipc_socket_path_is_correct() {
        assert_eq!(IPC_SOCKET_PATH, "/tmp/copper-hollow.sock");
    }

    #[test]
    fn multiple_undo_redo_cycle() {
        let mut state = AppState::new(1);

        for i in 0..5 {
            state.song.tempo = 100.0 + i as f64;
            state.take_snapshot(&format!("tempo {}", 100 + i));
        }
        // history: [Initial(120), 100, 101, 102, 103, 104], index=5
        assert_eq!(state.history_len(), 6);
        assert!((state.song.tempo - 104.0).abs() < f64::EPSILON);

        // Undo all the way back
        for _ in 0..5 {
            assert!(state.undo());
        }
        assert!(!state.undo()); // at start
        assert!((state.song.tempo - 120.0).abs() < f64::EPSILON);

        // Redo all the way forward
        for _ in 0..5 {
            assert!(state.redo());
        }
        assert!(!state.redo()); // at end
        assert!((state.song.tempo - 104.0).abs() < f64::EPSILON);
    }
}
