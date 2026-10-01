// Shared terminal typography for the bundled games.
use super::{BACKGROUND, CYAN, MUTED, WHITE, cell, text};

pub(super) fn mode_options(width: i32, y: i32, computer: bool, large: bool) {
    let labels = ["1P", "2P"];
    // Match the selector width parity to the viewport so both outer margins are
    // identical on even and odd terminal widths.
    let gap = if width % 2 == 0 { 6 } else { 7 };
    let label_width = if large { 7 } else { 2 };
    // Reserve two blank cells between each chevron and its label.
    let option_width = label_width + 6;
    let mut x = (width - (option_width * 2 + gap)) / 2;
    for (index, label) in labels.into_iter().enumerate() {
        let selected = (index == 0) == computer;
        if selected {
            let row = y + i32::from(large);
            cell(x, row, '❯', CYAN, BACKGROUND);
            cell(x + option_width - 1, row, '❮', CYAN, BACKGROUND);
        }
        let color = if selected { WHITE } else { MUTED };
        if large {
            for (offset, byte) in label.bytes().enumerate() {
                let rows = match byte {
                    b'1' | b'2' => DIGITS[(byte - b'0') as usize],
                    b'P' => [6, 5, 6, 4, 4],
                    _ => [0; 5],
                };
                glyph(x + 3 + offset as i32 * 4, y, rows, color);
            }
        } else {
            text(x + 3, y, label, color, BACKGROUND);
        }
        x += option_width + gap;
    }
}

const DIGITS: [[u8; 5]; 10] = [
    [7, 5, 5, 5, 7],
    [2, 6, 2, 2, 7],
    [7, 1, 7, 4, 7],
    [7, 1, 7, 1, 7],
    [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7],
    [7, 4, 7, 5, 7],
    [7, 1, 1, 1, 1],
    [7, 5, 7, 5, 7],
    [7, 5, 7, 1, 7],
];

pub(super) fn control(x: i32, y: i32, key: &str, action: &str) {
    let button = format!("[{key}]");
    text(x, y, &button, CYAN, BACKGROUND);
    text(
        x + button.chars().count() as i32 + 1,
        y,
        action,
        MUTED,
        BACKGROUND,
    );
}

pub(super) fn centered_control(width: i32, y: i32, key: &str, action: &str) {
    let length = key.chars().count() + action.chars().count() + 3;
    control((width - length as i32) / 2, y, key, action);
}

pub(super) fn scoreboard(width: i32, y: i32, left: u32, right: u32, large: bool) {
    let left = format!("{left:02}");
    let right = format!("{right:02}");
    let large_width = ((left.len() + right.len()) * 4 + 1) as i32;
    let large = large && large_width <= width;
    let score_width = if large {
        large_width
    } else {
        (left.len() + right.len() + 3) as i32
    };
    let start = (width - score_width) / 2;
    // Align the caption with the actual score bounds, with no empty row before the digits.
    text(start + (score_width - 5) / 2, y, "SCORE", MUTED, BACKGROUND);
    if !large {
        text(start, y + 1, &left, CYAN, BACKGROUND);
        text(start + left.len() as i32, y + 1, " : ", MUTED, BACKGROUND);
        text(
            start + left.len() as i32 + 3,
            y + 1,
            &right,
            WHITE,
            BACKGROUND,
        );
        return;
    }
    let right_start = start + left.len() as i32 * 4 + 2;
    for (value, x, color) in [(&left, start, CYAN), (&right, right_start, WHITE)] {
        for (index, byte) in value.bytes().enumerate() {
            glyph(
                x + index as i32 * 4,
                y + 1,
                DIGITS[(byte - b'0') as usize],
                color,
            );
        }
    }
    // Five half-cell pixels form a digit. Put the colon at pixels 1 and 3,
    // symmetrically around the middle pixel, with equal horizontal gaps.
    let colon_x = start + left.len() as i32 * 4;
    cell(colon_x, y + 1, '▄', MUTED, BACKGROUND);
    cell(colon_x, y + 2, '▄', MUTED, BACKGROUND);
}

fn glyph(x: i32, y: i32, rows: [u8; 5], color: i32) {
    for row in 0..3 {
        for column in 0..3 {
            let upper = rows[row * 2] & (1 << (2 - column)) != 0;
            let lower = row * 2 + 1 < 5 && rows[row * 2 + 1] & (1 << (2 - column)) != 0;
            let character = match (upper, lower) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                _ => ' ',
            };
            cell(x + column, y + row as i32, character, color, BACKGROUND);
        }
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use std::cell::RefCell;

    pub(crate) type Pixel = (char, i32);
    thread_local! {
        static FRAME: RefCell<Vec<Pixel>> = const { RefCell::new(Vec::new()) };
    }
    pub(crate) const WIDTH: i32 = 120;
    const HEIGHT: i32 = 80;

    #[unsafe(no_mangle)]
    extern "C" fn draw_cell(x: i32, y: i32, character: i32, foreground: i32, _background: i32) {
        assert!((0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y));
        FRAME.with(|frame| {
            frame.borrow_mut()[(y * WIDTH + x) as usize] =
                (char::from_u32(character as u32).unwrap(), foreground)
        });
    }

    pub(crate) fn capture(draw: impl FnOnce()) -> Vec<Pixel> {
        FRAME
            .with(|frame| *frame.borrow_mut() = vec![(' ', BACKGROUND); (WIDTH * HEIGHT) as usize]);
        draw();
        FRAME.with(|frame| frame.borrow().clone())
    }

    pub(crate) fn assert_in_bounds(width: i32, height: i32, draw: impl FnOnce()) {
        let frame = capture(draw);
        for (index, pixel) in frame.iter().enumerate() {
            let x = index as i32 % WIDTH;
            let y = index as i32 / WIDTH;
            if x >= width || y >= height {
                assert_eq!(
                    *pixel,
                    (' ', BACKGROUND),
                    "drawing outside {width}x{height} at {x},{y}"
                );
            }
        }
    }

    #[test]
    fn score_caption_and_colon_align_with_the_digit_pixels() {
        let frame = capture(|| scoreboard(80, 0, 0, 8, true));
        let at = |x: i32, y: i32| frame[(y * WIDTH + x) as usize].0;
        assert_eq!((37..42).map(|x| at(x, 0)).collect::<String>(), "SCORE");
        assert_eq!(at(31, 1), '█'); // Digits immediately follow the caption row.
        assert_eq!(at(39, 1), '▄'); // Colon pixels 1 and 3, centered within 0..4.
        assert_eq!(at(39, 2), '▄');
        assert_eq!(at(39, 3), ' ');
        assert_eq!(at(38, 1), ' ');
        assert_eq!(at(40, 1), ' ');
        for scores in [(99, 100), (u32::MAX, u32::MAX)] {
            capture(|| scoreboard(80, 0, scores.0, scores.1, true));
        }
    }

    #[test]
    fn controls_color_only_the_bracketed_button_orange() {
        let frame = capture(|| centered_control(80, 0, "Enter", "Play"));
        assert_eq!(
            frame[34..46]
                .iter()
                .map(|pixel| pixel.0)
                .collect::<String>(),
            "[Enter] Play"
        );
        assert!(frame[34..41].iter().all(|pixel| pixel.1 == CYAN));
        assert!(frame[42..46].iter().all(|pixel| pixel.1 == MUTED));
    }

    #[test]
    fn both_mode_choices_leave_two_cells_between_the_arrows_and_labels() {
        for (width, large) in [(24, false), (80, true)] {
            for computer in [false, true] {
                let frame = capture(|| mode_options(width, 0, computer, large));
                let row = i32::from(large);
                let line = &frame[(row * WIDTH) as usize..(row * WIDTH + width) as usize];
                let left = line.iter().position(|pixel| pixel.0 == '❯').unwrap();
                let right = line.iter().position(|pixel| pixel.0 == '❮').unwrap();
                assert!(line[left + 1..left + 3].iter().all(|pixel| pixel.0 == ' '));
                assert!(line[right - 2..right].iter().all(|pixel| pixel.0 == ' '));
                assert_eq!(line[left].1, CYAN);
                assert_eq!(line[right].1, CYAN);
                if !large {
                    let label = if computer { "1P" } else { "2P" };
                    assert_eq!(
                        line[left + 3..right - 2]
                            .iter()
                            .map(|pixel| pixel.0)
                            .collect::<String>(),
                        label
                    );
                }
            }
        }
    }
}
