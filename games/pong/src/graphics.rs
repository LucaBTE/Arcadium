use crate::{BALL_RADIUS_X, BALL_WIDTH, Game, PADDLE_HEIGHT, PADDLE_INSET, PLAY_TOP};

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
    // Arcadium validates coordinates and colors before writing its framebuffer.
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

fn render_title(width: i32, top: i32, large: bool) {
    if !large {
        centered(width, top, "P O N G", CYAN);
        return;
    }
    let letters = [
        ["╭────╮", "│    │", "├────╯", "│     ", "╵     "],
        ["╭────╮", "│    │", "│    │", "│    │", "╰────╯"],
        ["╷    ╷", "│╲   │", "│ ╲  │", "│  ╲ │", "╵   ╲╵"],
        ["╭────╮", "│     ", "│  ──┐", "│    │", "╰────╯"],
    ];
    let x = (width - 33) / 2;
    for (letter, rows) in letters.iter().enumerate() {
        for (row, line) in rows.iter().enumerate() {
            text(
                x + letter as i32 * 9,
                top + row as i32,
                line,
                CYAN,
                BACKGROUND,
            );
        }
    }
}

pub(super) fn render_selection(width: i32, height: i32, computer: bool) {
    clear(width, height);
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

fn digits(mut score: u32, buffer: &mut [u8; 10]) -> &[u8] {
    let mut start = buffer.len();
    loop {
        start -= 1;
        buffer[start] = b'0' + (score % 10) as u8;
        score /= 10;
        if score == 0 {
            return &buffer[start..];
        }
    }
}

pub(super) fn render(game: &Game, width: i32, height: i32) {
    clear(width, height);
    for x in 0..width {
        cell(x, 1, '─', LINE, BACKGROUND);
    }
    for y in (PLAY_TOP..height).step_by(2) {
        cell(width / 2, y, '┊', LINE, BACKGROUND);
    }
    let mut left_buffer = [0; 10];
    let mut right_buffer = [0; 10];
    let left = digits(game.left_score, &mut left_buffer);
    let right = digits(game.right_score, &mut right_buffer);
    let score_width = (left.len() + right.len() + 3) as i32;
    let start = (width - score_width) / 2;
    if width >= score_width + 32 {
        text(
            2,
            0,
            if game.computer {
                "YOU  ↑/↓"
            } else {
                "PLAYER 1  W/S"
            },
            CYAN,
            BACKGROUND,
        );
        let label = if game.computer {
            "COMPUTER"
        } else {
            "PLAYER 2  ↑/↓"
        };
        text(
            width - 2 - label.chars().count() as i32,
            0,
            label,
            PINK,
            BACKGROUND,
        );
    }
    for (index, &byte) in left.iter().chain(b" : ").chain(right).enumerate() {
        let color = if index < left.len() {
            CYAN
        } else if index >= left.len() + 3 {
            PINK
        } else {
            MUTED
        };
        cell(start + index as i32, 0, byte as char, color, BACKGROUND);
    }
    for offset in 0..PADDLE_HEIGHT {
        cell(
            PADDLE_INSET,
            game.left_paddle_y.round() as i32 + offset,
            '█',
            CYAN,
            BACKGROUND,
        );
        cell(
            width - 1 - PADDLE_INSET,
            game.right_paddle_y.round() as i32 + offset,
            '█',
            PINK,
            BACKGROUND,
        );
    }
    for offset in 0..BALL_WIDTH {
        cell(
            (game.ball_x - BALL_RADIUS_X).round() as i32 + offset,
            game.ball_y.round() as i32,
            '█',
            WHITE,
            BACKGROUND,
        );
    }
}

pub(super) fn render_small(width: i32, height: i32) {
    clear(width, height);
    if width >= 19 && height >= 2 {
        centered(width, height / 2 - 1, "Enlarge to 24 x 10", CYAN);
        centered(width, height / 2, "to resume Pong", MUTED);
    }
}
