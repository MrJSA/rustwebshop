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

    info!("Schema migrations successfully applied!");

    Ok(pool)
}
