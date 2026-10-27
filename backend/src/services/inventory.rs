//! Bill-of-materials inventory. Shared parts (`bom_parts`, one row per SKU) are the single source of
//! truth for part stock; the stock of every physical variant with a BOM is derived from them as the
//! number of units that can be built.

use crate::models::StoreSettings;
use sqlx::{PgConnection, PgPool, Row};
use tracing::error;
use uuid::Uuid;

/// Links every physical BOM line to its shared part (creating missing ones by SKU). Idempotent.
pub const LINK_SHARED_PARTS_SQL: &str = include_str!("../../migrations/0020_link_shared_parts.sql");

pub async fn link_shared_parts(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::Executor::execute(conn, LINK_SHARED_PARTS_SQL).await?;
    Ok(())
}

/// Recomputes the buildable stock of every variant that has physical BOM parts and notifies
/// back-in-stock subscribers of variants that became available again.
pub async fn recalculate_bom_stock(pool: &PgPool) -> Result<(), sqlx::Error> {
    let changed = sqlx::query(
        r#"
        WITH buildable AS (
            SELECT pv.id AS variant_id, MIN(GREATEST(bp.stock_quantity, 0) / pp.quantity) AS units
            FROM product_variants pv
            JOIN products p ON p.id = pv.product_id AND p.product_type <> 'digital'
            JOIN product_parts pp ON pp.variant_id = pv.id OR (pp.product_id = pv.product_id AND pp.variant_id IS NULL)
            JOIN bom_parts bp ON bp.id = pp.part_id
            WHERE pp.quantity > 0 AND COALESCE(pp.part_sku, '') <> 'DIGITAL_FILE'
            GROUP BY pv.id
        )
        UPDATE product_variants pv
        SET stock_quantity = b.units, updated_at = NOW()
        FROM buildable b, product_variants prev
        WHERE pv.id = b.variant_id AND prev.id = pv.id AND pv.stock_quantity IS DISTINCT FROM b.units
        RETURNING pv.id, prev.stock_quantity AS old_stock, pv.stock_quantity AS new_stock
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in changed {
        if row.get::<i32, _>("old_stock") <= 0 && row.get::<i32, _>("new_stock") > 0 {
            notify_back_in_stock(pool, row.get("id"));
        }
    }
    Ok(())
}

/// Emails everyone waiting for this variant (or its product) and clears the waitlist.
pub fn notify_back_in_stock(pool: &PgPool, variant_id: Uuid) {
    let pool = pool.clone();
    tokio::spawn(async move {
        let product: Option<(Uuid, String, String)> = sqlx::query_as(
            "SELECT p.id, p.title, p.slug FROM product_variants v JOIN products p ON p.id = v.product_id WHERE v.id = $1",
        )
        .bind(variant_id)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);
        let Some((product_id, title, slug)) = product else { return };

        let emails: Vec<String> = sqlx::query_scalar("SELECT email FROM stock_notifications WHERE product_id = $1 OR variant_id = $2")
            .bind(product_id)
            .bind(variant_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
        if emails.is_empty() {
            return;
        }

        match sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(&pool).await {
            Ok(settings) => {
                let url = format!("{}/products/{}", crate::services::email::shop_public_url(), slug);
                for email in emails {
                    let _ = crate::services::email::send_back_in_stock_email(&settings, &email, &title, &url).await;
                }
            }
            Err(e) => {
                error!("Back-in-stock emails skipped, settings unavailable: {}", e);
                return;
            }
        }

        let _ = sqlx::query("DELETE FROM stock_notifications WHERE product_id = $1 OR variant_id = $2")
            .bind(product_id)
            .bind(variant_id)
            .execute(&pool)
            .await;
    });
}

/// Takes (`sign` = -1) or returns (`sign` = 1) the parts used by `units` of a variant.
pub async fn adjust_parts_for_variant(
    conn: &mut PgConnection,
    product_id: Uuid,
    variant_id: Uuid,
    units: i32,
    sign: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE bom_parts bp
        SET stock_quantity = GREATEST(0, bp.stock_quantity + $1 * x.qty)::int
        FROM (
            SELECT pp.part_id, SUM(pp.quantity)::int AS qty
            FROM product_parts pp
            WHERE (pp.variant_id = $2 OR (pp.product_id = $3 AND pp.variant_id IS NULL))
              AND pp.part_id IS NOT NULL AND COALESCE(pp.part_sku, '') <> 'DIGITAL_FILE'
            GROUP BY pp.part_id
        ) x
        WHERE bp.id = x.part_id
        "#,
    )
    .bind(sign * units)
    .bind(variant_id)
    .bind(product_id)
    .execute(conn)
    .await?;
    Ok(())
}
