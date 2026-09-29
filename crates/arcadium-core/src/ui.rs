use ratatui::{
    Frame,
    layout::{Alignment, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{app::App, mode::AppMode};

const BACKGROUND: Color = Color::Rgb(11, 16, 32);
const PANEL: Color = Color::Rgb(18, 28, 50);
const SELECTED: Color = Color::Rgb(23, 51, 71);
const ACCENT: Color = Color::Rgb(103, 232, 249);
const TEXT: Color = Color::Rgb(241, 245, 249);
const MUTED: Color = Color::Rgb(148, 163, 184);
const BORDER: Color = Color::Rgb(41, 58, 85);

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame
        .buffer_mut()
        .set_style(area, Style::default().fg(TEXT).bg(BACKGROUND));
    match app.mode {
        AppMode::Library => render_library(frame, app),
        AppMode::Playing => render_game(frame, app),
    }
}

fn shortcuts<'a>(items: &[(&'a str, &'a str)]) -> Line<'a> {
    let mut spans = Vec::new();
    for (index, &(key, label)) in items.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("   "));
        }
        spans.push(Span::styled(key, Style::default().fg(ACCENT)));
        spans.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(MUTED),
        ));
    }
    Line::from(spans)
}

fn render_library(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = area.width.saturating_sub(4).min(64);
    let row_height = if area.height >= 18 { 3 } else { 1 };
    let show_description = area.height >= 16 && width >= 36 && app.game_count() > 0;
    let details_height = if show_description { 3 } else { 0 };
    let available = area.height.saturating_sub(6 + details_height);
    let list_height = (app.game_count().max(1).min(u16::MAX as usize) as u16)
        .saturating_mul(row_height)
        .saturating_add(2)
        .min(available);
    let height = (list_height + 4 + details_height).min(area.height);
    let content = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(
        Paragraph::new("ARCADIUM").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Rect::new(content.x, content.y, content.width, content.height.min(1)),
    );
    let list_area = Rect::new(
        content.x,
        content.y + content.height.min(2),
        width,
        list_height,
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL));
    if app.game_count() == 0 {
        frame.render_widget(
            Paragraph::new("No games installed")
                .style(Style::default().fg(MUTED))
                .alignment(Alignment::Center)
                .block(block),
            list_area,
        );
    } else {
        let items: Vec<ListItem> = app
            .games()
            .enumerate()
            .map(|(index, game)| {
                let marker = if index == app.selected_game {
                    "›"
                } else {
                    " "
                };
                let name = Line::from(format!(" {marker} {}", game.metadata.name));
                if row_height == 3 {
                    ListItem::new(vec![Line::default(), name, Line::default()])
                } else {
                    ListItem::new(name)
                }
            })
            .collect();
        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .fg(ACCENT)
                .bg(SELECTED)
                .add_modifier(Modifier::BOLD),
        );
        let mut state = ListState::default().with_selected(Some(app.selected_game));
        frame.render_stateful_widget(list, list_area, &mut state);
    }
    if show_description && let Some(game) = app.selected_game() {
        frame.render_widget(
            Paragraph::new(game.metadata.description.as_str())
                .style(Style::default().fg(MUTED))
                .wrap(Wrap { trim: true }),
            Rect::new(
                content.x + 1,
                list_area.bottom(),
                width.saturating_sub(2),
                2,
            ),
        );
    }
    let keys = if app.game_count() == 0 {
        shortcuts(&[("Q", "Quit")])
    } else if width < 36 {
        shortcuts(&[("↑↓", ""), ("Enter", "Play"), ("Q", "Quit")])
    } else {
        shortcuts(&[("↑↓", "Select"), ("Enter", "Play"), ("Q", "Quit")])
    };
    frame.render_widget(
        Paragraph::new(keys).alignment(Alignment::Center),
        Rect::new(
            content.x,
            content.bottom().saturating_sub(1),
            width,
            content.height.min(1),
        ),
    );
}

fn render_game(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let Some(installed_game) = app.selected_game() else {
        return;
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .title(
            Line::from(format!(" {} ", installed_game.metadata.name))
                .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        );
    frame.render_widget(block, area);
    let surface = game_surface(area);
    if let Some(screen) = app.screen() {
        for y in 0..surface.height.min(screen.height()) {
            for x in 0..surface.width.min(screen.width()) {
                let cell =
                    screen.screen()[usize::from(y) * usize::from(screen.width()) + usize::from(x)];
                frame.buffer_mut()[(surface.x + x, surface.y + y)]
                    .set_char(cell.character)
                    .set_fg(cell.foreground)
                    .set_bg(cell.background);
            }
        }
    } else {
        let message = app.runtime_message.as_deref().unwrap_or("");
        let reason = if message.contains("Expected arcadium_") {
            "This game needs an update."
        } else {
            "Return to the library and try again."
        };
        let height = surface.height.min(4);
        let error_area = Rect::new(
            surface.x,
            surface.y + (surface.height - height) / 2,
            surface.width,
            height,
        );
        frame.render_widget(
            Paragraph::new(vec![
                Line::from("Game unavailable")
                    .style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
                Line::default(),
                Line::from(reason).style(Style::default().fg(MUTED)),
            ])
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
            error_area,
        );
    }
    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    frame.render_widget(
        Paragraph::new(shortcuts(&[("Esc", "Library")])).alignment(Alignment::Center),
        Rect::new(
            inner.x,
            inner.y + surface.height,
            inner.width,
            inner.height.min(1),
        ),
    );
}

/// The border and one footer row are reserved by Arcadium.
pub(crate) fn game_surface(area: Rect) -> Rect {
    let inner = Block::default().borders(Borders::ALL).inner(area);
    Rect::new(
        inner.x,
        inner.y,
        inner.width,
        inner.height.saturating_sub(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameRegistry, InstalledGame};
    use arcadium_sdk::GameMetadata;
    use ratatui::{Terminal, backend::TestBackend};

    fn app(count: usize) -> App {
        let mut registry = GameRegistry::new();
        for index in 0..count {
            registry.register(InstalledGame::new(
                GameMetadata {
                    id: index.to_string(),
                    name: format!("Game {index}"),
                    author: "Arcadium".into(),
                    version: "0.1.0".into(),
                    description: "A short game description.".into(),
                },
                "unused.adm".into(),
                1,
                "game.wasm".into(),
            ));
        }
        App::new(registry)
    }

    fn draw(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn library_scrolls_to_selection_and_handles_empty_and_tiny_screens() {
        let mut library = app(50);
        library.selected_game = 49;
        for (width, height) in [(80, 24), (40, 12), (26, 10)] {
            let text = draw(&library, width, height);
            assert!(text.contains("Game 49"));
            assert!(text.contains("Play"));
        }
        assert!(draw(&app(0), 80, 24).contains("No games installed"));
        for width in 0..12 {
            for height in 0..12 {
                draw(&library, width, height);
                draw(&app(0), width, height);
            }
        }
    }

    #[test]
    fn game_errors_show_recovery_without_raw_runtime_details() {
        let mut game = app(1);
        game.mode = AppMode::Playing;
        game.runtime_message = Some("Failed to load game: Expected arcadium_update(f32)".into());
        let text = draw(&game, 80, 24);
        assert!(text.contains("Game unavailable"));
        assert!(text.contains("This game needs an update."));
        assert!(text.contains("Esc Library"));
        assert!(!text.contains("arcadium_update"));
        for width in 0..4 {
            for height in 0..4 {
                draw(&game, width, height);
            }
        }
    }

    #[test]
    fn surface_reserves_border_and_footer_even_on_tiny_terminals() {
        assert_eq!(
            game_surface(Rect::new(0, 0, 80, 24)),
            Rect::new(1, 1, 78, 21)
        );
        for width in 0..4 {
            for height in 0..4 {
                let area = Rect::new(0, 0, width, height);
                let surface = game_surface(area);
                assert!(surface.right() <= area.right());
                assert!(surface.bottom() <= area.bottom());
            }
        }
    }
}
