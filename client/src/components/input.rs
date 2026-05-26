use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct InputField<'a> {
    pub label: &'a str,
    pub value: &'a str,
    pub focused: bool,
    pub masked: bool,
}

impl<'a> InputField<'a> {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let display = if self.masked {
            "*".repeat(self.value.len())
        } else {
            self.value.to_string()
        };

        let cursor = if self.focused { "█" } else { "" };
        let text = format!("{}{}", display, cursor);

        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(self.label)
            .borders(Borders::ALL)
            .border_style(border_style);

        let para = Paragraph::new(text).block(block);
        f.render_widget(para, area);
    }
}
