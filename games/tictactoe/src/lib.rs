#![cfg(any(target_arch = "wasm32", test))]

#[cfg(target_arch = "wasm32")]
mod graphics;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    Empty,
    X,
    O,
}

impl Mark {
    fn opponent(self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
            Self::Empty => Self::Empty,
        }
    }
}

const WINS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];
const MOVE_ORDER: [usize; 9] = [4, 0, 2, 6, 8, 1, 3, 5, 7];

fn winning_line(board: &[Mark; 9]) -> Option<[usize; 3]> {
    WINS.into_iter()
        .find(|&[a, b, c]| board[a] != Mark::Empty && board[a] == board[b] && board[a] == board[c])
}

// Some(Empty) is a draw; None means the board is still in play.
fn result(board: &[Mark; 9]) -> Option<Mark> {
    if let Some([a, _, _]) = winning_line(board) {
        return Some(board[a]);
    }
    board
        .iter()
        .all(|&mark| mark != Mark::Empty)
        .then_some(Mark::Empty)
}

fn minimax(mut board: [Mark; 9], turn: Mark, depth: i32) -> i32 {
    if let Some(winner) = result(&board) {
        return match winner {
            Mark::O => 10 - depth,
            Mark::X => depth - 10,
            Mark::Empty => 0,
        };
    }
    let mut best = if turn == Mark::O { -100 } else { 100 };
    for index in MOVE_ORDER {
        if board[index] == Mark::Empty {
            board[index] = turn;
            let score = minimax(board, turn.opponent(), depth + 1);
            board[index] = Mark::Empty;
            best = if turn == Mark::O {
                best.max(score)
            } else {
                best.min(score)
            };
        }
    }
    best
}

fn ai_move(mut board: [Mark; 9]) -> Option<usize> {
    if result(&board).is_some() {
        return None;
    }
    let mut best_score = -100;
    let mut best_move = None;
    for index in MOVE_ORDER {
        if board[index] == Mark::Empty {
            board[index] = Mark::O;
            let score = minimax(board, Mark::X, 1);
            board[index] = Mark::Empty;
            if score > best_score {
                best_score = score;
                best_move = Some(index);
            }
        }
    }
    best_move
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Singleplayer,
    Multiplayer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    ModeSelect,
    Playing,
    Finished(Mark),
}

#[derive(Clone, Copy)]
struct Game {
    board: [Mark; 9],
    cursor: usize,
    turn: Mark,
    mode: Mode,
    state: State,
}

impl Game {
    fn new() -> Self {
        Self {
            board: [Mark::Empty; 9],
            cursor: 4,
            turn: Mark::X,
            mode: Mode::Singleplayer,
            state: State::ModeSelect,
        }
    }

    fn update(&mut self, horizontal: i32, vertical: i32, enter: bool) {
        match self.state {
            State::ModeSelect => {
                let direction = if horizontal != 0 {
                    horizontal
                } else {
                    vertical
                };
                if direction < 0 {
                    self.mode = Mode::Singleplayer;
                } else if direction > 0 {
                    self.mode = Mode::Multiplayer;
                }
                if enter {
                    self.state = State::Playing;
                }
            }
            State::Playing => {
                let row = (self.cursor as i32 / 3 + vertical).clamp(0, 2);
                let column = (self.cursor as i32 % 3 + horizontal).clamp(0, 2);
                self.cursor = (row * 3 + column) as usize;
                if enter {
                    self.play();
                }
            }
            State::Finished(_) => {
                if enter {
                    self.restart();
                }
            }
        }
    }

    fn restart(&mut self) {
        *self = Self {
            mode: self.mode,
            state: State::Playing,
            ..Self::new()
        };
    }

    fn finish_round(&mut self) -> bool {
        match result(&self.board) {
            Some(Mark::Empty) => self.restart(),
            Some(winner) => self.state = State::Finished(winner),
            None => return false,
        }
        true
    }

    fn play(&mut self) {
        if self.board[self.cursor] != Mark::Empty {
            return;
        }
        self.board[self.cursor] = self.turn;
        if self.finish_round() {
            return;
        }
        self.turn = self.turn.opponent();
        if self.mode == Mode::Singleplayer {
            if let Some(index) = ai_move(self.board) {
                self.board[index] = Mark::O;
            }
            self.finish_round();
            self.turn = Mark::X;
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod guest {
    use super::*;
    use crate::graphics::{render, render_small};
    use std::cell::Cell;

    const MIN_WIDTH: i32 = 26;
    const MIN_HEIGHT: i32 = 13;
    const UP: i32 = 0;
    const DOWN: i32 = 1;
    const LEFT: i32 = 2;
    const RIGHT: i32 = 3;
    const W: i32 = 4;
    const A: i32 = 5;
    const S: i32 = 6;
    const D: i32 = 7;
    const ENTER: i32 = 9;

    #[link(wasm_import_module = "arcadium")]
    unsafe extern "C" {
        fn screen_width() -> i32;
        fn screen_height() -> i32;
        fn key_pressed(key: i32) -> i32;
    }

    thread_local! {
        static GAME: Cell<Option<Game>> = const { Cell::new(None) };
    }

    fn pressed(key: i32) -> bool {
        unsafe { key_pressed(key) != 0 }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_init() -> i32 {
        GAME.set(Some(Game::new()));
        1
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_update(_delta_seconds: f32) {
        let width = unsafe { screen_width() };
        let height = unsafe { screen_height() };
        if width < MIN_WIDTH || height < MIN_HEIGHT {
            render_small(width, height);
            return;
        }
        if let Some(mut game) = GAME.get() {
            let horizontal =
                i32::from(pressed(RIGHT) || pressed(D)) - i32::from(pressed(LEFT) || pressed(A));
            let vertical =
                i32::from(pressed(DOWN) || pressed(S)) - i32::from(pressed(UP) || pressed(W));
            game.update(horizontal, vertical, pressed(ENTER));
            render(&game, width, height);
            GAME.set(Some(game));
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn arcadium_shutdown() {
        GAME.set(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Mark::{Empty as E, O, X};

    #[test]
    fn all_winning_lines_and_draws_are_detected() {
        for line in WINS {
            for mark in [X, O] {
                let mut board = [E; 9];
                for index in line {
                    board[index] = mark;
                }
                assert_eq!(result(&board), Some(mark));
            }
        }
        assert_eq!(result(&[X, O, X, X, O, O, O, X, X]), Some(E));
        assert_eq!(result(&[E; 9]), None);
    }

    #[test]
    fn selection_moves_and_replay_are_separate_actions() {
        let mut game = Game::new();
        game.update(0, 1, true);
        assert_eq!(game.mode, Mode::Multiplayer);
        assert_eq!(game.board, [E; 9]);
        game.update(-1, -1, false);
        game.update(-1, -1, false);
        assert_eq!(game.cursor, 0);
        game.update(0, 0, true);
        assert_eq!(game.turn, O);
        game.update(0, 0, true);
        assert_eq!(game.board[0], X);
        assert_eq!(game.turn, O);
        for index in [3, 1, 4, 2] {
            game.cursor = index;
            game.update(0, 0, true);
        }
        assert_eq!(game.state, State::Finished(X));
        game.update(0, 0, true);
        assert_eq!(game.state, State::Playing);
        assert_eq!(game.mode, Mode::Multiplayer);
        assert_eq!(game.turn, X);
        assert_eq!(game.board, [E; 9]);
        game.update(1, 1, false);
        game.update(1, 1, false);
        assert_eq!(game.cursor, 8);
    }

    #[test]
    fn multiplayer_draw_and_singleplayer_terminal_move() {
        let mut game = Game::new();
        game.update(0, 1, true);
        for index in [0, 1, 2, 4, 3, 5, 7, 6, 8] {
            game.cursor = index;
            game.update(0, 0, true);
        }
        assert_eq!(game.state, State::Playing);
        assert_eq!(game.board, [E; 9]);
        assert_eq!(game.mode, Mode::Multiplayer);
        assert_eq!(game.turn, X);
        game.mode = Mode::Singleplayer;
        game.state = State::Playing;
        game.board = [X, X, E, O, O, E, E, E, E];
        game.turn = X;
        game.cursor = 2;
        game.update(0, 0, true);
        assert_eq!(game.state, State::Finished(X));
        assert_eq!(game.board.iter().filter(|&&mark| mark == O).count(), 2);
    }

    #[test]
    fn repeated_draws_reset_silently_in_both_modes_without_an_extra_move() {
        for mode in [Mode::Singleplayer, Mode::Multiplayer] {
            let mut game = Game {
                mode,
                state: State::Playing,
                ..Game::new()
            };
            for _ in 0..3 {
                game.board = [X, O, X, X, O, O, O, X, E];
                game.cursor = 8;
                game.update(0, 0, true);
                assert_eq!(game.board, [E; 9]);
                assert_eq!(game.turn, X);
                assert_eq!(game.cursor, 4);
                assert_eq!(game.mode, mode);
                assert_eq!(game.state, State::Playing);
            }
        }
    }

    #[test]
    fn ai_wins_blocks_and_leaves_existing_marks_untouched() {
        assert_eq!(ai_move([O, O, E, X, X, E, X, E, E]), Some(2));
        assert_eq!(ai_move([X, X, E, E, O, E, E, E, E]), Some(2));
        assert_eq!(ai_move([X, O, X, X, O, O, O, X, X]), None);
        let mut game = Game::new();
        game.update(0, 0, true);
        game.cursor = 0;
        game.update(0, 0, true);
        assert_eq!(game.board[0], X);
        assert_eq!(game.board[4], O);
        assert_eq!(game.board.iter().filter(|&&mark| mark != E).count(), 2);
        let board = game.board;
        game.update(0, 0, true);
        assert_eq!(game.board, board);
        assert_eq!(game.turn, X);
    }

    #[test]
    fn ai_cannot_lose_against_any_human_move_sequence() {
        fn explore(game: Game) {
            if let State::Finished(winner) = game.state {
                assert_ne!(winner, X);
                return;
            }
            for index in 0..9 {
                if game.board[index] == E {
                    let mut next = game;
                    next.cursor = index;
                    next.update(0, 0, true);
                    if next.board == [E; 9] {
                        assert_eq!(game.board.iter().filter(|&&mark| mark == E).count(), 1);
                        assert_eq!(next.state, State::Playing);
                        continue;
                    }
                    assert_eq!(next.board[index], X);
                    for (before, after) in game.board.iter().zip(next.board) {
                        if *before != E {
                            assert_eq!(*before, after);
                        }
                    }
                    explore(next);
                }
            }
        }
        let mut game = Game::new();
        game.update(0, 0, true);
        explore(game);
    }
}
