use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const BASE_URL: &str = "http://172.16.173.140:3000";

// ── 类型定义（与 server 共享） ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: u32,
    pub username: String,
    pub nickname: String,
    pub role: String,
    pub avatar: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentWithAuthor {
    pub id: u32,
    pub article_id: u32,
    pub author_id: u32,
    pub author_name: String,
    pub author_nickname: String,
    pub parent_id: Option<u32>,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleListData {
    pub articles: Vec<ArticleWithAuthor>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub token: String,
    pub user: UserPublic,
}

pub struct Api {
    client: Client,
    token: Option<String>,
}

impl Api {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token.as_deref().unwrap_or(""))
    }

    pub async fn register(
        &self,
        username: &str,
        nickname: &str,
        password: &str,
    ) -> Result<AuthData, String> {
        let resp = self
            .client
            .post(format!("{}/api/register", BASE_URL))
            .json(&serde_json::json!({
                "username": username,
                "nickname": nickname,
                "password": password,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: ApiResponse<AuthData> = resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(body.data.unwrap())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<AuthData, String> {
        let resp = self
            .client
            .post(format!("{}/api/login", BASE_URL))
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: ApiResponse<AuthData> = resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(body.data.unwrap())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn list_articles(
        &self,
        page: u32,
        page_size: u32,
        category: Option<&str>,
        search: Option<&str>,
    ) -> Result<ArticleListData, String> {
        let mut url = format!(
            "{}/api/articles?page={}&page_size={}",
            BASE_URL, page, page_size
        );
        if let Some(cat) = category {
            url.push_str(&format!("&category={}", cat));
        }
        if let Some(s) = search {
            url.push_str(&format!("&search={}", s.replace(' ', "+")));
        }
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let body: ApiResponse<ArticleListData> = resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(body.data.unwrap())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn get_article(&self, id: u32) -> Result<ArticleWithAuthor, String> {
        let resp = self
            .client
            .get(format!("{}/api/articles/{}", BASE_URL, id))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<ArticleWithAuthor> = resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(body.data.unwrap())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn create_article(
        &self,
        title: &str,
        content: &str,
        category: &str,
    ) -> Result<(), String> {
        let resp = self
            .client
            .post(format!("{}/api/articles", BASE_URL))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({
                "title": title,
                "content": content,
                "category": category,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<serde_json::Value> =
            resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn update_article(
        &self,
        id: u32,
        title: &str,
        content: &str,
        category: &str,
    ) -> Result<(), String> {
        let resp = self
            .client
            .put(format!("{}/api/articles/{}", BASE_URL, id))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({
                "title": title,
                "content": content,
                "category": category,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<serde_json::Value> =
            resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn delete_article(&self, id: u32) -> Result<(), String> {
        let resp = self
            .client
            .delete(format!("{}/api/articles/{}", BASE_URL, id))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<serde_json::Value> =
            resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn list_comments(&self, article_id: u32) -> Result<Vec<CommentWithAuthor>, String> {
        let resp = self
            .client
            .get(format!("{}/api/articles/{}/comments", BASE_URL, article_id))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<Vec<CommentWithAuthor>> =
            resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(body.data.unwrap_or_default())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn create_comment(
        &self,
        article_id: u32,
        parent_id: Option<u32>,
        content: &str,
    ) -> Result<(), String> {
        let resp = self
            .client
            .post(format!("{}/api/articles/{}/comments", BASE_URL, article_id))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({
                "parent_id": parent_id,
                "content": content,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<serde_json::Value> =
            resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn list_members(&self) -> Result<Vec<UserPublic>, String> {
        let resp = self
            .client
            .get(format!("{}/api/members", BASE_URL))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<Vec<UserPublic>> = resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(body.data.unwrap_or_default())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn update_member_role(&self, id: u32, role: &str) -> Result<(), String> {
        let resp = self
            .client
            .put(format!("{}/api/members/{}/role", BASE_URL, id))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "role": role }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<serde_json::Value> =
            resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn delete_member(&self, id: u32) -> Result<(), String> {
        let resp = self
            .client
            .delete(format!("{}/api/members/{}", BASE_URL, id))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<serde_json::Value> =
            resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn get_profile(&self) -> Result<UserPublic, String> {
        let resp = self
            .client
            .get(format!("{}/api/me", BASE_URL))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<UserPublic> = resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(body.data.unwrap())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn update_profile(
        &self,
        nickname: Option<&str>,
        password: Option<&str>,
    ) -> Result<(), String> {
        let mut json = serde_json::json!({});
        if let Some(n) = nickname {
            json["nickname"] = serde_json::Value::String(n.to_string());
        }
        if let Some(p) = password {
            json["password"] = serde_json::Value::String(p.to_string());
        }
        let resp = self
            .client
            .put(format!("{}/api/me", BASE_URL))
            .header("Authorization", self.auth_header())
            .json(&json)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<serde_json::Value> =
            resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }

    pub async fn list_categories(&self) -> Result<Vec<Category>, String> {
        let resp = self
            .client
            .get(format!("{}/api/categories", BASE_URL))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiResponse<Vec<Category>> = resp.json().await.map_err(|e| e.to_string())?;
        if body.success {
            Ok(body.data.unwrap_or_default())
        } else {
            Err(body.error.unwrap_or_default())
        }
    }
}
