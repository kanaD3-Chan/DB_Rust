use sqlx::MySqlPool;

use crate::models::{User, UserPublic};

pub struct UserRepo<'a>(pub &'a MySqlPool);

impl<'a> UserRepo<'a> {
    pub async fn count_active(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(self.0)
            .await
            .unwrap_or(1)
    }

    pub async fn username_taken(&self, username: &str) -> bool {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = ? AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_one(self.0)
        .await
        .unwrap_or(1)
            > 0
    }

    pub async fn create(
        &self,
        username: &str,
        nickname: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<u32, sqlx::Error> {
        let res = sqlx::query(
            "INSERT INTO users (username, nickname, password_hash, role) VALUES (?, ?, ?, ?)",
        )
        .bind(username)
        .bind(nickname)
        .bind(password_hash)
        .bind(role)
        .execute(self.0)
        .await?;
        Ok(res.last_insert_id() as u32)
    }

    pub async fn find_by_username(&self, username: &str) -> Option<User> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ? AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(self.0)
        .await
        .ok()
        .flatten()
    }

    pub async fn find_by_id(&self, id: u32) -> Option<User> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(self.0)
        .await
        .ok()
        .flatten()
    }

    pub async fn list_all(&self) -> Vec<UserPublic> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC",
        )
        .fetch_all(self.0)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|u| u.into())
        .collect()
    }

    pub async fn update_role(&self, id: u32, role: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET role = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(role)
            .bind(id)
            .execute(self.0)
            .await?;
        Ok(())
    }

    pub async fn soft_delete(&self, id: u32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = ?")
            .bind(id)
            .execute(self.0)
            .await?;
        Ok(())
    }

    pub async fn update_nickname(&self, id: u32, nickname: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET nickname = ? WHERE id = ?")
            .bind(nickname)
            .bind(id)
            .execute(self.0)
            .await?;
        Ok(())
    }

    pub async fn update_password(&self, id: u32, password_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(password_hash)
            .bind(id)
            .execute(self.0)
            .await?;
        Ok(())
    }
}
