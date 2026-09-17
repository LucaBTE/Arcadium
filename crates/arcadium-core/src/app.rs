use arcadium_sdk::{ArcadeGame, GameMetadata};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

use crate::{mode::AppMode, registry::GameRegistry};

//app structure, very easy
pub struct App {
    pub should_quit: bool,
    pub selected_game: usize,
    pub mode: AppMode,
    registry: GameRegistry,
}

impl App {
    //function to initialize a new application boot
    pub fn new(registry: GameRegistry) -> Self {
        Self {
            should_quit: false,
            selected_game: 0,
            mode: AppMode::Library,
            registry,
        }
    }

    //function to handle events.
    //awaits some key pressed included in the 'library_input', other keys are simply ignored
    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match self.mode {
                AppMode::Library => self.handle_library_input(key.code),
                AppMode::Playing => self.handle_game_input(key.code),
            }
        }

        Ok(())
    }

    pub fn games(&self) -> impl Iterator<Item = &dyn ArcadeGame> {
        self.registry.iter()
    }

    pub fn game_count(&self) -> usize {
        self.registry.len()
    }

    pub fn selected_game(&self) -> Option<&dyn ArcadeGame> {
        self.registry.get(self.selected_game)
    }

    pub fn selected_game_metadata(&self) -> Option<&GameMetadata> {
        self.selected_game().map(ArcadeGame::metadata)
    }

    //this is the library mentioned in the function above.
    //simply defines what the app should do at the press of specific keys
    fn handle_library_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }

            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
            }

            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
            }

            KeyCode::Enter if !self.registry.is_empty() => {
                self.mode = AppMode::Playing;
            }

            _ => {}
        }
    }

    //yes, now you can't do anything special in the games.
    //I think this function will be implemented in each game with their specific keys instructions
    fn handle_game_input(&mut self, key: KeyCode) {
        if key == KeyCode::Esc {
            self.mode = AppMode::Library;
        }
    }

    fn select_next(&mut self) {
        if self.registry.is_empty() {
            return;
        }

        //when the function is called, the game selected becomes the next one. '% self.games.len()' is used to go back at the beginning of the list if
        //this function is called when the selected game is the last of the list.
        self.selected_game = (self.selected_game + 1) % self.registry.len();
    }

    //called by pressing 'up' or 'k', the selected game becomes the previous one. It goes at the last element if pressed when the selected item is the first of the list
    fn select_previous(&mut self) {
        if self.registry.is_empty() {
            return;
        }

        if self.selected_game == 0 {
            self.selected_game = self.registry.len() - 1;
        } else {
            self.selected_game -= 1;
        }
    }
}
