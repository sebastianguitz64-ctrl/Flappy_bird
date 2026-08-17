mod audio;
mod background;
mod bird;
mod camera_fx;
mod constants;
mod particles;
mod pipe;
mod popup;
mod powerup;
mod score;
mod state;

use audio::Sfx;
use background::Background;
use bird::Bird;
use camera_fx::CameraFx;
use constants::*;
use macroquad::prelude::*;
use particles::ParticleSystem;
use pipe::{PipeEvent, PipeManager};
use popup::PopupManager;
use powerup::{PowerUpManager, PowerUpPickupEvent};
use score::ScoreManager;
use state::GameState;

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Bird — AAA Edition".to_owned(),
        window_width: 480,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

struct Game {
    state: GameState,
    bird: Bird,
    pipes: PipeManager,
    particles: ParticleSystem,
    powerups: PowerUpManager,
    popups: PopupManager,
    background: Background,
    camera: CameraFx,
    sfx: Sfx,
    score: ScoreManager,

    menu_time: f32,
    death_timer: f32, // grace period before a restart tap is accepted
}

impl Game {
    async fn new() -> Self {
        Self {
            state: GameState::Menu,
            bird: Bird::new(Self::bird_rest_pos()),
            pipes: PipeManager::new(),
            particles: ParticleSystem::new(),
            powerups: PowerUpManager::new(),
            popups: PopupManager::new(),
            background: Background::new(),
            camera: CameraFx::new(),
            sfx: Sfx::load().await,
            score: ScoreManager::new(),
            menu_time: 0.0,
            death_timer: 0.0,
        }
    }

    fn bird_rest_pos() -> Vec2 {
        vec2(screen_width() * BIRD_X_FRACTION, screen_height() * 0.4)
    }

    fn ground_y(&self) -> f32 {
        screen_height() - GROUND_HEIGHT
    }

    fn reset_run(&mut self) {
        self.bird = Bird::new(Self::bird_rest_pos());
        self.pipes.reset();
        self.powerups.reset();
        self.background.reset();
        self.particles = ParticleSystem::new();
        self.popups = PopupManager::new();
        self.score.reset_run();
        self.death_timer = 0.0;
        self.state = GameState::Playing;
    }

    fn jump_or_confirm_pressed() -> bool {
        is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Up)
            || is_mouse_button_pressed(MouseButton::Left)
    }

    // ---- per-state update ---------------------------------------------

    fn update_menu(&mut self, dt: f32) {
        self.background.update(dt * 0.4); // gentle idle scroll
        if Self::jump_or_confirm_pressed() {
            self.reset_run();
        }
    }

    fn update_playing(&mut self, raw_dt: f32) {
        let dt = self.camera.tick(raw_dt, self.bird.vel_y);
        if dt <= 0.0 {
            return; // hitstop freeze
        }

        if Self::jump_or_confirm_pressed() {
            self.bird.jump();
            self.particles.spawn_feathers(self.bird.pos);
            self.sfx.play_jump();
        }

        let world_mult = self.powerups.world_speed_mult();
        let scaled_dt = dt * world_mult;

        self.bird.update(dt);
        self.particles.update(dt);
        self.popups.update(dt);
        self.background.update(scaled_dt);
        self.powerups.update(scaled_dt, PIPE_SPEED);

        let ground_y = self.ground_y();

        // Ceiling: soft clamp, no death.
        let r = self.bird.collider_radius();
        if self.bird.pos.y - r < 0.0 {
            self.bird.pos.y = r;
            self.bird.vel_y = self.bird.vel_y.max(0.0);
        }

        // Power-up pickups.
        for ev in self.powerups.check_pickups(&mut self.bird) {
            let PowerUpPickupEvent::Collected(kind, pos) = ev;
            self.sfx.play_powerup();
            self.popups.spawn(pos, kind.label(), kind_popup_color(kind));
        }

        // Pipe scrolling / spawning / scoring.
        let bird_x = self.bird.pos.x;
        for ev in self.pipes.update(scaled_dt, ground_y, bird_x) {
            match ev {
                PipeEvent::Passed(pos) => {
                    let broke_record = self.score.add_point();
                    self.popups.spawn(pos, "+1", WHITE);
                    self.background.register_pipe_passed();
                    self.sfx.play_score();
                    if broke_record {
                        self.particles.spawn_confetti(vec2(screen_width() / 2.0, 80.0));
                    }
                }
                PipeEvent::Spawned(gap_center) => {
                    self.powerups.maybe_spawn(gap_center);
                }
            }
        }

        // Ground collision -> instant death, no shield protection.
        if self.bird.pos.y + r >= ground_y {
            self.bird.pos.y = ground_y - r;
            self.kill_bird(SHAKE_GROUND_HIT);
            return;
        }

        // Pipe collision -> shield absorbs one hit, otherwise death. Skipped
        // entirely while invulnerable, which is what stops a just-broken
        // shield from getting the bird killed one frame later by the same
        // pipe rect it's still overlapping.
        if !self.bird.is_invulnerable() {
            if let Some(hit_point) = self.pipes.check_collision(self.bird.pos, r, ground_y) {
                if self.bird.shield {
                    self.bird.shield = false;
                    self.bird.invuln_timer = 0.6;
                    self.camera.add_trauma(0.35);
                    self.particles.spawn_explosion(hit_point);
                    self.sfx.play_hit();
                } else {
                    self.kill_bird(SHAKE_PIPE_HIT);
                }
            }
        }
    }

    fn kill_bird(&mut self, shake: f32) {
        self.camera.add_trauma(shake / 16.0); // normalize roughly into 0..1 trauma
        self.camera.trigger_hitstop(HITSTOP_FRAMES_ON_DEATH);
        self.particles.spawn_explosion(self.bird.pos);
        self.sfx.play_hit();
        self.state = GameState::Dead;
        self.death_timer = 0.0;
    }

    fn update_dead(&mut self, raw_dt: f32) {
        self.death_timer += raw_dt;
        let dt = self.camera.tick(raw_dt, self.bird.vel_y);
        if dt > 0.0 {
            // Let the bird keep falling and settle on the ground; pipes and
            // the background stay frozen so the crash site stays readable.
            self.bird.update(dt);
            let ground_y = self.ground_y();
            let r = self.bird.collider_radius();
            if self.bird.pos.y + r > ground_y {
                self.bird.pos.y = ground_y - r;
                self.bird.vel_y = 0.0;
            }
            self.particles.update(dt);
            self.popups.update(dt);
        }

        if self.death_timer > 0.5 && Self::jump_or_confirm_pressed() {
            self.reset_run();
        }
    }

    // ---- drawing -----------------------------------------------------

    fn draw_world(&self) {
        let offset = self.camera.offset();
        let ground_y = self.ground_y();
        self.background.draw(ground_y, offset);
        self.pipes.draw(ground_y, offset, self.background.is_night());
        self.powerups.draw(offset);
        self.particles.draw(offset);
        self.bird.draw(offset);
        self.popups.draw(offset);
    }

    fn draw_hud_playing(&self) {
        draw_text_ex(
            &format!("{}", self.score.current),
            screen_width() / 2.0 - 12.0,
            70.0,
            TextParams {
                font_size: 48,
                color: WHITE,
                ..Default::default()
            },
        );

        let mut y = 20.0;
        if self.bird.shield {
            draw_text("SHIELD", 16.0, y, 22.0, SKYBLUE);
            y += 24.0;
        }
        if self.powerups.slowmo_timer > 0.0 {
            draw_text(
                &format!("SLOW-MO {:.1}s", self.powerups.slowmo_timer),
                16.0,
                y,
                22.0,
                Color::new(0.75, 0.4, 1.0, 1.0),
            );
            y += 24.0;
        }
        if self.bird.is_shrunk() {
            draw_text(
                &format!("SHRINK {:.1}s", self.bird.shrink_timer),
                16.0,
                y,
                22.0,
                Color::new(1.0, 0.85, 0.25, 1.0),
            );
        }
    }

    fn draw_menu_overlay(&self) {
        let w = screen_width();
        let h = screen_height();

        let bob = (self.menu_time * 2.0).sin() * 10.0;
        let title = "FLAPPY BIRD";
        let title_size = 54.0;
        let dims = measure_text(title, None, title_size as u16, 1.0);
        draw_text_ex(
            title,
            w / 2.0 - dims.width / 2.0,
            h * 0.32 + bob,
            TextParams {
                font_size: title_size as u16,
                color: Color::new(1.0, 0.82, 0.15, 1.0),
                ..Default::default()
            },
        );

        let pulse = 0.6 + 0.4 * (self.menu_time * 3.2).sin().abs();
        let prompt = "Press SPACE to start";
        let p_size = 26.0;
        let p_dims = measure_text(prompt, None, p_size as u16, 1.0);
        draw_text_ex(
            prompt,
            w / 2.0 - p_dims.width / 2.0,
            h * 0.55,
            TextParams {
                font_size: p_size as u16,
                color: Color::new(1.0, 1.0, 1.0, pulse),
                ..Default::default()
            },
        );

        let high = format!("Best: {}", self.score.high);
        let h_dims = measure_text(&high, None, 24, 1.0);
        draw_text(&high, w / 2.0 - h_dims.width / 2.0, h * 0.62, 24.0, LIGHTGRAY);
    }

    fn draw_dead_overlay(&self) {
        let w = screen_width();
        let h = screen_height();

        draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.35));

        let title = "GAME OVER";
        let title_size = 46.0;
        let dims = measure_text(title, None, title_size as u16, 1.0);
        draw_text_ex(
            title,
            w / 2.0 - dims.width / 2.0,
            h * 0.32,
            TextParams {
                font_size: title_size as u16,
                color: Color::new(1.0, 0.35, 0.3, 1.0),
                ..Default::default()
            },
        );

        let score_line = format!("Score: {}", self.score.current);
        let s_dims = measure_text(&score_line, None, 30, 1.0);
        draw_text(&score_line, w / 2.0 - s_dims.width / 2.0, h * 0.42, 30.0, WHITE);

        let high_line = format!("Best: {}", self.score.high);
        let h_dims = measure_text(&high_line, None, 24, 1.0);
        draw_text(&high_line, w / 2.0 - h_dims.width / 2.0, h * 0.48, 24.0, LIGHTGRAY);

        if self.death_timer > 0.5 {
            let pulse = 0.6 + 0.4 * (self.menu_time * 3.2).sin().abs();
            let prompt = "Tap to try again";
            let p_dims = measure_text(prompt, None, 24, 1.0);
            draw_text_ex(
                prompt,
                w / 2.0 - p_dims.width / 2.0,
                h * 0.58,
                TextParams {
                    font_size: 24,
                    color: Color::new(1.0, 1.0, 1.0, pulse),
                    ..Default::default()
                },
            );
        }
    }
}

fn kind_popup_color(kind: powerup::PowerUpKind) -> Color {
    match kind {
        powerup::PowerUpKind::Shield => SKYBLUE,
        powerup::PowerUpKind::SlowMo => Color::new(0.75, 0.4, 1.0, 1.0),
        powerup::PowerUpKind::Shrink => Color::new(1.0, 0.85, 0.25, 1.0),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    loop {
        let raw_dt = get_frame_time().min(1.0 / 20.0); // clamp huge stalls/tab-switches

        // Keep menu/death pulse animations alive regardless of state so the
        // sine timer doesn't jump when switching screens.
        game.menu_time += raw_dt;

        match game.state {
            GameState::Menu => game.update_menu(raw_dt),
            GameState::Playing => game.update_playing(raw_dt),
            GameState::Dead => game.update_dead(raw_dt),
        }

        clear_background(BLACK);
        game.draw_world();
        match game.state {
            GameState::Menu => game.draw_menu_overlay(),
            GameState::Playing => game.draw_hud_playing(),
            GameState::Dead => {
                game.draw_hud_playing();
                game.draw_dead_overlay();
            }
        }

        next_frame().await;
    }
}
