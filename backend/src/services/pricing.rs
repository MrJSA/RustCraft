//! Prices as customers see and pay them.
//!
//! - A sale price applies only when sale prices are switched on in the settings, the product's sale
//!   window is open and the sale price is below the regular price.
//! - A reduction may only be advertised against the lowest price of the 30 days before it
//!   (§ 11 PAngV, CJEU C-330/23). `variant_price_history` records every price a variant was sold at;
//!   without such a reference the sale price is simply shown as the price.
//! - Products sold by weight, volume, length or area show the unit price (§ 4 PAngV).

use crate::models::{Product, ProductVariant, ProductWithVariants, StoreSettings};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct VariantPricing {
    /// Regular price (base price or variant price)
    pub regular_cents: i32,
    /// What the customer pays now
    pub price_cents: i32,
    /// A reduction that may be advertised (price below the 30-day reference)
    pub on_sale: bool,
    /// Lowest price of the 30 days before the reduction (struck through)
    pub reference_cents: Option<i32>,
    pub sale_ends_at: Option<DateTime<Utc>>,
    /// e.g. "35,96 € / 1 kg"
    pub unit_price: Option<String>,
}

/// Regular price of a variant.
pub fn regular_price(product: &Product, variant: &ProductVariant) -> i32 {
    variant.price_override_cents.unwrap_or(product.base_price_cents)
}

/// Whether a sale price applies right now.
pub fn sale_applies(
    enabled: bool,
    regular: i32,
    sale_price: Option<i32>,
    starts: Option<DateTime<Utc>>,
    ends: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<i32> {
    let sale = sale_price?;
    let open = starts.is_none_or(|s| s <= now) && ends.is_none_or(|e| e > now);
    (enabled && open && sale > 0 && sale < regular).then_some(sale)
}

/// Price per 1 kg / 1 l / 1 m / 1 m² (§ 4 PAngV), formatted for display.
pub fn unit_price(price_cents: i32, content: Option<f64>, unit: Option<&str>) -> Option<String> {
    let content = content.filter(|c| *c > 0.0)?;
    let (factor, label) = match unit?.trim() {
        "kg" => (1.0, "1 kg"),
        "g" => (1000.0, "1 kg"),
        "l" => (1.0, "1 l"),
        "ml" => (1000.0, "1 l"),
        "m" => (1.0, "1 m"),
        "cm" => (100.0, "1 m"),
        "m2" | "m²" => (1.0, "1 m²"),
        _ => return None,
    };
    let per_unit = (price_cents as f64 * factor / content).round() as i64;
    Some(format!("{} / {}", crate::services::invoice_pdf::money(per_unit, "€"), label))
}

/// Shipping weight of a variant: its own weight, otherwise the product's.
pub fn weight_grams(product_weight: Option<i32>, variant_weight: Option<i32>) -> i32 {
    variant_weight.or(product_weight).unwrap_or(0).max(0)
}

/// Lowest price of the 30 days before the current price of each variant began (only variants whose
/// recorded current price equals `current`).
async fn lowest_before_current(pool: &PgPool, current: &HashMap<Uuid, i32>) -> HashMap<Uuid, i32> {
    if current.is_empty() {
        return HashMap::new();
    }
    let ids: Vec<Uuid> = current.keys().copied().collect();
    let rows = sqlx::query(
        r#"
        WITH h AS (
            SELECT variant_id, price_cents, valid_from,
                   LEAD(valid_from) OVER (PARTITION BY variant_id ORDER BY valid_from, id) AS valid_to,
                   ROW_NUMBER() OVER (PARTITION BY variant_id ORDER BY valid_from DESC, id DESC) AS rn
            FROM variant_price_history
            WHERE variant_id = ANY($1)
        )
        SELECT cur.variant_id, cur.price_cents,
               (SELECT MIN(h2.price_cents) FROM h h2
                WHERE h2.variant_id = cur.variant_id
                  AND h2.valid_from < cur.valid_from
                  AND COALESCE(h2.valid_to, 'infinity'::timestamptz) > cur.valid_from - INTERVAL '30 days') AS lowest
        FROM h cur
        WHERE cur.rn = 1
        "#,
    )
    .bind(&ids)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .filter_map(|r| {
            let id: Uuid = r.get("variant_id");
            let recorded: i32 = r.get("price_cents");
            let lowest: Option<i32> = r.get("lowest");
            // The history may lag a scheduled sale start by a few minutes; until then no reduction is advertised
            (current.get(&id) == Some(&recorded)).then_some(()).and(lowest.map(|l| (id, l)))
        })
        .collect()
}

/// Fills `pricing` of every variant.
pub async fn attach(pool: &PgPool, settings: &StoreSettings, products: &mut [ProductWithVariants]) {
    let now = Utc::now();
    let mut sale_prices: HashMap<Uuid, i32> = HashMap::new();
    for p in products.iter_mut() {
        let product = &p.product;
        for v in p.variants.iter_mut() {
            let regular = regular_price(product, v);
            let price = sale_applies(settings.sale_prices_enabled, regular, v.sale_price_cents, product.sale_starts_at, product.sale_ends_at, now)
                .unwrap_or(regular);
            if price < regular {
                sale_prices.insert(v.id, price);
            }
            v.pricing = Some(VariantPricing {
                regular_cents: regular,
                price_cents: price,
                on_sale: false,
                reference_cents: None,
                sale_ends_at: if price < regular { product.sale_ends_at } else { None },
                unit_price: unit_price(price, product.unit_content, product.unit_content_unit.as_deref()),
            });
        }
    }
    let references = lowest_before_current(pool, &sale_prices).await;
    for p in products.iter_mut() {
        for v in p.variants.iter_mut() {
            if let (Some(pricing), Some(reference)) = (v.pricing.as_mut(), references.get(&v.id)) {
                if *reference > pricing.price_cents {
                    pricing.on_sale = true;
                    pricing.reference_cents = Some(*reference);
                }
            }
        }
    }
}

/// Records effective prices every few minutes so scheduled sale starts and ends enter the history.
pub fn spawn_price_recorder(pool: PgPool) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            tick.tick().await;
            if let Err(e) = sqlx::query("SELECT record_variant_prices(NULL)").execute(&pool).await {
                tracing::warn!("Recording the price history failed: {}", e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn sale_needs_switch_window_and_lower_price() {
        let now = Utc::now();
        assert_eq!(sale_applies(true, 1000, Some(800), None, None, now), Some(800));
        assert_eq!(sale_applies(false, 1000, Some(800), None, None, now), None);
        assert_eq!(sale_applies(true, 1000, Some(1000), None, None, now), None);
        assert_eq!(sale_applies(true, 1000, Some(800), Some(now + Duration::hours(1)), None, now), None);
        assert_eq!(sale_applies(true, 1000, Some(800), None, Some(now - Duration::hours(1)), now), None);
        assert_eq!(sale_applies(true, 1000, None, None, None, now), None);
    }

    #[test]
    fn unit_prices_per_kg_and_litre() {
        assert_eq!(unit_price(899, Some(250.0), Some("g")).as_deref(), Some("35,96 € / 1 kg"));
        assert_eq!(unit_price(299, Some(0.5), Some("l")).as_deref(), Some("5,98 € / 1 l"));
        assert_eq!(unit_price(1500, Some(2.0), Some("m2")).as_deref(), Some("7,50 € / 1 m²"));
        assert_eq!(unit_price(1500, None, Some("kg")), None);
        assert_eq!(unit_price(1500, Some(1.0), Some("piece")), None);
    }
}
