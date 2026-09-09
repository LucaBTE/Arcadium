use arcadium_sdk::GameMetadata;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

use crate::mode::AppMode;

//app structure, very easy
pub struct App {
    pub should_quit: bool,
    pub games: Vec<GameMetadata>,
    pub selected_game: usize,
    pub mode: AppMode,
}

impl App {
    //function to initialize a new application boot
    pub fn new() -> Self {
        //list of games are selected from below, currently are just a mock
        let games = games_mock();

        //these are just the properties of the app
        Self {
            should_quit: false,
            games,
            selected_game: 0,
            mode: AppMode::Library,
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

            KeyCode::Enter if !self.games.is_empty() => {
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
        if self.games.is_empty() {
            return;
        }

        //when the function is called, the game selected becomes the next one. '% self.games.len()' is used to go back at the beginning of the list if
        //this function is called when the selected game is the last of the list.
        self.selected_game = (self.selected_game + 1) % self.games.len();
    }

    //called by pressing 'up' or 'k', the selected game becomes the previous one. It goes at the last element if pressed when the selected item is the first of the list
    fn select_previous(&mut self) {
        if self.games.is_empty() {
            return;
        }

        if self.selected_game == 0 {
            self.selected_game = self.games.len() - 1;
        } else {
            self.selected_game -= 1;
        }
    }
}

//vector of mock games to have a reference while building the rest of the application logic.
fn games_mock() -> Vec<GameMetadata> {
    vec![
        GameMetadata {
            id: String::from("pong"),
            name: String::from("Pong"),
            author: String::from("Arcadium"),
            version: String::from("0.1.0"),
            description: String::from("Classic two-player paddle game."),
        },
        GameMetadata {
            id: String::from("snake"),
            name: String::from("Snake"),
            author: String::from("Arcadium"),
            version: String::from("0.1.0"),
            description: String::from("Classic terminal snake."),
        },
        GameMetadata {
            id: String::from("tic-tac-toe"),
            name: String::from("Tic Tac Toe"),
            author: String::from("Arcadium"),
            version: String::from("0.1.0"),
            description: String::from("Classic three-in-a-row game."),
        },
    ]
}
