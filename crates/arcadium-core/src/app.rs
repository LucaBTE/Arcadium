use arcadium_sdk::GameMetadata;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

pub struct App {
    pub should_quit: bool,
    pub games: Vec<GameMetadata>,
    pub selected_game: usize,
}

impl App {



    pub fn new() -> Self {
        let games = games_mock(); //it will be ' let games = discover_games();

        Self {
            should_quit: false,
            games,
            selected_game: 0,
        }
    }


    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                        }

                        KeyCode::Up | KeyCode::Char('o') => {
                            self.select_previous();
                        }

                        KeyCode::Down | KeyCode::Char('k') => {
                            self.select_next();
                        }

                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }



    fn select_next(&mut self) {
        if self.games.is_empty() {
            return;
        }
        self.selected_game = (self.selected_game + 1) % self.games.len();
    }


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