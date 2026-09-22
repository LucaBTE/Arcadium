use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use std::{io, time::Duration};

use crate::{
    installed_game::InstalledGame, mode::AppMode, registry::GameRegistry, runtime::GameRuntime,
};

pub struct App {
    pub should_quit: bool,
    pub selected_game: usize,
    pub mode: AppMode,
    pub runtime_message: Option<String>,

    registry: GameRegistry,
    runtime: Option<GameRuntime>,
}

impl App {
    pub fn new(registry: GameRegistry) -> Self {
        Self {
            should_quit: false,
            selected_game: 0,
            mode: AppMode::Library,
            runtime_message: None,
            registry,
            runtime: None,
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

    pub fn update(&mut self) {
        if self.mode != AppMode::Playing {
            return;
        }

        let update_result = match self.runtime.as_mut() {
            Some(runtime) => Some(runtime.update()),

            None => None,
        };

        if let Some(Err(error)) = update_result {
            self.runtime_message = Some(format!("Game runtime error: {}", error));

            self.runtime = None;
        }
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
            self.stop_game();

            self.mode = AppMode::Library;

            self.runtime_message = None;
        }
    }

    fn launch_selected_game(&mut self) {
        let runtime_result = {
            let Some(game) = self.registry.get(self.selected_game) else {
                return;
            };

            GameRuntime::load(game)
        };

        let mut runtime = match runtime_result {
            Ok(runtime) => runtime,

            Err(error) => {
                self.runtime_message = Some(format!("Failed to load game: {}", error));

                self.mode = AppMode::Playing;

                return;
            }
        };

        match runtime.init() {
            Ok(0) => {
                self.runtime_message = Some("Game initialization failed".to_string());
            }

            Ok(code) => {
                self.runtime_message = Some(format!("Game initialized successfully ({})", code));

                self.runtime = Some(runtime);
            }

            Err(error) => {
                self.runtime_message = Some(format!("Failed to initialize game: {}", error));
            }
        }

        self.mode = AppMode::Playing;
    }

    fn stop_game(&mut self) {
        let Some(mut runtime) = self.runtime.take() else {
            return;
        };

        if let Err(error) = runtime.shutdown() {
            eprintln!("Failed to shut down game runtime: {}", error);
        }
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
