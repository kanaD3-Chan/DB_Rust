use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ws::Notification;

pub fn render_toast(f: &mut Frame, notif: &Notification) {
    let area = toast_rect(f.area());

    let kind_label = match notif.kind.as_str() {
        "new_article" => "📄 新文章",
        "new_comment" => "💬 新评论",
        _ => "通知",
    };

    let author = notif.author_name.as_deref().unwrap_or("?");
    let title = notif.article_title.as_deref().unwrap_or("");
    let preview = notif.preview.as_deref().unwrap_or("");

    let text = format!("{} by {}\n{}\n{}", kind_label, author, title, &preview[..preview.len().min(40)]);

    let block = Block::default()
        .title("通知")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

pub fn render_notification_list(f: &mut Frame, notifications: &std::collections::VecDeque<Notification>) {
    use ratatui::widgets::{List, ListItem};
    use ratatui::layout::Constraint;
    use ratatui::layout::Direction;
    use ratatui::layout::Layout;
    use ratatui::widgets::Clear;
    use crate::components::popup::centered_rect;

    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = notifications
        .iter()
        .map(|n| {
            let kind = match n.kind.as_str() {
                "new_article" => "文章",
                "new_comment" => "评论",
                _ => "通知",
            };
            let author = n.author_name.as_deref().unwrap_or("?");
            let title = n.article_title.as_deref().unwrap_or("");
            ListItem::new(format!("[{}] {} - {}", kind, author, title))
        })
        .collect();

    let list = ratatui::widgets::List::new(items)
        .block(
            Block::default()
                .title("通知列表 [Esc 关闭]")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

    f.render_widget(list, area);
}

fn toast_rect(r: Rect) -> Rect {
    Rect {
        x: r.width.saturating_sub(42),
        y: 1,
        width: 40,
        height: 5,
    }
}
