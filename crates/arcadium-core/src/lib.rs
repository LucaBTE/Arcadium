mod app;
mod discovery;
mod installed_game;
mod mode;
mod registry;
mod runtime;
mod ui;

pub use discovery::{discover_games, discover_games_from_directory};

pub use installed_game::InstalledGame;
pub use registry::GameRegistry;

use app::App;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};

use std::io::{self, stdout};

pub fn run(registry: GameRegistry) -> io::Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, registry);

    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    registry: GameRegistry,
) -> io::Result<()> {
    let mut app = App::new(registry);

    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, &app))?;

        app.handle_events()?;
    }

    Ok(())
}
