use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{app::App, mode::AppMode};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    frame
        .buffer_mut()
        .set_style(area, Style::default().fg(Color::White));

    match app.mode {
        AppMode::Library => {
            render_library(frame, app);
        }

        AppMode::Playing => {
            render_game(frame, app);
        }
    }
}

fn render_library(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let main_block = Block::default().borders(Borders::ALL).title(" ARCADIUM ");

    frame.render_widget(main_block, area);

    let inner_area = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(inner_area);

    let title = Paragraph::new(vec![
        Line::from("ARCADIUM").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from("TERMINAL ARCADE SYSTEM"),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(title, chunks[1]);

    let game_items: Vec<ListItem> = app
        .games()
        .enumerate()
        .map(|(index, game)| {
            let metadata = &game.metadata;

            let label = if index == app.selected_game {
                format!("> {}", metadata.name)
            } else {
                format!("  {}", metadata.name)
            };

            ListItem::new(Line::from(label).alignment(Alignment::Center))
        })
        .collect();

    let game_list = List::new(game_items)
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" GAME LIBRARY "),
        );

    let mut list_state =
        ListState::default().with_selected((app.game_count() > 0).then_some(app.selected_game));

    frame.render_stateful_widget(game_list, chunks[3], &mut list_state);

    let footer = Paragraph::new(
        "[↑ ↓ / J K] SELECT    \
         [ENTER] PLAY    \
         [Q] QUIT",
    )
    .alignment(Alignment::Center);

    frame.render_widget(footer, chunks[4]);
}

fn render_game(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let Some(installed_game) = app.selected_game() else {
        return;
    };

    let game = &installed_game.metadata;

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", game.name.to_uppercase()));

    frame.render_widget(main_block, area);

    let surface = game_surface(area);
    if let Some(screen) = app.screen() {
        for y in 0..surface.height.min(screen.height()) {
            for x in 0..surface.width.min(screen.width()) {
                let character =
                    screen.screen()[usize::from(y) * usize::from(screen.width()) + usize::from(x)];
                frame.buffer_mut()[(surface.x + x, surface.y + y)].set_char(character);
            }
        }
    } else {
        frame.render_widget(
            Paragraph::new(
                app.runtime_message
                    .as_deref()
                    .unwrap_or("Runtime not started"),
            )
            .alignment(Alignment::Center),
            surface,
        );
    }
    let inner = Block::default().borders(Borders::ALL).inner(area);
    let footer = Rect::new(
        inner.x,
        inner.y + surface.height,
        inner.width,
        inner.height.min(1),
    );
    frame.render_widget(
        Paragraph::new("[ESC] BACK").alignment(Alignment::Center),
        footer,
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
