use crate::services::auth::{bearer_token, verify_token, ROLE_ADMIN_TOKEN};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// The authenticated admin, available to handlers via `Extension<CurrentAdmin>`.
#[derive(Debug, Clone)]
pub struct CurrentAdmin {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub is_default: bool,
}

impl CurrentAdmin {
    pub fn is_superadmin(&self) -> bool {
        self.role == "superadmin"
    }
    pub fn can_manage_store(&self) -> bool {
        self.role == "superadmin" || self.role == "admin"
    }
}

fn deny(status: StatusCode, code: &str, message: &str) -> Response {
    (status, Json(json!({ "error": message, "code": code }))).into_response()
}

/// Paths an account that still has its initial password may use.
fn allowed_with_default_password(path: &str) -> bool {
    matches!(path, "/auth/status" | "/auth/me" | "/auth/change-credentials")
}

/// Store-wide configuration & data that editors must not touch.
fn requires_store_manager(method: &Method, path: &str) -> bool {
    path.starts_with("/users")
        || path.starts_with("/settings/payments")
        || path.starts_with("/export/")
        || path.starts_with("/import/")
        || path.starts_with("/settings/email")
        || (path.starts_with("/settings/system") && method != Method::GET)
}

pub async fn admin_auth_middleware(State(pool): State<PgPool>, mut req: Request<Body>, next: Next) -> Result<Response, Response> {
    let claims = bearer_token(req.headers())
        .and_then(|t| verify_token(t, ROLE_ADMIN_TOKEN))
        .ok_or_else(|| deny(StatusCode::UNAUTHORIZED, "unauthenticated", "Please log in"))?;

    let admin_id = Uuid::parse_str(&claims.sub).map_err(|_| deny(StatusCode::UNAUTHORIZED, "unauthenticated", "Please log in again"))?;
    let row = sqlx::query("SELECT id, username, role, is_default FROM admin_users WHERE id = $1")
        .bind(admin_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| deny(StatusCode::SERVICE_UNAVAILABLE, "unavailable", "Database unavailable"))?
        .ok_or_else(|| deny(StatusCode::UNAUTHORIZED, "unauthenticated", "This admin account no longer exists"))?;

    let admin = CurrentAdmin {
        id: row.get("id"),
        username: row.get("username"),
        role: row.get("role"),
        is_default: row.get("is_default"),
    };

    // Nested routers see the path without the /api/v1/admin prefix
    let path = req.uri().path().trim_start_matches("/api/v1/admin").to_string();

    if admin.is_default && !allowed_with_default_password(&path) {
        return Err(deny(StatusCode::FORBIDDEN, "password_change_required", "Please change the initial password before using the admin area"));
    }
    if requires_store_manager(req.method(), &path) && !admin.can_manage_store() {
        return Err(deny(StatusCode::FORBIDDEN, "forbidden", "Your role does not allow this action"));
    }

    req.extensions_mut().insert(admin);
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editors_cannot_reach_store_configuration() {
        assert!(requires_store_manager(&Method::GET, "/users"));
        assert!(requires_store_manager(&Method::PUT, "/settings/payments/stripe"));
        assert!(requires_store_manager(&Method::GET, "/export/store-data"));
        assert!(requires_store_manager(&Method::POST, "/import/store-data"));
        assert!(requires_store_manager(&Method::PUT, "/settings/system"));
        assert!(!requires_store_manager(&Method::GET, "/settings/system"));
        assert!(!requires_store_manager(&Method::GET, "/orders"));
        assert!(!requires_store_manager(&Method::POST, "/products"));
    }

    #[test]
    fn default_password_accounts_are_limited() {
        assert!(allowed_with_default_password("/auth/change-credentials"));
        assert!(!allowed_with_default_password("/orders"));
    }
}
