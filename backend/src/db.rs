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

    info!("Database connection established. Checking schema migrations...");

    // Create migrations tracking table if not exists
    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS _schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#
    )
    .execute(&pool)
    .await?;

    // Check if store_settings already exists in DB (meaning previously initialized DB without tracking)
    let has_existing_schema: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'store_settings')"
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    let migration_files: &[(&str, &str)] = &[
        ("0001_initial_schema", include_str!("../migrations/0001_initial_schema.sql")),
        ("0002_enhanced_features", include_str!("../migrations/0002_enhanced_features.sql")),
        ("0003_extended_features", include_str!("../migrations/0003_extended_features.sql")),
        ("0004_media_email_auth_extended", include_str!("../migrations/0004_media_email_auth_extended.sql")),
        ("0005_cookie_gallery_seo_analytics", include_str!("../migrations/0005_cookie_gallery_seo_analytics.sql")),
        ("0006_menu_dropdown_single_variant_slip", include_str!("../migrations/0006_menu_dropdown_single_variant_slip.sql")),
        ("0007_tax_notice_and_migrations_table", include_str!("../migrations/0007_tax_notice_and_migrations_table.sql")),
        ("0008_legal_identity_fields", include_str!("../migrations/0008_legal_identity_fields.sql")),
        ("0009_footer_and_system_modes", include_str!("../migrations/0009_footer_and_system_modes.sql")),
        ("0010_tax_modes_and_product_vat", include_str!("../migrations/0010_tax_modes_and_product_vat.sql")),
        ("0011_custom_order_numbers", include_str!("../migrations/0011_custom_order_numbers.sql")),
    ];

    // If schema already existed prior to migration tracking, mark initial migrations 0001..0006 as applied if not tracked
    if has_existing_schema {
        for (version, _) in &migration_files[0..6] {
            let _ = sqlx::query(
                "INSERT INTO _schema_migrations (version) VALUES ($1) ON CONFLICT (version) DO NOTHING"
            )
            .bind(version)
            .execute(&pool)
            .await;
        }
    }

    // Apply any unapplied migrations in order
    for (version, sql) in migration_files {
        let is_applied: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM _schema_migrations WHERE version = $1)"
        )
        .bind(version)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        if !is_applied {
            info!("Applying migration {}...", version);
            sqlx::raw_sql(sql).execute(&pool).await?;
            sqlx::query("INSERT INTO _schema_migrations (version) VALUES ($1)")
                .bind(version)
                .execute(&pool)
                .await?;
            info!("Migration {} applied successfully!", version);
        } else {
            tracing::debug!("Migration {} already applied, skipping.", version);
        }
    }

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

    info!("Schema migrations check complete!");

    Ok(pool)
}
