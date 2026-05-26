use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, LoginMode};
use crate::components::input::InputField;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // 居中容器
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(14),
            Constraint::Percentage(20),
        ])
        .split(area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Min(40),
            Constraint::Percentage(30),
        ])
        .split(vert[1]);

    let container = horiz[1];

    // 标题
    let title = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "SLsec 实验室论坛",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Security Lab Forum",
            Style::default().fg(Color::DarkGray),
        )]),
    ])
    .alignment(Alignment::Center);

    let title_area = Rect {
        x: container.x,
        y: container.y,
        width: container.width,
        height: 2,
    };
    f.render_widget(title, title_area);

    // 模式切换提示
    let mode_hint = match app.login_mode {
        LoginMode::Login => "[Tab] 切换到注册",
        LoginMode::Register => "[Tab] 切换到登录",
    };
    let mode_para = Paragraph::new(mode_hint)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    let mode_area = Rect {
        x: container.x,
        y: container.y + 2,
        width: container.width,
        height: 1,
    };
    f.render_widget(mode_para, mode_area);

    // 表单区域
    let form_y = container.y + 4;
    let field_h = 3u16;

    let fields_count = if app.login_mode == LoginMode::Register { 3 } else { 2 };

    // 用户名
    InputField {
        label: "用户名",
        value: &app.input_username,
        focused: app.login_focused_field == 0,
        masked: false,
    }
    .render(
        f,
        Rect {
            x: container.x,
            y: form_y,
            width: container.width,
            height: field_h,
        },
    );

    // 密码
    InputField {
        label: "密码",
        value: &app.input_password,
        focused: app.login_focused_field == 1,
        masked: true,
    }
    .render(
        f,
        Rect {
            x: container.x,
            y: form_y + field_h,
            width: container.width,
            height: field_h,
        },
    );

    // 昵称（仅注册）
    if app.login_mode == LoginMode::Register {
        InputField {
            label: "昵称",
            value: &app.input_nickname,
            focused: app.login_focused_field == 2,
            masked: false,
        }
        .render(
            f,
            Rect {
                x: container.x,
                y: form_y + field_h * 2,
                width: container.width,
                height: field_h,
            },
        );
    }

    // 按钮
    let btn_y = form_y + field_h * fields_count as u16;
    let btn_label = match app.login_mode {
        LoginMode::Login => "[ 登录 ]",
        LoginMode::Register => "[ 注册 ]",
    };
    let btn = Paragraph::new(btn_label)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(
        btn,
        Rect {
            x: container.x + container.width / 4,
            y: btn_y,
            width: container.width / 2,
            height: 1,
        },
    );

    // 状态消息
    if let Some((msg, is_err)) = &app.status_msg {
        let color = if *is_err { Color::Red } else { Color::Green };
        let status = Paragraph::new(msg.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(color));
        f.render_widget(
            status,
            Rect {
                x: container.x,
                y: btn_y + 2,
                width: container.width,
                height: 1,
            },
        );
    }

    // 快捷键提示
    let hint = Paragraph::new("Tab: 切换字段/模式  Enter: 确认  Ctrl+C: 退出")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(
        hint,
        Rect {
            x: area.x,
            y: area.height.saturating_sub(2),
            width: area.width,
            height: 1,
        },
    );
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Option<LoginAction> {
    let field_count = if app.login_mode == LoginMode::Register { 3 } else { 2 };

    match key.code {
        KeyCode::Tab => {
            app.login_focused_field = (app.login_focused_field + 1) % (field_count + 1);
            // 如果超出字段范围，切换模式
            if app.login_focused_field == field_count {
                app.login_focused_field = 0;
                app.login_mode = match app.login_mode {
                    LoginMode::Login => LoginMode::Register,
                    LoginMode::Register => LoginMode::Login,
                };
            }
        }
        KeyCode::Enter => {
            return Some(LoginAction::Submit);
        }
        KeyCode::Backspace => match app.login_focused_field {
            0 => { app.input_username.pop(); }
            1 => { app.input_password.pop(); }
            2 => { app.input_nickname.pop(); }
            _ => {}
        },
        KeyCode::Char(c) => match app.login_focused_field {
            0 => app.input_username.push(c),
            1 => app.input_password.push(c),
            2 => app.input_nickname.push(c),
            _ => {}
        },
        _ => {}
    }
    None
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Option<LoginAction> {
    if mouse.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
        // 简单的点击区域检测（基于固定布局）
        // 实际坐标依赖终端大小，这里做简化处理
    }
    None
}

pub enum LoginAction {
    Submit,
}
