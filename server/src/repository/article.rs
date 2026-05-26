use chrono::NaiveDateTime;
use sqlx::{FromRow, MySqlPool};

use crate::models::{Article, ArticleWithAuthor};

#[derive(FromRow)]
#[allow(dead_code)]
pub struct ArticleJoinRow {
    pub id: u32,
    pub author_id: u32,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub category: String,
    pub pinned: i8,
    pub view_count: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub author_name: String,
    pub author_nickname: String,
    pub comment_count: i64,
}

impl ArticleJoinRow {
    pub fn into_model(self) -> ArticleWithAuthor {
        ArticleWithAuthor {
            id: self.id,
            author_id: self.author_id,
            author_name: self.author_name,
            author_nickname: self.author_nickname,
            title: self.title,
            content: self.content,
            summary: self.summary,
            category: self.category,
            pinned: self.pinned != 0,
            view_count: self.view_count,
            comment_count: self.comment_count as u32,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

const JOIN_SELECT: &str =
    "SELECT a.*, u.username AS author_name, u.nickname AS author_nickname, \
     (SELECT COUNT(*) FROM comments c WHERE c.article_id = a.id AND c.deleted_at IS NULL) AS comment_count \
     FROM articles a JOIN users u ON u.id = a.author_id \
     WHERE a.deleted_at IS NULL";

pub struct ArticleRepo<'a>(pub &'a MySqlPool);

impl<'a> ArticleRepo<'a> {
    pub async fn count(&self, category: Option<&str>, search: Option<&str>) -> i64 {
        match (category, search) {
            (Some(cat), None) => sqlx::query_scalar(
                "SELECT COUNT(*) FROM articles WHERE category = ? AND deleted_at IS NULL",
            )
            .bind(cat)
            .fetch_one(self.0)
            .await
            .unwrap_or(0),

            (None, Some(s)) => {
                let pat = format!("%{}%", s);
                sqlx::query_scalar(
                    "SELECT COUNT(*) FROM articles WHERE deleted_at IS NULL \
                     AND (title LIKE ? OR summary LIKE ?)",
                )
                .bind(&pat)
                .bind(&pat)
                .fetch_one(self.0)
                .await
                .unwrap_or(0)
            }

            (Some(cat), Some(s)) => {
                let pat = format!("%{}%", s);
                sqlx::query_scalar(
                    "SELECT COUNT(*) FROM articles WHERE category = ? AND deleted_at IS NULL \
                     AND (title LIKE ? OR summary LIKE ?)",
                )
                .bind(cat)
                .bind(&pat)
                .bind(&pat)
                .fetch_one(self.0)
                .await
                .unwrap_or(0)
            }

            (None, None) => {
                sqlx::query_scalar("SELECT COUNT(*) FROM articles WHERE deleted_at IS NULL")
                    .fetch_one(self.0)
                    .await
                    .unwrap_or(0)
            }
        }
    }

    pub async fn list(
        &self,
        category: Option<&str>,
        search: Option<&str>,
        page_size: u32,
        offset: u32,
    ) -> Vec<ArticleWithAuthor> {
        let rows: Vec<ArticleJoinRow> = match (category, search) {
            (Some(cat), None) => sqlx::query_as(&format!(
                "{} AND a.category = ? ORDER BY a.pinned DESC, a.created_at DESC LIMIT ? OFFSET ?",
                JOIN_SELECT
            ))
            .bind(cat)
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.0)
            .await
            .unwrap_or_default(),

            (None, Some(s)) => {
                let pat = format!("%{}%", s);
                sqlx::query_as(&format!(
                    "{} AND (a.title LIKE ? OR a.summary LIKE ?) \
                     ORDER BY a.pinned DESC, a.created_at DESC LIMIT ? OFFSET ?",
                    JOIN_SELECT
                ))
                .bind(&pat)
                .bind(&pat)
                .bind(page_size)
                .bind(offset)
                .fetch_all(self.0)
                .await
                .unwrap_or_default()
            }

            (Some(cat), Some(s)) => {
                let pat = format!("%{}%", s);
                sqlx::query_as(&format!(
                    "{} AND a.category = ? AND (a.title LIKE ? OR a.summary LIKE ?) \
                     ORDER BY a.pinned DESC, a.created_at DESC LIMIT ? OFFSET ?",
                    JOIN_SELECT
                ))
                .bind(cat)
                .bind(&pat)
                .bind(&pat)
                .bind(page_size)
                .bind(offset)
                .fetch_all(self.0)
                .await
                .unwrap_or_default()
            }

            (None, None) => sqlx::query_as(&format!(
                "{} ORDER BY a.pinned DESC, a.created_at DESC LIMIT ? OFFSET ?",
                JOIN_SELECT
            ))
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.0)
            .await
            .unwrap_or_default(),
        };

        rows.into_iter().map(|r| r.into_model()).collect()
    }

    pub async fn find_by_id(&self, id: u32) -> Option<ArticleWithAuthor> {
        sqlx::query_as::<_, ArticleJoinRow>(&format!(
            "{} AND a.id = ?",
            JOIN_SELECT
        ))
        .bind(id)
        .fetch_optional(self.0)
        .await
        .ok()
        .flatten()
        .map(|r| r.into_model())
    }

    pub async fn find_raw(&self, id: u32) -> Option<Article> {
        sqlx::query_as::<_, Article>(
            "SELECT * FROM articles WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(self.0)
        .await
        .ok()
        .flatten()
    }

    pub async fn increment_view(&self, id: u32) {
        let _ = sqlx::query("UPDATE articles SET view_count = view_count + 1 WHERE id = ?")
            .bind(id)
            .execute(self.0)
            .await;
    }

    pub async fn create(
        &self,
        author_id: u32,
        title: &str,
        content: &str,
        summary: &str,
        category: &str,
    ) -> Result<u32, sqlx::Error> {
        let res = sqlx::query(
            "INSERT INTO articles (author_id, title, content, summary, category) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(author_id)
        .bind(title)
        .bind(content)
        .bind(summary)
        .bind(category)
        .execute(self.0)
        .await?;
        Ok(res.last_insert_id() as u32)
    }

    pub async fn update(
        &self,
        id: u32,
        title: &str,
        content: &str,
        summary: &str,
        category: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE articles SET title=?, content=?, summary=?, category=? WHERE id=?",
        )
        .bind(title)
        .bind(content)
        .bind(summary)
        .bind(category)
        .bind(id)
        .execute(self.0)
        .await?;
        Ok(())
    }

    pub async fn soft_delete(&self, id: u32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE articles SET deleted_at = NOW() WHERE id = ?")
            .bind(id)
            .execute(self.0)
            .await?;
        Ok(())
    }

    pub async fn fetch_created(&self, id: u32) -> Result<Article, sqlx::Error> {
        sqlx::query_as::<_, Article>("SELECT * FROM articles WHERE id = ?")
            .bind(id)
            .fetch_one(self.0)
            .await
    }
}
