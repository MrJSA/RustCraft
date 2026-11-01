use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tracing::info;

/// Schema migrations in the order they are applied; each one is recorded in `_schema_migrations`.
pub const MIGRATIONS: &[(&str, &str)] = &[
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
    ("0012_stock_template_and_english_tax", include_str!("../migrations/0012_stock_template_and_english_tax.sql")),
    ("0013_coupons_and_admin_users", include_str!("../migrations/0013_coupons_and_admin_users.sql")),
    ("0014_payment_gateway_overhaul", include_str!("../migrations/0014_payment_gateway_overhaul.sql")),
    ("0015_security_hardening", include_str!("../migrations/0015_security_hardening.sql")),
    ("0016_admin_permissions", include_str!("../migrations/0016_admin_permissions.sql")),
    ("0017_bom_parts_stock", include_str!("../migrations/0017_bom_parts_stock.sql")),
    ("0018_centralized_bom_parts", include_str!("../migrations/0018_centralized_bom_parts.sql")),
    ("0019_part_stock_sync_and_low_stock_alerts", include_str!("../migrations/0019_part_stock_sync_and_low_stock_alerts.sql")),
    ("0020_link_shared_parts", include_str!("../migrations/0020_link_shared_parts.sql")),
    ("0021_structured_address_and_legal_placeholders", include_str!("../migrations/0021_structured_address_and_legal_placeholders.sql")),
    ("0022_cloud_backups", include_str!("../migrations/0022_cloud_backups.sql")),
    ("0023_test_mode_orders", include_str!("../migrations/0023_test_mode_orders.sql")),
    ("0024_invoice_pdf_and_einvoicing", include_str!("../migrations/0024_invoice_pdf_and_einvoicing.sql")),
    ("0025_reviews_newsletter_sales_downloads_b2b_legal", include_str!("../migrations/0025_reviews_newsletter_sales_downloads_b2b_legal.sql")),
    ("0026_package_sizes", include_str!("../migrations/0026_package_sizes.sql")),
    ("0027_order_notifications", include_str!("../migrations/0027_order_notifications.sql")),
    ("0028_withdrawal_texts_used_and_digital_goods", include_str!("../migrations/0028_withdrawal_texts_used_and_digital_goods.sql")),
];

/// Passwords published in this repository (compose fallback and `.env.example`).
const PUBLISHED_DB_PASSWORDS: &[&str] = &["local_dev_password", "change-me-to-a-long-random-password"];

/// True when the password in a `postgres://user:password@host/db` URL is a published default or shorter than 16 characters.
pub fn weak_database_password(database_url: &str) -> bool {
    let Some((_, rest)) = database_url.split_once("://") else { return false };
    let Some((credentials, _)) = rest.rsplit_once('@') else { return true };
    let password = credentials.split_once(':').map(|(_, p)| p).unwrap_or("");
    password.len() < 16 || PUBLISHED_DB_PASSWORDS.contains(&password)
}

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

    let migration_files = MIGRATIONS;

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

    // Apply any unapplied migrations in order.
    // Each migration runs in its own transaction together with its `_schema_migrations` record: a failing
    // migration leaves the database exactly as it was (no half-applied schema), and a failed lookup aborts
    // the start instead of re-running a migration that was already applied.
    for (version, sql) in migration_files {
        let is_applied: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM _schema_migrations WHERE version = $1)"
        )
        .bind(version)
        .fetch_one(&pool)
        .await?;

        if !is_applied {
            info!("Applying migration {}...", version);
            let mut tx = pool.begin().await?;
            // Only one backend may migrate at a time (e.g. an old and a new container during an update)
            sqlx::query("SELECT pg_advisory_xact_lock(72_270_001)").execute(&mut *tx).await?;
            let raced: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM _schema_migrations WHERE version = $1)")
                .bind(version)
                .fetch_one(&mut *tx)
                .await?;
            if raced {
                tx.rollback().await?;
                continue;
            }
            if let Err(e) = sqlx::raw_sql(sql).execute(&mut *tx).await {
                tracing::error!("Migration {} failed and was rolled back, the database is unchanged: {}", version, e);
                return Err(e);
            }
            sqlx::query("INSERT INTO _schema_migrations (version) VALUES ($1)")
                .bind(version)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
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
        // Never ship a publicly known password: use ADMIN_INITIAL_PASSWORD or generate a random one.
        let (password, generated) = match std::env::var("ADMIN_INITIAL_PASSWORD") {
            Ok(p) if p.len() >= 12 => (p, false),
            _ => (crate::services::auth::random_token(10), true),
        };
        if let Ok(hash) = bcrypt::hash(&password, 12) {
            let _ = sqlx::query(
                "INSERT INTO admin_users (username, password_hash, is_default, role) VALUES ($1, $2, TRUE, 'superadmin') ON CONFLICT (username) DO NOTHING"
            )
            .bind("admin")
            .bind(hash)
            .execute(&pool)
            .await;
            if generated {
                tracing::warn!(
                    "Initial admin account created: username='admin' password='{}' — you must change it on first login. (Set ADMIN_INITIAL_PASSWORD to choose it yourself.)",
                    password
                );
            } else {
                info!("Initial admin account created from ADMIN_INITIAL_PASSWORD: username='admin'");
            }
        }
    }

    // Older versions seeded a password that is published in this repository's history.
    // Any account still using it gets a fresh random password (shown once in the log).
    const PUBLISHED_DEFAULT_PASSWORD: &str = "RustCraftAdmin2026!";
    let default_accounts: Vec<(uuid::Uuid, String, String)> =
        sqlx::query_as("SELECT id, username, password_hash FROM admin_users WHERE is_default = TRUE")
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
    for (id, username, hash) in default_accounts {
        if bcrypt::verify(PUBLISHED_DEFAULT_PASSWORD, &hash).unwrap_or(false) {
            let password = crate::services::auth::random_token(10);
            if let Ok(new_hash) = bcrypt::hash(&password, 12) {
                let _ = sqlx::query("UPDATE admin_users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
                    .bind(new_hash)
                    .bind(id)
                    .execute(&pool)
                    .await;
                tracing::warn!(
                    "Admin '{}' still used the published default password. It was replaced by '{}' — log in and choose your own password.",
                    username, password
                );
            }
        }
    }

    info!("Schema migrations check complete!");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_weak_database_passwords() {
        assert!(weak_database_password("postgres://shop_user:local_dev_password@db:5432/shop_db"));
        assert!(weak_database_password("postgres://shop_user:change-me-to-a-long-random-password@db:5432/shop_db"));
        assert!(weak_database_password("postgres://shop_user:short@db:5432/shop_db"));
        assert!(weak_database_password("postgres://shop_user@db:5432/shop_db"));
        assert!(!weak_database_password("postgres://shop_user:3f9a1c0e8b7d6a5f4e3d2c1b0a9f8e7d6c5b4a39@db:5432/shop_db"));
    }
}
