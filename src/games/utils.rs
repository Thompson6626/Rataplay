use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};

pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);

    area
}

pub fn line_with_color<T: Into<String>>(text: T, color: Color) -> Line<'static> {
    Line::from(Span::styled(
        text.into(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

pub fn center_horizontal(area: Rect, width: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    area
}
