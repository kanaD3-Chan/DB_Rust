use sqlx::mysql::MySqlPool;
use tokio::sync::broadcast;

use crate::models::Notification;

pub struct AppState {
    pub db: MySqlPool,
    pub jwt_secret: String,
    pub notify_tx: broadcast::Sender<Notification>,
}

pub async fn create_pool() -> MySqlPool {
    let base_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:mysecretpassword@localhost:3306/slsec_forum".into());

    // 先连接不含数据库名的 URL，创建数据库
    let (root_url, db_name) = split_db(&base_url);
    let root_pool = MySqlPool::connect(&root_url)
        .await
        .expect("无法连接 MySQL，请确认已启动");

    sqlx::query(&format!(
        "CREATE DATABASE IF NOT EXISTS `{}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
        db_name
    ))
    .execute(&root_pool)
    .await
    .expect("创建数据库失败");

    root_pool.close().await;

    MySqlPool::connect(&base_url)
        .await
        .expect("无法连接数据库")
}

fn split_db(url: &str) -> (String, String) {
    let (base, db) = url.rsplit_once('/').unwrap_or((url, "slsec_forum"));
    let base = base.trim_end_matches('/');
    let db = db.split('?').next().unwrap_or(db);
    (format!("{}/mysql", base), db.to_string())
}
