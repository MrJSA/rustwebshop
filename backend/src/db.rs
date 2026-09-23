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
    let migration_sql = include_str!("../migrations/0001_initial_schema.sql");
    sqlx::raw_sql(migration_sql).execute(&pool).await?;

    info!("Schema migration successfully applied!");

    Ok(pool)
}
