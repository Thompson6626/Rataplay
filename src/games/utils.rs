use ratatui::prelude::{Color, Line, Modifier, Span, Style};

pub fn line_with_color<T: Into<String>>(text: T, color: Color) -> Line<'static> {
    Line::from(Span::styled(
        text.into(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}