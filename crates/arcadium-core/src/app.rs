use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::{Position, Rect};

use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    host::{HostState, key},
    installed_game::InstalledGame,
    installer::install_game,
    mode::AppMode,
    registry::GameRegistry,
    runtime::GameRuntime,
};

pub struct App {
    pub should_quit: bool,
    pub selected_game: usize,
    pub mode: AppMode,
    pub runtime_message: Option<String>,
    pub library_message: Option<String>,
    pub(crate) install_focused: bool,

    registry: GameRegistry,
    runtime: Option<GameRuntime>,
    bundled_directory: PathBuf,
    user_directory: PathBuf,
    install_requested: bool,
}

impl App {
    pub fn new(
        registry: GameRegistry,
        bundled_directory: PathBuf,
        user_directory: PathBuf,
    ) -> Self {
        let install_focused = registry.is_empty();
        Self {
            should_quit: false,
            selected_game: 0,
            mode: AppMode::Library,
            runtime_message: None,
            library_message: None,
            install_focused,
            registry,
            runtime: None,
            bundled_directory,
            user_directory,
            install_requested: false,
        }
    }

    pub fn handle_events(
        &mut self,
        width: u16,
        height: u16,
        button: Option<Rect>,
    ) -> io::Result<()> {
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(key)
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                {
                    match self.mode {
                        AppMode::Library => {
                            self.handle_library_input(key.code, width, height, button.is_some());
                        }

                        AppMode::Playing => {
                            self.handle_game_input(key.code);
                        }
                    }
                }
                Event::Mouse(mouse)
                    if self.mode == AppMode::Library
                        && mouse.kind == MouseEventKind::Down(MouseButton::Left)
                        && button.is_some_and(|rect| {
                            rect.contains(Position::new(mouse.column, mouse.row))
                        }) =>
                {
                    self.install_focused = true;
                    self.request_install();
                }
                _ => {}
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
        } else if self
            .runtime
            .as_ref()
            .is_some_and(GameRuntime::exit_requested)
        {
            self.stop_game();
            self.mode = AppMode::Library;
            self.runtime_message = None;
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

    pub fn take_install_request(&mut self) -> bool {
        std::mem::take(&mut self.install_requested)
    }

    pub fn install_selected_file(&mut self, path: &Path) {
        match self.install_and_refresh(path) {
            Ok(message) => self.library_message = Some(message),
            Err(error) => {
                eprintln!("Could not install game: {error}");
                let message = if error.kind() == io::ErrorKind::InvalidInput {
                    error.to_string()
                } else {
                    "Filesystem error. Check the games directory.".to_owned()
                };
                self.library_message = Some(format!("Could not install game: {message}"));
            }
        }
    }

    fn install_and_refresh(&mut self, path: &Path) -> io::Result<String> {
        let installed = install_game(path, &self.bundled_directory, &self.user_directory)?;
        let registry = crate::discover_games(&self.bundled_directory, &self.user_directory)?;
        let selected = registry
            .iter()
            .position(|game| game.metadata.id == installed.id);
        self.registry = registry;
        self.install_focused = false;
        self.selected_game = selected.unwrap_or_else(|| {
            self.selected_game
                .min(self.registry.len().saturating_sub(1))
        });
        if selected.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Installed game was not found after refresh",
            ));
        }
        let verb = if installed.updated {
            "Updated"
        } else {
            "Installed"
        };
        Ok(format!("{verb} \"{}\".", installed.name))
    }

    fn handle_library_input(
        &mut self,
        key: KeyCode,
        width: u16,
        height: u16,
        button_available: bool,
    ) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }

            KeyCode::Char('i' | 'I') => {
                self.request_install();
            }

            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous(button_available);
            }

            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next(button_available);
            }

            KeyCode::Enter if self.install_focused => {
                if button_available {
                    self.request_install();
                }
            }

            KeyCode::Enter if !self.registry.is_empty() => {
                self.launch_selected_game(width, height);
            }

            _ => {}
        }
    }

    fn request_install(&mut self) {
        self.install_requested = true;
        self.library_message = None;
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

    fn select_next(&mut self, button_available: bool) {
        if self.registry.is_empty() {
            return;
        }
        if self.install_focused {
            self.install_focused = false;
            self.selected_game = 0;
        } else if self.selected_game + 1 == self.registry.len() && button_available {
            self.install_focused = true;
        } else {
            self.selected_game = (self.selected_game + 1) % self.registry.len();
        }
    }

    fn select_previous(&mut self, button_available: bool) {
        if self.registry.is_empty() {
            return;
        }
        if self.install_focused {
            self.install_focused = false;
            self.selected_game = self.registry.len() - 1;
        } else if self.selected_game == 0 && button_available {
            self.install_focused = true;
        } else if self.selected_game == 0 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use arcadium_sdk::GameMetadata;

    fn app_with_games() -> App {
        let mut registry = GameRegistry::new();
        for id in ["one", "two"] {
            registry.register(InstalledGame::new(
                GameMetadata {
                    id: id.into(),
                    name: id.into(),
                    author: "Test".into(),
                    version: "1".into(),
                    description: String::new(),
                },
                PathBuf::new(),
                1,
                "game.wasm".into(),
            ));
        }
        App::new(registry, PathBuf::new(), PathBuf::new())
    }

    #[test]
    fn cancellation_keeps_library_unchanged() {
        let mut app = App::new(GameRegistry::new(), PathBuf::new(), PathBuf::new());
        app.handle_library_input(KeyCode::Char('I'), 80, 24, true);
        assert!(app.take_install_request());
        assert!(!app.take_install_request());
        assert_eq!(app.game_count(), 0);
        assert_eq!(app.selected_game, 0);
        assert!(app.library_message.is_none());
    }

    #[test]
    fn arrow_navigation_reaches_install_button() {
        let mut app = app_with_games();
        app.handle_library_input(KeyCode::Down, 80, 24, true);
        assert_eq!(app.selected_game, 1);
        app.handle_library_input(KeyCode::Down, 80, 24, true);
        assert!(app.install_focused);
        app.handle_library_input(KeyCode::Enter, 80, 24, true);
        assert!(app.take_install_request());
        app.handle_library_input(KeyCode::Up, 80, 24, true);
        assert_eq!(app.selected_game, 1);
        assert!(!app.install_focused);

        app.handle_library_input(KeyCode::Down, 26, 10, false);
        assert_eq!(app.selected_game, 0);
        assert!(!app.install_focused);
    }
}
