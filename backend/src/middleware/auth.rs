use crate::models::Claims;
use axum::{
    body::Body,
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;

pub async fn admin_auth_middleware(req: Request<Body>, next: Next) -> Result<Response, Response> {
    // Check for development bypass header or query param
    if let Some(dev_hdr) = req.headers().get("X-Dev-Mode") {
        if dev_hdr == "true" {
            return Ok(next.run(req).await);
        }
    }

    if let Some(query) = req.uri().query() {
        if query.contains("dev=true") {
            return Ok(next.run(req).await);
        }
    }

    // Check Authorization header or query param "token=" or cookie "admin_token="
    let mut token_opt = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(header[7..].to_string())
            } else {
                None
            }
        });

    if token_opt.is_none() {
        if let Some(query) = req.uri().query() {
            for param in query.split('&') {
                if let Some(val) = param.strip_prefix("token=") {
                    token_opt = Some(val.to_string());
                    break;
                }
            }
        }
    }

    if token_opt.is_none() {
        if let Some(cookie) = req.headers().get(axum::http::header::COOKIE).and_then(|c| c.to_str().ok()) {
            for part in cookie.split(';') {
                let part = part.trim();
                if let Some(val) = part.strip_prefix("admin_token=") {
                    token_opt = Some(val.to_string());
                    break;
                }
            }
        }
    }

    let token = match token_opt {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing or invalid Authorization header" })),
            )
                .into_response());
        }
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_rustwebshop_jwt_token_2026".to_string());

    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            if token_data.claims.role != "admin" {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Insufficient permissions" })),
                )
                    .into_response());
            }
            Ok(next.run(req).await)
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid or expired token" })),
        )
            .into_response()),
    }
}
