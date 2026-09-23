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

#[derive(Default)]
pub(crate) struct HostState {
    width: u16,
    height: u16,
    screen: Vec<char>,
    keys: [bool; 10],
}

impl HostState {
    pub fn resize(&mut self, width: u16, height: u16) {
        if (self.width, self.height) != (width, height) {
            self.width = width;
            self.height = height;
            self.screen = vec![' '; usize::from(width) * usize::from(height)];
        }
    }

    pub fn begin_frame(&mut self) {
        self.screen.fill(' ');
        self.keys.fill(false);
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn draw_char(&mut self, x: i32, y: i32, character: i32) {
        if x < 0 || y < 0 || x >= i32::from(self.width) || y >= i32::from(self.height) {
            return;
        }
        if let Some(character) = char::from_u32(character as u32)
            && !character.is_control()
        {
            self.screen[y as usize * usize::from(self.width) + x as usize] = character;
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

    pub fn screen(&self) -> &[char] {
        &self.screen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(host.screen(), &[' ', ' ', ' ', ' ', ' ', '@']);
        host.press_key(key::LEFT);
        host.press_key(-1);
        assert!(host.key_pressed(key::LEFT));
        assert!(!host.key_pressed(10));
        host.begin_frame();
        assert!(!host.key_pressed(key::LEFT));
        assert!(host.screen().iter().all(|&c| c == ' '));
        host.resize(0, 0);
        host.draw_char(0, 0, 65);
        assert!(host.screen().is_empty());
    }
}
