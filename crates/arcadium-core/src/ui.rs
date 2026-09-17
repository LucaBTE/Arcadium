use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
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

    let inner_area = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(inner_area);

    let content = Paragraph::new(vec![
        Line::from(game.name.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from(game.description.clone()),
        Line::from(""),
        Line::from(format!("Author: {}", game.author)),
        Line::from(format!("Version: {}", game.version)),
        Line::from(""),
        Line::from("GAME NOT IMPLEMENTED"),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(content, chunks[1]);

    let footer = Paragraph::new("[ESC] BACK").alignment(Alignment::Center);

    frame.render_widget(footer, chunks[2]);
}
