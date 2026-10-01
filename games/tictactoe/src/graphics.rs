#[path = "../../common/hud.rs"]
mod hud;

use crate::{Game, Mark, Mode, State, winning_line};

const BACKGROUND: i32 = 0x0e101d;
const PANEL: i32 = 0x181a2a;
const DRAW_PANEL: i32 = 0x302f3f;
const CYAN: i32 = 0xff7338;
const TITLE: i32 = 0xffa04d;
const TITLE_SHADOW: i32 = 0x753914;
const PINK: i32 = 0xf4e7d3;
const WHITE: i32 = 0xf4e7d3;
const MUTED: i32 = 0xa99e94;
const LINE: i32 = 0x68402f;

#[link(wasm_import_module = "arcadium")]
unsafe extern "C" {
    fn draw_cell(x: i32, y: i32, character: i32, foreground: i32, background: i32);
}

fn cell(x: i32, y: i32, character: char, foreground: i32, background: i32) {
    // The host validates coordinates and colors.
    unsafe { draw_cell(x, y, character as i32, foreground, background) }
}

fn text(x: i32, y: i32, value: &str, foreground: i32, background: i32) {
    for (offset, character) in value.chars().enumerate() {
        cell(x + offset as i32, y, character, foreground, background);
    }
}

fn centered(width: i32, y: i32, value: &str, foreground: i32) {
    text(
        (width - value.chars().count() as i32) / 2,
        y,
        value,
        foreground,
        BACKGROUND,
    );
}

fn clear(width: i32, height: i32) {
    for y in 0..height {
        for x in 0..width {
            cell(x, y, ' ', WHITE, BACKGROUND);
        }
    }
}

pub(super) fn render_small(width: i32, height: i32) {
    clear(width, height);
    if width >= 18 && height > 0 {
        centered(width, height / 2, "TERMINAL TOO SMALL", MUTED);
    }
}

fn render_title(width: i32, top: i32, large: bool) {
    if !large {
        centered(width, top, "T I C   T A C   T O E", TITLE);
        return;
    }
    let letters = [
        [31, 4, 4, 4, 4],
        [14, 4, 4, 4, 14],
        [15, 16, 16, 16, 15],
        [31, 4, 4, 4, 4],
        [14, 17, 31, 17, 17],
        [15, 16, 16, 16, 15],
        [31, 4, 4, 4, 4],
        [14, 17, 17, 17, 14],
        [31, 16, 30, 16, 31],
    ];
    let x = (width - 57) / 2;
    for (letter, rows) in letters.iter().enumerate() {
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    cell(
                        x + letter as i32 * 6 + (letter / 3) as i32 * 2 + column + 1,
                        top + row as i32 + 1,
                        '▒',
                        TITLE_SHADOW,
                        BACKGROUND,
                    );
                }
            }
        }
    }
    for (letter, rows) in letters.iter().enumerate() {
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    cell(
                        x + letter as i32 * 6 + (letter / 3) as i32 * 2 + column,
                        top + row as i32,
                        '▓',
                        TITLE,
                        BACKGROUND,
                    );
                }
            }
        }
    }
}

fn selection(game: &Game, width: i32, height: i32) {
    let computer = game.mode == Mode::Singleplayer;
    let large_title = width >= 60 && height >= 17;
    let title_height = if large_title { 5 } else { 1 };
    let large_options = width >= 48 && height >= 15;
    let option_height = if large_options { 3 } else { 1 };
    let gap = if height >= 17 { 3 } else { 2 };
    let instruction_gap = if height >= 13 { 2 } else { 1 };
    let menu_height = title_height + gap + option_height + gap + instruction_gap + 1;
    let top = (height - menu_height) / 2;
    render_title(width, top, large_title);
    let options_y = top + title_height + gap;
    hud::mode_options(width, options_y, computer, large_options);
    let controls_y = options_y + option_height + gap;
    hud::centered_control(width, controls_y, "Enter", "Play");
    hud::centered_control(width, controls_y + instruction_gap, "Esc", "Exit");
}

pub(super) fn render(game: &Game, width: i32, height: i32) {
    clear(width, height);
    if game.state == State::ModeSelect {
        selection(game, width, height);
        return;
    }
    let large = width >= 32 && height >= 27;
    let bordered = width >= 26 && height >= 19;
    let large_score = width >= 32 && height >= 21;
    let score_height = if large_score { 4 } else { 2 };
    // Terminal cells are about twice as tall as they are wide. The large layout
    // uses an even cell width and four-column marks, keeping both the board and
    // every label on the same visual center line.
    let (cell_width, cell_height) = if large {
        (10, 5)
    } else if bordered {
        (5, 3)
    } else {
        (3, 1)
    };
    let board_width = cell_width * 3 + 2;
    let board_height = cell_height * 3 + 2;
    let base_height = board_height + score_height + 6;
    let spacing = i32::from(height >= base_height + 3);
    let top = (height - (base_height + spacing * 3)) / 2;
    let x = (width - board_width) / 2;
    let y = top + score_height + 3 + spacing * 2;
    centered(width, top, "T I C   T A C   T O E", TITLE);
    hud::scoreboard(
        width,
        top + 1 + spacing,
        game.x_score,
        game.o_score,
        large_score,
    );
    let status = match game.state {
        State::Draw => "DRAW",
        State::Finished(Mark::X) => "X WINS",
        State::Finished(Mark::O) if game.mode == Mode::Singleplayer => "COMPUTER WINS",
        State::Finished(Mark::O) => "O WINS",
        _ if game.mode == Mode::Singleplayer => "YOUR TURN (X)",
        _ if game.turn == Mark::X => "X TURN",
        _ => "O TURN",
    };
    let active_mark = match game.state {
        State::Finished(mark) => mark,
        _ => game.turn,
    };
    centered(
        width,
        top + score_height + 2 + spacing,
        status,
        if game.state == State::Draw {
            WHITE
        } else if active_mark == Mark::O {
            PINK
        } else {
            CYAN
        },
    );
    let winning_line = winning_line(&game.board);
    let draw_lit = game.draw_is_lit();
    for index in 0..9 {
        let selected = game.state == State::Playing && game.cursor == index;
        let winning = winning_line.is_some_and(|line| line.contains(&index));
        let bg = if draw_lit { DRAW_PANEL } else { PANEL };
        let color = if game.board[index] == Mark::O {
            PINK
        } else {
            CYAN
        };
        let cx = x + index as i32 % 3 * (cell_width + 1);
        let cy = y + index as i32 / 3 * (cell_height + 1);
        for row in 0..cell_height {
            for column in 0..cell_width {
                cell(cx + column, cy + row, ' ', WHITE, bg);
            }
        }
        if large {
            let glyph = match game.board[index] {
                Mark::X => ["█  █", " ██ ", "█  █"],
                Mark::O => ["████", "█  █", "████"],
                Mark::Empty => ["    "; 3],
            };
            for (row, value) in glyph.iter().enumerate() {
                text(
                    cx + (cell_width - 4) / 2,
                    cy + 1 + row as i32,
                    value,
                    color,
                    bg,
                );
            }
        } else {
            cell(
                cx + cell_width / 2,
                cy + cell_height / 2,
                match game.board[index] {
                    Mark::X => 'X',
                    Mark::O => 'O',
                    Mark::Empty => ' ',
                },
                color,
                bg,
            );
        }
        if selected || winning {
            let border_color = if selected { CYAN } else { color };
            if cell_height == 1 {
                cell(cx, cy, '[', border_color, bg);
                cell(cx + cell_width - 1, cy, ']', border_color, bg);
            } else {
                for row in 0..cell_height {
                    for column in 0..cell_width {
                        let glyph = match (column, row) {
                            (0, 0) => '┌',
                            (c, 0) if c == cell_width - 1 => '┐',
                            (0, r) if r == cell_height - 1 => '└',
                            (c, r) if c == cell_width - 1 && r == cell_height - 1 => '┘',
                            (_, r) if r == 0 || r == cell_height - 1 => '─',
                            (c, _) if c == 0 || c == cell_width - 1 => '│',
                            _ => continue,
                        };
                        cell(cx + column, cy + row, glyph, border_color, bg);
                    }
                }
            }
        }
    }
    for row in 0..board_height {
        for column in 0..board_width {
            let vertical = column == cell_width || column == 2 * cell_width + 1;
            let horizontal = row == cell_height || row == 2 * cell_height + 1;
            if vertical || horizontal {
                cell(
                    x + column,
                    y + row,
                    if vertical && horizontal {
                        '┼'
                    } else if vertical {
                        '│'
                    } else {
                        '─'
                    },
                    LINE,
                    BACKGROUND,
                );
            }
        }
    }
    if game.state == State::Draw {
        centered(
            width,
            y + board_height + 2 + spacing,
            "New round shortly",
            MUTED,
        );
    } else if matches!(game.state, State::Finished(_)) {
        hud::centered_control(width, y + board_height + 2 + spacing, "Enter", "Play again");
    } else {
        hud::centered_control(width, y + board_height + 1, "↑↓←→ / WASD", "Move");
        hud::centered_control(width, y + board_height + 2 + spacing, "Enter", "Place");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hud::tests::{WIDTH, assert_in_bounds, capture};

    fn assert_text_centered(frame: &[(char, i32)], width: i32, text: &str) {
        let expected_x = (width - text.chars().count() as i32) / 2;
        let found = frame
            .chunks(WIDTH as usize)
            .take(80)
            .enumerate()
            .find_map(|(y, row)| {
                row.windows(text.chars().count())
                    .position(|cells| cells.iter().map(|cell| cell.0).collect::<String>() == text)
                    .map(|x| (x as i32, y))
            });
        assert_eq!(
            found.map(|(x, _)| x),
            Some(expected_x),
            "{text:?} is not centered"
        );
    }

    #[test]
    fn every_centered_label_uses_the_viewport_center() {
        for width in [79, 80, 81] {
            let height = 40;
            let mut game = Game::new();
            let frame = capture(|| render(&game, width, height));
            assert_text_centered(&frame, width, "[Enter] Play");
            assert_text_centered(&frame, width, "[Esc] Exit");

            game.state = State::Playing;
            let frame = capture(|| render(&game, width, height));
            for text in [
                "T I C   T A C   T O E",
                "SCORE",
                "YOUR TURN (X)",
                "[↑↓←→ / WASD] Move",
                "[Enter] Place",
            ] {
                assert_text_centered(&frame, width, text);
            }

            game.state = State::Draw;
            game.board = [
                Mark::X,
                Mark::O,
                Mark::X,
                Mark::X,
                Mark::O,
                Mark::O,
                Mark::O,
                Mark::X,
                Mark::X,
            ];
            let frame = capture(|| render(&game, width, height));
            assert_text_centered(&frame, width, "DRAW");
            assert_text_centered(&frame, width, "New round shortly");

            game.state = State::Finished(Mark::X);
            let frame = capture(|| render(&game, width, height));
            assert_text_centered(&frame, width, "X WINS");
            assert_text_centered(&frame, width, "[Enter] Play again");
        }
    }

    #[test]
    fn screens_fit_at_every_layout_breakpoint() {
        for width in [26, 31, 32, 47, 48, 59, 60, 80, 120] {
            for height in [13, 15, 17, 18, 19, 20, 21, 24, 26, 27, 30, 40] {
                for mode in [Mode::Singleplayer, Mode::Multiplayer] {
                    let mut game = Game::new();
                    game.mode = mode;
                    assert_in_bounds(width, height, || render(&game, width, height));
                    game.state = State::Playing;
                    game.x_score = u32::MAX;
                    game.o_score = u32::MAX;
                    assert_in_bounds(width, height, || render(&game, width, height));
                    game.board = [
                        Mark::X,
                        Mark::O,
                        Mark::X,
                        Mark::X,
                        Mark::O,
                        Mark::O,
                        Mark::O,
                        Mark::X,
                        Mark::X,
                    ];
                    game.state = State::Draw;
                    for elapsed in [0.0, 0.5, 1.0, 1.5, 2.0, 2.5] {
                        game.draw_elapsed = elapsed;
                        assert_in_bounds(width, height, || render(&game, width, height));
                    }
                    game.board = [Mark::X; 9];
                    game.state = State::Finished(Mark::X);
                    assert_in_bounds(width, height, || render(&game, width, height));
                }
            }
        }
        for width in 0..26 {
            for height in 0..13 {
                assert_in_bounds(width, height, || render_small(width, height));
            }
        }
    }
}
