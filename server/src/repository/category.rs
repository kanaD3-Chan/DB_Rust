use sqlx::MySqlPool;

use crate::models::Category;

pub struct CategoryRepo<'a>(pub &'a MySqlPool);

impl<'a> CategoryRepo<'a> {
    pub async fn list(&self) -> Vec<Category> {
        sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY sort_order")
            .fetch_all(self.0)
            .await
            .unwrap_or_default()
    }
}
