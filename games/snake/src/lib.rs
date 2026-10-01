#![cfg(any(target_arch = "wasm32", test))]

mod graphics;

const MIN_WIDTH: i32 = 24;
const MIN_HEIGHT: i32 = 10;
const PLAY_TOP: i32 = 3;
const INITIAL_INTERVAL: f32 = 0.12;
const MIN_INTERVAL: f32 = 0.07;
const SPEED_STEP: f32 = 0.005;
const MAX_DELTA: f32 = 0.25;
const BOOST_FACTOR: f32 = 0.65;
const REPEAT_WINDOW: f32 = 0.22;
const ROCK_EVERY: u32 = 3;
const MAX_ROCKS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Up, Self::Down)
                | (Self::Down, Self::Up)
                | (Self::Left, Self::Right)
                | (Self::Right, Self::Left)
        )
    }

    fn next(self, head: Point) -> Point {
        match self {
            Self::Up => Point {
                x: head.x,
                y: head.y - 1,
            },
            Self::Down => Point {
                x: head.x,
                y: head.y + 1,
            },
            Self::Left => Point {
                x: head.x - 1,
                y: head.y,
            },
            Self::Right => Point {
                x: head.x + 1,
                y: head.y,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Ready,
    Playing,
    GameOver,
    Won,
}

#[derive(Clone, Copy)]
struct Field {
    width: i32,
    height: i32,
}

impl Field {
    fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    fn columns(self) -> i32 {
        (self.width - 2) / 2
    }

    fn contains(self, point: Point) -> bool {
        point.x >= 0 && point.x < self.columns() && point.y >= PLAY_TOP && point.y < self.height - 1
    }

    fn cells(self) -> usize {
        (self.columns() * (self.height - PLAY_TOP - 1)) as usize
    }

    fn point(self, index: usize) -> Point {
        let columns = self.columns() as usize;
        Point {
            x: (index % columns) as i32,
            y: PLAY_TOP + (index / columns) as i32,
        }
    }
}

struct Game {
    snake: Vec<Point>, // Head first.
    rocks: Vec<Point>,
    direction: Direction,
    pending_direction: Direction,
    last_input: Option<Direction>,
    since_input: f32,
    boost_remaining: f32,
    food: Point,
    score: u32,
    best_score: u32,
    exit_selected: bool,
    exit_requested: bool,
    accumulator: f32,
    state: State,
    rng: u32,
}

impl Game {
    fn new(field: Field) -> Self {
        let head = Point {
            x: field.columns() / 2,
            y: (PLAY_TOP + field.height - 2) / 2,
        };
        let mut game = Self {
            snake: (0..4)
                .map(|offset| Point {
                    x: head.x - offset,
                    y: head.y,
                })
                .collect(),
            rocks: Vec::new(),
            direction: Direction::Right,
            pending_direction: Direction::Right,
            last_input: None,
            since_input: REPEAT_WINDOW,
            boost_remaining: 0.0,
            food: head,
            score: 0,
            best_score: 0,
            exit_selected: false,
            exit_requested: false,
            accumulator: 0.0,
            state: State::Ready,
            rng: ((field.width as u32).wrapping_mul(0x9e3779b9)
                ^ (field.height as u32)
                ^ 0xa341316c)
                | 1,
        };
        game.spawn_food(field);
        game
    }

    fn move_interval(&self) -> f32 {
        let base = (INITIAL_INTERVAL - (self.score / 3) as f32 * SPEED_STEP).max(MIN_INTERVAL);
        if self.boost_remaining > 0.0 {
            base * BOOST_FACTOR
        } else {
            base
        }
    }

    fn input(&mut self, direction: Direction) {
        // Keep the first valid turn until the next step. A second key cannot turn
        // around the original direction before the snake has actually moved.
        if self.state == State::Playing
            && self.pending_direction == self.direction
            && direction != self.direction
            && !direction.opposite(self.direction)
        {
            self.pending_direction = direction;
        }
    }

    fn update(&mut self, field: Field, delta: f32, input: Option<Direction>, enter: bool) {
        if self.snake.iter().any(|&point| !field.contains(point)) {
            if self.state == State::Ready {
                let best_score = self.best_score;
                *self = Self::new(field);
                self.best_score = best_score;
            } else {
                // A shrink that cuts through the snake ends the round; score stays visible.
                self.state = State::GameOver;
            }
        }
        self.rocks.retain(|&point| field.contains(point));
        if matches!(self.state, State::Ready | State::GameOver | State::Won) {
            if self.state != State::Ready {
                match input {
                    Some(Direction::Up | Direction::Left) => self.exit_selected = false,
                    Some(Direction::Down | Direction::Right) => self.exit_selected = true,
                    None => {}
                }
            }
            if enter {
                if self.state != State::Ready && self.exit_selected {
                    self.exit_requested = true;
                } else {
                    let best_score = self.best_score;
                    *self = Self::new(field);
                    self.best_score = best_score;
                    self.state = State::Playing;
                }
            } else if !field.contains(self.food) && self.state == State::Ready {
                self.spawn_food(field);
            }
            return;
        }
        if !field.contains(self.food) {
            self.spawn_food(field);
        }
        let delta = if delta.is_finite() {
            delta.clamp(0.0, MAX_DELTA)
        } else {
            0.0
        };
        self.since_input += delta;
        self.boost_remaining = (self.boost_remaining - delta).max(0.0);
        if let Some(direction) = input {
            self.input(direction);
            if self.last_input == Some(direction)
                && self.since_input <= REPEAT_WINDOW
                && direction == self.direction
            {
                self.boost_remaining = REPEAT_WINDOW;
            }
            self.last_input = Some(direction);
            self.since_input = 0.0;
        }
        self.accumulator += delta;
        while self.accumulator >= self.move_interval() && self.state == State::Playing {
            self.accumulator -= self.move_interval();
            self.step(field);
        }
    }

    fn step(&mut self, field: Field) {
        self.direction = self.pending_direction;
        let next = self.direction.next(self.snake[0]);
        let eating = next == self.food;
        // The tail leaves on a normal move, so entering its current cell is legal.
        let occupied = if eating {
            self.snake.len()
        } else {
            self.snake.len() - 1
        };
        if !field.contains(next)
            || self.snake[..occupied].contains(&next)
            || self.rocks.contains(&next)
        {
            self.state = State::GameOver;
            return;
        }
        self.snake.insert(0, next);
        if eating {
            self.score = self.score.saturating_add(1);
            self.best_score = self.best_score.max(self.score);
            self.spawn_food(field);
            if self.state == State::Playing
                && self.score.is_multiple_of(ROCK_EVERY)
                && self.rocks.len() < MAX_ROCKS
            {
                self.spawn_rock(field);
            }
        } else {
            self.snake.pop();
        }
        self.pending_direction = self.direction;
    }

    fn random(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    fn spawn_food(&mut self, field: Field) {
        let cells = field.cells();
        if self.snake.len() + self.rocks.len() >= cells {
            self.state = State::Won;
            return;
        }
        let start = self.random() as usize % cells;
        for offset in 0..cells {
            let point = field.point((start + offset) % cells);
            if !self.snake.contains(&point) && !self.rocks.contains(&point) {
                self.food = point;
                return;
            }
        }
    }

    fn spawn_rock(&mut self, field: Field) {
        let cells = field.cells();
        let start = self.random() as usize % cells;
        for offset in 0..cells {
            let point = field.point((start + offset) % cells);
            if point != self.food
                && !self.snake.contains(&point)
                && !self.rocks.contains(&point)
                && ![
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ]
                .iter()
                .any(|&direction| direction.next(self.snake[0]) == point)
            {
                self.rocks.push(point);
                return;
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod guest {
    use super::*;
    use std::cell::RefCell;

    #[link(wasm_import_module = "arcadium")]
    unsafe extern "C" {
        fn screen_width() -> i32;
        fn screen_height() -> i32;
        fn key_pressed(key: i32) -> i32;
        fn load_score() -> i64;
        fn save_score(score: i64) -> i32;
        fn request_exit();
    }

    thread_local! {
        static GAME: RefCell<Option<Game>> = const { RefCell::new(None) };
    }

    fn pressed(key: i32) -> bool {
        unsafe { key_pressed(key) != 0 }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_init() -> i32 {
        GAME.with(|game| *game.borrow_mut() = None);
        1
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_update(delta_seconds: f32) {
        let width = unsafe { screen_width() };
        let height = unsafe { screen_height() };
        if width < MIN_WIDTH || height < MIN_HEIGHT {
            graphics::render_small(width, height);
            return;
        }
        let field = Field::new(width, height);
        let direction = if pressed(0) || pressed(4) {
            Some(Direction::Up)
        } else if pressed(1) || pressed(6) {
            Some(Direction::Down)
        } else if pressed(2) || pressed(5) {
            Some(Direction::Left)
        } else if pressed(3) || pressed(7) {
            Some(Direction::Right)
        } else {
            None
        };
        GAME.with(|slot| {
            let mut slot = slot.borrow_mut();
            let game = slot.get_or_insert_with(|| {
                let mut game = Game::new(field);
                game.best_score = unsafe { load_score() }.clamp(0, u32::MAX as i64) as u32;
                game
            });
            let previous_best = game.best_score;
            game.update(field, delta_seconds, direction, pressed(9));
            if game.best_score > previous_best {
                unsafe { save_score(i64::from(game.best_score)) };
            }
            if game.exit_requested {
                unsafe { request_exit() };
                return;
            }
            graphics::render(game, width, height);
        });
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_shutdown() {
        GAME.with(|game| *game.borrow_mut() = None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELD: Field = Field {
        width: MIN_WIDTH,
        height: MIN_HEIGHT,
    };

    fn playing() -> Game {
        let mut game = Game::new(FIELD);
        game.state = State::Playing;
        game
    }

    #[test]
    fn movement_and_accumulated_time() {
        let mut game = playing();
        let head = game.snake[0];
        game.update(FIELD, 0.05, None, false);
        assert_eq!(game.snake[0], head);
        game.update(FIELD, 0.07, None, false);
        assert_eq!(
            game.snake[0],
            Point {
                x: head.x + 1,
                y: head.y
            }
        );
        let x = game.snake[0].x;
        game.update(FIELD, 100.0, None, false);
        assert!(game.snake[0].x <= x + 3);
    }

    #[test]
    fn growth_and_food_placement() {
        let mut game = playing();
        game.food = game.direction.next(game.snake[0]);
        game.step(FIELD);
        assert_eq!(game.snake.len(), 5);
        assert_eq!(game.score, 1);
        assert!(FIELD.contains(game.food));
        assert!(!game.snake.contains(&game.food));
    }

    #[test]
    fn direction_buffer_rejects_reversal_and_second_turn() {
        let mut game = playing();
        game.input(Direction::Left);
        assert_eq!(game.pending_direction, Direction::Right);
        game.input(Direction::Up);
        assert!(Direction::Up.opposite(Direction::Down));
        game.input(Direction::Left);
        assert_eq!(game.pending_direction, Direction::Up);
        game.step(FIELD);
        assert_eq!(game.direction, Direction::Up);
        game.input(Direction::Down);
        assert_eq!(game.pending_direction, Direction::Up);
    }

    #[test]
    fn wall_and_body_collisions() {
        let mut wall = playing();
        wall.snake[0].x = FIELD.columns() - 1;
        wall.step(FIELD);
        assert_eq!(wall.state, State::GameOver);
        let mut body = playing();
        body.snake = vec![
            Point { x: 5, y: 5 },
            Point { x: 5, y: 4 },
            Point { x: 6, y: 4 },
            Point { x: 6, y: 5 },
            Point { x: 7, y: 5 },
        ];
        body.step(FIELD);
        assert_eq!(body.state, State::GameOver);
    }

    #[test]
    fn moving_into_departing_tail_is_legal() {
        let mut game = playing();
        game.snake = vec![
            Point { x: 5, y: 5 },
            Point { x: 5, y: 4 },
            Point { x: 6, y: 4 },
            Point { x: 6, y: 5 },
        ];
        game.step(FIELD);
        assert_eq!(game.state, State::Playing);
        assert_eq!(game.snake[0], Point { x: 6, y: 5 });
    }

    #[test]
    fn nearly_full_field_finds_last_open_cell() {
        let field = Field::new(6, 6);
        let mut game = Game::new(FIELD);
        game.snake = (0..field.cells()).map(|index| field.point(index)).collect();
        let open = game.snake.pop().unwrap();
        game.spawn_food(field);
        assert_eq!(game.food, open);
        game.snake.push(open);
        game.spawn_food(field);
        assert_eq!(game.state, State::Won);
    }

    #[test]
    fn repeated_direction_temporarily_boosts_speed() {
        let mut game = playing();
        game.food = Point { x: 0, y: PLAY_TOP };
        let mut normal = playing();
        normal.food = game.food;
        game.update(FIELD, 0.02, Some(Direction::Right), false);
        assert_eq!(game.move_interval(), INITIAL_INTERVAL);
        game.update(FIELD, 0.02, Some(Direction::Right), false);
        assert_eq!(game.move_interval(), INITIAL_INTERVAL * BOOST_FACTOR);
        game.update(FIELD, 0.05, None, false);
        normal.update(FIELD, 0.09, None, false);
        assert_eq!(game.snake[0].x, normal.snake[0].x + 1);
        game.update(FIELD, REPEAT_WINDOW + 0.01, None, false);
        assert_eq!(game.move_interval(), INITIAL_INTERVAL);
    }

    #[test]
    fn rocks_appear_after_three_food_and_block_movement() {
        let mut game = playing();
        for _ in 0..3 {
            game.food = game.direction.next(game.snake[0]);
            game.step(FIELD);
        }
        assert_eq!(game.rocks.len(), 1);
        assert!(FIELD.contains(game.rocks[0]));
        assert!(!game.snake.contains(&game.rocks[0]));
        assert_ne!(game.food, game.rocks[0]);
        game.rocks = vec![game.direction.next(game.snake[0])];
        game.step(FIELD);
        assert_eq!(game.state, State::GameOver);
    }

    #[test]
    fn resize_pauses_and_recovers_or_ends_round() {
        let mut game = playing();
        let head = game.snake[0];
        game.update(Field::new(30, 12), 0.0, None, false);
        assert_eq!(game.snake[0], head);
        game.update(Field::new(10, 10), 0.0, None, false);
        assert_eq!(game.state, State::GameOver);
        assert_eq!(game.score, 0);
        game.update(FIELD, 0.0, None, true);
        assert_eq!(game.state, State::Playing);
    }

    #[test]
    fn game_over_menu_restarts_or_requests_exit() {
        let mut game = playing();
        game.score = 5;
        game.best_score = 8;
        game.state = State::GameOver;
        game.update(FIELD, 0.0, Some(Direction::Down), false);
        assert!(game.exit_selected);
        game.update(FIELD, 0.0, None, true);
        assert!(game.exit_requested);
        assert_eq!(game.score, 5);

        game.update(FIELD, 0.0, Some(Direction::Up), false);
        game.update(FIELD, 0.0, None, true);
        assert_eq!(game.state, State::Playing);
        assert_eq!(game.score, 0);
        assert_eq!(game.best_score, 8);
        assert!(!game.exit_requested);
    }

    #[test]
    fn eating_updates_best_score_only_when_beaten() {
        let mut game = playing();
        game.best_score = 3;
        game.food = game.direction.next(game.snake[0]);
        game.step(FIELD);
        assert_eq!(game.best_score, 3);
        game.score = 3;
        game.food = game.direction.next(game.snake[0]);
        game.step(FIELD);
        assert_eq!(game.best_score, 4);
    }
}
