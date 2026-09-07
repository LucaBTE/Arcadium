use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(" ARCADIUM ");

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
        Line::from("ARCADIUM")
            .style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from("TERMINAL ARCADE SYSTEM"),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(title, chunks[1]);

    let game_lines: Vec<Line> = app
        .games
        .iter()
        .enumerate()
        .map(|(index, game)| {
            if index == app.selected_game {
                Line::from(format!("> {}", game.name))
                    .style(Style::default().add_modifier(Modifier::BOLD))
            } else {
                Line::from(format!("  {}", game.name))
            }
        })
        .collect();

    let game_list = Paragraph::new(game_lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" GAME LIBRARY "),
        );

    frame.render_widget(game_list, chunks[3]);

    let footer = Paragraph::new("[↑ ↓ / O K] SELECT    [ENTER] PLAY    [Q] QUIT")
        .alignment(Alignment::Center);

    frame.render_widget(footer, chunks[4]);
}