use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ── 用户 ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub nickname: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub avatar: Option<String>,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub nickname: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPublic {
    pub id: u32,
    pub username: String,
    pub nickname: String,
    pub role: String,
    pub avatar: Option<String>,
    pub created_at: NaiveDateTime,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            nickname: u.nickname,
            role: u.role,
            avatar: u.avatar,
            created_at: u.created_at,
        }
    }
}

// ── 文章 ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Article {
    pub id: u32,
    pub author_id: u32,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub category: String,
    pub pinned: bool,
    pub view_count: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct ArticleWithAuthor {
    pub id: u32,
    pub author_id: u32,
    pub author_name: String,
    pub author_nickname: String,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub category: String,
    pub pinned: bool,
    pub view_count: u32,
    pub comment_count: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateArticleRequest {
    pub title: String,
    pub content: String,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ArticleListResponse {
    pub articles: Vec<ArticleWithAuthor>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

// ── 评论 ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: u32,
    pub article_id: u32,
    pub author_id: u32,
    pub parent_id: Option<u32>,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct CommentWithAuthor {
    pub id: u32,
    pub article_id: u32,
    pub author_id: u32,
    pub author_name: String,
    pub author_nickname: String,
    pub parent_id: Option<u32>,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub parent_id: Option<u32>,
    pub content: String,
}

// ── 分类 ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

// ── 通用 ──

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(msg.into()) }
    }
}

// ── WebSocket 通知 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    #[serde(rename = "type")]
    pub kind: String,        // "new_article" | "new_comment"
    pub article_id: Option<u32>,
    pub article_title: Option<String>,
    pub author_name: Option<String>,
    pub preview: Option<String>,
    pub time: NaiveDateTime,
}
