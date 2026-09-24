use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tracing::info;

pub async fn init_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Connecting to PostgreSQL database...");

    let pool = PgPoolOptions::new()
        .max_connections(25)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    info!("Database connection established. Running schema migrations...");

    // Execute initial schema and seed directly to ensure tables are always ready
    let migration_sql_1 = include_str!("../migrations/0001_initial_schema.sql");
    sqlx::raw_sql(migration_sql_1).execute(&pool).await?;

    let migration_sql_2 = include_str!("../migrations/0002_enhanced_features.sql");
    sqlx::raw_sql(migration_sql_2).execute(&pool).await?;

    let migration_sql_3 = include_str!("../migrations/0003_extended_features.sql");
    sqlx::raw_sql(migration_sql_3).execute(&pool).await?;

    let migration_sql_4 = include_str!("../migrations/0004_media_email_auth_extended.sql");
    sqlx::raw_sql(migration_sql_4).execute(&pool).await?;

    // Seed default admin user if none exists
    let admin_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    if admin_count == 0 {
        if let Ok(hash) = bcrypt::hash("RustCraftAdmin2026!", 10) {
            let _ = sqlx::query(
                "INSERT INTO admin_users (username, password_hash, is_default) VALUES ($1, $2, TRUE) ON CONFLICT (username) DO NOTHING"
            )
            .bind("admin")
            .bind(hash)
            .execute(&pool)
            .await;
            info!("Default admin user created: username='admin'");
        }
    }

    info!("Schema migrations successfully applied!");

    Ok(pool)
}
