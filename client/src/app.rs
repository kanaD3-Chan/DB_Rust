use std::collections::VecDeque;

use crate::api::{Api, ArticleWithAuthor, Category, CommentWithAuthor, UserPublic};
use crate::ws::Notification;

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Login,
    Home,
    Article(u32),
    Editor { article_id: Option<u32> },
    Members,
    Profile,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginMode {
    Login,
    Register,
}

pub struct App {
    pub api: Api,
    pub page: Page,
    pub prev_page: Option<Page>,

    // 认证
    pub current_user: Option<UserPublic>,
    pub login_mode: LoginMode,

    // 首页
    pub articles: Vec<ArticleWithAuthor>,
    pub article_total: u32,
    pub article_page: u32,
    pub article_selected: usize,
    pub categories: Vec<Category>,
    pub filter_category: Option<String>,
    pub search_query: String,
    pub searching: bool,

    // 文章详情
    pub current_article: Option<ArticleWithAuthor>,
    pub comments: Vec<CommentWithAuthor>,
    pub article_scroll: u16,
    pub comment_selected: usize,
    pub comment_input: String,
    pub commenting: bool,
    pub article_focus_comments: bool, // true = 焦点在评论区
    pub article_area: (u16, u16, u16, u16),  // (x, y, w, h) 文章内容区
    pub comments_area: (u16, u16, u16, u16), // (x, y, w, h) 评论区

    // 编辑器
    pub editor_title: String,
    pub editor_title_cursor: usize,
    pub editor_content: String,
    pub editor_cursor: usize, // byte offset
    pub editor_category: String,
    pub editor_preview: bool,
    pub editor_focused_title: bool,

    // 成员管理
    pub members: Vec<UserPublic>,
    pub member_selected: usize,

    // 个人中心
    pub profile_nickname: String,
    pub profile_password: String,
    pub profile_focused_nick: bool,

    // 通知
    pub notifications: VecDeque<Notification>,
    pub show_notifications: bool,

    // 弹窗
    pub popup: Option<PopupState>,

    // 状态消息
    pub status_msg: Option<(String, bool)>, // (message, is_error)

    // 输入焦点
    pub input_username: String,
    pub input_password: String,
    pub input_nickname: String,
    pub login_focused_field: usize, // 0=username, 1=password, 2=nickname

    pub should_quit: bool,
}

#[derive(Debug, Clone)]
pub struct PopupState {
    pub title: String,
    pub message: String,
    pub confirm_action: PopupAction,
}

#[derive(Debug, Clone)]
pub enum PopupAction {
    DeleteArticle(u32),
    DeleteMember(u32),
    ChangeRole(u32, String),
    None,
}

impl App {
    pub fn new() -> Self {
        Self {
            api: Api::new(),
            page: Page::Login,
            prev_page: None,

            current_user: None,
            login_mode: LoginMode::Login,

            articles: vec![],
            article_total: 0,
            article_page: 1,
            article_selected: 0,
            categories: vec![],
            filter_category: None,
            search_query: String::new(),
            searching: false,

            current_article: None,
            comments: vec![],
            article_scroll: 0,
            comment_selected: 0,
            comment_input: String::new(),
            commenting: false,
            article_focus_comments: false,
            article_area: (0, 0, 0, 0),
            comments_area: (0, 0, 0, 0),

            editor_title: String::new(),
            editor_title_cursor: 0,
            editor_content: String::new(),
            editor_cursor: 0,
            editor_category: "general".into(),
            editor_preview: false,
            editor_focused_title: true,

            members: vec![],
            member_selected: 0,

            profile_nickname: String::new(),
            profile_password: String::new(),
            profile_focused_nick: true,

            notifications: VecDeque::new(),
            show_notifications: false,

            popup: None,
            status_msg: None,

            input_username: String::new(),
            input_password: String::new(),
            input_nickname: String::new(),
            login_focused_field: 0,

            should_quit: false,
        }
    }

    pub fn navigate(&mut self, page: Page) {
        let old = self.page.clone();
        self.prev_page = Some(old);
        self.page = page;
        self.status_msg = None;
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.prev_page.take() {
            self.page = prev;
        } else {
            self.page = Page::Home;
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_msg = Some((msg.into(), is_error));
    }

    pub fn push_notification(&mut self, n: Notification) {
        self.notifications.push_front(n);
        if self.notifications.len() > 20 {
            self.notifications.pop_back();
        }
    }

    pub fn is_admin(&self) -> bool {
        self.current_user
            .as_ref()
            .map(|u| u.role == "admin")
            .unwrap_or(false)
    }

    pub fn open_editor_new(&mut self) {
        self.editor_title.clear();
        self.editor_title_cursor = 0;
        self.editor_content.clear();
        self.editor_cursor = 0;
        self.editor_category = "general".into();
        self.editor_preview = false;
        self.editor_focused_title = true;
        self.navigate(Page::Editor { article_id: None });
    }

    pub fn open_editor_edit(&mut self, article: &ArticleWithAuthor) {
        self.editor_title = article.title.clone();
        self.editor_title_cursor = article.title.len();
        self.editor_content = article.content.clone();
        self.editor_cursor = article.content.len();
        self.editor_category = article.category.clone();
        self.editor_preview = false;
        self.editor_focused_title = false;
        self.navigate(Page::Editor {
            article_id: Some(article.id),
        });
    }
}
