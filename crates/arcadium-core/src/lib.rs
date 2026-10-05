mod app;
mod discovery;
mod host;
mod installed_game;
mod installer;
mod limits;
mod mode;
pub mod paths;
mod registry;
mod runtime;
mod score_store;
mod ui;

pub use discovery::{discover_games, discover_games_from_directory};

pub use installed_game::InstalledGame;
pub use registry::GameRegistry;

use app::App;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};

use std::{
    io::{self, stdout},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const DEVELOPER_GUIDE_URL: &str =
    "https://github.com/LucaBTE/Arcadium/blob/main/docs/creating-games.md";

pub fn run(
    registry: GameRegistry,
    bundled_directory: PathBuf,
    user_directory: PathBuf,
) -> io::Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, registry, bundled_directory, user_directory);

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;

    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    registry: GameRegistry,
    bundled_directory: PathBuf,
    user_directory: PathBuf,
) -> io::Result<()> {
    let mut app = App::new(registry, bundled_directory, user_directory);

    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    let mut previous_frame = Instant::now();
    let mut actions = app::LibraryActions::default();
    let mut guide_processes = Vec::new();
    while !app.should_quit {
        let frame_start = Instant::now();
        let delta_seconds = frame_start.duration_since(previous_frame).as_secs_f32();
        previous_frame = frame_start;
        terminal.autoresize()?;
        let size = terminal.size()?;
        let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        let surface = ui::game_surface(area);
        app.begin_frame(surface.width, surface.height);
        app.handle_events(surface.width, surface.height, actions)?;
        if app.take_install_request() {
            if let Some(path) = select_adm_file(terminal)? {
                app.install_selected_file(&path);
            }
            previous_frame = Instant::now();
        }
        let mut guide_failed = false;
        if app.take_guide_request() {
            match Command::new(guide_open_command())
                .arg(DEVELOPER_GUIDE_URL)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => guide_processes.push(child),
                Err(_) => guide_failed = true,
            }
        }
        guide_processes.retain_mut(|child| match child.try_wait() {
            Ok(None) => true,
            Ok(Some(status)) => {
                guide_failed |= !status.success();
                false
            }
            Err(_) => {
                guide_failed = true;
                false
            }
        });
        if guide_failed {
            app.library_message = Some(format!(
                "Could not open browser. Guide:\n{DEVELOPER_GUIDE_URL}"
            ));
        }
        app.update(delta_seconds);
        terminal.draw(|frame| actions = ui::render(frame, &app))?;
        thread::sleep(frame_duration.saturating_sub(frame_start.elapsed()));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn guide_open_command() -> &'static str {
    "xdg-open"
}

#[cfg(target_os = "macos")]
fn guide_open_command() -> &'static str {
    "open"
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn guide_open_command() -> &'static str {
    "unsupported-platform"
}

fn select_adm_file(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> io::Result<Option<PathBuf>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    let selected = rfd::FileDialog::new()
        .add_filter("Arcadium games", &["adm"])
        .pick_file();

    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    enable_raw_mode()?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    while crossterm::event::poll(Duration::ZERO)? {
        let _ = crossterm::event::read()?;
    }
    Ok(selected)
}
