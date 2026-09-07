use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

pub struct App {
    pub should_quit: bool,
}

impl App{
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let KeyCode::Char('q') = key.code {
                        self.should_quit = true;
                    }
                }
            }
        }

        Ok(())
    }
}

