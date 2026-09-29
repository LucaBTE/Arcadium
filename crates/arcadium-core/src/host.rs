use ratatui::style::Color;

/// Integer key codes shared with Core WebAssembly guests.
pub(crate) mod key {
    pub const UP: i32 = 0;
    pub const DOWN: i32 = 1;
    pub const LEFT: i32 = 2;
    pub const RIGHT: i32 = 3;
    pub const W: i32 = 4;
    pub const A: i32 = 5;
    pub const S: i32 = 6;
    pub const D: i32 = 7;
    pub const SPACE: i32 = 8;
    pub const ENTER: i32 = 9;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScreenCell {
    pub character: char,
    pub foreground: Color,
    pub background: Color,
}

impl Default for ScreenCell {
    fn default() -> Self {
        Self {
            character: ' ',
            foreground: Color::White,
            background: Color::Reset,
        }
    }
}

fn color(rgb: i32) -> Option<Color> {
    match rgb {
        -1 => Some(Color::Reset),
        0..=0xffffff => Some(Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)),
        _ => None,
    }
}

#[derive(Default)]
pub(crate) struct HostState {
    width: u16,
    height: u16,
    screen: Vec<ScreenCell>,
    keys: [bool; 10],
}

impl HostState {
    pub fn resize(&mut self, width: u16, height: u16) {
        if (self.width, self.height) != (width, height) {
            self.width = width;
            self.height = height;
            self.screen = vec![ScreenCell::default(); usize::from(width) * usize::from(height)];
        }
    }

    pub fn begin_frame(&mut self) {
        self.screen.fill(ScreenCell::default());
        self.keys.fill(false);
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn draw_char(&mut self, x: i32, y: i32, character: i32) {
        self.draw_cell(x, y, character, 0xffffff, -1);
    }

    pub fn draw_cell(&mut self, x: i32, y: i32, character: i32, foreground: i32, background: i32) {
        if x < 0 || y < 0 || x >= i32::from(self.width) || y >= i32::from(self.height) {
            return;
        }
        if let Some(character) = char::from_u32(character as u32)
            && !character.is_control()
        {
            let (Some(foreground), Some(background)) = (color(foreground), color(background))
            else {
                return;
            };
            self.screen[y as usize * usize::from(self.width) + x as usize] = ScreenCell {
                character,
                foreground,
                background,
            };
        }
    }

    pub fn press_key(&mut self, key: i32) {
        if let Some(pressed) = self.keys.get_mut(key as usize) {
            *pressed = true;
        }
    }

    pub fn key_pressed(&self, key: i32) -> bool {
        self.keys.get(key as usize).copied().unwrap_or(false)
    }

    pub fn screen(&self) -> &[ScreenCell] {
        &self.screen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colored_cells_validate_reset_and_resize() {
        let mut host = HostState::default();
        host.resize(2, 1);
        host.draw_cell(0, 0, '█' as i32, 0x12abef, 0x102030);
        let cell = host.screen()[0];
        assert_eq!(cell.foreground, Color::Rgb(0x12, 0xab, 0xef));
        assert_eq!(cell.background, Color::Rgb(0x10, 0x20, 0x30));
        for (x, y, ch, fg, bg) in [
            (-1, 0, 65, 0, 0),
            (2, 0, 65, 0, 0),
            (0, 1, 65, 0, 0),
            (0, 0, 27, 0, 0),
            (0, 0, 0xd800, 0, 0),
            (0, 0, 65, -2, 0),
            (0, 0, 65, 0, 0x1000000),
        ] {
            host.draw_cell(x, y, ch, fg, bg);
            assert_eq!(host.screen()[0], cell);
        }
        host.draw_char(0, 0, 'A' as i32);
        assert_eq!(host.screen()[0].foreground, Color::Rgb(255, 255, 255));
        assert_eq!(host.screen()[0].background, Color::Reset);
        host.draw_cell(1, 0, 66, -1, -1);
        assert_eq!(host.screen()[1].foreground, Color::Reset);
        host.begin_frame();
        assert!(host.screen().iter().all(|c| *c == ScreenCell::default()));
        host.draw_cell(0, 0, 65, 0, 0);
        host.resize(1, 1);
        assert_eq!(host.screen()[0], ScreenCell::default());
    }

    #[test]
    fn frame_state_and_safe_drawing() {
        let mut host = HostState::default();
        host.resize(3, 2);
        host.draw_char(2, 1, '@' as i32);
        for (x, y, character) in [
            (-1, 0, 65),
            (3, 0, 65),
            (0, 2, 65),
            (0, 0, -1),
            (0, 0, 0xd800),
            (0, 0, 27),
        ] {
            host.draw_char(x, y, character);
        }
        assert_eq!(
            host.screen()
                .iter()
                .map(|cell| cell.character)
                .collect::<String>(),
            "     @"
        );
        host.press_key(key::LEFT);
        host.press_key(-1);
        assert!(host.key_pressed(key::LEFT));
        assert!(!host.key_pressed(10));
        host.begin_frame();
        assert!(!host.key_pressed(key::LEFT));
        assert!(host.screen().iter().all(|c| *c == ScreenCell::default()));
        host.resize(0, 0);
        host.draw_char(0, 0, 65);
        assert!(host.screen().is_empty());
    }
}
