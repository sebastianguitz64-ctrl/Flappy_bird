use crate::constants::*;
use crate::state::circle_rect_overlap;
use macroquad::prelude::*;

pub struct Pipe {
    pub x: f32,
    pub gap_center_y: f32,
    pub gap_size: f32,
    pub passed: bool,
}

impl Pipe {
    fn top_rect(&self) -> Rect {
        let bottom = self.gap_center_y - self.gap_size / 2.0;
        Rect::new(self.x, 0.0, PIPE_WIDTH, bottom.max(0.0))
    }

    fn bottom_rect(&self, ground_y: f32) -> Rect {
        let top = self.gap_center_y + self.gap_size / 2.0;
        Rect::new(self.x, top, PIPE_WIDTH, (ground_y - top).max(0.0))
    }

    pub fn gap_center(&self) -> Vec2 {
        vec2(self.x + PIPE_WIDTH / 2.0, self.gap_center_y)
    }

    fn draw(&self, ground_y: f32, offset: Vec2, palette: PipeColors) {
        let top = self.top_rect();
        let bottom = self.bottom_rect(ground_y);

        for rect in [top, bottom] {
            if rect.h <= 0.0 {
                continue;
            }
            let x = rect.x + offset.x;
            let y = rect.y + offset.y;
            draw_rectangle(x, y, rect.w, rect.h, palette.body);
            draw_rectangle_lines(x, y, rect.w, rect.h, 3.0, palette.outline);
        }

        // Pipe "caps" (the lip at the mouth of each pipe) for a less flat look.
        let cap_h = 26.0;
        let cap_overhang = 6.0;
        if top.h > 0.0 {
            draw_rectangle(
                top.x - cap_overhang + offset.x,
                top.y + top.h - cap_h + offset.y,
                top.w + cap_overhang * 2.0,
                cap_h,
                palette.cap,
            );
        }
        if bottom.h > 0.0 {
            draw_rectangle(
                bottom.x - cap_overhang + offset.x,
                bottom.y + offset.y,
                bottom.w + cap_overhang * 2.0,
                cap_h,
                palette.cap,
            );
        }
    }
}

#[derive(Clone, Copy)]
struct PipeColors {
    body: Color,
    outline: Color,
    cap: Color,
}

pub enum PipeEvent {
    /// Bird flew past this pipe pair; carries the point where a "+1" popup
    /// should appear.
    Passed(Vec2),
    /// A new pipe pair was just spawned; carries the gap center, letting the
    /// power-up system decide whether to place a collectible there.
    Spawned(Vec2),
}

pub struct PipeManager {
    pub pipes: Vec<Pipe>,
    spawn_timer: f32,
}

impl PipeManager {
    pub fn new() -> Self {
        Self {
            pipes: Vec::new(),
            spawn_timer: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.pipes.clear();
        self.spawn_timer = 0.0;
    }

    /// `dt` should already be scaled by the world speed multiplier (slow-mo
    /// etc.) by the caller.
    pub fn update(&mut self, dt: f32, ground_y: f32, bird_x: f32) -> Vec<PipeEvent> {
        let mut events = Vec::new();
        let scaled_speed = PIPE_SPEED;

        for pipe in &mut self.pipes {
            pipe.x -= scaled_speed * dt;
            if !pipe.passed && pipe.x + PIPE_WIDTH < bird_x {
                pipe.passed = true;
                events.push(PipeEvent::Passed(pipe.gap_center()));
            }
        }
        self.pipes.retain(|p| p.x + PIPE_WIDTH > -20.0);

        self.spawn_timer -= scaled_speed * dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer = PIPE_SPACING;
            let gap_size = rand::gen_range(PIPE_GAP_MIN, PIPE_GAP_MAX);
            let usable_top = PIPE_GAP_MARGIN;
            let usable_bottom = ground_y - PIPE_GAP_MARGIN;
            let gap_center_y = rand::gen_range(
                usable_top + gap_size / 2.0,
                (usable_bottom - gap_size / 2.0).max(usable_top + gap_size / 2.0 + 1.0),
            );
            let pipe = Pipe {
                x: screen_width() + PIPE_WIDTH,
                gap_center_y,
                gap_size,
                passed: false,
            };
            events.push(PipeEvent::Spawned(pipe.gap_center()));
            self.pipes.push(pipe);
        }

        events
    }

    /// Returns Some(hit_point) for the first pipe the bird overlaps, or None.
    pub fn check_collision(&self, bird_pos: Vec2, bird_radius: f32, ground_y: f32) -> Option<Vec2> {
        for pipe in &self.pipes {
            let top = pipe.top_rect();
            let bottom = pipe.bottom_rect(ground_y);
            if top.h > 0.0 && circle_rect_overlap(bird_pos, bird_radius, top) {
                return Some(bird_pos);
            }
            if bottom.h > 0.0 && circle_rect_overlap(bird_pos, bird_radius, bottom) {
                return Some(bird_pos);
            }
        }
        None
    }

    pub fn draw(&self, ground_y: f32, offset: Vec2, is_night: bool) {
        let palette = if is_night {
            PipeColors {
                body: Color::new(0.16, 0.42, 0.24, 1.0),
                outline: Color::new(0.08, 0.22, 0.13, 1.0),
                cap: Color::new(0.20, 0.50, 0.28, 1.0),
            }
        } else {
            PipeColors {
                body: Color::new(0.30, 0.72, 0.28, 1.0),
                outline: Color::new(0.16, 0.45, 0.15, 1.0),
                cap: Color::new(0.38, 0.82, 0.34, 1.0),
            }
        };
        for pipe in &self.pipes {
            pipe.draw(ground_y, offset, palette);
        }
    }
}
