use crate::bird::Bird;
use crate::constants::*;
use crate::state::circle_rect_overlap;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PowerUpKind {
    Shield,
    SlowMo,
    Shrink,
}

impl PowerUpKind {
    fn color(self) -> Color {
        match self {
            PowerUpKind::Shield => Color::new(0.35, 0.75, 1.0, 1.0),
            PowerUpKind::SlowMo => Color::new(0.75, 0.4, 1.0, 1.0),
            PowerUpKind::Shrink => Color::new(1.0, 0.85, 0.25, 1.0),
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            PowerUpKind::Shield => "S",
            PowerUpKind::SlowMo => "Z",
            PowerUpKind::Shrink => "V",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PowerUpKind::Shield => "Shield",
            PowerUpKind::SlowMo => "Slow-Mo",
            PowerUpKind::Shrink => "Shrink",
        }
    }
}

struct PowerUpEntity {
    pos: Vec2,
    kind: PowerUpKind,
    bob_phase: f32,
}

pub struct PowerUpManager {
    entities: Vec<PowerUpEntity>,
    pub slowmo_timer: f32,
}

pub enum PowerUpPickupEvent {
    Collected(PowerUpKind, Vec2),
}

impl PowerUpManager {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            slowmo_timer: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.entities.clear();
        self.slowmo_timer = 0.0;
    }

    /// Called when the pipe manager spawns a new pair; rolls the dice on
    /// whether to place a collectible at the gap center.
    pub fn maybe_spawn(&mut self, gap_center: Vec2) {
        if rand::gen_range(0.0, 1.0) > POWERUP_SPAWN_CHANCE {
            return;
        }
        let roll = rand::gen_range(0, 3);
        let kind = match roll {
            0 => PowerUpKind::Shield,
            1 => PowerUpKind::SlowMo,
            _ => PowerUpKind::Shrink,
        };
        self.entities.push(PowerUpEntity {
            pos: gap_center,
            kind,
            bob_phase: rand::gen_range(0.0, std::f32::consts::TAU),
        });
    }

    /// `dt` is the *scaled* delta (already reduced by slow-mo if active) so
    /// power-ups drift with the same world speed as the pipes.
    pub fn update(&mut self, dt: f32, pipe_speed: f32) {
        for e in &mut self.entities {
            e.pos.x -= pipe_speed * dt;
            e.bob_phase += dt * 3.0;
        }
        self.entities.retain(|e| e.pos.x > -40.0);

        if self.slowmo_timer > 0.0 {
            self.slowmo_timer = (self.slowmo_timer - dt).max(0.0);
        }
    }

    pub fn world_speed_mult(&self) -> f32 {
        if self.slowmo_timer > 0.0 {
            SLOWMO_WORLD_SPEED_MULT
        } else {
            1.0
        }
    }

    /// Checks bird overlap against all live power-ups, applies effects
    /// directly to the bird / self, and returns pickup events for FX/audio.
    pub fn check_pickups(&mut self, bird: &mut Bird) -> Vec<PowerUpPickupEvent> {
        let mut events = Vec::new();
        let radius = POWERUP_RADIUS;
        self.entities.retain(|e| {
            let rect = Rect::new(e.pos.x - radius, e.pos.y - radius, radius * 2.0, radius * 2.0);
            let hit = circle_rect_overlap(bird.pos, bird.collider_radius(), rect);
            if hit {
                match e.kind {
                    PowerUpKind::Shield => bird.shield = true,
                    PowerUpKind::SlowMo => self.slowmo_timer = SLOWMO_DURATION,
                    PowerUpKind::Shrink => bird.shrink_timer = SHRINK_DURATION,
                }
                events.push(PowerUpPickupEvent::Collected(e.kind, e.pos));
            }
            !hit
        });
        events
    }

    pub fn draw(&self, offset: Vec2) {
        for e in &self.entities {
            let bob = (e.bob_phase.sin()) * 5.0;
            let pos = e.pos + offset + vec2(0.0, bob);
            let pulse = 0.85 + 0.15 * (e.bob_phase * 1.5).sin();
            draw_circle(pos.x, pos.y, POWERUP_RADIUS * pulse, e.kind.color());
            draw_circle_lines(pos.x, pos.y, POWERUP_RADIUS * pulse, 2.0, WHITE);
            let dims = measure_text(e.kind.glyph(), None, 18, 1.0);
            draw_text(
                e.kind.glyph(),
                pos.x - dims.width / 2.0,
                pos.y + dims.height / 2.0,
                18.0,
                WHITE,
            );
        }
    }
}
