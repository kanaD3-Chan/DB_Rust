use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let items: Vec<ListItem> = app
        .members
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let selected = i == app.member_selected;
            let role_color = if u.role == "admin" {
                Color::Red
            } else {
                Color::Green
            };
            let style = if selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(vec![Line::from(vec![
                Span::styled(format!("{:<20}", u.nickname), style),
                Span::styled(format!("@{:<16}", u.username), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("[{}]", u.role),
                    Style::default().fg(role_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  加入: {}", &u.created_at[..10]),
                    Style::default().fg(Color::DarkGray),
                ),
            ])])
        })
        .collect();

    let title = format!("成员管理 ({}) [j/k:移动 r:切换角色 d:删除 q:返回]", app.members.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = ListState::default();
    if !app.members.is_empty() {
        state.select(Some(app.member_selected));
    }
    f.render_stateful_widget(list, chunks[0], &mut state);

    let hint = "j/k:移动  r:切换角色(admin↔member)  d:删除成员  q:返回";
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Option<MembersAction> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            return Some(MembersAction::Back);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if app.member_selected + 1 < app.members.len() {
                app.member_selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.member_selected > 0 {
                app.member_selected -= 1;
            }
        }
        KeyCode::Char('g') => {
            app.member_selected = 0;
        }
        KeyCode::Char('G') => {
            if !app.members.is_empty() {
                app.member_selected = app.members.len() - 1;
            }
        }
        KeyCode::Char('r') => {
            if let Some(m) = app.members.get(app.member_selected) {
                let new_role = if m.role == "admin" { "member" } else { "admin" };
                return Some(MembersAction::ToggleRole(m.id, new_role.to_string()));
            }
        }
        KeyCode::Char('d') => {
            if let Some(m) = app.members.get(app.member_selected) {
                return Some(MembersAction::Delete(m.id));
            }
        }
        _ => {}
    }
    None
}

pub enum MembersAction {
    Back,
    ToggleRole(u32, String),
    Delete(u32),
}
