mod app;
mod discovery;
mod host;
mod installed_game;
mod mode;
mod registry;
mod runtime;
mod score_store;
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

use std::{
    io::{self, stdout},
    thread,
    time::{Duration, Instant},
};

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

    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    let mut previous_frame = Instant::now();
    while !app.should_quit {
        let frame_start = Instant::now();
        let delta_seconds = frame_start.duration_since(previous_frame).as_secs_f32();
        previous_frame = frame_start;
        terminal.autoresize()?;
        let size = terminal.size()?;
        let surface = ui::game_surface(ratatui::layout::Rect::new(0, 0, size.width, size.height));
        app.begin_frame(surface.width, surface.height);
        app.handle_events(surface.width, surface.height)?;
        app.update(delta_seconds);
        terminal.draw(|frame| ui::render(frame, &app))?;
        thread::sleep(frame_duration.saturating_sub(frame_start.elapsed()));
    }

    Ok(())
}
