use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{app::App, mode::AppMode};

const BACKGROUND: Color = Color::Rgb(14, 16, 29);
const PANEL: Color = Color::Rgb(24, 26, 42);
const ACCENT: Color = Color::Rgb(255, 115, 56);
const TEXT: Color = Color::Rgb(244, 231, 211);
const MUTED: Color = Color::Rgb(169, 158, 148);
const BORDER: Color = Color::Rgb(104, 64, 47);
const SHADOW: Color = Color::Rgb(112, 43, 30);

// Five-row letterforms for the ARCADIUM wordmark.
const WORDMARK: [[u8; 5]; 8] = [
    [14, 17, 31, 17, 17],
    [30, 17, 30, 18, 17],
    [15, 16, 16, 16, 15],
    [14, 17, 31, 17, 17],
    [30, 17, 17, 17, 30],
    [31, 4, 4, 4, 31],
    [17, 17, 17, 17, 14],
    [17, 27, 21, 17, 17],
];
const WORDMARK_WIDTH: u16 = 54;

fn render_wordmark(frame: &mut Frame, area: Rect, large: bool) {
    if !large {
        frame.render_widget(
            Paragraph::new("A R C A D I U M")
                .alignment(Alignment::Center)
                .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            area,
        );
        return;
    }
    let start = area.x + (area.width - WORDMARK_WIDTH) / 2;
    for (letter, rows) in WORDMARK.iter().enumerate() {
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0
                    && start + letter as u16 * 7 + column + 1 < area.right()
                    && area.y + row as u16 + 1 < area.bottom()
                {
                    frame.buffer_mut()[(
                        start + letter as u16 * 7 + column + 1,
                        area.y + row as u16 + 1,
                    )]
                        .set_char('▓')
                        .set_fg(SHADOW);
                }
            }
        }
    }
    for (letter, rows) in WORDMARK.iter().enumerate() {
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    frame.buffer_mut()[(start + letter as u16 * 7 + column, area.y + row as u16)]
                        .set_char('▓')
                        .set_fg(ACCENT)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
            }
        }
    }
}

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
        spans.push(Span::styled(
            format!("[{key}]"),
            Style::default().fg(ACCENT),
        ));
        spans.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(MUTED),
        ));
    }
    Line::from(spans)
}

fn shortcut_rows<'a>(items: &[(&'a str, &'a str)], width: u16) -> Vec<Line<'a>> {
    let mut rows = Vec::new();
    let mut current = Vec::new();
    for &item in items {
        let mut candidate = current.clone();
        candidate.push(item);
        if !current.is_empty() && shortcuts(&candidate).width() > usize::from(width) {
            rows.push(shortcuts(&current));
            current.clear();
        }
        current.push(item);
    }
    if !current.is_empty() {
        rows.push(shortcuts(&current));
    }
    rows
}

fn render_library(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < 20 || area.height < 7 {
        frame.render_widget(
            Paragraph::new("Enlarge terminal")
                .style(Style::default().fg(MUTED))
                .alignment(Alignment::Center),
            Rect::new(
                area.x,
                area.y + area.height / 2,
                area.width,
                area.height.min(1),
            ),
        );
        return;
    }
    let width = area.width.saturating_sub(4).min(72);
    let large_wordmark = width >= WORDMARK_WIDTH && area.height >= 22;
    let header_height = if large_wordmark { 8 } else { 2 };
    let items: &[(&str, &str)] = if app.game_count() == 0 {
        &[("Q", "Exit")]
    } else {
        &[("Enter", "Play"), ("Q", "Exit")]
    };
    let keys = shortcut_rows(items, width);
    let footer_height = keys.len() as u16;
    let row_height = if area.height >= 18 { 3 } else { 1 };
    let show_description = area.height >= 16 && width >= 36 && app.game_count() > 0;
    let details_height = if show_description { 3 } else { 0 };
    let available = area
        .height
        .saturating_sub(header_height + 1 + details_height + footer_height);
    let list_height = (app.game_count().max(1).min(u16::MAX as usize) as u16)
        .saturating_mul(row_height)
        .saturating_add(2)
        .min(available);
    let height =
        (header_height + list_height + 1 + details_height + footer_height).min(area.height);
    let content = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    render_wordmark(
        frame,
        Rect::new(
            content.x,
            content.y,
            width,
            content.height.min(header_height),
        ),
        large_wordmark,
    );
    let list_area = Rect::new(
        content.x,
        content.y + content.height.min(header_height),
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
                let selected = index == app.selected_game;
                let name = Line::from(vec![
                    Span::styled(
                        if selected { "  ❯  " } else { "     " },
                        Style::default().fg(ACCENT),
                    ),
                    Span::styled(
                        game.metadata.name.to_uppercase(),
                        Style::default().fg(if selected { TEXT } else { MUTED }),
                    ),
                ]);
                if row_height == 3 {
                    ListItem::new(vec![Line::default(), name, Line::default()])
                } else {
                    ListItem::new(name)
                }
            })
            .collect();
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
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
                list_area.bottom() + 1,
                width.saturating_sub(2),
                2,
            ),
        );
    }
    frame.render_widget(
        Paragraph::new(keys).alignment(Alignment::Center),
        Rect::new(
            content.x,
            content
                .bottom()
                .saturating_sub(footer_height.min(content.height)),
            width,
            footer_height.min(content.height),
        ),
    );
}

fn render_game(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let Some(installed_game) = app.selected_game() else {
        return;
    };
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT))
        .title(
            Line::from(format!(" {} ", installed_game.metadata.name.to_uppercase()))
                .style(Style::default().fg(ACCENT)),
        );
    if area.width >= 48 {
        block = block.title(
            Line::from(" ARCADIUM ")
                .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
                .right_aligned(),
        );
    }
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
                Line::from("Game unavailable").style(Style::default().fg(TEXT)),
                Line::default(),
                Line::from(reason).style(Style::default().fg(MUTED)),
            ])
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
            error_area,
        );
    }
}

/// Arcadium reserves the outer border; the game owns the full inner surface.
pub(crate) fn game_surface(area: Rect) -> Rect {
    let inner = Block::default().borders(Borders::ALL).inner(area);
    Rect::new(inner.x, inner.y, inner.width, inner.height)
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
        for (width, height) in [
            (120, 40),
            (80, 24),
            (58, 22),
            (57, 22),
            (80, 21),
            (40, 12),
            (26, 10),
        ] {
            let text = draw(&library, width, height);
            assert!(text.contains("GAME 49"));
            assert!(text.contains("[Enter] Play"));
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
    fn arcadium_wordmark_uses_the_platform_accent() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| render(frame, &app(2))).unwrap();
        assert!(
            terminal
                .backend()
                .buffer()
                .content
                .iter()
                .any(|cell| cell.symbol() == "▓" && cell.fg == ACCENT)
        );
    }

    #[test]
    fn menu_and_game_controls_use_orange_buttons_on_the_dark_background() {
        for (width, height) in [(80, 24), (40, 12), (26, 10)] {
            let library = app(2);
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| render(frame, &library)).unwrap();
            let buffer = terminal.backend().buffer();
            for (button, action) in [("[Enter]", "Play"), ("[Q]", "Exit")] {
                let length = button.chars().count();
                let start = buffer
                    .content
                    .windows(length)
                    .position(|cells| {
                        cells.iter().map(|cell| cell.symbol()).collect::<String>() == button
                    })
                    .expect("every control remains visible");
                assert!(
                    buffer.content[start..start + length]
                        .iter()
                        .all(|cell| cell.fg == ACCENT && cell.bg == BACKGROUND)
                );
                let action_start = start + length + 1;
                let cells = &buffer.content[action_start..action_start + action.len()];
                assert_eq!(
                    cells.iter().map(|cell| cell.symbol()).collect::<String>(),
                    action
                );
                assert!(
                    cells
                        .iter()
                        .all(|cell| cell.fg == MUTED && cell.bg == BACKGROUND)
                );
                assert_eq!(
                    start / width as usize,
                    (action_start + action.len() - 1) / width as usize
                );
            }
            let mut game = app(1);
            game.mode = AppMode::Playing;
            terminal.draw(|frame| render(frame, &game)).unwrap();
            let buffer = terminal.backend().buffer();
            assert!(!buffer.content.windows(5).any(|cells| {
                cells.iter().map(|cell| cell.symbol()).collect::<String>() == "[Esc]"
            }));
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
        assert!(!text.contains("[Esc] Library"));
        assert!(!text.contains("arcadium_update"));
        for width in 0..4 {
            for height in 0..4 {
                draw(&game, width, height);
            }
        }
    }

    #[test]
    fn surface_reserves_the_border_even_on_tiny_terminals() {
        assert_eq!(
            game_surface(Rect::new(0, 0, 80, 24)),
            Rect::new(1, 1, 78, 22)
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
