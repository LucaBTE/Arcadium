use crate::{Game, PADDLE_HEIGHT, PADDLE_INSET, PLAY_TOP};

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

pub(super) fn render_selection(width: i32, height: i32, computer: bool) {
    clear(width, height);
    if width >= 40 && height >= 18 {
        let top = (height - 16) / 2;
        for (row, line) in [
            "█▀▀█  █▀▀█  █▄  █  █▀▀▀",
            "█▄▄█  █  █  █ █ █  █ ▄▄",
            "█     █▄▄█  █  ▀█  █▄▄█",
        ]
        .iter()
        .enumerate()
        {
            centered(
                width,
                top + row as i32,
                line,
                if row == 1 { WHITE } else { CYAN },
            );
        }
        centered(width, top + 4, "T H E   T E R M I N A L   C O U R T", MUTED);
        let card_width = 34;
        let x = (width - card_width) / 2;
        for (index, label, detail, selected) in [
            (0, "One player", "You vs Computer", computer),
            (1, "Two players", "Local head-to-head", !computer),
        ] {
            let y = top + 6 + index * 4;
            let background = if selected { SELECTED } else { PANEL };
            let accent = if selected { CYAN } else { MUTED };
            for row in 0..3 {
                for col in 0..card_width {
                    cell(
                        x + col,
                        y + row,
                        if col == 0 { '▌' } else { ' ' },
                        accent,
                        background,
                    );
                }
            }
            text(
                x + 2,
                y,
                if selected { "›" } else { " " },
                accent,
                background,
            );
            text(x + 4, y, label, WHITE, background);
            text(x + 4, y + 1, detail, MUTED, background);
        }
        centered(width, top + 14, "W/S or ↑/↓  SELECT    ENTER  PLAY", WHITE);
        centered(
            width,
            top + 15,
            if computer {
                "W/S moves your paddle"
            } else {
                "Left: W/S    Right: ↑/↓"
            },
            MUTED,
        );
    } else {
        let top = (height - 8) / 2;
        centered(width, top, "P O N G", CYAN);
        centered(
            width,
            top + 2,
            if computer {
                "› One player"
            } else {
                "  One player"
            },
            if computer { CYAN } else { MUTED },
        );
        centered(
            width,
            top + 3,
            if computer {
                "  Two players"
            } else {
                "› Two players"
            },
            if computer { MUTED } else { CYAN },
        );
        centered(width, top + 5, "W/S or ↑/↓: select", WHITE);
        centered(width, top + 6, "ENTER: play", WHITE);
        centered(
            width,
            top + 7,
            if computer {
                "W/S vs Computer"
            } else {
                "W/S vs ↑/↓"
            },
            MUTED,
        );
    }
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
        text(2, 0, "YOU  W/S", CYAN, BACKGROUND);
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
    cell(
        game.ball_x.round() as i32,
        game.ball_y.round() as i32,
        '●',
        WHITE,
        BACKGROUND,
    );
}

pub(super) fn render_small(width: i32, height: i32) {
    clear(width, height);
    if width >= 19 && height >= 2 {
        centered(width, height / 2 - 1, "Enlarge to 24 x 10", CYAN);
        centered(width, height / 2, "to resume Pong", MUTED);
    }
}
