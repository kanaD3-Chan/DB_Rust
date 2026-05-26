use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;
use crate::components::markdown::render_markdown;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(area);

    render_article(f, app, chunks[0]);
    render_comments(f, app, chunks[1]);
}

pub fn layout_areas(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    (chunks[0], chunks[1])
}

fn render_article<'a>(f: &mut Frame, app: &'a App, area: Rect) {
    let Some(article) = &app.current_article else {
        f.render_widget(
            Paragraph::new("加载中...").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    };

    let inner_width = area.width.saturating_sub(4) as usize;
    let header: Vec<Line<'static>> = vec![
        Line::from(vec![Span::styled(
            article.title.clone(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!(
                "作者: {}  分类: {}  👁{}  💬{}  {}",
                article.author_nickname,
                article.category,
                article.view_count,
                article.comment_count,
                &article.created_at[..10]
            ),
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from("─".repeat(inner_width.min(60))),
        Line::from(""),
    ];

    let md = render_markdown(&article.content);
    let mut all_lines: Vec<Line<'a>> = header;
    all_lines.extend(md.lines);

    let can_edit = app
        .current_user
        .as_ref()
        .map(|u| u.id == article.author_id || u.role == "admin")
        .unwrap_or(false);

    let focused = !app.article_focus_comments;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if can_edit {
        "文章 [e:编辑 d:删除 ↑↓:滚动 Tab:切换焦点]"
    } else {
        "文章 [↑↓:滚动 Tab:切换焦点]"
    };

    let para = Paragraph::new(Text::from(all_lines))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .scroll((app.article_scroll, 0));

    f.render_widget(para, area);
}

fn render_comments(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let items: Vec<ListItem> = app
        .comments
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let selected = i == app.comment_selected;
            let indent = if c.parent_id.is_some() { "  ↳ " } else { "" };
            let style = if selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::raw(indent),
                    Span::styled(&c.author_nickname, Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("  {}", &c.created_at[..16]),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(vec![
                    Span::raw(if c.parent_id.is_some() { "    " } else { "  " }),
                    Span::styled(&c.content, style),
                ]),
            ])
        })
        .collect();

    let focused = app.article_focus_comments;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let comment_title = format!(
        "评论 ({}) [j/k:移动 c:评论 r:回复 q:返回]",
        app.comments.len()
    );
    let list = List::new(items)
        .block(
            Block::default()
                .title(comment_title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = ListState::default();
    if !app.comments.is_empty() {
        state.select(Some(app.comment_selected));
    }
    f.render_stateful_widget(list, chunks[0], &mut state);

    let input_text = if app.commenting {
        format!("{}█", app.comment_input)
    } else {
        String::new()
    };
    let input_style = if app.commenting {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let input = Paragraph::new(input_text).block(
        Block::default()
            .title(if app.commenting {
                "输入评论 [Enter:发送 Esc:取消]"
            } else {
                "c:评论  r:回复选中评论"
            })
            .borders(Borders::ALL)
            .border_style(input_style),
    );
    f.render_widget(input, chunks[1]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Option<ArticleAction> {
    if app.commenting {
        match key.code {
            KeyCode::Esc => {
                app.commenting = false;
                app.comment_input.clear();
            }
            KeyCode::Enter => return Some(ArticleAction::SubmitComment { reply_to: None }),
            KeyCode::Backspace => { app.comment_input.pop(); }
            KeyCode::Char(c) => { app.comment_input.push(c); }
            _ => {}
        }
        return None;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Some(ArticleAction::Back),

        // Tab 切换焦点
        KeyCode::Tab => {
            app.article_focus_comments = !app.article_focus_comments;
        }

        // ↑/↓ 始终滚动文章内容
        KeyCode::Up => {
            app.article_scroll = app.article_scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            app.article_scroll += 1;
        }
        KeyCode::PageUp => {
            app.article_scroll = app.article_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.article_scroll += 10;
        }

        // j/k 根据焦点决定行为
        KeyCode::Char('j') => {
            if app.article_focus_comments {
                if app.comment_selected + 1 < app.comments.len() {
                    app.comment_selected += 1;
                }
            } else {
                app.article_scroll += 1;
            }
        }
        KeyCode::Char('k') => {
            if app.article_focus_comments {
                if app.comment_selected > 0 {
                    app.comment_selected -= 1;
                }
            } else {
                app.article_scroll = app.article_scroll.saturating_sub(1);
            }
        }

        KeyCode::Char('c') => {
            app.article_focus_comments = true;
            app.commenting = true;
            app.comment_input.clear();
        }
        KeyCode::Char('r') => {
            let parent_id = app.comments.get(app.comment_selected).map(|c| c.id);
            app.article_focus_comments = true;
            app.commenting = true;
            app.comment_input.clear();
            return Some(ArticleAction::StartReply(parent_id));
        }
        KeyCode::Char('e') => {
            if let Some(article) = &app.current_article {
                let can_edit = app
                    .current_user
                    .as_ref()
                    .map(|u| u.id == article.author_id || u.role == "admin")
                    .unwrap_or(false);
                if can_edit {
                    return Some(ArticleAction::Edit);
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(article) = &app.current_article {
                let can_delete = app
                    .current_user
                    .as_ref()
                    .map(|u| u.id == article.author_id || u.role == "admin")
                    .unwrap_or(false);
                if can_delete {
                    return Some(ArticleAction::Delete(article.id));
                }
            }
        }
        _ => {}
    }
    None
}

fn in_rect(x: u16, y: u16, r: (u16, u16, u16, u16)) -> bool {
    x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Option<ArticleAction> {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            if in_rect(mouse.column, mouse.row, app.comments_area) && app.article_focus_comments {
                if app.comment_selected > 0 {
                    app.comment_selected -= 1;
                }
            } else {
                app.article_scroll = app.article_scroll.saturating_sub(2);
            }
        }
        MouseEventKind::ScrollDown => {
            if in_rect(mouse.column, mouse.row, app.comments_area) && app.article_focus_comments {
                if app.comment_selected + 1 < app.comments.len() {
                    app.comment_selected += 1;
                }
            } else {
                app.article_scroll += 2;
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if in_rect(mouse.column, mouse.row, app.article_area) {
                app.article_focus_comments = false;
            } else if in_rect(mouse.column, mouse.row, app.comments_area) {
                app.article_focus_comments = true;
                // 计算点击的是哪条评论（每条评论占2行，列表从 y+1 开始）
                let list_y = app.comments_area.1 + 1; // 跳过边框
                if mouse.row >= list_y {
                    let row_offset = (mouse.row - list_y) as usize;
                    let idx = row_offset / 2; // 每条评论2行
                    if idx < app.comments.len() {
                        app.comment_selected = idx;
                    }
                }
            }
        }
        _ => {}
    }
    None
}

pub enum ArticleAction {
    Back,
    Edit,
    Delete(u32),
    StartReply(Option<u32>),
    SubmitComment { reply_to: Option<u32> },
}
