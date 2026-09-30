#![cfg(any(target_arch = "wasm32", test))]

#[cfg(target_arch = "wasm32")]
mod graphics;

const PADDLE_HEIGHT: i32 = 5;
const PADDLE_STEP: f32 = 2.0;
const BALL_WIDTH: i32 = 2;
const BALL_RADIUS_X: f32 = (BALL_WIDTH - 1) as f32 / 2.0;
// Velocities are fractions of the court per second, independent of terminal size.
const BALL_SPEED: f32 = 1.0 / 2.6;
const MAX_VERTICAL_SPEED: f32 = 14.4 / 21.0;
const SERVE_VERTICAL_SPEED: f32 = 7.2 / 21.0;
const COMPUTER_SPEED: f32 = 10.0 / 21.0;
const COMPUTER_SPEEDUP_PER_POINT: f32 = 0.15;
const MAX_COMPUTER_MULTIPLIER: f32 = 3.0;
const HITS_PER_SPEEDUP: u32 = 2;
const SPEEDUP: f32 = 0.1;
const MAX_SPEEDUPS: u32 = 10;
const MAX_STEP_DISTANCE: f32 = 0.5;
const MAX_DELTA: f32 = 0.05;
const PHYSICS_STEP: f32 = 1.0 / 240.0;
const PLAY_TOP: i32 = 2;
const MIN_WIDTH: i32 = 24;
const MIN_HEIGHT: i32 = 10;
const PADDLE_INSET: i32 = 1;

#[derive(Clone, Copy)]
struct Game {
    computer: bool,
    rally_hits: u32,
    left_paddle_y: f32,
    right_paddle_y: f32,
    ball_x: f32,
    ball_y: f32,
    ball_velocity_x: f32,
    ball_velocity_y: f32,
    left_score: u32,
    right_score: u32,
}

impl Game {
    fn new(width: i32, height: i32) -> Self {
        let width = width.max(MIN_WIDTH);
        let height = height.max(MIN_HEIGHT);
        let paddle_y = ((PLAY_TOP + height - PADDLE_HEIGHT) as f32 / 2.0).round();
        let mut game = Self {
            computer: false,
            rally_hits: 0,
            left_paddle_y: paddle_y,
            right_paddle_y: paddle_y,
            ball_x: 0.0,
            ball_y: 0.0,
            ball_velocity_x: 0.0,
            ball_velocity_y: 0.0,
            left_score: 0,
            right_score: 0,
        };
        game.reset_ball(width, height, 1.0);
        game
    }

    fn reset_ball(&mut self, width: i32, height: i32, direction: f32) {
        self.rally_hits = 0;
        self.ball_x = (width - 1) as f32 / 2.0;
        self.ball_y = (PLAY_TOP + height - 1) as f32 / 2.0;
        self.ball_velocity_x = direction * BALL_SPEED;
        self.ball_velocity_y = SERVE_VERTICAL_SPEED;
    }

    fn speed_multiplier(&self) -> f32 {
        1.0 + (self.rally_hits / HITS_PER_SPEEDUP).min(MAX_SPEEDUPS) as f32 * SPEEDUP
    }

    fn computer_speed(&self) -> f32 {
        COMPUTER_SPEED
            * (1.0 + self.left_score as f32 * COMPUTER_SPEEDUP_PER_POINT)
                .min(MAX_COMPUTER_MULTIPLIER)
    }

    fn update(&mut self, width: i32, height: i32, delta: f32, left: i32, right: i32) {
        if width < MIN_WIDTH || height < MIN_HEIGHT {
            return;
        }
        let delta = if delta.is_finite() {
            delta.clamp(0.0, MAX_DELTA)
        } else {
            0.0
        };
        let court_width = (width - 1 - 2 * PADDLE_INSET) as f32;
        let court_height = (height - 1 - PLAY_TOP) as f32;
        let paddle_bottom = (height - PADDLE_HEIGHT) as f32;
        self.left_paddle_y =
            (self.left_paddle_y + left as f32 * PADDLE_STEP).clamp(PLAY_TOP as f32, paddle_bottom);
        let right_movement = if self.computer {
            let target = if self.ball_velocity_x > 0.0 {
                self.ball_y
            } else {
                (PLAY_TOP + height - 1) as f32 / 2.0
            };
            let center = self.right_paddle_y + (PADDLE_HEIGHT - 1) as f32 / 2.0;
            let movement = self.computer_speed() * court_height * delta;
            (target - center).clamp(-movement, movement)
        } else {
            right as f32 * PADDLE_STEP
        };
        self.right_paddle_y =
            (self.right_paddle_y + right_movement).clamp(PLAY_TOP as f32, paddle_bottom);
        self.ball_x = self
            .ball_x
            .clamp(-BALL_RADIUS_X, (width - 1) as f32 + BALL_RADIUS_X);
        self.ball_y = self.ball_y.clamp(PLAY_TOP as f32, (height - 1) as f32);

        // Limit travel per step so fast rallies cannot skip a paddle on large courts.
        let mut remaining = delta;
        while remaining > 0.0 {
            let velocity_x = self.ball_velocity_x * court_width;
            let velocity_y = self.ball_velocity_y * court_height;
            let max_velocity = velocity_x.abs().max(velocity_y.abs());
            let step = remaining
                .min(PHYSICS_STEP)
                .min(MAX_STEP_DISTANCE / max_velocity);
            remaining -= step;
            let previous_x = self.ball_x;
            self.ball_x += velocity_x * step;
            self.ball_y += velocity_y * step;
            let top = PLAY_TOP as f32;
            let bottom = (height - 1) as f32;
            if self.ball_y < top {
                self.ball_y = 2.0 * top - self.ball_y;
                self.ball_velocity_y = self.ball_velocity_y.abs();
            } else if self.ball_y > bottom {
                self.ball_y = 2.0 * bottom - self.ball_y;
                self.ball_velocity_y = -self.ball_velocity_y.abs();
            }

            let left_x = PADDLE_INSET as f32 + BALL_RADIUS_X;
            let right_x = (width - 1 - PADDLE_INSET) as f32 - BALL_RADIUS_X;
            if self.ball_velocity_x < 0.0 && previous_x >= left_x && self.ball_x <= left_x {
                self.hit_paddle(left_x, self.left_paddle_y, 1.0);
            } else if self.ball_velocity_x > 0.0 && previous_x <= right_x && self.ball_x >= right_x
            {
                self.hit_paddle(right_x, self.right_paddle_y, -1.0);
            }

            if self.ball_x < -BALL_RADIUS_X {
                self.right_score = self.right_score.saturating_add(1);
                self.reset_ball(width, height, -1.0);
                break;
            } else if self.ball_x > (width - 1) as f32 + BALL_RADIUS_X {
                self.left_score = self.left_score.saturating_add(1);
                self.reset_ball(width, height, 1.0);
                break;
            }
        }
    }

    fn hit_paddle(&mut self, x: f32, paddle_y: f32, direction: f32) {
        let center = paddle_y.round() + (PADDLE_HEIGHT - 1) as f32 / 2.0;
        let offset = (self.ball_y - center) / (PADDLE_HEIGHT as f32 / 2.0);
        if offset.abs() <= 1.0 {
            self.ball_x = 2.0 * x - self.ball_x;
            self.rally_hits = self.rally_hits.saturating_add(1);
            let speed = self.speed_multiplier();
            self.ball_velocity_x = direction * BALL_SPEED * speed;
            self.ball_velocity_y = offset * MAX_VERTICAL_SPEED * speed;
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod guest {
    use super::*;
    use crate::graphics::{render, render_selection, render_small};
    use std::cell::Cell;

    #[link(wasm_import_module = "arcadium")]
    unsafe extern "C" {
        fn screen_width() -> i32;
        fn screen_height() -> i32;
        fn key_pressed(key: i32) -> i32;
    }

    const UP: i32 = 0;
    const DOWN: i32 = 1;
    const LEFT: i32 = 2;
    const RIGHT: i32 = 3;
    const A: i32 = 5;
    const D: i32 = 7;
    const W: i32 = 4;
    const S: i32 = 6;
    const ENTER: i32 = 9;

    #[derive(Clone, Copy)]
    enum Screen {
        Select { computer: bool },
        Playing(Game),
    }

    thread_local! {
        static SCREEN: Cell<Option<Screen>> = const { Cell::new(None) };
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_init() -> i32 {
        SCREEN.set(Some(Screen::Select { computer: true }));
        1
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_update(delta_seconds: f32) {
        let width = unsafe { screen_width() };
        let height = unsafe { screen_height() };
        if width < MIN_WIDTH || height < MIN_HEIGHT {
            render_small(width, height);
            return;
        }
        let left = unsafe { key_pressed(S) - key_pressed(W) };
        let right = unsafe { key_pressed(DOWN) - key_pressed(UP) };
        match SCREEN.get() {
            Some(Screen::Select { mut computer }) => {
                let horizontal = unsafe {
                    i32::from(key_pressed(RIGHT) != 0 || key_pressed(D) != 0)
                        - i32::from(key_pressed(LEFT) != 0 || key_pressed(A) != 0)
                };
                let direction = if horizontal != 0 {
                    horizontal
                } else {
                    left + right
                };
                if direction < 0 {
                    computer = true;
                } else if direction > 0 {
                    computer = false;
                }
                if unsafe { key_pressed(ENTER) } != 0 {
                    let mut game = Game::new(width, height);
                    game.computer = computer;
                    render(&game, width, height);
                    SCREEN.set(Some(Screen::Playing(game)));
                } else {
                    render_selection(width, height, computer);
                    SCREEN.set(Some(Screen::Select { computer }));
                }
            }
            Some(Screen::Playing(mut game)) => {
                game.update(
                    width,
                    height,
                    delta_seconds,
                    if game.computer { right } else { left },
                    right,
                );
                render(&game, width, height);
                SCREEN.set(Some(Screen::Playing(game)));
            }
            None => {}
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_shutdown() {
        SCREEN.set(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computer_gets_faster_only_when_the_human_scores() {
        let mut game = Game::new(80, 24);
        game.computer = true;
        let base = game.computer_speed();
        game.right_score = 5;
        assert_eq!(game.computer_speed(), base);
        for points in 1..=10 {
            let previous = game.computer_speed();
            game.left_score = points;
            assert!(game.computer_speed() > previous);
        }
        let difficulty = game.computer_speed();
        game.reset_ball(80, 24, 1.0);
        assert_eq!(game.computer_speed(), difficulty);
        game.left_score = u32::MAX;
        assert_eq!(
            game.computer_speed(),
            COMPUTER_SPEED * MAX_COMPUTER_MULTIPLIER
        );
        let mut beginner = Game::new(80, 24);
        beginner.computer = true;
        beginner.ball_y = 22.0;
        game.ball_y = 22.0;
        beginner.update(80, 24, 0.04, 0, 0);
        game.update(80, 24, 0.04, 0, 0);
        assert!(game.right_paddle_y > beginner.right_paddle_y);
        assert_eq!(Game::new(80, 24).computer_speed(), base);
    }

    #[test]
    fn wider_ball_hits_with_its_edge_and_scores_only_after_leaving() {
        for (x, direction) in [(1.7, -1.0), (77.3, 1.0)] {
            let mut game = Game::new(80, 24);
            game.ball_x = x;
            game.ball_y = game.left_paddle_y + 2.0;
            game.ball_velocity_x = direction * BALL_SPEED;
            game.ball_velocity_y = 0.0;
            game.update(80, 24, 0.01, 0, 0);
            assert_eq!(game.rally_hits, 1);
            assert!(game.ball_velocity_x * direction < 0.0);
        }
        for (x, direction) in [(-0.1, -1.0), (79.1, 1.0)] {
            let mut game = Game::new(80, 24);
            game.ball_x = x;
            game.ball_velocity_x = direction * BALL_SPEED;
            game.ball_velocity_y = 0.0;
            game.update(80, 24, 0.005, 0, 0);
            assert_eq!(game.left_score + game.right_score, 0);
            game.update(80, 24, MAX_DELTA, 0, 0);
            assert_eq!(game.left_score + game.right_score, 1);
        }
    }

    #[test]
    fn speed_increases_only_every_two_paddle_hits_and_is_capped() {
        let mut game = Game::new(80, 24);
        for hit in 1..=30 {
            game.ball_x = 1.0;
            game.ball_y = game.left_paddle_y.round() + 3.0;
            game.hit_paddle(1.0, game.left_paddle_y, 1.0);
            let expected = 1.0 + ((hit / 2).min(10) as f32 * 0.1);
            assert_eq!(game.rally_hits, hit);
            assert!((game.ball_velocity_x - BALL_SPEED * expected).abs() < 0.00001);
            assert!((game.ball_velocity_y - 0.4 * MAX_VERTICAL_SPEED * expected).abs() < 0.00001);
        }
        let hits = game.rally_hits;
        game.ball_y = 2.01;
        game.ball_velocity_y = -SERVE_VERTICAL_SPEED;
        game.ball_x = 40.0;
        game.update(80, 24, 0.02, 0, 0);
        assert_eq!(game.rally_hits, hits);
    }

    #[test]
    fn ball_covers_same_fraction_of_small_and_large_courts() {
        let mut fractions = Vec::new();
        for (width, height) in [(24, 10), (80, 24), (240, 70), (500, 120)] {
            let mut game = Game::new(width, height);
            let (x, y) = (game.ball_x, game.ball_y);
            game.update(width, height, 0.04, 0, 0);
            fractions.push((
                (game.ball_x - x) / (width - 3) as f32,
                (game.ball_y - y) / (height - 3) as f32,
            ));
        }
        for (x, y) in fractions {
            assert!((x - BALL_SPEED * 0.04).abs() < 0.00001);
            assert!((y - SERVE_VERTICAL_SPEED * 0.04).abs() < 0.00001);
        }
        let mut game = Game::new(80, 24);
        let x = game.ball_x;
        game.update(240, 70, 0.04, 0, 0);
        assert!((game.ball_x - x - BALL_SPEED * 237.0 * 0.04).abs() < 0.001);
    }

    #[test]
    fn fast_ball_still_hits_paddles_on_a_large_court() {
        let mut game = Game::new(4000, 100);
        game.rally_hits = 20;
        game.ball_x = 1.7;
        game.ball_y = game.left_paddle_y.round() + 2.0;
        game.ball_velocity_x = -BALL_SPEED * game.speed_multiplier();
        game.ball_velocity_y = 0.0;
        game.update(4000, 100, MAX_DELTA, 0, 0);
        assert!(game.ball_velocity_x > 0.0);
        assert_eq!(game.rally_hits, 21);
        assert_eq!(game.right_score, 0);
    }

    #[test]
    fn computer_tracks_ball_with_limited_speed_and_ignores_right_input() {
        let mut game = Game::new(80, 24);
        game.computer = true;
        game.ball_y = 22.0;
        let before = game.right_paddle_y;
        let mut without_input = game;
        game.update(80, 24, 0.04, -1, -1);
        without_input.update(80, 24, 0.04, -1, 0);
        assert_eq!(game.right_paddle_y, without_input.right_paddle_y);
        assert!((game.right_paddle_y - before - 0.4).abs() < 0.001);
        assert_eq!(game.left_paddle_y, before - 2.0);
        game.ball_y = 2.0;
        game.update(80, 24, 0.04, 0, 0);
        assert!((game.right_paddle_y - before).abs() < 0.001);
        game.right_paddle_y = 100.0;
        game.update(24, 10, MAX_DELTA, 0, 0);
        assert_eq!(game.right_paddle_y, 5.0);
        game.reset_ball(80, 24, -1.0);
        assert!(game.computer);
    }

    #[test]
    fn walls_reflect_in_both_directions() {
        for (y, velocity) in [(2.01, -SERVE_VERTICAL_SPEED), (22.99, SERVE_VERTICAL_SPEED)] {
            let mut game = Game::new(80, 24);
            game.ball_y = y;
            game.ball_velocity_y = velocity;
            game.update(80, 24, 0.02, 0, 0);
            assert!(game.ball_velocity_y * velocity < 0.0);
            assert!((2.0..=23.0).contains(&game.ball_y));
        }
    }

    #[test]
    fn paddles_reflect_and_aim_without_repeated_hits() {
        for (x, velocity, paddle, direction) in
            [(1.7, -BALL_SPEED, 0, 1.0), (77.3, BALL_SPEED, 1, -1.0)]
        {
            for offset in [-2.0, 0.0, 2.0] {
                let mut game = Game::new(80, 24);
                let y = if paddle == 0 {
                    game.left_paddle_y
                } else {
                    game.right_paddle_y
                };
                game.ball_x = x;
                game.ball_y = y.round() + 2.0 + offset;
                game.ball_velocity_x = velocity;
                game.ball_velocity_y = 0.0;
                game.update(80, 24, MAX_DELTA, 0, 0);
                assert_eq!(game.ball_velocity_x, direction * BALL_SPEED);
                assert_eq!(game.ball_velocity_y.signum(), offset.signum());
                game.update(80, 24, MAX_DELTA, 0, 0);
                assert_eq!(game.ball_velocity_x, direction * BALL_SPEED);
            }
        }
    }

    #[test]
    fn misses_score_once_and_serve_toward_conceding_player() {
        for direction in [-1.0, 1.0] {
            let mut game = Game::new(80, 24);
            game.rally_hits = 8;
            game.ball_x = if direction < 0.0 { 0.6 } else { 78.4 };
            game.ball_y = PLAY_TOP as f32;
            game.ball_velocity_x = direction * BALL_SPEED;
            game.ball_velocity_y = 0.0;
            game.update(80, 24, MAX_DELTA, 0, 0);
            assert_eq!(game.left_score, u32::from(direction > 0.0));
            assert_eq!(game.right_score, u32::from(direction < 0.0));
            assert_eq!(game.rally_hits, 0);
            assert_eq!(game.speed_multiplier(), 1.0);
            assert_eq!(game.ball_x, 39.5);
            assert_eq!(game.ball_y, 12.5);
            assert_eq!(game.ball_velocity_x, direction * BALL_SPEED);
        }
    }

    #[test]
    fn resize_small_screens_and_invalid_deltas_are_safe() {
        let mut game = Game::new(0, 0);
        game.update(0, 0, 1.0, -1, 1);
        game.left_paddle_y = -100.0;
        game.right_paddle_y = 100.0;
        game.ball_x = 100.0;
        game.ball_y = 100.0;
        game.update(24, 10, f32::NAN, 0, 0);
        assert_eq!(game.left_paddle_y, 2.0);
        assert_eq!(game.right_paddle_y, 5.0);
        assert_eq!(game.ball_x, 23.5);
        assert_eq!(game.ball_y, 9.0);
        game.update(80, 24, -1.0, 0, 0);
        assert_eq!(game.ball_x, 23.5);
        game.update(80, 24, f32::INFINITY, 0, 0);
        assert_eq!(game.ball_x, 23.5);
    }

    #[test]
    fn paddles_step_per_press_while_ball_uses_delta_and_caps_stalls() {
        let mut game = Game::new(80, 24);
        let mut split = game;
        game.update(80, 24, 0.04, -1, 1);
        assert_eq!(game.left_paddle_y, split.left_paddle_y - 2.0);
        assert_eq!(game.right_paddle_y, split.right_paddle_y + 2.0);
        split.update(80, 24, 0.01, -1, 1);
        for _ in 0..3 {
            split.update(80, 24, 0.01, 0, 0);
        }
        assert!((game.ball_x - split.ball_x).abs() < 0.001);
        assert!((game.left_paddle_y - split.left_paddle_y).abs() < 0.001);
        assert!((game.right_paddle_y - split.right_paddle_y).abs() < 0.001);
        let mut capped = game;
        game.update(80, 24, 100.0, 0, 0);
        capped.update(80, 24, MAX_DELTA, 0, 0);
        assert_eq!(game.ball_x, capped.ball_x);
    }
}
