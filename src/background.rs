use crate::constants::*;
use crate::state::lerp_color;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Day,
    Night,
}

struct SkyPalette {
    top: Color,
    bottom: Color,
    mountains: Color,
    trees: Color,
    ground_top: Color,
    ground_bottom: Color,
}

fn day_palette() -> SkyPalette {
    SkyPalette {
        top: Color::new(0.45, 0.78, 0.95, 1.0),
        bottom: Color::new(0.78, 0.90, 0.85, 1.0),
        mountains: Color::new(0.55, 0.62, 0.72, 1.0),
        trees: Color::new(0.30, 0.55, 0.30, 1.0),
        ground_top: Color::new(0.82, 0.70, 0.35, 1.0),
        ground_bottom: Color::new(0.62, 0.48, 0.22, 1.0),
    }
}

fn night_palette() -> SkyPalette {
    SkyPalette {
        top: Color::new(0.04, 0.05, 0.18, 1.0),
        bottom: Color::new(0.12, 0.13, 0.30, 1.0),
        mountains: Color::new(0.10, 0.11, 0.22, 1.0),
        trees: Color::new(0.06, 0.14, 0.10, 1.0),
        ground_top: Color::new(0.20, 0.18, 0.28, 1.0),
        ground_bottom: Color::new(0.10, 0.09, 0.16, 1.0),
    }
}

/// A single scrolling layer of the parallax background (mountains, trees,
/// or the ground strip). Each layer just tracks how far it has scrolled and
/// how fast, relative to the world speed multiplier.
struct ParallaxLayer {
    scroll_x: f32,
    speed: f32,
}

impl ParallaxLayer {
    fn new(speed: f32) -> Self {
        Self {
            scroll_x: 0.0,
            speed,
        }
    }

    fn update(&mut self, dt: f32) {
        self.scroll_x = (self.scroll_x + self.speed * dt).rem_euclid(10_000.0);
    }
}

pub struct Background {
    mountains: ParallaxLayer,
    trees: ParallaxLayer,
    ground: ParallaxLayer,

    time_of_day: TimeOfDay,
    /// 0 = fully settled on `time_of_day`, ramps to 1 mid-transition then
    /// flips `time_of_day` and resets to 0.
    transition_t: f32,
    transitioning: bool,
    pipes_since_switch: u32,
}

impl Background {
    pub fn new() -> Self {
        Self {
            mountains: ParallaxLayer::new(18.0),
            trees: ParallaxLayer::new(45.0),
            ground: ParallaxLayer::new(PIPE_SPEED),
            time_of_day: TimeOfDay::Day,
            transition_t: 0.0,
            transitioning: false,
            pipes_since_switch: 0,
        }
    }

    pub fn reset(&mut self) {
        self.time_of_day = TimeOfDay::Day;
        self.transition_t = 0.0;
        self.transitioning = false;
        self.pipes_since_switch = 0;
    }

    pub fn update(&mut self, dt: f32) {
        self.mountains.update(dt);
        self.trees.update(dt);
        self.ground.update(dt);

        if self.transitioning {
            self.transition_t += dt / DAY_NIGHT_TRANSITION_SECONDS;
            if self.transition_t >= 1.0 {
                self.transition_t = 0.0;
                self.transitioning = false;
                self.time_of_day = match self.time_of_day {
                    TimeOfDay::Day => TimeOfDay::Night,
                    TimeOfDay::Night => TimeOfDay::Day,
                };
            }
        }
    }

    /// Call once per pipe passed; flips day/night every N pipes.
    pub fn register_pipe_passed(&mut self) {
        self.pipes_since_switch += 1;
        if self.pipes_since_switch >= PIPES_PER_DAY_NIGHT_SWITCH && !self.transitioning {
            self.pipes_since_switch = 0;
            self.transitioning = true;
        }
    }

    pub fn is_night(&self) -> bool {
        match self.time_of_day {
            TimeOfDay::Day => self.transitioning && self.transition_t > 0.5,
            TimeOfDay::Night => !self.transitioning || self.transition_t < 0.5,
        }
    }

    fn current_palette(&self) -> SkyPalette {
        let (from, to) = match self.time_of_day {
            TimeOfDay::Day => (day_palette(), night_palette()),
            TimeOfDay::Night => (night_palette(), day_palette()),
        };
        if !self.transitioning {
            from
        } else {
            let t = self.transition_t;
            SkyPalette {
                top: lerp_color(from.top, to.top, t),
                bottom: lerp_color(from.bottom, to.bottom, t),
                mountains: lerp_color(from.mountains, to.mountains, t),
                trees: lerp_color(from.trees, to.trees, t),
                ground_top: lerp_color(from.ground_top, to.ground_top, t),
                ground_bottom: lerp_color(from.ground_bottom, to.ground_bottom, t),
            }
        }
    }

    pub fn draw(&self, ground_y: f32, offset: Vec2) {
        let p = self.current_palette();
        let w = screen_width();
        let h = screen_height();

        // Sky gradient (approximated with horizontal bands; cheap and looks
        // smooth enough at typical portrait window sizes).
        let bands = 24;
        for i in 0..bands {
            let t0 = i as f32 / bands as f32;
            let t1 = (i + 1) as f32 / bands as f32;
            let y0 = t0 * ground_y;
            let y1 = t1 * ground_y;
            let color = lerp_color(p.top, p.bottom, t0);
            draw_rectangle(offset.x, y0 + offset.y, w, (y1 - y0) + 1.0, color);
        }

        // Distant mountains: a jagged silhouette built from a repeating
        // triangle strip, offset by the slowest-scrolling layer.
        let mountain_w = 140.0;
        let mountain_h = 110.0;
        let base_y = ground_y * 0.72;
        let start = -(self.mountains.scroll_x % mountain_w);
        let mut x = start;
        while x < w {
            let peak = vec2(x + mountain_w * 0.5 + offset.x, base_y - mountain_h + offset.y);
            let left = vec2(x + offset.x, base_y + offset.y);
            let right = vec2(x + mountain_w + offset.x, base_y + offset.y);
            draw_triangle(left, right, peak, p.mountains);
            x += mountain_w;
        }

        // Midground trees: simple triangle "pines" on a trunk, faster layer.
        let tree_w = 60.0;
        let tree_h = 70.0;
        let tree_base_y = ground_y * 0.92;
        let start = -(self.trees.scroll_x % tree_w);
        let mut x = start;
        while x < w {
            let trunk_w = 6.0;
            draw_rectangle(
                x + tree_w * 0.5 - trunk_w * 0.5 + offset.x,
                tree_base_y - 10.0 + offset.y,
                trunk_w,
                10.0,
                Color::new(0.35, 0.24, 0.14, 1.0),
            );
            let peak = vec2(x + tree_w * 0.5 + offset.x, tree_base_y - tree_h + offset.y);
            let left = vec2(x + tree_w * 0.15 + offset.x, tree_base_y - 10.0 + offset.y);
            let right = vec2(x + tree_w * 0.85 + offset.x, tree_base_y - 10.0 + offset.y);
            draw_triangle(left, right, peak, p.trees);
            x += tree_w;
        }

        // Ground strip with a scrolling "texture" of dashes so motion reads
        // clearly even at a glance.
        draw_rectangle(offset.x, ground_y + offset.y, w, h - ground_y, p.ground_bottom);
        draw_rectangle(offset.x, ground_y + offset.y, w, 14.0, p.ground_top);
        let dash_w = 28.0;
        let start = -(self.ground.scroll_x % (dash_w * 2.0));
        let mut x = start;
        while x < w {
            draw_rectangle(
                x + offset.x,
                ground_y + 4.0 + offset.y,
                dash_w,
                6.0,
                p.ground_bottom,
            );
            x += dash_w * 2.0;
        }

        // Stars at night, faded in with the transition.
        let night_amount = match self.time_of_day {
            TimeOfDay::Day => {
                if self.transitioning {
                    self.transition_t
                } else {
                    0.0
                }
            }
            TimeOfDay::Night => {
                if self.transitioning {
                    1.0 - self.transition_t
                } else {
                    1.0
                }
            }
        };
        if night_amount > 0.01 {
            for i in 0..40u32 {
                // Deterministic pseudo-random-looking star field based on index.
                let sx = ((i as f32 * 61.8) % w as f32 + self.mountains.scroll_x * 0.05) % w;
                let sy = (i as f32 * 37.2) % (ground_y * 0.6);
                draw_circle(
                    sx + offset.x,
                    sy + offset.y,
                    1.6,
                    Color::new(1.0, 1.0, 1.0, night_amount * 0.85),
                );
            }
        }
    }
}
