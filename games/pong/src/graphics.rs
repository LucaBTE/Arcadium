#[path = "../../common/hud.rs"]
mod hud;

use crate::{BALL_RADIUS_X, BALL_WIDTH, Game, PADDLE_HEIGHT, PADDLE_INSET, play_top};

const BACKGROUND: i32 = 0x0e101d;
const CYAN: i32 = 0xff7338;
const PINK: i32 = 0xf4e7d3;
const WHITE: i32 = 0xf4e7d3;
const MUTED: i32 = 0xa99e94;
const LINE: i32 = 0x68402f;
const SHADOW: i32 = 0x702b1e;

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
        [30, 17, 30, 16, 16],
        [14, 17, 17, 17, 14],
        [17, 25, 21, 19, 17],
        [15, 16, 23, 17, 15],
    ];
    let x = (width - 46) / 2;
    for (letter, rows) in letters.iter().enumerate() {
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    text(
                        x + letter as i32 * 12 + column * 2 + 1,
                        top + row as i32 + 1,
                        "▓▓",
                        SHADOW,
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
                    text(
                        x + letter as i32 * 12 + column * 2,
                        top + row as i32,
                        "▓▓",
                        CYAN,
                        BACKGROUND,
                    );
                }
            }
        }
    }
}

pub(super) fn render_selection(width: i32, height: i32, computer: bool) {
    clear(width, height);
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
    hud::centered_control(width, controls_y, "←/→", "Choose");
    hud::centered_control(width, controls_y + instruction_gap, "Enter", "Play");
}

pub(super) fn render(game: &Game, width: i32, height: i32) {
    clear(width, height);
    let top = play_top(width, height);
    for x in 0..width {
        cell(x, top - 1, '─', LINE, BACKGROUND);
    }
    for y in (top..height).step_by(2) {
        cell(width / 2, y, '┊', LINE, BACKGROUND);
    }
    hud::scoreboard(width, 0, game.left_score, game.right_score, top > 3);
    // Player identities sit beside the caption; key hints sit beside the digits.
    // Keep enough room for all score digits before showing the side hints.
    let score_width = if top > 3 {
        ((format!("{:02}", game.left_score).len() + format!("{:02}", game.right_score).len()) * 4
            + 1) as i32
    } else {
        (format!("{:02}", game.left_score).len() + format!("{:02}", game.right_score).len() + 3)
            as i32
    };
    if width >= score_width + 32 {
        text(
            2,
            0,
            if game.computer { "YOU" } else { "1P" },
            CYAN,
            BACKGROUND,
        );
        let right_label = if game.computer { "CPU" } else { "2P" };
        text(
            width - 2 - right_label.len() as i32,
            0,
            right_label,
            WHITE,
            BACKGROUND,
        );
        hud::control(2, 1, if game.computer { "↑/↓" } else { "W/S" }, "Move");
        if !game.computer {
            hud::control(width - 14, 1, "↑/↓", "Move");
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use hud::tests::assert_in_bounds;

    #[test]
    fn screens_fit_at_every_layout_breakpoint() {
        for width in [24, 47, 48, 59, 60, 80, 120] {
            for height in [10, 14, 15, 16, 17, 18, 24, 40] {
                for computer in [false, true] {
                    assert_in_bounds(width, height, || render_selection(width, height, computer));
                    let mut game = Game::new(width, height);
                    game.computer = computer;
                    assert_in_bounds(width, height, || render(&game, width, height));
                    game.left_score = u32::MAX;
                    game.right_score = u32::MAX;
                    assert_in_bounds(width, height, || render(&game, width, height));
                }
            }
        }
        for width in 0..24 {
            for height in 0..10 {
                assert_in_bounds(width, height, || render_small(width, height));
            }
        }
    }
}
