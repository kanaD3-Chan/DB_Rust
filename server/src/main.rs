use std::sync::Arc;

use axum::{
    Router,
    middleware,
    routing::{delete, get, post, put},
};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod config;
mod handlers;
mod models;
mod repository;
mod ws;

use config::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = config::create_pool().await;
    println!("数据库连接成功");

    init_db(&pool).await;

    let (notify_tx, _) = broadcast::channel::<models::Notification>(128);

    let state = Arc::new(AppState {
        db: pool,
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "slsec-secret-key".into()),
        notify_tx,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public = Router::new()
        .route("/api/register", post(handlers::register))
        .route("/api/login", post(handlers::login))
        .route("/api/articles", get(handlers::list_articles))
        .route("/api/articles/{id}", get(handlers::get_article))
        .route("/api/articles/{id}/comments", get(handlers::list_comments))
        .route("/api/categories", get(handlers::list_categories))
        .route("/ws", get(ws::ws_handler));

    let protected = Router::new()
        .route("/api/articles", post(handlers::create_article))
        .route("/api/articles/{id}", put(handlers::update_article))
        .route("/api/articles/{id}", delete(handlers::delete_article))
        .route("/api/articles/{id}/comments", post(handlers::create_comment))
        .route("/api/members", get(handlers::list_members))
        .route("/api/members/{id}/role", put(handlers::update_member_role))
        .route("/api/members/{id}", delete(handlers::delete_member))
        .route("/api/me", get(handlers::get_profile))
        .route("/api/me", put(handlers::update_profile))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("0.0.0.0:{}", port);
    println!("SLsec 论坛服务启动: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn init_db(pool: &sqlx::MySqlPool) {
    let sql = include_str!("../sql/init.sql");
    for statement in sql.split(';') {
        let s = statement.trim();
        if !s.is_empty() {
            sqlx::query(s).execute(pool).await.ok();
        }
    }
    println!("数据库初始化完成");
}
