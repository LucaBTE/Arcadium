use crate::{Game, Mark, Mode, State, winning_line};

const BACKGROUND: i32 = 0x0b1020;
const PANEL: i32 = 0x121c32;
const SELECTED: i32 = 0x173347;
const CYAN: i32 = 0x67e8f9;
const PINK: i32 = 0xf0abfc;
const WHITE: i32 = 0xf1f5f9;
const MUTED: i32 = 0x94a3b8;
const LINE: i32 = 0x293a55;

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
        centered(width, top, "T I C   T A C   T O E", CYAN);
        return;
    }
    let letters = [
        ["─────", "  │  ", "  │  ", "  │  ", "  ╵  "],
        ["──┬──", "  │  ", "  │  ", "  │  ", "──┴──"],
        ["╭────", "│    ", "│    ", "│    ", "╰────"],
        ["─────", "  │  ", "  │  ", "  │  ", "  ╵  "],
        ["╭───╮", "│   │", "├───┤", "│   │", "╵   ╵"],
        ["╭────", "│    ", "│    ", "│    ", "╰────"],
        ["─────", "  │  ", "  │  ", "  │  ", "  ╵  "],
        ["╭───╮", "│   │", "│   │", "│   │", "╰───╯"],
        ["╭────", "│    ", "├─── ", "│    ", "╰────"],
    ];
    let x = (width - 57) / 2;
    for (letter, rows) in letters.iter().enumerate() {
        for (row, line) in rows.iter().enumerate() {
            text(
                x + letter as i32 * 6 + (letter / 3) as i32 * 2,
                top + row as i32,
                line,
                CYAN,
                BACKGROUND,
            );
        }
    }
}

fn selection(game: &Game, width: i32, height: i32) {
    let computer = game.mode == Mode::Singleplayer;
    let large_title = width >= 60 && height >= 17;
    let title_height = if large_title { 5 } else { 1 };
    let menu_height = title_height + 8;
    let top = (height - menu_height) / 2;
    render_title(width, top, large_title);
    // Terminal cells are roughly twice as tall as they are wide.
    let card_width = 10.min((width - 4) / 2);
    let x = (width - (card_width * 2 + 2)) / 2;
    let y = top + title_height + 1;
    for (index, label, selected) in [(0, "1P", computer), (1, "2P", !computer)] {
        let left = x + index * (card_width + 2);
        let background = if selected { SELECTED } else { PANEL };
        let foreground = if selected { CYAN } else { MUTED };
        let border = if selected {
            ['╔', '╗', '╚', '╝', '═', '║']
        } else {
            ['┌', '┐', '└', '┘', '─', '│']
        };
        for row in 0..5 {
            for column in 0..card_width {
                let character = match (column, row) {
                    (0, 0) => border[0],
                    (c, 0) if c == card_width - 1 => border[1],
                    (0, 4) => border[2],
                    (c, 4) if c == card_width - 1 => border[3],
                    (_, 0 | 4) => border[4],
                    (c, _) if c == 0 || c == card_width - 1 => border[5],
                    _ => ' ',
                };
                cell(left + column, y + row, character, foreground, background);
            }
        }
        text(
            left + (card_width - 2) / 2,
            y + 2,
            label,
            foreground,
            background,
        );
    }
    centered(
        width,
        top + menu_height - 1,
        "←→ Choose   Enter Play",
        MUTED,
    );
}

pub(super) fn render(game: &Game, width: i32, height: i32) {
    clear(width, height);
    if game.state == State::ModeSelect {
        selection(game, width, height);
        return;
    }
    let large = width >= 32 && height >= 19;
    let (cell_width, cell_height) = if large { (7, 3) } else { (3, 1) };
    let board_width = cell_width * 3 + 2;
    let board_height = cell_height * 3 + 2;
    let top = (height - (board_height + 8)) / 2;
    let x = (width - board_width) / 2;
    let y = top + 4;
    centered(width, top, "T I C   T A C   T O E", CYAN);
    let status = match game.state {
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
        top + 2,
        status,
        if active_mark == Mark::O { PINK } else { CYAN },
    );
    let winning_line = winning_line(&game.board);
    for index in 0..9 {
        let selected = game.state == State::Playing && game.cursor == index;
        let winning = winning_line.is_some_and(|line| line.contains(&index));
        let bg = if selected || winning { SELECTED } else { PANEL };
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
                Mark::X => ["█ █", " █ ", "█ █"],
                Mark::O => ["█▀█", "█ █", "█▄█"],
                Mark::Empty => ["   "; 3],
            };
            for (row, value) in glyph.iter().enumerate() {
                text(cx + 2, cy + row as i32, value, color, bg);
            }
        } else {
            cell(
                cx + 1,
                cy,
                match game.board[index] {
                    Mark::X => 'X',
                    Mark::O => 'O',
                    Mark::Empty => ' ',
                },
                color,
                bg,
            );
        }
        if selected {
            cell(cx, cy + cell_height / 2, '[', WHITE, bg);
            cell(cx + cell_width - 1, cy + cell_height / 2, ']', WHITE, bg);
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
    if matches!(game.state, State::Finished(_)) {
        centered(width, y + board_height + 2, "Enter  Play again", WHITE);
    } else {
        centered(width, y + board_height + 1, "Arrows / WASD  Move", MUTED);
        centered(width, y + board_height + 2, "Enter  Place", WHITE);
    }
}
