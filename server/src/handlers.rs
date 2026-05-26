use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::{CurrentUser, create_token};
use crate::config::AppState;
use crate::models::*;
use crate::repository::{ArticleRepo, CategoryRepo, CommentRepo, UserRepo};

type ApiResult<T> = Result<Json<ApiResponse<T>>, (StatusCode, Json<ApiResponse<String>>)>;

fn internal_err(msg: &'static str) -> (StatusCode, Json<ApiResponse<String>>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(msg)))
}

// ── 注册 ──

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<AuthResponse> {
    if body.username.len() < 3 || body.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err("用户名至少3位，密码至少6位")),
        ));
    }

    let repo = UserRepo(&state.db);

    if repo.username_taken(&body.username).await {
        return Err((StatusCode::CONFLICT, Json(ApiResponse::err("用户名已被注册"))));
    }

    let password_hash = hash(&body.password, DEFAULT_COST)
        .map_err(|_| internal_err("密码加密失败"))?;

    let role = if repo.count_active().await == 0 { "admin" } else { "member" };

    let user_id = repo
        .create(&body.username, &body.nickname, &password_hash, role)
        .await
        .map_err(|_| internal_err("注册失败"))?;

    let token = create_token(user_id, &body.username, role, &state.jwt_secret);

    Ok(Json(ApiResponse::ok(AuthResponse {
        token,
        user: UserPublic {
            id: user_id,
            username: body.username,
            nickname: body.nickname,
            role: role.into(),
            avatar: None,
            created_at: Utc::now().naive_utc(),
        },
    })))
}

// ── 登录 ──

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<AuthResponse> {
    let repo = UserRepo(&state.db);
    let user = repo.find_by_username(&body.username).await;

    match user {
        Some(u) if verify(&body.password, &u.password_hash).unwrap_or(false) => {
            let token = create_token(u.id, &u.username, &u.role, &state.jwt_secret);
            Ok(Json(ApiResponse::ok(AuthResponse { token, user: u.into() })))
        }
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::err("用户名或密码错误")),
        )),
    }
}

// ── 文章列表 ──

#[derive(Deserialize)]
pub struct ArticleListQuery {
    page: Option<u32>,
    page_size: Option<u32>,
    category: Option<String>,
    search: Option<String>,
}

pub async fn list_articles(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ArticleListQuery>,
) -> Json<ApiResponse<ArticleListResponse>> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(50);
    let offset = (page - 1) * page_size;

    let repo = ArticleRepo(&state.db);
    let cat = q.category.as_deref();
    let search = q.search.as_deref();

    let total = repo.count(cat, search).await;
    let articles = repo.list(cat, search, page_size, offset).await;

    Json(ApiResponse::ok(ArticleListResponse {
        articles,
        total: total as u32,
        page,
        page_size,
    }))
}

// ── 文章详情 ──

pub async fn get_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
) -> ApiResult<ArticleWithAuthor> {
    let repo = ArticleRepo(&state.db);
    repo.increment_view(id).await;

    match repo.find_by_id(id).await {
        Some(a) => Ok(Json(ApiResponse::ok(a))),
        None => Err((StatusCode::NOT_FOUND, Json(ApiResponse::err("文章不存在")))),
    }
}

// ── 创建文章 ──

pub async fn create_article(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(body): Json<CreateArticleRequest>,
) -> ApiResult<Article> {
    let summary: String = body.content.chars().take(200).collect();
    let category = body.category.as_deref().unwrap_or("general");

    let repo = ArticleRepo(&state.db);
    let article_id = repo
        .create(user.id, &body.title, &body.content, &summary, category)
        .await
        .map_err(|_| internal_err("发布失败"))?;

    let _ = state.notify_tx.send(Notification {
        kind: "new_article".into(),
        article_id: Some(article_id),
        article_title: Some(body.title.clone()),
        author_name: Some(user.username.clone()),
        preview: Some(summary),
        time: Utc::now().naive_utc(),
    });

    let article = repo
        .fetch_created(article_id)
        .await
        .map_err(|_| internal_err("获取文章失败"))?;

    Ok(Json(ApiResponse::ok(article)))
}

// ── 更新文章 ──

pub async fn update_article(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<u32>,
    Json(body): Json<UpdateArticleRequest>,
) -> ApiResult<()> {
    let repo = ArticleRepo(&state.db);

    match repo.find_raw(id).await {
        Some(a) if a.author_id == user.id || user.role == "admin" => {
            let title = body.title.unwrap_or(a.title);
            let content = body.content.unwrap_or(a.content);
            let summary: String = content.chars().take(200).collect();
            let category = body.category.unwrap_or(a.category);

            repo.update(id, &title, &content, &summary, &category)
                .await
                .map_err(|_| internal_err("更新失败"))?;

            Ok(Json(ApiResponse::ok(())))
        }
        Some(_) => Err((StatusCode::FORBIDDEN, Json(ApiResponse::err("无权编辑此文章")))),
        None => Err((StatusCode::NOT_FOUND, Json(ApiResponse::err("文章不存在")))),
    }
}

// ── 删除文章 ──

pub async fn delete_article(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<u32>,
) -> ApiResult<()> {
    let repo = ArticleRepo(&state.db);

    match repo.find_raw(id).await {
        Some(a) if a.author_id == user.id || user.role == "admin" => {
            repo.soft_delete(id).await.ok();
            Ok(Json(ApiResponse::ok(())))
        }
        Some(_) => Err((StatusCode::FORBIDDEN, Json(ApiResponse::err("无权删除此文章")))),
        None => Err((StatusCode::NOT_FOUND, Json(ApiResponse::err("文章不存在")))),
    }
}

// ── 评论列表 ──

pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    Path(article_id): Path<u32>,
) -> Json<ApiResponse<Vec<CommentWithAuthor>>> {
    let repo = CommentRepo(&state.db);
    Json(ApiResponse::ok(repo.list_for_article(article_id).await))
}

// ── 创建评论 ──

pub async fn create_comment(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(article_id): Path<u32>,
    Json(body): Json<CreateCommentRequest>,
) -> ApiResult<Comment> {
    let repo = CommentRepo(&state.db);

    if !repo.article_exists(article_id).await {
        return Err((StatusCode::NOT_FOUND, Json(ApiResponse::err("文章不存在"))));
    }

    if let Some(pid) = body.parent_id {
        if !repo.parent_exists(pid, article_id).await {
            return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::err("父评论不存在"))));
        }
    }

    let comment_id = repo
        .create(article_id, user.id, body.parent_id, &body.content)
        .await
        .map_err(|_| internal_err("评论失败"))?;

    let title = repo.get_article_title(article_id).await;

    let _ = state.notify_tx.send(Notification {
        kind: "new_comment".into(),
        article_id: Some(article_id),
        article_title: title,
        author_name: Some(user.username),
        preview: Some(body.content.chars().take(100).collect()),
        time: Utc::now().naive_utc(),
    });

    let comment = repo
        .fetch_created(comment_id)
        .await
        .map_err(|_| internal_err("获取评论失败"))?;

    Ok(Json(ApiResponse::ok(comment)))
}

// ── 成员管理 ──

pub async fn list_members(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> ApiResult<Vec<UserPublic>> {
    if user.role != "admin" {
        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::err("仅管理员可访问"))));
    }
    Ok(Json(ApiResponse::ok(UserRepo(&state.db).list_all().await)))
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub async fn update_member_role(
    State(state): State<Arc<AppState>>,
    admin: CurrentUser,
    Path(id): Path<u32>,
    Json(body): Json<UpdateRoleRequest>,
) -> ApiResult<()> {
    if admin.role != "admin" {
        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::err("仅管理员可操作"))));
    }
    if body.role != "admin" && body.role != "member" {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::err("无效角色"))));
    }

    UserRepo(&state.db).update_role(id, &body.role).await.ok();
    Ok(Json(ApiResponse::ok(())))
}

pub async fn delete_member(
    State(state): State<Arc<AppState>>,
    admin: CurrentUser,
    Path(id): Path<u32>,
) -> ApiResult<()> {
    if admin.role != "admin" {
        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::err("仅管理员可操作"))));
    }
    if id == admin.id {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::err("不能删除自己"))));
    }

    UserRepo(&state.db).soft_delete(id).await.ok();
    Ok(Json(ApiResponse::ok(())))
}

// ── 个人中心 ──

pub async fn get_profile(
    user: CurrentUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult<UserPublic> {
    match UserRepo(&state.db).find_by_id(user.id).await {
        Some(u) => Ok(Json(ApiResponse::ok(u.into()))),
        None => Err((StatusCode::NOT_FOUND, Json(ApiResponse::err("用户不存在")))),
    }
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub password: Option<String>,
}

pub async fn update_profile(
    user: CurrentUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateProfileRequest>,
) -> ApiResult<()> {
    let repo = UserRepo(&state.db);

    if let Some(ref nick) = body.nickname {
        repo.update_nickname(user.id, nick).await.ok();
    }

    if let Some(ref pwd) = body.password {
        if pwd.len() < 6 {
            return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::err("密码至少6位"))));
        }
        let h = hash(pwd, DEFAULT_COST).map_err(|_| internal_err("密码加密失败"))?;
        repo.update_password(user.id, &h).await.ok();
    }

    Ok(Json(ApiResponse::ok(())))
}

// ── 分类列表 ──

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<Category>>> {
    Json(ApiResponse::ok(CategoryRepo(&state.db).list().await))
}
