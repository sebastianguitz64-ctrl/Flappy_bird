//! Central place for every tunable number in the game.
//! Tweak these to change the feel without hunting through gameplay code.

// ---- World / physics ------------------------------------------------------
pub const GRAVITY: f32 = 1500.0; // px/s^2
pub const JUMP_VELOCITY: f32 = -480.0; // px/s, negative = up
pub const MAX_FALL_SPEED: f32 = 900.0;
pub const GROUND_HEIGHT: f32 = 90.0;

// ---- Bird -------------------------------------------------------------
pub const BIRD_RADIUS: f32 = 18.0;
pub const BIRD_X_FRACTION: f32 = 0.30; // fraction of screen width from the left
pub const BIRD_MAX_ROTATION_UP: f32 = -35.0_f32.to_radians();
pub const BIRD_MAX_ROTATION_DOWN: f32 = 90.0_f32.to_radians();
pub const BIRD_ROTATION_LERP_SPEED: f32 = 12.0; // higher = snappier
pub const BIRD_SQUASH_SCALE_X: f32 = 0.68;
pub const BIRD_SQUASH_SCALE_Y: f32 = 1.32;
pub const BIRD_SQUASH_RECOVER_SPEED: f32 = 9.0; // how fast scale relaxes to 1.0

// ---- Pipes --------------------------------------------------------------
pub const PIPE_WIDTH: f32 = 70.0;
pub const PIPE_SPEED: f32 = 220.0; // px/s, scaled by the world speed multiplier
pub const PIPE_SPACING: f32 = 260.0; // horizontal distance between pipe pairs
pub const PIPE_GAP_MIN: f32 = 165.0;
pub const PIPE_GAP_MAX: f32 = 190.0;
pub const PIPE_GAP_MARGIN: f32 = 90.0; // keep gap away from the very top/ground

// ---- Day / night cycle -----------------------------------------------
pub const PIPES_PER_DAY_NIGHT_SWITCH: u32 = 10;
pub const DAY_NIGHT_TRANSITION_SECONDS: f32 = 2.5;

// ---- Power-ups ------------------------------------------------------------
pub const POWERUP_SPAWN_CHANCE: f32 = 0.22; // per pipe pair
pub const POWERUP_RADIUS: f32 = 14.0;
pub const SLOWMO_DURATION: f32 = 5.0;
pub const SLOWMO_WORLD_SPEED_MULT: f32 = 0.7; // "30% slower"
pub const SHRINK_DURATION: f32 = 6.0;
pub const SHRINK_COLLIDER_MULT: f32 = 0.5;

// ---- Camera / juice -----------------------------------------------------
pub const SHAKE_DECAY: f32 = 6.0; // how fast shake magnitude decays
pub const SHAKE_PIPE_HIT: f32 = 10.0;
pub const SHAKE_GROUND_HIT: f32 = 16.0;
pub const HITSTOP_FRAMES_ON_DEATH: u32 = 5;
pub const CAMERA_INERTIA_LERP: f32 = 4.0; // lower = laggier / floatier follow

// ---- Particles ------------------------------------------------------------
pub const FEATHER_BURST_COUNT: usize = 6;
pub const DEATH_EXPLOSION_COUNT: usize = 28;
pub const CONFETTI_BURST_COUNT: usize = 40;

// ---- Score popup ------------------------------------------------------
pub const POPUP_LIFETIME: f32 = 0.9;
pub const POPUP_RISE_SPEED: f32 = 55.0;

// ---- Misc -----------------------------------------------------------------
pub const HIGH_SCORE_FILE: &str = "highscore.txt";
