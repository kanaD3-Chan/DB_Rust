use chrono::NaiveDateTime;
use sqlx::{FromRow, MySqlPool};

use crate::models::{Comment, CommentWithAuthor};

#[derive(FromRow)]
pub struct CommentJoinRow {
    pub id: u32,
    pub article_id: u32,
    pub author_id: u32,
    pub parent_id: Option<u32>,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub author_name: String,
    pub author_nickname: String,
}

impl CommentJoinRow {
    pub fn into_model(self) -> CommentWithAuthor {
        CommentWithAuthor {
            id: self.id,
            article_id: self.article_id,
            author_id: self.author_id,
            author_name: self.author_name,
            author_nickname: self.author_nickname,
            parent_id: self.parent_id,
            content: self.content,
            created_at: self.created_at,
        }
    }
}

pub struct CommentRepo<'a>(pub &'a MySqlPool);

impl<'a> CommentRepo<'a> {
    pub async fn list_for_article(&self, article_id: u32) -> Vec<CommentWithAuthor> {
        sqlx::query_as::<_, CommentJoinRow>(
            "SELECT c.id, c.article_id, c.author_id, c.parent_id, c.content, c.created_at, \
             u.username AS author_name, u.nickname AS author_nickname \
             FROM comments c JOIN users u ON u.id = c.author_id \
             WHERE c.article_id = ? AND c.deleted_at IS NULL \
             ORDER BY c.created_at ASC",
        )
        .bind(article_id)
        .fetch_all(self.0)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.into_model())
        .collect()
    }

    pub async fn article_exists(&self, article_id: u32) -> bool {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM articles WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(article_id)
        .fetch_one(self.0)
        .await
        .unwrap_or(0)
            > 0
    }

    pub async fn parent_exists(&self, parent_id: u32, article_id: u32) -> bool {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM comments WHERE id = ? AND article_id = ? AND deleted_at IS NULL",
        )
        .bind(parent_id)
        .bind(article_id)
        .fetch_one(self.0)
        .await
        .unwrap_or(0)
            > 0
    }

    pub async fn create(
        &self,
        article_id: u32,
        author_id: u32,
        parent_id: Option<u32>,
        content: &str,
    ) -> Result<u32, sqlx::Error> {
        let res = sqlx::query(
            "INSERT INTO comments (article_id, author_id, parent_id, content) VALUES (?, ?, ?, ?)",
        )
        .bind(article_id)
        .bind(author_id)
        .bind(parent_id)
        .bind(content)
        .execute(self.0)
        .await?;
        Ok(res.last_insert_id() as u32)
    }

    pub async fn fetch_created(&self, id: u32) -> Result<Comment, sqlx::Error> {
        sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = ?")
            .bind(id)
            .fetch_one(self.0)
            .await
    }

    pub async fn get_article_title(&self, article_id: u32) -> Option<String> {
        sqlx::query_scalar("SELECT title FROM articles WHERE id = ?")
            .bind(article_id)
            .fetch_optional(self.0)
            .await
            .unwrap_or(None)
    }
}
