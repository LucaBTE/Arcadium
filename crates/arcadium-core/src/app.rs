use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use std::{io, time::Duration};

use crate::{
    installed_game::InstalledGame, mode::AppMode, registry::GameRegistry, runtime::initialize_game,
};

pub struct App {
    pub should_quit: bool,
    pub selected_game: usize,
    pub mode: AppMode,
    pub runtime_message: Option<String>,

    registry: GameRegistry,
}

impl App {
    pub fn new(registry: GameRegistry) -> Self {
        Self {
            should_quit: false,
            selected_game: 0,
            mode: AppMode::Library,
            runtime_message: None,
            registry,
        }
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match self.mode {
                AppMode::Library => {
                    self.handle_library_input(key.code);
                }

                AppMode::Playing => {
                    self.handle_game_input(key.code);
                }
            }
        }

        Ok(())
    }

    pub fn games(&self) -> impl Iterator<Item = &InstalledGame> {
        self.registry.iter()
    }

    pub fn game_count(&self) -> usize {
        self.registry.len()
    }

    pub fn selected_game(&self) -> Option<&InstalledGame> {
        self.registry.get(self.selected_game)
    }

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
                self.launch_selected_game();
            }

            _ => {}
        }
    }

    fn handle_game_input(&mut self, key: KeyCode) {
        if key == KeyCode::Esc {
            self.mode = AppMode::Library;

            self.runtime_message = None;
        }
    }

    fn launch_selected_game(&mut self) {
        let result = {
            let Some(game) = self.registry.get(self.selected_game) else {
                return;
            };

            initialize_game(game)
        };

        self.runtime_message = Some(match result {
            Ok(code) => {
                format!("arcadium_init() returned {}", code)
            }

            Err(error) => {
                format!("Failed to initialize game: {}", error)
            }
        });

        self.mode = AppMode::Playing;
    }

    fn select_next(&mut self) {
        if self.registry.is_empty() {
            return;
        }

        self.selected_game = (self.selected_game + 1) % self.registry.len();
    }

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
