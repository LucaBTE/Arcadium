#[path = "../../common/hud.rs"]
#[allow(dead_code)] // Snake uses controls, while the shared module also offers two-player UI.
mod hud;

use crate::{Field, Game, PLAY_TOP, Point, State};

const BACKGROUND: i32 = 0x0e101d;
const CYAN: i32 = 0xff7338;
const TITLE: i32 = 0xffa04d;
const TITLE_SHADOW: i32 = 0x753914;
const WHITE: i32 = 0xf4e7d3;
const MUTED: i32 = 0xa99e94;
const LINE: i32 = 0x68402f;
const RED: i32 = 0xf05252;
const RED_SHADOW: i32 = 0x6d252a;

#[link(wasm_import_module = "arcadium")]
unsafe extern "C" {
    fn draw_cell(x: i32, y: i32, character: i32, foreground: i32, background: i32);
}

fn cell(x: i32, y: i32, character: char, foreground: i32, background: i32) {
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

fn tile(point: Point, character: char, color: i32) {
    let x = 1 + point.x * 2;
    cell(x, point.y, character, color, BACKGROUND);
    cell(x + 1, point.y, character, color, BACKGROUND);
}

fn title_letter(letter: char) -> [u8; 5] {
    match letter {
        'A' => [14, 17, 31, 17, 17],
        'E' => [31, 16, 30, 16, 31],
        'G' => [15, 16, 23, 17, 15],
        'K' => [17, 18, 28, 18, 17],
        'M' => [17, 27, 21, 17, 17],
        'N' => [17, 25, 21, 19, 17],
        'O' => [14, 17, 17, 17, 14],
        'R' => [30, 17, 30, 18, 17],
        'S' => [31, 16, 31, 1, 31],
        'V' => [17, 17, 17, 10, 4],
        _ => [0; 5],
    }
}

fn arcade_title(width: i32, y: i32, word: &str, scale: i32, foreground: i32, shadow: i32) {
    let title_width = word
        .chars()
        .map(|character| if character == ' ' { 2 } else { 5 * scale + 2 })
        .sum::<i32>()
        - 1;
    let start = (width - title_width) / 2;
    for (offset_x, offset_y, color) in [(1, 1, shadow), (0, 0, foreground)] {
        let mut x = start;
        for character in word.chars() {
            if character == ' ' {
                x += 2;
                continue;
            }
            for (row, bits) in title_letter(character).into_iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) != 0 {
                        for part in 0..scale {
                            cell(
                                x + column * scale + part + offset_x,
                                y + row as i32 + offset_y,
                                '▓',
                                color,
                                BACKGROUND,
                            );
                        }
                    }
                }
            }
            x += 5 * scale + 2;
        }
    }
}

pub(super) fn render_small(width: i32, height: i32) {
    clear(width, height);
    if width >= 18 && height > 0 {
        centered(width, height / 2, "TERMINAL TOO SMALL", MUTED);
    }
}

pub(super) fn render(game: &Game, width: i32, height: i32) {
    clear(width, height);
    if game.state == State::Ready {
        render_start(width, height);
        return;
    }
    centered(width, 0, "S N A K E", TITLE);
    let scores = format!(
        "SCORE: {:04}    BEST SCORE: {:04}",
        game.score, game.best_score
    );
    let scores = if scores.len() <= width as usize {
        scores
    } else {
        format!("S:{}  BEST:{}", game.score, game.best_score)
    };
    text(
        ((width - scores.len() as i32) / 2).max(0),
        1,
        &scores.chars().take(width as usize).collect::<String>(),
        WHITE,
        BACKGROUND,
    );
    for x in 0..width {
        cell(x, PLAY_TOP - 1, '─', LINE, BACKGROUND);
    }
    let field = Field::new(width, height);
    for &point in &game.rocks {
        if field.contains(point) {
            tile(point, '▓', LINE);
        }
    }
    for (index, &point) in game.snake.iter().enumerate() {
        if field.contains(point) {
            tile(point, '█', if index == 0 { CYAN } else { WHITE });
        }
    }
    if field.contains(game.food)
        && !game.snake.contains(&game.food)
        && !game.rocks.contains(&game.food)
    {
        tile(game.food, '█', WHITE);
    }
    if matches!(game.state, State::GameOver | State::Won) {
        render_end_screen(game, width, height);
        return;
    }
    let hint_y = height - 1;
    match game.state {
        State::Ready => unreachable!(),
        State::Playing => hud::centered_control(
            width,
            hint_y,
            "↑↓←→ / WASD",
            if width >= 38 {
                "Move / hold to dash"
            } else {
                "Move"
            },
        ),
        State::GameOver | State::Won => unreachable!(),
    }
}

fn render_start(width: i32, height: i32) {
    let control_gap = if height >= 13 { 2 } else { 1 };
    if width >= 60 && height >= 17 {
        let title_to_actions_gap = 6;
        let top = (height - (6 + title_to_actions_gap + control_gap)) / 2;
        arcade_title(width, top, "SNAKE", 2, TITLE, TITLE_SHADOW);
        let actions_y = top + 5 + title_to_actions_gap;
        hud::centered_control(width, actions_y, "Enter", "Play");
        hud::centered_control(width, actions_y + control_gap, "Esc", "Exit");
    } else {
        let top = (height - (3 + control_gap)) / 2;
        centered(width, top, "S N A K E", TITLE);
        hud::centered_control(width, top + 2, "Enter", "Play");
        hud::centered_control(width, top + 2 + control_gap, "Esc", "Exit");
    }
}

fn render_end_screen(game: &Game, width: i32, height: i32) {
    let large_title = width >= 52 && height >= 15 && game.state == State::GameOver;
    let top = if large_title {
        (height - 10) / 2
    } else {
        (height - 5) / 2
    };
    let panel_width = if large_title { 58 } else { width.min(30) };
    let panel_x = (width - panel_width) / 2;
    let panel_height = if large_title { 10 } else { 5 };
    for y in top..top + panel_height {
        for x in panel_x..panel_x + panel_width {
            cell(x, y, ' ', WHITE, BACKGROUND);
        }
    }
    if large_title {
        arcade_title(width, top, "GAME OVER", 1, RED, RED_SHADOW);
    } else {
        centered(
            width,
            top,
            if game.state == State::Won {
                "BOARD COMPLETE"
            } else {
                "GAME OVER"
            },
            CYAN,
        );
    }
    let choices_y = top + if large_title { 7 } else { 2 };
    menu_choice(width, choices_y, "RESTART", !game.exit_selected);
    menu_choice(
        width,
        choices_y + if large_title { 2 } else { 1 },
        "EXIT",
        game.exit_selected,
    );
    let hint_y = height - 1;
    if large_title {
        let x = (width - 29) / 2;
        hud::control(x, hint_y, "↑/↓", "Choose");
        hud::control(x + 15, hint_y, "Enter", "Select");
    } else {
        centered(width, hint_y, "↑↓ CHOOSE  ENTER SELECT", MUTED);
    }
}

fn menu_choice(width: i32, y: i32, label: &str, selected: bool) {
    let left = (width - 20) / 2;
    if selected {
        cell(left, y, '❯', CYAN, BACKGROUND);
        cell(left + 19, y, '❮', CYAN, BACKGROUND);
    }
    text(
        left + (20 - label.len() as i32) / 2,
        y,
        label,
        if selected { WHITE } else { MUTED },
        BACKGROUND,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use hud::tests::{WIDTH, assert_in_bounds, capture};

    #[test]
    fn logical_cells_are_two_columns_wide_and_one_row_high() {
        let mut game = Game::new(Field::new(40, 20));
        game.state = State::Playing;
        let head = game.snake[0];
        let frame = capture(|| render(&game, 40, 20));
        let x = 1 + head.x * 2;
        let at = |x: i32, y: i32| frame[(y * WIDTH + x) as usize].0;
        assert_eq!(at(x, head.y), '█');
        assert_eq!(at(x + 1, head.y), '█');
        assert_eq!(at(x - 1, head.y), '█');
        assert_ne!(at(x, head.y - 1), '█');
    }

    #[test]
    fn screens_stay_within_viewport() {
        for width in [24, 25, 40, 80, 120] {
            for height in [10, 11, 24, 80] {
                let field = Field::new(width, height);
                let mut game = Game::new(field);
                for state in [State::Ready, State::Playing, State::GameOver, State::Won] {
                    game.state = state;
                    assert_in_bounds(width, height, || render(&game, width, height));
                }
                game.score = u32::MAX;
                game.best_score = u32::MAX;
                assert_in_bounds(width, height, || render(&game, width, height));
            }
        }
        for width in 0..24 {
            for height in 0..10 {
                assert_in_bounds(width, height, || render_small(width, height));
            }
        }
    }

    #[test]
    fn start_screen_hides_board_and_menu_rows_share_a_center() {
        let mut ready = Game::new(Field::new(80, 24));
        ready.food = Point { x: 0, y: PLAY_TOP };
        let frame = capture(|| render(&ready, 80, 24));
        assert_eq!(frame[(PLAY_TOP * WIDTH + 1) as usize].0, ' ');
        assert!(
            frame[(8 * WIDTH) as usize..(14 * WIDTH) as usize]
                .iter()
                .any(|cell| cell.0 == '▓' && cell.1 == TITLE)
        );
        assert!(
            !frame
                .iter()
                .map(|cell| cell.0)
                .collect::<String>()
                .contains("EAT FOOD")
        );

        let mut game = Game::new(Field::new(80, 24));
        game.state = State::GameOver;
        game.best_score = 9;
        let frame = capture(|| render(&game, 80, 24));
        let row = |y: usize| {
            frame[y * WIDTH as usize..y * WIDTH as usize + 80]
                .iter()
                .map(|cell| cell.0)
                .collect::<String>()
        };
        assert!(row(1).contains("BEST SCORE: 0009"));
        assert_eq!(
            row(14).chars().skip(30).take(20).collect::<String>(),
            "❯     RESTART      ❮"
        );
        assert_eq!(row(16).chars().skip(38).take(4).collect::<String>(), "EXIT");
        assert!(
            frame[(7 * WIDTH) as usize..(12 * WIDTH) as usize]
                .iter()
                .any(|cell| cell.0 == '▓' && cell.1 == RED)
        );

        game.exit_selected = true;
        let frame = capture(|| render(&game, 80, 24));
        let row = frame[(16 * WIDTH) as usize..(17 * WIDTH) as usize]
            .iter()
            .map(|cell| cell.0)
            .collect::<String>();
        assert_eq!(
            row.chars().skip(30).take(20).collect::<String>(),
            "❯       EXIT       ❮"
        );
    }
}
