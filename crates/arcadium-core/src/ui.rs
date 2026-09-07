use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame) {
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
            Constraint::Percentage(30),
            Constraint::Length(5),
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(inner_area);

    let title = Paragraph::new(vec![
        Line::from("ARCADIUM").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from(""),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(title, chunks[1]);

    let status = Paragraph::new("NO GAMES INSTALLED")
        .alignment(Alignment::Center);

    frame.render_widget(status, chunks[3]);

    let footer = Paragraph::new("[Q] QUIT")
        .alignment(Alignment::Center);

    frame.render_widget(footer, chunks[4]);
}