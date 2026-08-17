use crate::constants::HIGH_SCORE_FILE;

/// Tracks the in-run score plus the persisted best.
///
/// Persistence uses `std::fs` on native targets, which is the closest
/// native equivalent to "local storage" for a desktop build. That API does
/// not exist on `wasm32-unknown-unknown` (there's no filesystem in the
/// browser sandbox); if you ship this to the web, replace `load`/`save`
/// with a small JS interop shim (e.g. the `quad-storage` crate, which
/// wraps the browser's real `localStorage`) — the call sites below are the
/// only places that would need to change.
pub struct ScoreManager {
    pub current: u32,
    pub high: u32,
}

impl ScoreManager {
    pub fn new() -> Self {
        Self {
            current: 0,
            high: Self::load_high_score(),
        }
    }

    pub fn reset_run(&mut self) {
        self.current = 0;
    }

    /// Increments the score and returns true the moment a new high score is
    /// set (i.e. exactly on the frame it's broken), so callers can trigger
    /// a one-shot confetti burst instead of firing every frame afterward.
    pub fn add_point(&mut self) -> bool {
        self.current += 1;
        if self.current > self.high {
            self.high = self.current;
            self.save_high_score();
            true
        } else {
            false
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_high_score() -> u32 {
        std::fs::read_to_string(HIGH_SCORE_FILE)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0)
    }

    #[cfg(target_arch = "wasm32")]
    fn load_high_score() -> u32 {
        0
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_high_score(&self) {
        if let Err(e) = std::fs::write(HIGH_SCORE_FILE, self.high.to_string()) {
            eprintln!("[score] failed to persist high score: {e}");
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save_high_score(&self) {
        // No-op on web builds; see the module doc comment above.
    }
}
