use crate::constants::*;
use macroquad::prelude::*;

struct FloatingText {
    pos: Vec2,
    text: String,
    life: f32,
    color: Color,
}

pub struct PopupManager {
    popups: Vec<FloatingText>,
}

impl PopupManager {
    pub fn new() -> Self {
        Self { popups: Vec::new() }
    }

    pub fn spawn(&mut self, pos: Vec2, text: &str, color: Color) {
        self.popups.push(FloatingText {
            pos,
            text: text.to_string(),
            life: POPUP_LIFETIME,
            color,
        });
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.popups {
            p.pos.y -= POPUP_RISE_SPEED * dt;
            p.life -= dt;
        }
        self.popups.retain(|p| p.life > 0.0);
    }

    pub fn draw(&self, offset: Vec2) {
        for p in &self.popups {
            let t = (p.life / POPUP_LIFETIME).clamp(0.0, 1.0);
            let color = Color::new(p.color.r, p.color.g, p.color.b, p.color.a * t);
            let font_size = 28.0 + (1.0 - t) * 8.0; // slight grow-as-it-fades punch
            let dims = measure_text(&p.text, None, font_size as u16, 1.0);
            draw_text(
                &p.text,
                p.pos.x - dims.width / 2.0 + offset.x,
                p.pos.y + offset.y,
                font_size,
                color,
            );
        }
    }
}
