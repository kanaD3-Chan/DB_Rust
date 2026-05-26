use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::components::input::InputField;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Min(12),
            Constraint::Percentage(15),
        ])
        .split(area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Min(40),
            Constraint::Percentage(25),
        ])
        .split(vert[1]);

    let container = horiz[1];

    // 用户信息展示
    if let Some(user) = &app.current_user {
        let info = format!(
            "用户名: {}\n昵称: {}\n角色: {}\n加入: {}",
            user.username,
            user.nickname,
            user.role,
            &user.created_at[..10]
        );
        let info_para = Paragraph::new(info)
            .block(
                Block::default()
                    .title("个人信息")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(
            info_para,
            Rect {
                x: container.x,
                y: container.y,
                width: container.width,
                height: 6,
            },
        );
    }

    let form_y = container.y + 7;

    // 昵称修改
    InputField {
        label: "新昵称",
        value: &app.profile_nickname,
        focused: app.profile_focused_nick,
        masked: false,
    }
    .render(
        f,
        Rect {
            x: container.x,
            y: form_y,
            width: container.width,
            height: 3,
        },
    );

    // 密码修改
    InputField {
        label: "新密码（留空不修改）",
        value: &app.profile_password,
        focused: !app.profile_focused_nick,
        masked: true,
    }
    .render(
        f,
        Rect {
            x: container.x,
            y: form_y + 3,
            width: container.width,
            height: 3,
        },
    );

    // 状态消息
    if let Some((msg, is_err)) = &app.status_msg {
        let color = if *is_err { Color::Red } else { Color::Green };
        f.render_widget(
            Paragraph::new(msg.as_str()).style(Style::default().fg(color)),
            Rect {
                x: container.x,
                y: form_y + 7,
                width: container.width,
                height: 1,
            },
        );
    }

    // 快捷键提示
    let hint = "Tab:切换字段  Enter:保存  q:返回";
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        Rect {
            x: area.x,
            y: area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        },
    );
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Option<ProfileAction> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            return Some(ProfileAction::Back);
        }
        KeyCode::Tab => {
            app.profile_focused_nick = !app.profile_focused_nick;
        }
        KeyCode::Enter => {
            return Some(ProfileAction::Save);
        }
        KeyCode::Backspace => {
            if app.profile_focused_nick {
                app.profile_nickname.pop();
            } else {
                app.profile_password.pop();
            }
        }
        KeyCode::Char(c) => {
            if app.profile_focused_nick {
                app.profile_nickname.push(c);
            } else {
                app.profile_password.push(c);
            }
        }
        _ => {}
    }
    None
}

pub enum ProfileAction {
    Back,
    Save,
}
