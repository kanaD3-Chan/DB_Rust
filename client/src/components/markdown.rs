use ratatui::text::Text;

pub fn render_markdown(md: &str) -> Text<'_> {
    tui_markdown::from_str(md)
}
