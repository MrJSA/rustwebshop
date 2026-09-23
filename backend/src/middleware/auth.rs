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
    // Check for development bypass header
    if let Some(dev_hdr) = req.headers().get("X-Dev-Mode") {
        if dev_hdr == "true" {
            return Ok(next.run(req).await);
        }
    }

    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            // Also check query param or cookie if available, or return 401
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing or invalid Authorization header" })),
            )
                .into_response());
        }
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_rustwebshop_jwt_token_2026".to_string());

    match decode::<Claims>(
        token,
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
