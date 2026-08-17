use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    Menu,
    Playing,
    Dead,
}

/// Closest-point circle vs. AABB test, used for bird/pipe and bird/powerup
/// collisions. Cheap, exact, and doesn't need macroquad's `Rect::overlaps`
/// (which is AABB-only) to fake curved collision shapes.
pub fn circle_rect_overlap(circle_center: Vec2, radius: f32, rect: Rect) -> bool {
    let closest_x = circle_center.x.clamp(rect.x, rect.x + rect.w);
    let closest_y = circle_center.y.clamp(rect.y, rect.y + rect.h);
    let dx = circle_center.x - closest_x;
    let dy = circle_center.y - closest_y;
    (dx * dx + dy * dy) <= radius * radius
}

/// Rotate a point around a pivot by `angle` radians. Used everywhere we draw
/// procedural (non-textured) shapes that need to follow the bird's rotation.
pub fn rotate_point(p: Vec2, pivot: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    let t = p - pivot;
    pivot + vec2(t.x * c - t.y * s, t.x * s + t.y * c)
}

/// Draws a filled, rotated ellipse by fanning triangles out from its center.
/// Macroquad's built-in `draw_poly` only supports regular (equal-radius)
/// polygons, so this is what gives the bird its squash-and-stretch silhouette.
pub fn draw_ellipse_rotated(center: Vec2, rx: f32, ry: f32, rotation: f32, sides: u32, color: Color) {
    let (s, c) = rotation.sin_cos();
    let mut prev: Option<Vec2> = None;
    let first_point = {
        let (x, y) = (rx, 0.0);
        vec2(center.x + x * c - y * s, center.y + x * s + y * c)
    };
    for i in 1..=sides {
        let theta = (i as f32 / sides as f32) * std::f32::consts::TAU;
        let (lx, ly) = (theta.cos() * rx, theta.sin() * ry);
        let p = vec2(center.x + lx * c - ly * s, center.y + lx * s + ly * c);
        let prev_p = prev.unwrap_or(first_point);
        draw_triangle(center, prev_p, p, color);
        prev = Some(p);
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        lerp(a.r, b.r, t),
        lerp(a.g, b.g, t),
        lerp(a.b, b.b, t),
        lerp(a.a, b.a, t),
    )
}

/// Frame-rate independent exponential smoothing factor, the standard trick
/// for turning a "lerp speed" constant into a stable `lerp(current, target, k)`
/// regardless of dt. See Freya Holmer's "lerp smoothing is broken" talk.
pub fn smoothing(speed: f32, dt: f32) -> f32 {
    1.0 - (-speed * dt).exp()
}
