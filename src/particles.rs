use crate::constants::*;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum ParticleShape {
    Circle,
    Rect,
    /// A little quad that spins as it falls; used for confetti.
    Confetti,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    gravity: f32,
    life: f32,
    max_life: f32,
    size: f32,
    color: Color,
    shape: ParticleShape,
    spin: f32,
    angle: f32,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.vel.y += p.gravity * dt;
            p.pos += p.vel * dt;
            p.angle += p.spin * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    pub fn draw(&self, offset: Vec2) {
        for p in &self.particles {
            let t = (p.life / p.max_life).clamp(0.0, 1.0);
            let color = Color::new(p.color.r, p.color.g, p.color.b, p.color.a * t);
            let pos = p.pos + offset;
            match p.shape {
                ParticleShape::Circle => draw_circle(pos.x, pos.y, p.size * t.max(0.2), color),
                ParticleShape::Rect | ParticleShape::Confetti => {
                    let half = p.size * 0.5;
                    let corners = [
                        vec2(-half, -half * 0.5),
                        vec2(half, -half * 0.5),
                        vec2(half, half * 0.5),
                        vec2(-half, half * 0.5),
                    ];
                    let (s, c) = p.angle.sin_cos();
                    let rotated: Vec<Vec2> = corners
                        .iter()
                        .map(|v| pos + vec2(v.x * c - v.y * s, v.x * s + v.y * c))
                        .collect();
                    draw_triangle(rotated[0], rotated[1], rotated[2], color);
                    draw_triangle(rotated[0], rotated[2], rotated[3], color);
                }
            }
        }
    }

    /// Downward burst of feathers, triggered every time the jump key fires.
    pub fn spawn_feathers(&mut self, origin: Vec2) {
        for _ in 0..FEATHER_BURST_COUNT {
            let angle = rand::gen_range(60.0_f32.to_radians(), 120.0_f32.to_radians());
            let speed = rand::gen_range(60.0, 160.0);
            let vel = vec2(angle.cos(), angle.sin()) * speed + vec2(rand::gen_range(-20.0, 20.0), 0.0);
            self.particles.push(Particle {
                pos: origin,
                vel,
                gravity: 260.0,
                life: rand::gen_range(0.25, 0.5),
                max_life: 0.5,
                size: rand::gen_range(4.0, 8.0),
                color: Color::new(1.0, 0.92, 0.75, 0.9),
                shape: ParticleShape::Rect,
                spin: rand::gen_range(-6.0, 6.0),
                angle: rand::gen_range(0.0, std::f32::consts::TAU),
            });
        }
    }

    /// Radial explosion at the point of death.
    pub fn spawn_explosion(&mut self, origin: Vec2) {
        for _ in 0..DEATH_EXPLOSION_COUNT {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(120.0, 420.0);
            let vel = vec2(angle.cos(), angle.sin()) * speed;
            let warm = rand::gen_range(0.0, 1.0) > 0.5;
            let color = if warm {
                Color::new(1.0, 0.55, 0.15, 1.0)
            } else {
                Color::new(1.0, 0.82, 0.2, 1.0)
            };
            self.particles.push(Particle {
                pos: origin,
                vel,
                gravity: 500.0,
                life: rand::gen_range(0.4, 0.9),
                max_life: 0.9,
                size: rand::gen_range(3.0, 7.0),
                color,
                shape: ParticleShape::Circle,
                spin: 0.0,
                angle: 0.0,
            });
        }
    }

    /// Colorful confetti burst for a new high score.
    pub fn spawn_confetti(&mut self, origin: Vec2) {
        const PALETTE: [Color; 5] = [
            Color::new(1.0, 0.25, 0.35, 1.0),
            Color::new(0.25, 0.75, 1.0, 1.0),
            Color::new(1.0, 0.85, 0.15, 1.0),
            Color::new(0.45, 0.9, 0.35, 1.0),
            Color::new(0.8, 0.4, 1.0, 1.0),
        ];
        for _ in 0..CONFETTI_BURST_COUNT {
            let angle = rand::gen_range(200.0_f32.to_radians(), 340.0_f32.to_radians());
            let speed = rand::gen_range(150.0, 380.0);
            let vel = vec2(angle.cos(), angle.sin()) * speed;
            let color = PALETTE[rand::gen_range(0, PALETTE.len() as i32) as usize];
            self.particles.push(Particle {
                pos: origin,
                vel,
                gravity: 340.0,
                life: rand::gen_range(0.8, 1.6),
                max_life: 1.6,
                size: rand::gen_range(5.0, 9.0),
                color,
                shape: ParticleShape::Confetti,
                spin: rand::gen_range(-10.0, 10.0),
                angle: rand::gen_range(0.0, std::f32::consts::TAU),
            });
        }
    }
}
