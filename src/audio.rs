use macroquad::audio::{load_sound, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

/// # A note on "pitch variation"
///
/// Macroquad's audio backend (`quad-snd`) only exposes `PlaySoundParams {
/// looped, volume }` -- there is no real-time pitch/playback-speed control.
/// So a literal "+/-5% pitch offset per play" isn't something macroquad can
/// do on its own. This wrapper approximates the requested effect two ways:
///
/// 1. It looks for up to 3 pre-baked pitch variants per effect on disk
///    (`jump.wav`, `jump_alt1.wav`, `jump_alt2.wav`) and picks one at
///    random each time, which is how most shipped games *actually* solve
///    "audio monotony" without a DSP pitch shifter.
/// 2. It also jitters playback volume by a few percent as a cheap,
///    always-available secondary variation, so even a single-file setup
///    doesn't sound identical on every jump.
///
/// If you need *true* real-time pitch shifting, swap this module for a
/// lower-level audio crate such as `kira` or `rodio`, both of which expose
/// a `playback_rate` / `speed` control per-instance.
pub struct Sfx {
    jump_variants: Vec<Sound>,
    score: Option<Sound>,
    hit: Option<Sound>,
    powerup: Option<Sound>,
}

async fn try_load(path: &str) -> Option<Sound> {
    match load_sound(path).await {
        Ok(s) => Some(s),
        Err(_) => {
            // Missing/optional asset: log and keep running silently rather
            // than crash the game over a sound file.
            eprintln!("[audio] couldn't load '{path}', continuing without it");
            None
        }
    }
}

impl Sfx {
    pub async fn load() -> Self {
        let mut jump_variants = Vec::new();
        for path in [
            "assets/sfx/jump.wav",
            "assets/sfx/jump_alt1.wav",
            "assets/sfx/jump_alt2.wav",
        ] {
            if let Some(s) = try_load(path).await {
                jump_variants.push(s);
            }
        }

        Self {
            jump_variants,
            score: try_load("assets/sfx/score.wav").await,
            hit: try_load("assets/sfx/hit.wav").await,
            powerup: try_load("assets/sfx/powerup.wav").await,
        }
    }

    pub fn play_jump(&self) {
        if self.jump_variants.is_empty() {
            return;
        }
        let idx = rand::gen_range(0, self.jump_variants.len() as i32) as usize;
        let volume = rand::gen_range(0.90, 1.0); // stand-in for +/-5% pitch jitter
        play_sound(
            &self.jump_variants[idx],
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }

    pub fn play_score(&self) {
        if let Some(s) = &self.score {
            play_sound(s, PlaySoundParams { looped: false, volume: 1.0 });
        }
    }

    pub fn play_hit(&self) {
        if let Some(s) = &self.hit {
            play_sound(s, PlaySoundParams { looped: false, volume: 1.0 });
        }
    }

    pub fn play_powerup(&self) {
        if let Some(s) = &self.powerup {
            play_sound(s, PlaySoundParams { looped: false, volume: 1.0 });
        }
    }
}
