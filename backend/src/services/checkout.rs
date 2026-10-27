use crate::models::{CheckoutQuote, CheckoutRequest, CheckoutResponse, DigitalDownloadItem};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgConnection, PgPool, Row};
use uuid::Uuid;

pub struct CheckoutService;

struct PricedLine {
    product_id: Uuid,
    variant_id: Uuid,
    product_title: String,
    variant_title: String,
    sku: String,
    unit_price_cents: i32,
    quantity: i32,
    total_price_cents: i32,
    is_digital: bool,
    download_url: Option<String>,
    tax_rate_percent: f64,
}

struct PricedOrder {
    lines: Vec<PricedLine>,
    tax_mode: String,
    digital_downloads: Vec<DigitalDownloadItem>,
    items_subtotal_cents: i32,
    discount_cents: i32,
    subtotal_cents: i32,
    shipping_cost_cents: i32,
    tax_cents: i32,
    total_cents: i32,
    is_digital_only: bool,
    coupon: Option<(Uuid, String)>,
    /// Physical order whose destination zone has rates but none was selected
    missing_shipping_rate: bool,
    total_weight_g: i32,
    /// Rates the parcel fits into for the destination (cheapest first)
    shipping_options: Vec<crate::models::ShippingRate>,
    /// VIES-confirmed VAT ID of a VAT-free EU business order
    reverse_charge: Option<crate::services::vies::VatCheck>,
}

/// VAT and total of an order. Product VAT is reduced pro rata by product discounts; shipping is taxed
/// with the VAT rate of the main delivery (largest share) — included in the price with gross pricing,
/// added on top with net pricing. Small businesses (§ 19 UStG) charge no VAT.
pub fn compute_tax(tax_mode: &str, rate_amounts: &[(f64, i64)], items_subtotal: i32, subtotal: i32, shipping: i32, reverse_charge: bool) -> (i32, i32) {
    let discount_factor = if items_subtotal > 0 { subtotal as f64 / items_subtotal as f64 } else { 0.0 };
    let amounts: Vec<i64> = rate_amounts.iter().map(|(_, a)| *a).collect();
    let shipping_rate = crate::services::invoice::main_rate_index(&amounts).map(|i| rate_amounts[i].0).unwrap_or(0.0) / 100.0;
    let items_vat = |gross: bool| -> f64 {
        rate_amounts
            .iter()
            .map(|(rate, amount)| {
                let r = rate / 100.0;
                let a = *amount as f64 * discount_factor;
                if r <= 0.0 { 0.0 } else if gross { a - a / (1.0 + r) } else { a * r }
            })
            .sum()
    };
    // VAT-free EU business order: the customer pays the net amounts
    if reverse_charge && tax_mode != "kleingewerbe" {
        return match tax_mode {
            "included" => {
                let items_net: f64 = rate_amounts
                    .iter()
                    .map(|(rate, amount)| *amount as f64 * discount_factor / (1.0 + rate / 100.0))
                    .sum();
                (0, (items_net + shipping as f64 / (1.0 + shipping_rate)).round() as i32)
            }
            _ => (0, subtotal + shipping),
        };
    }
    match tax_mode {
        "included" => {
            let shipping_vat = if shipping_rate > 0.0 { shipping as f64 - shipping as f64 / (1.0 + shipping_rate) } else { 0.0 };
            ((items_vat(true) + shipping_vat).round() as i32, subtotal + shipping)
        }
        "excluded" => {
            let vat = (items_vat(false) + shipping as f64 * shipping_rate).round() as i32;
            (vat, subtotal + shipping + vat)
        }
        _ => (0, subtotal + shipping),
    }
}

/// A payment that has been verified with the provider and may be turned into an order.
pub struct VerifiedPayment {
    pub provider: String,
    /// Refundable provider reference (Stripe PaymentIntent id, PayPal capture id).
    pub reference: String,
    pub amount_cents: i32,
    /// 'paid' or 'pending' (e.g. PayPal capture under review)
    pub status: String,
    /// Paid in test mode (Stripe `livemode: false`, PayPal sandbox): kept out of analytics
    pub is_test: bool,
}

pub enum FinalizeError {
    /// The order could not be created although the payment went through — the caller must refund.
    OrderRejected(anyhow::Error),
    /// Unknown reference or a checkout that already failed — nothing to do.
    Invalid(anyhow::Error),
    /// Transient failure (database) — safe to retry, no refund.
    Internal(anyhow::Error),
}

impl std::fmt::Display for FinalizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinalizeError::OrderRejected(e) => write!(f, "{}", e),
            FinalizeError::Invalid(e) => write!(f, "{}", e),
            FinalizeError::Internal(e) => write!(f, "{}", e),
        }
    }
}

pub struct FinalizeOutcome {
    pub response: CheckoutResponse,
    pub newly_created: bool,
}

impl CheckoutService {
    /// Enforces the store's "registered customers only" / "verified email" checkout policy.
    pub async fn enforce_checkout_policy(pool: &PgPool, customer_email: &str) -> Result<()> {
        let Ok(settings_row) = sqlx::query("SELECT require_registered_checkout, require_email_verification FROM store_settings WHERE id = 1")
            .fetch_one(pool)
            .await
        else {
            return Ok(());
        };
        let req_reg: bool = settings_row.get("require_registered_checkout");
        let req_verify: bool = settings_row.get("require_email_verification");
        if !req_reg {
            return Ok(());
        }
        let cust = sqlx::query("SELECT is_verified FROM customers WHERE email = $1")
            .bind(customer_email)
            .fetch_optional(pool)
            .await?;
        match cust {
            None => Err(anyhow!(
                "Purchases are restricted to registered customers. Please log in or create an account to complete checkout."
            )),
            Some(c) if req_verify && !c.get::<bool, _>("is_verified") => Err(anyhow!(
                "Your account email has not been verified yet. Please check your inbox and verify your email before placing an order."
            )),
            _ => Ok(()),
        }
    }

    /// Single source of truth for pricing. With `lock_rows` the variant and coupon rows are locked
    /// (`SELECT ... FOR UPDATE`) so the caller can safely decrement stock / increment coupon usage.
    async fn price_order(
        conn: &mut PgConnection,
        req: &CheckoutRequest,
        lock_rows: bool,
        reverse_charge: Option<&crate::services::vies::VatCheck>,
    ) -> Result<PricedOrder> {
        if req.items.is_empty() {
            return Err(anyhow!("Cart cannot be empty for checkout"));
        }
        let (store_tax_rate, tax_mode, sales_enabled): (f64, String, bool) = sqlx::query_as(
            "SELECT tax_rate_percent, COALESCE(tax_mode, 'kleingewerbe'), sale_prices_enabled FROM store_settings WHERE id = 1",
        )
        .fetch_one(&mut *conn)
        .await
        .unwrap_or((19.0, "kleingewerbe".to_string(), false));
        let now = Utc::now();

        let lock_clause = if lock_rows { "FOR UPDATE OF pv" } else { "" };
        let variant_sql = format!(
            r#"
            SELECT
                pv.id, pv.product_id, pv.sku, pv.title as variant_title,
                pv.price_override_cents, pv.stock_quantity, pv.sale_price_cents, pv.weight_grams AS variant_weight,
                p.title as product_title, p.base_price_cents, p.product_type,
                p.digital_download_url, p.tax_rate_percent, p.is_active,
                p.weight_grams AS product_weight, p.sale_starts_at, p.sale_ends_at,
                pv.length_mm AS v_length, pv.width_mm AS v_width, pv.height_mm AS v_height,
                p.length_mm AS p_length, p.width_mm AS p_width, p.height_mm AS p_height
            FROM product_variants pv
            JOIN products p ON p.id = pv.product_id
            WHERE pv.id = $1
            {}
            "#,
            lock_clause
        );

        let mut lines = Vec::new();
        let mut digital_downloads = Vec::new();
        let mut items_subtotal_cents = 0;
        let mut parcel = crate::services::packing::Parcel::default();
        // Line totals per VAT rate (first appearance order) — shipping is taxed with the main rate
        let mut rate_amounts: Vec<(f64, i64)> = Vec::new();

        for item in &req.items {
            if item.quantity <= 0 {
                return Err(anyhow!("Invalid quantity for item"));
            }

            let row = sqlx::query(&variant_sql)
                .bind(item.variant_id)
                .fetch_optional(&mut *conn)
                .await?
                .ok_or_else(|| anyhow!("A product in your cart is no longer available"))?;

            let product_id: Uuid = row.get("product_id");
            let sku: String = row.get("sku");
            let variant_title: String = row.get("variant_title");
            let stock_quantity: i32 = row.get("stock_quantity");
            let product_title: String = row.get("product_title");
            let is_active: bool = row.get("is_active");
            let is_digital = row.get::<String, _>("product_type") == "digital";
            let item_tax_rate = row.get::<Option<f64>, _>("tax_rate_percent").unwrap_or(store_tax_rate);

            if !is_active {
                return Err(anyhow!("'{}' is no longer available", product_title));
            }
            if !is_digital && stock_quantity < item.quantity {
                return Err(anyhow!(
                    "Insufficient stock for '{}' (SKU: {}). Available: {}, Requested: {}",
                    variant_title, sku, stock_quantity, item.quantity
                ));
            }

            let regular_price = row
                .get::<Option<i32>, _>("price_override_cents")
                .unwrap_or_else(|| row.get::<i32, _>("base_price_cents"));
            // An active sale price is what the customer pays
            let unit_price = crate::services::pricing::sale_applies(
                sales_enabled,
                regular_price,
                row.get("sale_price_cents"),
                row.get("sale_starts_at"),
                row.get("sale_ends_at"),
                now,
            )
            .unwrap_or(regular_price);
            if !is_digital {
                let weight = crate::services::pricing::weight_grams(row.get("product_weight"), row.get("variant_weight"));
                parcel.add(weight, Self::item_dims(&row), item.quantity);
            }
            let line_total = unit_price * item.quantity;
            items_subtotal_cents += line_total;

            match rate_amounts.iter_mut().find(|(r, _)| (*r - item_tax_rate).abs() < 1e-9) {
                Some((_, sum)) => *sum += line_total as i64,
                None => rate_amounts.push((item_tax_rate, line_total as i64)),
            }

            let download_url: Option<String> = if is_digital { row.get("digital_download_url") } else { None };
            if is_digital {
                if let Some(ref url_str) = download_url {
                    if let Ok(files) = serde_json::from_str::<Vec<serde_json::Value>>(url_str) {
                        for f in files {
                            let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("File");
                            let u = f.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            if !u.is_empty() {
                                digital_downloads.push(DigitalDownloadItem {
                                    title: format!("{} ({})", product_title, name),
                                    download_url: u.to_string(),
                                });
                            }
                        }
                    } else if !url_str.trim().is_empty() {
                        digital_downloads.push(DigitalDownloadItem {
                            title: product_title.clone(),
                            download_url: url_str.clone(),
                        });
                    }
                }

                // BOM product_parts can carry additional digital files
                let parts = sqlx::query("SELECT part_name, notes FROM product_parts WHERE product_id = $1 AND part_sku = 'DIGITAL_FILE'")
                    .bind(product_id)
                    .fetch_all(&mut *conn)
                    .await
                    .unwrap_or_default();
                for p in parts {
                    let pname: String = p.get("part_name");
                    if let Some(furl) = p.get::<Option<String>, _>("notes") {
                        if !furl.trim().is_empty() {
                            digital_downloads.push(DigitalDownloadItem {
                                title: format!("{} ({})", product_title, pname),
                                download_url: furl,
                            });
                        }
                    }
                }
            }

            lines.push(PricedLine {
                product_id,
                variant_id: row.get("id"),
                product_title,
                variant_title,
                sku,
                unit_price_cents: unit_price,
                quantity: item.quantity,
                total_price_cents: line_total,
                is_digital,
                download_url,
                tax_rate_percent: item_tax_rate,
            });
        }

        let is_digital_only = lines.iter().all(|l| l.is_digital);

        // Shipping: only rates of the zone serving the destination country whose weight and package
        // size fit the parcel; the selected one must be among them
        let mut shipping_cost_cents = 0;
        let mut missing_shipping_rate = false;
        let mut shipping_options = Vec::new();
        if !is_digital_only {
            let country = req.shipping_address.country_code.trim().to_uppercase();
            let (fitting, any_rates) = Self::fitting_rates(&mut *conn, &country, &parcel).await?;
            // Once shipping is set up, a country no zone serves cannot be shipped to (a shop
            // without any rates keeps shipping free to everywhere)
            if !any_rates && Self::shop_has_shipping_rates(&mut *conn).await? {
                return Err(anyhow!("Sorry, we do not ship to {} yet.", if country.is_empty() { "this country" } else { country.as_str() }));
            }
            if fitting.is_empty() && any_rates {
                return Err(anyhow!(
                    "No shipping option fits this parcel ({}). Please contact us for a shipping quote.",
                    parcel.describe()
                ));
            }
            match req.shipping_rate_id.and_then(|id| fitting.iter().find(|r| r.id == id)) {
                Some(rate) => shipping_cost_cents = rate.price_cents,
                // None chosen, or one for another country / parcel: the customer has to choose
                // (the quote still answers with the matching options)
                None => missing_shipping_rate = !fitting.is_empty(),
            }
            shipping_options = fitting;
        }

        // Coupon
        let mut coupon = None;
        let mut discount_cents = 0;
        let mut subtotal_cents = items_subtotal_cents;
        if let Some(code) = req.coupon_code.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
            let coupon_sql = format!(
                "SELECT id, code, discount_type, value_cents, min_order_cents, max_uses, used_count, expires_at FROM coupons WHERE UPPER(code) = UPPER($1) AND is_active = true {}",
                if lock_rows { "FOR UPDATE" } else { "" }
            );
            let c = sqlx::query(&coupon_sql)
                .bind(code)
                .fetch_optional(&mut *conn)
                .await?
                .ok_or_else(|| anyhow!("Coupon '{}' is invalid or inactive", code))?;

            if let Some(exp) = c.get::<Option<DateTime<Utc>>, _>("expires_at") {
                if exp < Utc::now() {
                    return Err(anyhow!("Coupon '{}' has expired", code));
                }
            }
            if let Some(max_u) = c.get::<Option<i32>, _>("max_uses") {
                if c.get::<i32, _>("used_count") >= max_u {
                    return Err(anyhow!("Coupon '{}' usage limit has been reached", code));
                }
            }
            let min_order_cents: i32 = c.get("min_order_cents");
            if items_subtotal_cents < min_order_cents {
                return Err(anyhow!(
                    "Order subtotal of {:.2} € is below the minimum {:.2} € required for coupon '{}'",
                    items_subtotal_cents as f64 / 100.0,
                    min_order_cents as f64 / 100.0,
                    code
                ));
            }

            let value_cents: i32 = c.get("value_cents");
            match c.get::<String, _>("discount_type").as_str() {
                "free_shipping" => {
                    discount_cents = shipping_cost_cents;
                    shipping_cost_cents = 0;
                }
                "fixed_amount" => {
                    discount_cents = value_cents.clamp(0, items_subtotal_cents);
                    subtotal_cents = items_subtotal_cents - discount_cents;
                }
                "percentage" => {
                    let pct = value_cents.clamp(0, 100);
                    discount_cents = ((items_subtotal_cents as f64) * (pct as f64 / 100.0)).round() as i32;
                    subtotal_cents = (items_subtotal_cents - discount_cents).max(0);
                }
                _ => {}
            }
            coupon = Some((c.get::<Uuid, _>("id"), c.get::<String, _>("code")));
        }

        let (tax_cents, total_cents) = compute_tax(
            &tax_mode,
            &rate_amounts,
            items_subtotal_cents,
            subtotal_cents,
            shipping_cost_cents,
            reverse_charge.is_some(),
        );

        Ok(PricedOrder {
            lines,
            tax_mode: tax_mode.clone(),
            digital_downloads,
            items_subtotal_cents,
            discount_cents,
            subtotal_cents,
            shipping_cost_cents,
            tax_cents,
            total_cents,
            is_digital_only,
            coupon,
            missing_shipping_rate,
            total_weight_g: parcel.weight_g,
            shipping_options,
            reverse_charge: reverse_charge.cloned(),
        })
    }

    /// Customer / address checks required before money is taken or an order is written.
    fn validate_for_order(req: &CheckoutRequest, priced: &PricedOrder) -> Result<()> {
        if req.customer_name.trim().is_empty() {
            return Err(anyhow!("Please enter your full name"));
        }
        let email = req.customer_email.trim();
        if email.len() < 3 || !email.contains('@') || email.contains(char::is_whitespace) {
            return Err(anyhow!("Please enter a valid email address"));
        }
        if !priced.is_digital_only {
            let addr = &req.shipping_address;
            if addr.street_address.trim().is_empty() || addr.city.trim().is_empty() || addr.postal_code.trim().is_empty() {
                return Err(anyhow!("Please enter a complete shipping address"));
            }
            if priced.missing_shipping_rate {
                return Err(anyhow!("Please select a shipping option"));
            }
        }
        Ok(())
    }

    /// Prices the cart without locking. With `strict` the customer details are validated as well.
    pub async fn quote(pool: &PgPool, req: &CheckoutRequest, strict: bool) -> Result<CheckoutQuote> {
        let reverse_charge = Self::reverse_charge(pool, req).await?;
        let mut conn = pool.acquire().await?;
        let priced = Self::price_order(&mut *conn, req, false, reverse_charge.as_ref()).await?;
        if strict {
            Self::validate_for_order(req, &priced)?;
        }
        Ok(CheckoutQuote {
            items_subtotal_cents: priced.items_subtotal_cents,
            discount_cents: priced.discount_cents,
            subtotal_cents: priced.subtotal_cents,
            shipping_cost_cents: priced.shipping_cost_cents,
            tax_cents: priced.tax_cents,
            total_cents: priced.total_cents,
            is_digital_only: priced.is_digital_only,
            coupon_code: priced.coupon.map(|(_, code)| code),
            currency: "EUR".to_string(),
            total_weight_g: priced.total_weight_g,
            shipping_options: priced.shipping_options.iter().map(|r| json!({ "id": r.id, "name": r.name, "price_cents": r.price_cents, "estimated_delivery_days": r.estimated_delivery_days })).collect(),
            reverse_charge: priced.reverse_charge.is_some(),
        })
    }

    /// Package size of a cart line: the variant's own dimensions, otherwise the product's.
    fn item_dims(row: &sqlx::postgres::PgRow) -> Option<crate::services::packing::Dims> {
        use crate::services::packing::sorted_dims;
        sorted_dims(row.get("v_length"), row.get("v_width"), row.get("v_height"))
            .or_else(|| sorted_dims(row.get("p_length"), row.get("p_width"), row.get("p_height")))
    }

    /// Rates of the zone serving `country` that the parcel fits into (cheapest first), and whether
    /// that zone has any rates at all.
    async fn fitting_rates(
        conn: &mut PgConnection,
        country: &str,
        parcel: &crate::services::packing::Parcel,
    ) -> Result<(Vec<crate::models::ShippingRate>, bool)> {
        // Zones listing the country explicitly; the default zone only if none does
        let rates = sqlx::query_as::<_, crate::models::ShippingRate>(
            r#"
            WITH matched AS (SELECT id FROM shipping_zones WHERE country_codes @> $1::jsonb),
                 serving AS (SELECT id FROM matched UNION ALL SELECT id FROM shipping_zones WHERE is_default = true AND NOT EXISTS (SELECT 1 FROM matched))
            SELECT id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days, max_length_mm, max_width_mm, max_height_mm
            FROM shipping_rates WHERE zone_id IN (SELECT id FROM serving) ORDER BY price_cents, name
            "#,
        )
        .bind(json!([country]).to_string())
        .fetch_all(&mut *conn)
        .await?;
        let any = !rates.is_empty();
        Ok((rates.into_iter().filter(|r| parcel.fits(&r.limits())).collect(), any))
    }

    /// Whether any shipping rate is configured at all.
    async fn shop_has_shipping_rates(conn: &mut PgConnection) -> Result<bool> {
        Ok(sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM shipping_rates)").fetch_one(&mut *conn).await?)
    }

    /// Shipping options for some products to a country (product page: "delivery to …").
    pub async fn shipping_options(pool: &PgPool, country: &str, items: &[crate::models::CartItemInput]) -> Result<(Vec<crate::models::ShippingRate>, bool)> {
        let mut conn = pool.acquire().await?;
        let mut parcel = crate::services::packing::Parcel::default();
        for item in items.iter().filter(|i| i.quantity > 0) {
            let row = sqlx::query(
                "SELECT p.product_type, p.weight_grams AS product_weight, pv.weight_grams AS variant_weight,
                        pv.length_mm AS v_length, pv.width_mm AS v_width, pv.height_mm AS v_height,
                        p.length_mm AS p_length, p.width_mm AS p_width, p.height_mm AS p_height
                 FROM product_variants pv JOIN products p ON p.id = pv.product_id WHERE pv.id = $1",
            )
            .bind(item.variant_id)
            .fetch_optional(&mut *conn)
            .await?;
            if let Some(row) = row.filter(|r| r.get::<String, _>("product_type") != "digital") {
                let weight = crate::services::pricing::weight_grams(row.get("product_weight"), row.get("variant_weight"));
                parcel.add(weight, Self::item_dims(&row), item.quantity);
            }
        }
        Self::fitting_rates(&mut *conn, &country.trim().to_uppercase(), &parcel).await
    }

    /// VIES-confirmed VAT-free EU business order, `None` for a normal order.
    pub async fn reverse_charge(pool: &PgPool, req: &CheckoutRequest) -> Result<Option<crate::services::vies::VatCheck>> {
        if req.vat_id.as_deref().map(str::trim).unwrap_or("").is_empty() {
            return Ok(None);
        }
        let settings = sqlx::query_as::<_, crate::models::StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await?;
        crate::services::vies::reverse_charge_for(pool, &settings, req).await.map_err(|e| anyhow!(e))
    }

    /// Creates the order inside the caller's transaction for a payment that was already verified.
    /// Re-prices with row locks and refuses if the verified amount differs from the order total.
    pub async fn create_order(
        conn: &mut PgConnection,
        req: &CheckoutRequest,
        payment: &VerifiedPayment,
        reverse_charge: Option<&crate::services::vies::VatCheck>,
    ) -> Result<CheckoutResponse> {
        let priced = Self::price_order(conn, req, true, reverse_charge).await?;
        Self::validate_for_order(req, &priced)?;

        if priced.total_cents != payment.amount_cents {
            return Err(anyhow!(
                "Paid amount ({:.2} €) does not match the order total ({:.2} €)",
                payment.amount_cents as f64 / 100.0,
                priced.total_cents as f64 / 100.0
            ));
        }

        for line in priced.lines.iter().filter(|l| !l.is_digital) {
            sqlx::query("UPDATE product_variants SET stock_quantity = stock_quantity - $1, updated_at = NOW() WHERE id = $2")
                .bind(line.quantity)
                .bind(line.variant_id)
                .execute(&mut *conn)
                .await?;

            // Shared BOM parts are consumed with the product
            crate::services::inventory::adjust_parts_for_variant(&mut *conn, line.product_id, line.variant_id, line.quantity, -1).await?;
        }

        if let Some((coupon_id, _)) = &priced.coupon {
            sqlx::query("UPDATE coupons SET used_count = used_count + 1, updated_at = NOW() WHERE id = $1")
                .bind(coupon_id)
                .execute(&mut *conn)
                .await?;
        }

        // GoBD-compliant consecutive order number
        let (prefix_enabled, order_prefix, date_enabled): (bool, String, bool) = sqlx::query_as(
            "SELECT COALESCE(order_prefix_enabled, true), COALESCE(order_prefix, 'ORD'), COALESCE(order_date_enabled, true) FROM store_settings WHERE id = 1",
        )
        .fetch_one(&mut *conn)
        .await
        .unwrap_or((true, "ORD".to_string(), true));
        let seq_num: i64 = sqlx::query_scalar("SELECT nextval('order_number_seq')").fetch_one(&mut *conn).await?;

        let mut parts = Vec::new();
        let clean_prefix = order_prefix.trim().to_uppercase();
        if prefix_enabled && !clean_prefix.is_empty() {
            parts.push(clean_prefix);
        }
        if date_enabled {
            parts.push(Utc::now().format("%Y%m%d").to_string());
        }
        parts.push(seq_num.to_string());
        let order_number = parts.join("-");

        let shipping_addr_json = serde_json::to_value(&req.shipping_address)?;
        let billing_addr_json = match req.billing_address {
            Some(ref b) => serde_json::to_value(b)?,
            None => shipping_addr_json.clone(),
        };

        // Digital-only orders auto-complete once paid (no shipping needed)
        let order_status = if priced.is_digital_only && payment.status == "paid" { "completed" } else { "processing" };
        let shipping_rate_id = if priced.is_digital_only { None } else { req.shipping_rate_id };
        let order_id = Uuid::new_v4();
        let access_token = crate::services::auth::random_token(32);

        sqlx::query(
            r#"
            INSERT INTO orders (
                id, order_number, customer_name, customer_email,
                shipping_address, billing_address, shipping_rate_id,
                shipping_cost_cents, subtotal_cents, tax_cents, total_cents,
                payment_provider, payment_status, order_status, coupon_code, discount_cents,
                payment_reference, access_token, is_test, tax_mode,
                buyer_company, buyer_vat_id, reverse_charge, vat_check, paid_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, CASE WHEN $25 THEN NOW() END)
            "#,
        )
        .bind(order_id)
        .bind(&order_number)
        .bind(req.customer_name.trim())
        .bind(req.customer_email.trim())
        .bind(shipping_addr_json)
        .bind(billing_addr_json)
        .bind(shipping_rate_id)
        .bind(priced.shipping_cost_cents)
        .bind(priced.subtotal_cents)
        .bind(priced.tax_cents)
        .bind(priced.total_cents)
        .bind(&payment.provider)
        .bind(&payment.status)
        .bind(order_status)
        .bind(priced.coupon.as_ref().map(|(_, code)| code.clone()))
        .bind(priced.discount_cents)
        .bind(&payment.reference)
        .bind(&access_token)
        .bind(payment.is_test)
        .bind(&priced.tax_mode)
        .bind(req.buyer_company.as_deref().map(str::trim).filter(|c| !c.is_empty()))
        .bind(req.vat_id.as_deref().map(crate::services::vies::normalize).filter(|v| !v.is_empty()))
        .bind(priced.reverse_charge.is_some())
        .bind(priced.reverse_charge.as_ref().map(crate::services::vies::proof_json))
        .bind(payment.status == "paid")
        .execute(&mut *conn)
        .await?;

        for line in &priced.lines {
            sqlx::query(
                r#"
                INSERT INTO order_items (
                    order_id, product_id, variant_id, product_title,
                    variant_title, sku, unit_price_cents, quantity,
                    total_price_cents, is_digital, download_url, tax_rate_percent
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
            )
            .bind(order_id)
            .bind(line.product_id)
            .bind(line.variant_id)
            .bind(&line.product_title)
            .bind(&line.variant_title)
            .bind(&line.sku)
            .bind(line.unit_price_cents)
            .bind(line.quantity)
            .bind(line.total_price_cents)
            .bind(line.is_digital)
            .bind(&line.download_url)
            .bind(line.tax_rate_percent)
            .execute(&mut *conn)
            .await?;
        }

        let paid = payment.status == "paid";
        Ok(CheckoutResponse {
            order_number,
            access_token,
            total_cents: priced.total_cents,
            payment_status: payment.status.clone(),
            order_status: order_status.to_string(),
            payment_redirect_url: None,
            digital_items: if paid { priced.digital_downloads } else { Vec::new() },
        })
    }

    /// Stores the checkout payload server-side before the customer is sent to the payment provider,
    /// so the order can be created from trusted data by the return page or by a webhook.
    pub async fn create_pending(pool: &PgPool, provider: &str, req: &CheckoutRequest, amount_cents: i32) -> Result<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO pending_checkouts (id, provider, checkout_payload, amount_cents) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(provider)
            .bind(serde_json::to_value(req)?)
            .bind(amount_cents)
            .execute(pool)
            .await?;
        Ok(id)
    }

    pub async fn attach_reference(pool: &PgPool, pending_id: Uuid, reference: &str) -> Result<()> {
        sqlx::query("UPDATE pending_checkouts SET provider_reference = $1, updated_at = NOW() WHERE id = $2")
            .bind(reference)
            .bind(pending_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn pending_exists(pool: &PgPool, provider: &str, reference: &str) -> Result<Option<Uuid>> {
        Ok(sqlx::query_scalar("SELECT id FROM pending_checkouts WHERE provider = $1 AND provider_reference = $2")
            .bind(provider)
            .bind(reference)
            .fetch_optional(pool)
            .await?)
    }

    /// Turns a pending checkout into an order exactly once. Safe to call concurrently from the
    /// customer's return page and the provider webhook: the pending row is locked for the duration.
    pub async fn finalize_pending(pool: &PgPool, provider_reference: &str, payment: VerifiedPayment) -> Result<FinalizeOutcome, FinalizeError> {
        let mut tx = pool.begin().await.map_err(|e| FinalizeError::Internal(e.into()))?;

        let row = sqlx::query(
            "SELECT id, provider, checkout_payload, amount_cents, status, order_number FROM pending_checkouts WHERE provider_reference = $1 FOR UPDATE",
        )
        .bind(provider_reference)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| FinalizeError::Internal(e.into()))?
        .ok_or_else(|| FinalizeError::Invalid(anyhow!("Unknown payment reference")))?;

        let pending_id: Uuid = row.get("id");
        let status: String = row.get("status");

        if row.get::<String, _>("provider") != payment.provider {
            return Err(FinalizeError::Invalid(anyhow!("Payment provider mismatch")));
        }

        if status == "completed" {
            let order_number: String = row.get::<Option<String>, _>("order_number").unwrap_or_default();
            let existing = sqlx::query("SELECT total_cents, payment_status, order_status, COALESCE(access_token, '') AS access_token FROM orders WHERE order_number = $1")
                .bind(&order_number)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| FinalizeError::Internal(e.into()))?;
            let (total_cents, payment_status, order_status, access_token) = match existing {
                Some(o) => (o.get("total_cents"), o.get("payment_status"), o.get("order_status"), o.get("access_token")),
                None => (payment.amount_cents, payment.status.clone(), "processing".to_string(), String::new()),
            };
            return Ok(FinalizeOutcome {
                response: CheckoutResponse {
                    order_number,
                    access_token,
                    total_cents,
                    payment_status,
                    order_status,
                    payment_redirect_url: None,
                    digital_items: Vec::new(),
                },
                newly_created: false,
            });
        }
        if status == "failed" {
            return Err(FinalizeError::Invalid(anyhow!("This checkout could not be completed and its payment was refunded.")));
        }

        let expected: i32 = row.get("amount_cents");
        let result = match serde_json::from_value::<CheckoutRequest>(row.get("checkout_payload")) {
            Err(e) => Err(anyhow!("Stored checkout is unreadable: {}", e)),
            Ok(_) if payment.amount_cents != expected => Err(anyhow!(
                "Paid amount ({:.2} €) does not match the checkout total ({:.2} €)",
                payment.amount_cents as f64 / 100.0,
                expected as f64 / 100.0
            )),
            Ok(req) => match Self::reverse_charge(pool, &req).await {
                Ok(rc) => Self::create_order(&mut *tx, &req, &payment, rc.as_ref()).await,
                Err(e) => Err(e),
            },
        };

        match result {
            Ok(response) => {
                sqlx::query("UPDATE pending_checkouts SET status = 'completed', order_number = $1, updated_at = NOW() WHERE id = $2")
                    .bind(&response.order_number)
                    .bind(pending_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| FinalizeError::Internal(e.into()))?;
                tx.commit().await.map_err(|e| FinalizeError::Internal(e.into()))?;
                Ok(FinalizeOutcome { response, newly_created: true })
            }
            Err(err) => {
                let _ = tx.rollback().await;
                let _ = sqlx::query("UPDATE pending_checkouts SET status = 'failed', failure_reason = $1, updated_at = NOW() WHERE id = $2")
                    .bind(err.to_string())
                    .bind(pending_id)
                    .execute(pool)
                    .await;
                Err(FinalizeError::OrderRejected(err))
            }
        }
    }

    /// A delayed payment (e.g. SEPA Direct Debit) was confirmed: mark the pending order paid and
    /// auto-complete digital-only orders. Returns the order number if an order changed.
    pub async fn mark_payment_succeeded(pool: &PgPool, payment_reference: &str) -> Result<Option<String>> {
        Ok(sqlx::query_scalar(
            r#"
            UPDATE orders o
            SET payment_status = 'paid',
                paid_at = NOW(),
                order_status = CASE
                    WHEN NOT EXISTS (SELECT 1 FROM order_items oi WHERE oi.order_id = o.id AND NOT oi.is_digital) THEN 'completed'
                    ELSE o.order_status
                END,
                updated_at = NOW()
            WHERE o.payment_reference = $1 AND o.payment_status = 'pending'
            RETURNING o.order_number
            "#,
        )
        .bind(payment_reference)
        .fetch_optional(pool)
        .await?)
    }

    /// A delayed payment failed: cancel the pending order, return its stock and coupon usage.
    pub async fn mark_payment_failed(pool: &PgPool, payment_reference: &str) -> Result<Option<String>> {
        let mut tx = pool.begin().await?;
        let order = sqlx::query(
            "SELECT id, order_number, coupon_code FROM orders WHERE payment_reference = $1 AND payment_status = 'pending' FOR UPDATE",
        )
        .bind(payment_reference)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(order) = order else {
            return Ok(None);
        };
        let order_id: Uuid = order.get("id");

        sqlx::query(
            r#"
            UPDATE product_variants pv
            SET stock_quantity = pv.stock_quantity + oi.quantity, updated_at = NOW()
            FROM order_items oi
            WHERE oi.order_id = $1 AND oi.variant_id = pv.id AND NOT oi.is_digital
            "#,
        )
        .bind(order_id)
        .execute(&mut *tx)
        .await?;
        let items = sqlx::query("SELECT product_id, variant_id, quantity FROM order_items WHERE order_id = $1 AND NOT is_digital")
            .bind(order_id)
            .fetch_all(&mut *tx)
            .await?;
        for item in items {
            crate::services::inventory::adjust_parts_for_variant(&mut *tx, item.get("product_id"), item.get("variant_id"), item.get("quantity"), 1).await?;
        }
        if let Some(code) = order.get::<Option<String>, _>("coupon_code") {
            sqlx::query("UPDATE coupons SET used_count = GREATEST(used_count - 1, 0), updated_at = NOW() WHERE code = $1")
                .bind(code)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query("UPDATE orders SET payment_status = 'failed', order_status = 'cancelled', updated_at = NOW() WHERE id = $1")
            .bind(order_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        if let Err(e) = crate::services::inventory::recalculate_bom_stock(pool).await {
            tracing::error!("Recalculating BOM stock after a failed payment failed: {}", e);
        }
        Ok(Some(order.get("order_number")))
    }

    /// Orders whose total is 0 € (e.g. 100 % coupon) need no payment provider.
    pub async fn create_free_order(pool: &PgPool, req: &CheckoutRequest) -> Result<CheckoutResponse> {
        let reverse_charge = Self::reverse_charge(pool, req).await?;
        let mut tx = pool.begin().await?;
        let payment = VerifiedPayment {
            provider: "free".to_string(),
            reference: format!("free-{}", Uuid::new_v4()),
            amount_cents: 0,
            status: "paid".to_string(),
            // No payment provider involved — a 100 % coupon is a real order
            is_test: false,
        };
        let response = Self::create_order(&mut *tx, req, &payment, reverse_charge.as_ref()).await?;
        tx.commit().await?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::compute_tax;
    use crate::services::invoice::tests::{item, sample_order, sample_settings};
    use crate::services::invoice::Invoice;

    #[test]
    fn shipping_carries_the_main_vat_rate() {
        // 100 € at 19 % + 20 € at 7 %, 10 € shipping
        let rates = [(19.0, 10000), (7.0, 2000)];
        // Net prices: VAT on items (19.00 + 1.40) plus 19 % on shipping (1.90)
        assert_eq!(compute_tax("excluded", &rates, 12000, 12000, 1000, false), (2230, 15230));
        // Gross prices: total unchanged, contained VAT includes the shipping share
        let (tax, total) = compute_tax("included", &rates, 12000, 12000, 1000, false);
        assert_eq!(total, 13000);
        assert_eq!(tax, (10000.0 - 10000.0 / 1.19 + 2000.0 - 2000.0 / 1.07 + 1000.0 - 1000.0 / 1.19_f64).round() as i32);
        assert_eq!(compute_tax("kleingewerbe", &rates, 12000, 12000, 1000, false), (0, 13000));
        // Equal shares: the first rate wins (same rule as the invoice)
        assert_eq!(compute_tax("excluded", &[(7.0, 1000), (19.0, 1000)], 2000, 2000, 1000, false).0, 70 + 190 + 70);
    }

    #[test]
    fn checkout_and_invoice_agree() {
        for mode in ["kleingewerbe", "included", "excluded"] {
            let items = vec![item("Gadget", "G", 1999, 3, 19.0, false), item("Book", "B", 1099, 2, 7.0, false)];
            let items_sum = 3 * 1999 + 2 * 1099;
            let subtotal = items_sum - 815; // coupon
            let rates = [(19.0, 3 * 1999), (7.0, 2 * 1099)];
            let (tax, total) = compute_tax(mode, &rates, items_sum, subtotal, 495, false);
            let mut order = sample_order(mode, subtotal, 495, total);
            order.tax_cents = tax;
            let inv = Invoice::build(&order, &items, &sample_settings(mode));
            assert_eq!(inv.payable_cents, total as i64, "{}", mode);
            assert!(inv.rounding_cents.abs() <= 2, "{}: rounding {}", mode, inv.rounding_cents);
            assert!((inv.tax_total_cents - tax as i64).abs() <= 2, "{}: invoice VAT {} vs checkout {}", mode, inv.tax_total_cents, tax);
        }
    }
}
