use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use std::{io, time::Duration};

use crate::{
    host::{HostState, key},
    installed_game::InstalledGame,
    mode::AppMode,
    registry::GameRegistry,
    runtime::GameRuntime,
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

    pub fn handle_events(&mut self, width: u16, height: u16) -> io::Result<()> {
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()?
                && matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
            {
                match self.mode {
                    AppMode::Library => {
                        self.handle_library_input(key.code, width, height);
                    }

                    AppMode::Playing => {
                        self.handle_game_input(key.code);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update(&mut self, delta_seconds: f32) {
        if self.mode != AppMode::Playing {
            return;
        }

        let update_result = self
            .runtime
            .as_mut()
            .map(|runtime| runtime.update(delta_seconds));

        if let Some(Err(error)) = update_result {
            self.runtime_message = Some(format!("Game runtime error: {}", error));

            self.stop_game();
        }
    }

    pub fn begin_frame(&mut self, width: u16, height: u16) {
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.begin_frame();
            runtime.resize(width, height);
        }
    }

    pub fn screen(&self) -> Option<&HostState> {
        self.runtime.as_ref().map(GameRuntime::screen)
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

    fn handle_library_input(&mut self, key: KeyCode, width: u16, height: u16) {
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
                self.launch_selected_game(width, height);
            }

            _ => {}
        }
    }

    fn handle_game_input(&mut self, key: KeyCode) {
        if key == KeyCode::Esc {
            self.stop_game();

            self.mode = AppMode::Library;

            self.runtime_message = None;
        } else if let Some(key) = host_key(key)
            && let Some(runtime) = self.runtime.as_mut()
        {
            runtime.press_key(key);
        }
    }

    fn launch_selected_game(&mut self, width: u16, height: u16) {
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

        runtime.resize(width, height);

        match runtime.init() {
            Ok(0) => {
                self.runtime_message = Some("Game initialization failed".to_string());
            }

            Ok(_) => {
                self.runtime_message = None;

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

fn host_key(code: KeyCode) -> Option<i32> {
    Some(match code {
        KeyCode::Up => key::UP,
        KeyCode::Down => key::DOWN,
        KeyCode::Left => key::LEFT,
        KeyCode::Right => key::RIGHT,
        KeyCode::Char('w' | 'W') => key::W,
        KeyCode::Char('a' | 'A') => key::A,
        KeyCode::Char('s' | 'S') => key::S,
        KeyCode::Char('d' | 'D') => key::D,
        KeyCode::Char(' ') => key::SPACE,
        KeyCode::Enter => key::ENTER,
        _ => return None,
    })
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop_game();
    }
}
