mod app;
mod mode;
mod registry;
mod ui;

pub use registry::GameRegistry;

use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, stdout};

//called by 'arcadium-cli', defines the basics of the applications
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
    //defines a new mutable app, calling App::new() from 'arcadium-core/app.rs'
    let mut app = App::new(registry);

    //until the app field 'should quit' is false ('true' when pressing 'q' or something else), the app continues to run
    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, &app))?;
        app.handle_events()?;
    }

    Ok(())
}
