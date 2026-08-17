use crate::constants::*;
use crate::state::smoothing;
use macroquad::prelude::*;

/// All of the "screen feel" effects live here: trauma-based shake, a short
/// freeze-frame (hitstop) on death, and a lightly-lagging camera that gives
/// the world a bit of inertia instead of rigidly tracking the bird.
pub struct CameraFx {
    /// 0..1 "trauma" value. Shake magnitude is trauma^2, which gives a much
    /// punchier falloff than a flat linear decay (a common juice trick).
    trauma: f32,
    shake_offset: Vec2,
    hitstop_frames_left: u32,
    /// Smoothed vertical camera offset that trails the bird's velocity.
    inertia_offset: f32,
}

impl CameraFx {
    pub fn new() -> Self {
        Self {
            trauma: 0.0,
            shake_offset: Vec2::ZERO,
            hitstop_frames_left: 0,
            inertia_offset: 0.0,
        }
    }

    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    pub fn trigger_hitstop(&mut self, frames: u32) {
        self.hitstop_frames_left = frames;
    }

    /// Returns true while the game should render but skip gameplay logic.
    pub fn is_frozen(&self) -> bool {
        self.hitstop_frames_left > 0
    }

    /// Call once per real frame (even while frozen, so the freeze eventually
    /// ends). Returns the dt gameplay code should actually simulate with:
    /// zero while frozen, the real dt otherwise.
    pub fn tick(&mut self, dt: f32, bird_vel: f32) -> f32 {
        if self.hitstop_frames_left > 0 {
            self.hitstop_frames_left -= 1;
            self.update_shake(dt);
            return 0.0;
        }
        self.update_shake(dt);

        // Camera inertia: let the offset lag behind the bird's vertical
        // speed, then relax back to zero. Purely cosmetic, applied as an
        // extra draw-time offset alongside shake.
        let target = (bird_vel * 0.03).clamp(-18.0, 18.0);
        let k = smoothing(CAMERA_INERTIA_LERP, dt);
        self.inertia_offset += (target - self.inertia_offset) * k;

        dt
    }

    fn update_shake(&mut self, dt: f32) {
        self.trauma = (self.trauma - SHAKE_DECAY * dt).max(0.0);
        let magnitude = self.trauma * self.trauma;
        if magnitude > 0.001 {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let dist = rand::gen_range(0.0, magnitude * 24.0);
            self.shake_offset = vec2(angle.cos(), angle.sin()) * dist;
        } else {
            self.shake_offset = Vec2::ZERO;
        }
    }

    /// Combined draw-time offset (shake + inertia). Add this to every world
    /// draw position for the frame.
    pub fn offset(&self) -> Vec2 {
        self.shake_offset + vec2(0.0, self.inertia_offset)
    }
}
