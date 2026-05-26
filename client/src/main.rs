use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

mod api;
mod app;
mod components;
mod pages;
mod ws;

use app::{App, Page, PopupAction};
use pages::{
    article::ArticleAction,
    editor::EditorAction,
    home::HomeAction,
    login::LoginAction,
    members::MembersAction,
    profile::ProfileAction,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // WebSocket 通知通道
    let (ws_tx, mut ws_rx) = mpsc::channel::<ws::Notification>(32);
    tokio::spawn(ws::connect(ws_tx));

    // 评论回复目标
    let mut reply_to: Option<u32> = None;

    loop {
        // 接收 WebSocket 通知
        while let Ok(notif) = ws_rx.try_recv() {
            app.push_notification(notif);
        }

        // 渲染
        terminal.draw(|f| {
            match &app.page {
                Page::Login => pages::login::render(f, &app),
                Page::Home => {
                    pages::home::render(f, &app);
                    if app.show_notifications {
                        components::toast::render_notification_list(f, &app.notifications);
                    }
                }
                Page::Article(_) => {
                    pages::article::render(f, &app);
                }
                Page::Editor { .. } => pages::editor::render(f, &app),
                Page::Members => pages::members::render(f, &app),
                Page::Profile => pages::profile::render(f, &app),
            }

            // 弹窗覆盖
            if let Some(popup) = &app.popup {
                components::popup::render_popup(f, &popup.title, &popup.message);
            }

            // Toast 通知（最新一条）
            if !app.show_notifications {
                if let Some(notif) = app.notifications.front() {
                    components::toast::render_toast(f, notif);
                }
            }
        })?;

        // 更新文章页布局坐标（用于鼠标命中检测）
        if matches!(app.page, Page::Article(_)) {
            let size = terminal.size()?;
            let rect = ratatui::layout::Rect::new(0, 0, size.width, size.height);
            let (art, cmt) = pages::article::layout_areas(rect);
            app.article_area = (art.x, art.y, art.width, art.height);
            app.comments_area = (cmt.x, cmt.y, cmt.width, cmt.height);
        }

        // 事件处理（100ms 超时）
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) => {
                // 全局退出
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    break;
                }

                // 弹窗处理
                if app.popup.is_some() {
                    match key.code {
                        KeyCode::Enter => {
                            let action = app.popup.take().map(|p| p.confirm_action);
                            if let Some(action) = action {
                                handle_popup_confirm(&mut app, action).await;
                            }
                        }
                        KeyCode::Esc => {
                            app.popup = None;
                        }
                        _ => {}
                    }
                    continue;
                }

                match &app.page.clone() {
                    Page::Login => {
                        if let Some(action) = pages::login::handle_key(&mut app, key) {
                            match action {
                                LoginAction::Submit => handle_login_submit(&mut app).await,
                            }
                        }
                    }
                    Page::Home => {
                        if key.code == KeyCode::Char('q') {
                            break;
                        }
                        if let Some(action) = pages::home::handle_key(&mut app, key) {
                            handle_home_action(&mut app, action).await;
                        }
                    }
                    Page::Article(_) => {
                        if let Some(action) = pages::article::handle_key(&mut app, key) {
                            handle_article_action(&mut app, action, &mut reply_to).await;
                        }
                    }
                    Page::Editor { .. } => {
                        if let Some(action) = pages::editor::handle_key(&mut app, key) {
                            handle_editor_action(&mut app, action).await;
                        }
                    }
                    Page::Members => {
                        if let Some(action) = pages::members::handle_key(&mut app, key) {
                            handle_members_action(&mut app, action).await;
                        }
                    }
                    Page::Profile => {
                        if let Some(action) = pages::profile::handle_key(&mut app, key) {
                            handle_profile_action(&mut app, action).await;
                        }
                    }
                }
            }
            Event::Mouse(mouse) => {
                match &app.page.clone() {
                    Page::Home => {
                        if let Some(action) = pages::home::handle_mouse(&mut app, mouse) {
                            handle_home_action(&mut app, action).await;
                        }
                    }
                    Page::Article(_) => {
                        pages::article::handle_mouse(&mut app, mouse);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn handle_login_submit(app: &mut App) {
    let username = app.input_username.trim().to_string();
    let password = app.input_password.clone();
    let nickname = app.input_nickname.trim().to_string();

    if username.is_empty() || password.is_empty() {
        app.set_status("用户名和密码不能为空", true);
        return;
    }

    let result = match app.login_mode {
        app::LoginMode::Login => app.api.login(&username, &password).await,
        app::LoginMode::Register => {
            if nickname.is_empty() {
                app.set_status("昵称不能为空", true);
                return;
            }
            app.api.register(&username, &nickname, &password).await
        }
    };

    match result {
        Ok(auth) => {
            app.api.set_token(auth.token);
            app.current_user = Some(auth.user);
            app.input_username.clear();
            app.input_password.clear();
            app.input_nickname.clear();
            app.navigate(Page::Home);
            load_home_data(app).await;
        }
        Err(e) => {
            app.set_status(e, true);
        }
    }
}

async fn load_home_data(app: &mut App) {
    let category = app.filter_category.as_deref().map(|s| s.to_string());
    let search = if app.search_query.is_empty() {
        None
    } else {
        Some(app.search_query.clone())
    };

    match app
        .api
        .list_articles(
            app.article_page,
            u32::MAX,
            category.as_deref(),
            search.as_deref(),
        )
        .await
    {
        Ok(data) => {
            app.articles = data.articles;
            app.article_total = data.total;
            app.article_selected = 0;
        }
        Err(e) => {
            app.set_status(e, true);
        }
    }
}

async fn handle_home_action(app: &mut App, action: HomeAction) {
    match action {
        HomeAction::OpenArticle(id) => {
            match app.api.get_article(id).await {
                Ok(article) => {
                    app.current_article = Some(article);
                    app.article_scroll = 0;
                    app.comment_selected = 0;
                    app.navigate(Page::Article(id));
                    match app.api.list_comments(id).await {
                        Ok(comments) => app.comments = comments,
                        Err(e) => app.set_status(e, true),
                    }
                }
                Err(e) => app.set_status(e, true),
            }
        }
        HomeAction::NewArticle => {
            if app.api.has_token() {
                app.open_editor_new();
            } else {
                app.set_status("请先登录", true);
            }
        }
        HomeAction::GoMembers => {
            if app.is_admin() {
                match app.api.list_members().await {
                    Ok(members) => {
                        app.members = members;
                        app.member_selected = 0;
                        app.navigate(Page::Members);
                    }
                    Err(e) => app.set_status(e, true),
                }
            } else {
                app.set_status("仅管理员可访问", true);
            }
        }
        HomeAction::GoProfile => {
            if let Some(user) = &app.current_user {
                app.profile_nickname = user.nickname.clone();
                app.profile_password.clear();
                app.navigate(Page::Profile);
            } else {
                app.set_status("请先登录", true);
            }
        }
        HomeAction::Search => {
            load_home_data(app).await;
        }
        HomeAction::NextCategory => {
            let cats = [None, Some("general"), Some("tech"), Some("security"), Some("life")];
            let current_idx = cats
                .iter()
                .position(|c| c.as_deref() == app.filter_category.as_deref())
                .unwrap_or(0);
            let next = cats[(current_idx + 1) % cats.len()];
            app.filter_category = next.map(|s| s.to_string());
            load_home_data(app).await;
        }
        HomeAction::Refresh => {
            load_home_data(app).await;
        }
    }
}

async fn handle_article_action(app: &mut App, action: ArticleAction, reply_to: &mut Option<u32>) {
    match action {
        ArticleAction::Back => {
            app.go_back();
        }
        ArticleAction::Edit => {
            if let Some(article) = app.current_article.clone() {
                app.open_editor_edit(&article);
            }
        }
        ArticleAction::Delete(id) => {
            app.popup = Some(app::PopupState {
                title: "确认删除".into(),
                message: "确定要删除这篇文章吗？".into(),
                confirm_action: PopupAction::DeleteArticle(id),
            });
        }
        ArticleAction::StartReply(parent_id) => {
            *reply_to = parent_id;
        }
        ArticleAction::SubmitComment { reply_to: _ } => {
            let content = app.comment_input.trim().to_string();
            if content.is_empty() {
                app.set_status("评论不能为空", true);
                app.commenting = false;
                return;
            }
            let article_id = match &app.page {
                Page::Article(id) => *id,
                _ => return,
            };
            match app.api.create_comment(article_id, *reply_to, &content).await {
                Ok(_) => {
                    app.comment_input.clear();
                    app.commenting = false;
                    *reply_to = None;
                    match app.api.list_comments(article_id).await {
                        Ok(comments) => app.comments = comments,
                        Err(e) => app.set_status(e, true),
                    }
                }
                Err(e) => {
                    app.set_status(e, true);
                    app.commenting = false;
                }
            }
        }
    }
}

async fn handle_editor_action(app: &mut App, action: EditorAction) {
    match action {
        EditorAction::Cancel => {
            app.go_back();
        }
        EditorAction::Save => {
            let title = app.editor_title.trim().to_string();
            let content = app.editor_content.trim().to_string();
            let category = app.editor_category.clone();

            if title.is_empty() || content.is_empty() {
                app.set_status("标题和内容不能为空", true);
                return;
            }

            let article_id = match &app.page {
                Page::Editor { article_id } => *article_id,
                _ => None,
            };

            let result = match article_id {
                Some(id) => app.api.update_article(id, &title, &content, &category).await,
                None => app.api.create_article(&title, &content, &category).await,
            };

            match result {
                Ok(_) => {
                    app.set_status("保存成功", false);
                    app.go_back();
                    load_home_data(app).await;
                }
                Err(e) => app.set_status(e, true),
            }
        }
    }
}

async fn handle_members_action(app: &mut App, action: MembersAction) {
    match action {
        MembersAction::Back => {
            app.go_back();
        }
        MembersAction::ToggleRole(id, role) => {
            match app.api.update_member_role(id, &role).await {
                Ok(_) => {
                    app.set_status(format!("已将角色改为 {}", role), false);
                    match app.api.list_members().await {
                        Ok(members) => app.members = members,
                        Err(e) => app.set_status(e, true),
                    }
                }
                Err(e) => app.set_status(e, true),
            }
        }
        MembersAction::Delete(id) => {
            app.popup = Some(app::PopupState {
                title: "确认删除".into(),
                message: "确定要删除该成员吗？".into(),
                confirm_action: PopupAction::DeleteMember(id),
            });
        }
    }
}

async fn handle_profile_action(app: &mut App, action: ProfileAction) {
    match action {
        ProfileAction::Back => {
            app.go_back();
        }
        ProfileAction::Save => {
            let nickname = if app.profile_nickname.is_empty() {
                None
            } else {
                Some(app.profile_nickname.as_str())
            };
            let password = if app.profile_password.is_empty() {
                None
            } else {
                Some(app.profile_password.as_str())
            };

            match app.api.update_profile(nickname, password).await {
                Ok(_) => {
                    app.set_status("保存成功", false);
                    if let Ok(user) = app.api.get_profile().await {
                        app.current_user = Some(user);
                    }
                }
                Err(e) => app.set_status(e, true),
            }
        }
    }
}

async fn handle_popup_confirm(app: &mut App, action: PopupAction) {
    match action {
        PopupAction::DeleteArticle(id) => {
            match app.api.delete_article(id).await {
                Ok(_) => {
                    app.set_status("文章已删除", false);
                    app.go_back();
                    load_home_data(app).await;
                }
                Err(e) => app.set_status(e, true),
            }
        }
        PopupAction::DeleteMember(id) => {
            match app.api.delete_member(id).await {
                Ok(_) => {
                    app.set_status("成员已删除", false);
                    match app.api.list_members().await {
                        Ok(members) => app.members = members,
                        Err(e) => app.set_status(e, true),
                    }
                }
                Err(e) => app.set_status(e, true),
            }
        }
        PopupAction::ChangeRole(id, role) => {
            match app.api.update_member_role(id, &role).await {
                Ok(_) => {
                    app.set_status(format!("角色已更新为 {}", role), false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        PopupAction::None => {}
    }
}
