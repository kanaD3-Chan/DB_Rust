use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::components::markdown::render_markdown;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // 标题输入
    let title_display = render_with_cursor(&app.editor_title, app.editor_title_cursor, app.editor_focused_title);
    let title_block = Block::default()
        .title("标题 [Tab:切换到内容]")
        .borders(Borders::ALL)
        .border_style(if app.editor_focused_title {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    f.render_widget(Paragraph::new(title_display).block(title_block), chunks[0]);

    // 分类选择
    let categories = ["general", "tech", "security", "life"];
    let cat_spans: Vec<Span> = categories
        .iter()
        .map(|c| {
            if *c == app.editor_category {
                Span::styled(
                    format!(" [{}] ", c),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!("  {}  ", c), Style::default().fg(Color::DarkGray))
            }
        })
        .collect();
    f.render_widget(Paragraph::new(Line::from(cat_spans)), chunks[1]);

    // 编辑/预览区
    if app.editor_preview {
        let md = render_markdown(&app.editor_content);
        let para = Paragraph::new(md)
            .block(
                Block::default()
                    .title("预览 [Ctrl+O:切换编辑]")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(para, chunks[2]);
    } else {
        let content_display = render_with_cursor(&app.editor_content, app.editor_cursor, !app.editor_focused_title);
        let content_block = Block::default()
            .title("内容 (Markdown) [Tab:切换到标题 Ctrl+O:预览 Ctrl+S:保存]")
            .borders(Borders::ALL)
            .border_style(if !app.editor_focused_title {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let para = Paragraph::new(content_display)
            .block(content_block)
            .wrap(Wrap { trim: false });
        f.render_widget(para, chunks[2]);
    }

    // 快捷键提示
    let hint = "Tab:切换焦点  Ctrl+S:保存  Ctrl+O:预览  Ctrl+←/→:切换分类  Esc:取消";
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        chunks[3],
    );
}

/// Insert a block cursor character at `cursor` byte offset, return as Line vec.
fn render_with_cursor(text: &str, cursor: usize, show_cursor: bool) -> Vec<Line<'static>> {
    if !show_cursor {
        return text.lines()
            .map(|l| Line::from(l.to_owned()))
            .collect();
    }
    let cursor = cursor.min(text.len());
    let before = &text[..cursor];
    let after = &text[cursor..];
    // Split into lines preserving the cursor position
    let combined = format!("{}\u{2588}{}", before, after); // █
    combined.lines()
        .map(|l| Line::from(l.to_owned()))
        .collect()
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Option<EditorAction> {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
            return Some(EditorAction::Save);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
            app.editor_preview = !app.editor_preview;
        }
        (KeyModifiers::CONTROL, KeyCode::Right) => {
            let cats = ["general", "tech", "security", "life"];
            let idx = cats.iter().position(|&c| c == app.editor_category).unwrap_or(0);
            app.editor_category = cats[(idx + 1) % cats.len()].to_string();
        }
        (KeyModifiers::CONTROL, KeyCode::Left) => {
            let cats = ["general", "tech", "security", "life"];
            let idx = cats.iter().position(|&c| c == app.editor_category).unwrap_or(0);
            app.editor_category = cats[(idx + cats.len() - 1) % cats.len()].to_string();
        }
        (_, KeyCode::Esc) => return Some(EditorAction::Cancel),
        (_, KeyCode::Tab) => {
            app.editor_focused_title = !app.editor_focused_title;
        }

        // 方向键移动光标
        (_, KeyCode::Left) => {
            if app.editor_focused_title {
                app.editor_title_cursor = prev_char_boundary(&app.editor_title, app.editor_title_cursor);
            } else if !app.editor_preview {
                app.editor_cursor = prev_char_boundary(&app.editor_content, app.editor_cursor);
            }
        }
        (_, KeyCode::Right) => {
            if app.editor_focused_title {
                app.editor_title_cursor = next_char_boundary(&app.editor_title, app.editor_title_cursor);
            } else if !app.editor_preview {
                app.editor_cursor = next_char_boundary(&app.editor_content, app.editor_cursor);
            }
        }
        (_, KeyCode::Up) => {
            if !app.editor_focused_title && !app.editor_preview {
                app.editor_cursor = move_cursor_vertical(&app.editor_content, app.editor_cursor, -1);
            }
        }
        (_, KeyCode::Down) => {
            if !app.editor_focused_title && !app.editor_preview {
                app.editor_cursor = move_cursor_vertical(&app.editor_content, app.editor_cursor, 1);
            }
        }
        (_, KeyCode::Home) => {
            if app.editor_focused_title {
                app.editor_title_cursor = 0;
            } else if !app.editor_preview {
                app.editor_cursor = line_start(&app.editor_content, app.editor_cursor);
            }
        }
        (_, KeyCode::End) => {
            if app.editor_focused_title {
                app.editor_title_cursor = app.editor_title.len();
            } else if !app.editor_preview {
                app.editor_cursor = line_end(&app.editor_content, app.editor_cursor);
            }
        }

        (_, KeyCode::Enter) => {
            if !app.editor_focused_title && !app.editor_preview {
                app.editor_content.insert(app.editor_cursor, '\n');
                app.editor_cursor += 1;
            }
        }
        (_, KeyCode::Backspace) => {
            if app.editor_focused_title {
                if app.editor_title_cursor > 0 {
                    let prev = prev_char_boundary(&app.editor_title, app.editor_title_cursor);
                    app.editor_title.drain(prev..app.editor_title_cursor);
                    app.editor_title_cursor = prev;
                }
            } else if !app.editor_preview && app.editor_cursor > 0 {
                let prev = prev_char_boundary(&app.editor_content, app.editor_cursor);
                app.editor_content.drain(prev..app.editor_cursor);
                app.editor_cursor = prev;
            }
        }
        (_, KeyCode::Delete) => {
            if app.editor_focused_title {
                if app.editor_title_cursor < app.editor_title.len() {
                    let next = next_char_boundary(&app.editor_title, app.editor_title_cursor);
                    app.editor_title.drain(app.editor_title_cursor..next);
                }
            } else if !app.editor_preview && app.editor_cursor < app.editor_content.len() {
                let next = next_char_boundary(&app.editor_content, app.editor_cursor);
                app.editor_content.drain(app.editor_cursor..next);
            }
        }
        (_, KeyCode::Char(c)) => {
            if app.editor_focused_title {
                app.editor_title.insert(app.editor_title_cursor, c);
                app.editor_title_cursor += c.len_utf8();
            } else if !app.editor_preview {
                app.editor_content.insert(app.editor_cursor, c);
                app.editor_cursor += c.len_utf8();
            }
        }
        _ => {}
    }
    None
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    if pos == 0 { return 0; }
    let mut p = pos - 1;
    while p > 0 && !s.is_char_boundary(p) { p -= 1; }
    p
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() { return s.len(); }
    let mut p = pos + 1;
    while p < s.len() && !s.is_char_boundary(p) { p += 1; }
    p
}

fn line_start(s: &str, pos: usize) -> usize {
    let before = &s[..pos];
    before.rfind('\n').map(|i| i + 1).unwrap_or(0)
}

fn line_end(s: &str, pos: usize) -> usize {
    s[pos..].find('\n').map(|i| pos + i).unwrap_or(s.len())
}

fn move_cursor_vertical(s: &str, pos: usize, delta: i32) -> usize {
    let start = line_start(s, pos);
    let col = pos - start; // byte column on current line

    if delta < 0 {
        // move up
        if start == 0 { return 0; }
        let prev_end = start - 1; // '\n' before current line
        let prev_start = line_start(s, prev_end);
        let prev_line_len = prev_end - prev_start;
        prev_start + col.min(prev_line_len)
    } else {
        // move down
        let end = line_end(s, pos);
        if end >= s.len() { return s.len(); }
        let next_start = end + 1;
        let next_end = line_end(s, next_start);
        let next_line_len = next_end - next_start;
        next_start + col.min(next_line_len)
    }
}

pub enum EditorAction {
    Save,
    Cancel,
}
