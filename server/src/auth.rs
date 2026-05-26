use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::{Json, Response},
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::AppState;
use crate::models::ApiResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: u32,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: u32,
    pub username: String,
    pub role: String,
}

pub fn create_token(user_id: u32, username: &str, role: &str, secret: &str) -> String {
    let exp = (Utc::now().naive_utc() + chrono::Duration::hours(72))
        .and_utc()
        .timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        role: role.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("JWT encode failed")
}

// 鉴权中间件: 从 Authorization header 解析 JWT，注入 CurrentUser 到 extensions
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: axum::http::Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<()>>)> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if let Some(token) = token {
        if let Some(claims) = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .ok()
        .map(|d| d.claims)
        {
            req.extensions_mut().insert(CurrentUser {
                id: claims.sub,
                username: claims.username,
                role: claims.role,
            });
            return Ok(next.run(req).await);
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(ApiResponse::<()>::err("未登录或令牌已过期")),
    ))
}

// 从 extensions 提取 CurrentUser
impl<S: Send + Sync> FromRequestParts<S> for CurrentUser {
    type Rejection = (StatusCode, Json<ApiResponse<()>>);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<CurrentUser>().cloned().ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::err("未登录")),
            )
        })
    }
}

// 可选鉴权：不强制要求登录
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OptionalUser(pub Option<CurrentUser>);

impl<S: Send + Sync> FromRequestParts<S> for OptionalUser {
    type Rejection = (StatusCode, Json<ApiResponse<()>>);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(OptionalUser(parts.extensions.get::<CurrentUser>().cloned()))
    }
}
