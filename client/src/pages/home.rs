use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;
use crate::api::ArticleWithAuthor;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // 顶栏（分类标签+搜索框）
            Constraint::Min(0),    // 主体
            Constraint::Length(1), // 底栏
        ])
        .split(area);

    render_topbar(f, app, chunks[0]);
    render_body(f, app, chunks[1]);
    render_statusbar(f, app, chunks[2]);
}

fn render_topbar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    // 分类标签行
    let cats: &[Option<&str>] = &[None, Some("general"), Some("tech"), Some("security"), Some("life")];
    let labels = ["全部", "综合", "技术", "安全", "生活"];
    let cat_spans: Vec<Span> = cats.iter().zip(labels.iter()).map(|(cat, label)| {
        let active = cat.map(|s| s.to_string()) == app.filter_category;
        if active {
            Span::styled(format!(" [{}] ", label), Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("  {}  ", label), Style::default().fg(Color::DarkGray))
        }
    }).collect();
    f.render_widget(Paragraph::new(Line::from(cat_spans)), chunks[0]);

    // 搜索框 + 用户信息
    let user_info = app
        .current_user
        .as_ref()
        .map(|u| format!(" {} [{}]", u.nickname, u.role))
        .unwrap_or_default();
    let notif_count = app.notifications.len();
    let notif_hint = if notif_count > 0 { format!(" 🔔{}", notif_count) } else { String::new() };
    let title = format!("SLsec 论坛{}{}", user_info, notif_hint);

    let search_text = if app.searching {
        format!("搜索: {}█", app.search_query)
    } else if !app.search_query.is_empty() {
        format!("搜索: {}", app.search_query)
    } else {
        String::new()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Paragraph::new(search_text).block(block), chunks[1]);
}

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55), // 文章列表
            Constraint::Percentage(45), // 预览
        ])
        .split(area);

    render_article_list(f, app, chunks[0]);
    render_preview(f, app, chunks[1]);
}

fn render_article_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .articles
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let selected = i == app.article_selected;
            let pin = if a.pinned { "📌 " } else { "" };
            let title_style = if selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let meta_style = Style::default().fg(Color::DarkGray);

            let cat_color = match a.category.as_str() {
                "security" => Color::Red,
                "tech" => Color::Blue,
                "life" => Color::Green,
                _ => Color::DarkGray,
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::raw(pin),
                    Span::styled(truncate(&a.title, 35), title_style),
                    Span::styled(
                        format!(" [{}]", a.category),
                        Style::default().fg(cat_color),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!(
                            "  {} | 👁{} 💬{} | {}",
                            a.author_nickname,
                            a.view_count,
                            a.comment_count,
                            &a.created_at[..10]
                        ),
                        meta_style,
                    ),
                ]),
            ])
        })
        .collect();

    let title = format!(
        "文章列表 ({}/{}) [j/k 移动 Enter 查看 n 写文章]",
        app.article_selected + 1,
        app.articles.len()
    );

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = ListState::default();
    if !app.articles.is_empty() {
        state.select(Some(app.article_selected));
    }

    f.render_stateful_widget(list, area, &mut state);
}

fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let content = app
        .articles
        .get(app.article_selected)
        .map(|a| {
            format!(
                "{}\n\n作者: {}  分类: {}\n{}\n\n{}",
                a.title,
                a.author_nickname,
                a.category,
                "─".repeat(30),
                a.summary
            )
        })
        .unwrap_or_else(|| "暂无文章".into());

    let para = Paragraph::new(content)
        .block(
            Block::default()
                .title("预览")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(para, area);
}

fn render_statusbar(f: &mut Frame, app: &App, area: Rect) {
    let msg = if let Some((msg, is_err)) = &app.status_msg {
        let color = if *is_err { Color::Red } else { Color::Green };
        Paragraph::new(msg.as_str()).style(Style::default().fg(color))
    } else {
        let hint = "q:退出  Enter:查看  n:写文章  /:搜索  m:成员  p:个人  N:通知  Tab:分类";
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray))
    };
    f.render_widget(msg, area);
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Option<HomeAction> {
    if app.searching {
        match key.code {
            KeyCode::Esc => {
                app.searching = false;
            }
            KeyCode::Enter => {
                app.searching = false;
                return Some(HomeAction::Search);
            }
            KeyCode::Backspace => {
                app.search_query.pop();
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
            }
            _ => {}
        }
        return None;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.article_selected + 1 < app.articles.len() {
                app.article_selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.article_selected > 0 {
                app.article_selected -= 1;
            }
        }
        KeyCode::Char('g') => {
            app.article_selected = 0;
        }
        KeyCode::Char('G') => {
            if !app.articles.is_empty() {
                app.article_selected = app.articles.len() - 1;
            }
        }
        KeyCode::Enter => {
            if let Some(a) = app.articles.get(app.article_selected) {
                return Some(HomeAction::OpenArticle(a.id));
            }
        }
        KeyCode::Char('n') => {
            return Some(HomeAction::NewArticle);
        }
        KeyCode::Char('/') => {
            app.searching = true;
        }
        KeyCode::Char('m') => {
            return Some(HomeAction::GoMembers);
        }
        KeyCode::Char('p') => {
            return Some(HomeAction::GoProfile);
        }
        KeyCode::Char('N') => {
            app.show_notifications = !app.show_notifications;
        }
        KeyCode::Tab => {
            return Some(HomeAction::NextCategory);
        }
        KeyCode::Char('r') | KeyCode::F(5) => {
            return Some(HomeAction::Refresh);
        }
        _ => {}
    }
    None
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Option<HomeAction> {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            if app.article_selected + 1 < app.articles.len() {
                app.article_selected += 1;
            }
        }
        MouseEventKind::ScrollUp => {
            if app.article_selected > 0 {
                app.article_selected -= 1;
            }
        }
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            // 点击列表项（简化：每项占2行，从y=1开始）
            let list_y_start = 5u16; // topbar(4) + border(1)
            if mouse.row >= list_y_start {
                let idx = ((mouse.row - list_y_start) / 2) as usize;
                if idx < app.articles.len() {
                    app.article_selected = idx;
                    return Some(HomeAction::OpenArticle(app.articles[idx].id));
                }
            }
        }
        _ => {}
    }
    None
}

pub enum HomeAction {
    OpenArticle(u32),
    NewArticle,
    GoMembers,
    GoProfile,
    Search,
    NextCategory,
    Refresh,
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}…", chars[..max - 1].iter().collect::<String>())
    }
}
