pub mod admin;
pub mod public;

use axum::Router;
use sqlx::PgPool;

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/v1", public::public_router())
        .nest("/api/v1/admin", admin::admin_router())
        .with_state(pool)
}
