use crate::constants::*;
use crate::state::{draw_ellipse_rotated, lerp, rotate_point, smoothing};
use macroquad::prelude::*;

pub struct Bird {
    pub pos: Vec2,
    pub vel_y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,

    // Power-up state lives on the bird because it's the thing they modify.
    pub shield: bool,
    pub shrink_timer: f32,
    /// Brief grace period after the shield absorbs a hit, so the bird isn't
    /// instantly killed by the same pipe rect it's still overlapping.
    pub invuln_timer: f32,
}

impl Bird {
    pub fn new(start_pos: Vec2) -> Self {
        Self {
            pos: start_pos,
            vel_y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shield: false,
            shrink_timer: 0.0,
            invuln_timer: 0.0,
        }
    }

    pub fn jump(&mut self) {
        self.vel_y = JUMP_VELOCITY;
        // Squash-and-stretch: contract horizontally, stretch vertically on
        // the flap, then let `update` ease it back to (1.0, 1.0) each frame.
        self.scale_x = BIRD_SQUASH_SCALE_X;
        self.scale_y = BIRD_SQUASH_SCALE_Y;
    }

    pub fn update(&mut self, dt: f32) {
        self.vel_y = (self.vel_y + GRAVITY * dt).min(MAX_FALL_SPEED);
        self.pos.y += self.vel_y * dt;

        // Pitch up sharply on jump, dive down as fall speed increases.
        let target_rotation = if self.vel_y < 0.0 {
            BIRD_MAX_ROTATION_UP
        } else {
            lerp(
                0.0,
                BIRD_MAX_ROTATION_DOWN,
                self.vel_y / MAX_FALL_SPEED,
            )
        };
        let rot_k = smoothing(BIRD_ROTATION_LERP_SPEED, dt);
        self.rotation += (target_rotation - self.rotation) * rot_k;

        let scale_k = smoothing(BIRD_SQUASH_RECOVER_SPEED, dt);
        self.scale_x += (1.0 - self.scale_x) * scale_k;
        self.scale_y += (1.0 - self.scale_y) * scale_k;

        if self.shrink_timer > 0.0 {
            self.shrink_timer = (self.shrink_timer - dt).max(0.0);
        }
        if self.invuln_timer > 0.0 {
            self.invuln_timer = (self.invuln_timer - dt).max(0.0);
        }
    }

    /// Effective collision radius, respecting the Shrink power-up.
    pub fn collider_radius(&self) -> f32 {
        if self.shrink_timer > 0.0 {
            BIRD_RADIUS * SHRINK_COLLIDER_MULT
        } else {
            BIRD_RADIUS
        }
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invuln_timer > 0.0
    }

    pub fn is_shrunk(&self) -> bool {
        self.shrink_timer > 0.0
    }

    pub fn draw(&self, offset: Vec2) {
        let center = self.pos + offset;
        let rx = BIRD_RADIUS * self.scale_x;
        let ry = BIRD_RADIUS * self.scale_y;

        // Shield glow, drawn first so the bird renders on top of it.
        if self.shield {
            let pulse = 0.75 + 0.25 * (macroquad::time::get_time() as f32 * 6.0).sin();
            draw_circle(
                center.x,
                center.y,
                BIRD_RADIUS * 1.6,
                Color::new(0.35, 0.75, 1.0, 0.25 * pulse),
            );
            draw_circle_lines(
                center.x,
                center.y,
                BIRD_RADIUS * 1.6,
                2.0,
                Color::new(0.55, 0.85, 1.0, 0.8),
            );
        }

        // Body.
        let body_color = if self.is_shrunk() {
            Color::new(1.0, 0.85, 0.3, 1.0)
        } else {
            Color::new(1.0, 0.78, 0.15, 1.0)
        };
        draw_ellipse_rotated(center, rx, ry, self.rotation, 20, body_color);

        // Wing: a small darker ellipse offset toward the tail, rotates with
        // the body for a cheap but effective "flap follows pitch" look.
        let wing_local = vec2(-rx * 0.25, ry * 0.05);
        let wing_center = rotate_point(center + wing_local, center, self.rotation);
        draw_ellipse_rotated(
            wing_center,
            rx * 0.55,
            ry * 0.4,
            self.rotation,
            14,
            Color::new(0.85, 0.55, 0.05, 1.0),
        );

        // Beak.
        let beak_tip = rotate_point(center + vec2(rx + 10.0, 0.0), center, self.rotation);
        let beak_top = rotate_point(center + vec2(rx * 0.6, -ry * 0.35), center, self.rotation);
        let beak_bot = rotate_point(center + vec2(rx * 0.6, ry * 0.35), center, self.rotation);
        draw_triangle(beak_top, beak_bot, beak_tip, ORANGE);

        // Eye.
        let eye_pos = rotate_point(center + vec2(rx * 0.35, -ry * 0.35), center, self.rotation);
        draw_circle(eye_pos.x, eye_pos.y, 4.5, WHITE);
        draw_circle(eye_pos.x + 1.0, eye_pos.y, 2.3, BLACK);
    }
}
