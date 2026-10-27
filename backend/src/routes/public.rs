use crate::models::{
    CartItemInput, Category, Claims, Coupon, CreateStockNotificationRequest, CustomerAddress,
    CustomerChangePasswordRequest, CustomerLoginRequest, CustomerRegisterRequest, EstimateShippingRequest,
    NavigationItem, OrderItem, PageContent, Product, ProductPart, ProductVariant, ProductWithVariants,
    PublicPaymentProviderInfo, ResetPasswordRequest, SaveAddressRequest, ShippingProvider,
    ShippingProviderWithZones, ShippingRate, ShippingZone, ShippingZoneWithRates, StoreSettings,
    StoreSettingsDTO, ToggleWishlistRequest, UpdateCustomerProfileRequest, ValidateCouponRequest,
    ValidateCouponResponse,
};
use axum::{
    extract::{Path, Query, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use crate::services::auth;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub product_type: Option<String>,
    pub search: Option<String>,
}

pub fn public_router() -> Router<PgPool> {
    Router::new()
        .route("/store/info", get(get_store_info))
        .route("/categories", get(public_list_categories))
        .route("/menu", get(public_get_menu))
        .route("/pages/:slug", get(public_get_page))
        .route("/shipping/providers", get(public_get_shipping_providers))
        .route("/products", get(list_products))
        .route("/products/carousels", get(public_get_carousels))
        .route("/products/:slug", get(get_product_by_slug))
        .route("/products/:id/related", get(public_get_related_products))
        .route("/products/:id/notify-stock", post(public_subscribe_stock_notification))
        .route("/products/:slug/parts", get(public_get_product_parts))
        .route("/cart/validate", post(validate_cart))
        .route("/coupons/validate", post(public_validate_coupon))
        .route("/shipping/rates", get(get_shipping_rates))
        .route("/orders/lookup/:order_number", get(lookup_order))
        // Customer Authentication & Profile
        .route("/auth/customer/register", post(public_customer_register))
        .route("/auth/customer/login", post(public_customer_login))
        .route("/auth/customer/reset-password", post(public_customer_reset_password))
        .route("/auth/customer/resend-verification", post(public_resend_verification))
        .route("/customer/verify", get(public_verify_customer_email))
        .route("/customer/change-password", post(public_customer_change_password))
        .route("/customer/profile", get(public_get_customer_profile).put(public_update_customer_profile))
        .route("/customer/orders", get(public_get_customer_orders))
        .route("/customer/downloads", get(public_get_customer_downloads))
        .route("/customer/wishlist", get(public_get_customer_wishlist))
        .route("/customer/wishlist/toggle", post(public_toggle_wishlist))
        .route("/customer/addresses", get(public_get_customer_addresses).post(public_save_customer_address))
        .route("/customer/addresses/:id", delete(public_delete_customer_address))
}

async fn get_store_info(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT * FROM store_settings WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let eu_business_vat_free = crate::services::vies::available(&settings);
    let store_dto = StoreSettingsDTO {
        store_name: settings.store_name,
        currency: settings.currency,
        currency_symbol: settings.currency_symbol,
        tax_rate_percent: settings.tax_rate_percent,
        tax_mode: settings.tax_mode,
        deployment_mode: settings.deployment_mode,
        debug_mode: settings.debug_mode,
        support_email: settings.support_email,
        company_address: settings.company_address,
        vat_id: settings.vat_id,
        logo_url: settings.logo_url,
        phone: settings.phone,
        hero_config: settings.hero_config,
        require_registered_checkout: settings.require_registered_checkout,
        require_email_verification: settings.require_email_verification,
        store_subtitle: settings.store_subtitle,
        show_store_title: settings.show_store_title,
        show_store_subtitle: settings.show_store_subtitle,
        carousels_config: settings.carousels_config,
        cookie_banner_enabled: settings.cookie_banner_enabled,
        cookie_banner_title: settings.cookie_banner_title,
        cookie_banner_description: settings.cookie_banner_description,
        cookie_banner_policy_url: settings.cookie_banner_policy_url,
        cookie_accept_label: settings.cookie_accept_label,
        cookie_deny_label: settings.cookie_deny_label,
        cookie_preferences_label: settings.cookie_preferences_label,
        tax_notice: settings.tax_notice,
        legal_name: settings.legal_name,
        store_owner: settings.store_owner,
        commercial_register: settings.commercial_register,
        dispute_resolution_notice: settings.dispute_resolution_notice,
        odr_url: settings.odr_url,
        footer_config: settings.footer_config,
        order_prefix_enabled: settings.order_prefix_enabled,
        order_prefix: settings.order_prefix,
        order_date_enabled: settings.order_date_enabled,
        stock_display_template: settings.stock_display_template,
        address_street: settings.address_street,
        address_house_number: settings.address_house_number,
        address_extra: settings.address_extra,
        address_postal_code: settings.address_postal_code,
        address_city: settings.address_city,
        address_country: settings.address_country,
        tax_number: settings.tax_number,
        reviews_enabled: settings.reviews_enabled,
        newsletter_enabled: settings.newsletter_enabled,
        sale_prices_enabled: settings.sale_prices_enabled,
        eu_business_vat_free,
        default_delivery_time: settings.default_delivery_time,
        download_limit: settings.download_limit,
        download_days: settings.download_days,
        gpsr_default_manufacturer: settings.gpsr_default_manufacturer,
    };

    let payment_providers = sqlx::query_as::<_, PaymentProviderRow>(
        "SELECT provider, display_name, is_enabled, is_sandbox, public_client_id, config_data FROM payment_configs WHERE is_enabled = true"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| PublicPaymentProviderInfo {
        provider: r.provider,
        display_name: r.display_name,
        is_enabled: r.is_enabled,
        is_sandbox: r.is_sandbox,
        public_client_id: r.public_client_id,
        config_data: r.config_data,
    })
    .collect::<Vec<_>>();

    Ok(Json(json!({
        "store": store_dto,
        "payment_providers": payment_providers
    })))
}

#[derive(sqlx::FromRow)]
struct PaymentProviderRow {
    provider: String,
    display_name: String,
    is_enabled: bool,
    is_sandbox: bool,
    public_client_id: String,
    config_data: serde_json::Value,
}

/// Download URLs of digital products are only handed out for paid orders, never in catalogue responses.
fn public_product(mut product: Product) -> Product {
    product.digital_download_url = None;
    product
}

async fn get_category_and_descendants(pool: &PgPool, cat_name_or_slug: &str) -> Vec<String> {
    let rows: Vec<String> = sqlx::query_scalar(
        r#"
        WITH RECURSIVE cat_tree AS (
            SELECT id, name, slug FROM categories 
            WHERE LOWER(name) = LOWER($1) OR LOWER(slug) = LOWER($1)
            UNION ALL
            SELECT c.id, c.name, c.slug 
            FROM categories c 
            JOIN cat_tree ct ON c.parent_id = ct.id
        )
        SELECT name FROM cat_tree
        "#
    )
    .bind(cat_name_or_slug)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        vec![cat_name_or_slug.to_string()]
    } else {
        rows
    }
}

async fn list_products(
    State(pool): State<PgPool>,
    Query(query): Query<ProductQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // All user input is passed as bind parameters — never interpolated into SQL.
    let category_names: Option<Vec<String>> = match query.category.as_deref().filter(|c| !c.is_empty()) {
        Some(cat) => Some(get_category_and_descendants(&pool, cat).await),
        None => None,
    };
    let subcategory_names: Option<Vec<String>> = match query.subcategory.as_deref().filter(|c| !c.is_empty()) {
        Some(sub) => Some(get_category_and_descendants(&pool, sub).await),
        None => None,
    };
    let product_type = query.product_type.as_deref().filter(|t| !t.is_empty());
    // Escape LIKE wildcards so a search for "%" or "_" is literal
    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            let term: String = s.chars().take(100).collect();
            format!("%{}%", term.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
        });

    let products = sqlx::query_as::<_, Product>(
        r#"
        SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent, weight_grams, delivery_time, sale_starts_at, sale_ends_at, gpsr_manufacturer, gpsr_responsible_person, gpsr_safety_info, unit_content, unit_content_unit, length_mm, width_mm, height_mm
        FROM products
        WHERE is_active = true
          AND ($1::text[] IS NULL OR category = ANY($1) OR subcategory = ANY($1))
          AND ($2::text[] IS NULL OR category = ANY($2) OR subcategory = ANY($2))
          AND ($3::text IS NULL OR product_type = $3)
          AND ($4::text IS NULL OR title ILIKE $4 OR description ILIKE $4)
        ORDER BY created_at DESC
        "#,
    )
    .bind(category_names)
    .bind(subcategory_names)
    .bind(product_type)
    .bind(search)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // One query for all variants instead of one per product
    let ids: Vec<Uuid> = products.iter().map(|p| p.id).collect();
    let all_variants = sqlx::query_as::<_, ProductVariant>(
        "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at, images, weight_grams, sale_price_cents, length_mm, width_mm, height_mm FROM product_variants WHERE product_id = ANY($1) ORDER BY created_at ASC"
    )
    .bind(&ids)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut by_product: std::collections::HashMap<Uuid, Vec<ProductVariant>> = std::collections::HashMap::new();
    for v in all_variants {
        by_product.entry(v.product_id).or_default().push(v);
    }

    let mut result: Vec<ProductWithVariants> = products
        .into_iter()
        .map(|product| {
            let variants = by_product.remove(&product.id).unwrap_or_default();
            ProductWithVariants { product: public_product(product), variants }
        })
        .collect();
    attach_pricing(&pool, &mut result).await;

    Ok(Json(result))
}

async fn get_product_by_slug(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let product = sqlx::query_as::<_, Product>(
        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent, weight_grams, delivery_time, sale_starts_at, sale_ends_at, gpsr_manufacturer, gpsr_responsible_person, gpsr_safety_info, unit_content, unit_content_unit, length_mm, width_mm, height_mm FROM products WHERE slug = $1 AND is_active = true"
    )
    .bind(slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let variants = sqlx::query_as::<_, ProductVariant>(
        "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at, images, weight_grams, sale_price_cents, length_mm, width_mm, height_mm FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
    )
    .bind(product.id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut result = [ProductWithVariants { product: public_product(product), variants }];
    attach_pricing(&pool, &mut result).await;
    let [result] = result;
    Ok(Json(result))
}

async fn validate_cart(
    State(pool): State<PgPool>,
    Json(items): Json<Vec<CartItemInput>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut validated_items = Vec::new();
    let mut total_cents = 0;

    for item in items {
        let variant_row = sqlx::query(
            r#"
            SELECT pv.id, pv.sku, pv.title as variant_title, pv.price_override_cents, pv.stock_quantity,
                   p.title as product_title, p.base_price_cents, p.product_type
            FROM product_variants pv
            JOIN products p ON p.id = pv.product_id
            WHERE pv.id = $1
            "#,
        )
        .bind(item.variant_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Some(v) = variant_row {
            let id: Uuid = v.get("id");
            let sku: String = v.get("sku");
            let variant_title: String = v.get("variant_title");
            let price_override_cents: Option<i32> = v.get("price_override_cents");
            let stock_quantity: i32 = v.get("stock_quantity");
            let product_title: String = v.get("product_title");
            let base_price_cents: i32 = v.get("base_price_cents");
            let product_type: String = v.get("product_type");

            let unit_price = price_override_cents.unwrap_or(base_price_cents);
            let item_total = unit_price * item.quantity;
            total_cents += item_total;

            let in_stock = product_type == "digital" || stock_quantity >= item.quantity;

            validated_items.push(json!({
                "variant_id": id,
                "sku": sku,
                "product_title": product_title,
                "variant_title": variant_title,
                "unit_price_cents": unit_price,
                "quantity": item.quantity,
                "item_total_cents": item_total,
                "available_stock": stock_quantity,
                "in_stock": in_stock,
                "is_digital": product_type == "digital"
            }));
        }
    }

    Ok(Json(json!({
        "items": validated_items,
        "subtotal_cents": total_cents
    })))
}

async fn get_shipping_rates(
    State(pool): State<PgPool>,
    Query(query): Query<EstimateShippingRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let country = query.country_code.to_uppercase();

    let zone = sqlx::query_as::<_, ShippingZone>(
        r#"
        SELECT id, provider_id, zone_name, country_codes, is_default, created_at
        FROM shipping_zones
        WHERE country_codes @> $1::jsonb OR is_default = true
        ORDER BY (country_codes @> $1::jsonb) DESC
        LIMIT 1

        "#,
    )
    .bind(json!([country]).to_string())
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(z) = zone {
        let rates = sqlx::query_as::<_, ShippingRate>(
            "SELECT id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days, max_length_mm, max_width_mm, max_height_mm FROM shipping_rates
             WHERE zone_id = $1 AND ($2::int IS NULL OR ($2 >= min_weight_g AND (max_weight_g <= 0 OR $2 <= max_weight_g)))
             ORDER BY price_cents ASC"
        )
        .bind(z.id)
        .bind(query.total_weight_g)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        Ok(Json(json!({
            "zone": z,
            "rates": rates
        })))
    } else {
        Ok(Json(json!({
            "zone": null,
            "rates": []
        })))
    }
}

#[derive(Debug, Deserialize)]
pub struct OrderLookupQuery {
    /// Secret returned to the buyer at checkout (order confirmation page)
    pub token: Option<String>,
    /// Alternative proof of ownership for the "track order" form
    pub email: Option<String>,
}

/// Order status for guests. Requires the order's secret access token or the matching email address,
/// because order numbers are sequential (GoBD) and therefore guessable.
async fn lookup_order(
    State(pool): State<PgPool>,
    Path(order_number): Path<String>,
    Query(proof): Query<OrderLookupQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let not_found = || (StatusCode::NOT_FOUND, "Order not found. Please check the order number and email address.".to_string());
    let token = proof.token.as_deref().map(str::trim).filter(|t| t.len() >= 32);
    let email = proof.email.as_deref().map(|e| e.trim().to_lowercase()).filter(|e| e.contains('@'));
    if token.is_none() && email.is_none() {
        return Err(not_found());
    }
    let order_row = sqlx::query(
        r#"
        SELECT id, order_number, customer_name, customer_email, total_cents,
               payment_provider, payment_status, order_status, tracking_number, created_at
        FROM orders
        WHERE order_number = $1
          AND (($2::text IS NOT NULL AND access_token = $2) OR ($3::text IS NOT NULL AND LOWER(customer_email) = $3))
        "#,
    )
    .bind(&order_number)
    .bind(token)
    .bind(email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(not_found)?;

    let order_id: Uuid = order_row.get("id");
    let num: String = order_row.get("order_number");
    let cust_name: String = order_row.get("customer_name");
    let cust_email: String = order_row.get("customer_email");
    let total: i32 = order_row.get("total_cents");
    let provider: String = order_row.get("payment_provider");
    let pay_status: String = order_row.get("payment_status");
    let ord_status: String = order_row.get("order_status");
    let tracking: Option<String> = order_row.get("tracking_number");
    let created: chrono::DateTime<chrono::Utc> = order_row.get("created_at");

    let items_rows = sqlx::query(
        r#"
        SELECT product_id, product_title, variant_title, sku, quantity, unit_price_cents, is_digital, download_url
        FROM order_items
        WHERE order_id = $1
        "#,
    )
    .bind(order_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut items = Vec::new();
    for r in items_rows {
        let pid: Option<Uuid> = r.try_get("product_id").ok().flatten();
        let ptitle: String = r.get("product_title");
        let vtitle: String = r.get("variant_title");
        let sku: String = r.get("sku");
        let qty: i32 = r.get("quantity");
        let unit_price: i32 = r.get("unit_price_cents");
        let is_digital: bool = r.get("is_digital");
        let orig_url: Option<String> = r.try_get("download_url").ok().flatten();

        // Never hand out download links for unpaid, failed or refunded orders; files are only
        // reachable through personal, counted links (never their storage address)
        let order_paid = pay_status == "paid";
        let files = match (is_digital && order_paid, pid) {
            (true, Some(product_id)) => crate::routes::shop_features::order_download_files(&pool, order_id, product_id, &ptitle, orig_url.as_deref()).await,
            _ => Vec::new(),
        };
        let effective_url: Option<String> = None;

        items.push(json!({
            "product_id": pid,
            "product_title": ptitle,
            "variant_title": vtitle,
            "sku": sku,
            "quantity": qty,
            "unit_price_cents": unit_price,
            "is_digital": is_digital,
            "download_url": effective_url,
            "files": files,
        }));
    }

    Ok(Json(json!({
        "order": {
            "order_number": num,
            "customer_name": cust_name,
            "customer_email": cust_email,
            "total_cents": total,
            "payment_provider": provider,
            "payment_status": pay_status,
            "order_status": ord_status,
            "tracking_number": tracking,
            "created_at": created
        },
        "items": items
    })))
}

#[derive(Debug, Deserialize)]
pub struct MenuQuery {
    pub location: Option<String>,
}

// 7. Dynamic Navigation Menu Handler
async fn public_get_menu(
    State(pool): State<PgPool>,
    Query(query): Query<MenuQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let loc = query.location.unwrap_or_else(|| "header".to_string());
    let items = if loc == "all" {
        sqlx::query_as::<_, NavigationItem>(
            "SELECT id, location, label, url, sort_order, is_active, parent_id, created_at FROM navigation_items WHERE is_active = true ORDER BY sort_order ASC, created_at ASC"
        )
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, NavigationItem>(
            "SELECT id, location, label, url, sort_order, is_active, parent_id, created_at FROM navigation_items WHERE is_active = true AND location = $1 ORDER BY sort_order ASC, created_at ASC"
        )
        .bind(loc)
        .fetch_all(&pool)
        .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(items))
}

/// Fills the shop's own data into a CMS page. A line whose placeholders are all empty (e.g. no
/// phone number or VAT ID set) is left out instead of showing an empty label.
fn render_page_placeholders(md: &str, s: &StoreSettings) -> String {
    let join = |parts: &[&str]| parts.iter().map(|p| p.trim()).filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ");
    let legal_name = if s.legal_name.trim().is_empty() { s.store_name.as_str() } else { s.legal_name.as_str() };
    let odr_url = if s.odr_url.trim().is_empty() { "https://ec.europa.eu/odr" } else { s.odr_url.as_str() };
    let street_line = join(&[&s.address_street, &s.address_house_number]);
    let city_line = join(&[&s.address_postal_code, &s.address_city]);
    let values: [(&str, &str); 19] = [
        ("{{STORE_NAME}}", &s.store_name),
        ("{{LEGAL_NAME}}", legal_name),
        ("{{STORE_OWNER}}", &s.store_owner),
        ("{{COMPANY_ADDRESS}}", &s.company_address),
        ("{{STREET_LINE}}", &street_line),
        ("{{STREET}}", &s.address_street),
        ("{{HOUSE_NUMBER}}", &s.address_house_number),
        ("{{ADDRESS_EXTRA}}", &s.address_extra),
        ("{{CITY_LINE}}", &city_line),
        ("{{POSTAL_CODE}}", &s.address_postal_code),
        ("{{CITY}}", &s.address_city),
        ("{{COUNTRY}}", &s.address_country),
        ("{{SUPPORT_EMAIL}}", &s.support_email),
        ("{{PHONE}}", &s.phone),
        ("{{VAT_ID}}", &s.vat_id),
        ("{{TAX_NOTICE}}", &s.tax_notice),
        ("{{COMMERCIAL_REGISTER}}", &s.commercial_register),
        ("{{DISPUTE_RESOLUTION_NOTICE}}", &s.dispute_resolution_notice),
        ("{{ODR_URL}}", odr_url),
    ];

    md.lines()
        .filter_map(|line| {
            let mut out = line.to_string();
            let (mut used, mut filled) = (false, false);
            for (key, value) in &values {
                if out.contains(key) {
                    used = true;
                    filled |= !value.trim().is_empty();
                    out = out.replace(key, value.trim());
                }
            }
            (!used || filled).then_some(out)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// 8. Public CMS Policy Page Handler
async fn public_get_page(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut page = sqlx::query_as::<_, PageContent>(
        "SELECT slug, title, content_markdown, is_published, updated_at FROM pages WHERE slug = $1 AND is_published = true"
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    // Load store_settings to dynamically interpolate Store Identity across all Policy pages
    let settings_opt = sqlx::query_as::<_, StoreSettings>(
        "SELECT * FROM store_settings WHERE id = 1"
    )
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    if let Some(s) = settings_opt {
        if page.content_markdown.contains("{{FEATURE_PRIVACY_SECTIONS}}") {
            let sections = feature_privacy_sections(&pool, &s).await;
            page.content_markdown = page.content_markdown.replace("{{FEATURE_PRIVACY_SECTIONS}}", &sections);
        }
        page.content_markdown = render_page_placeholders(&page.content_markdown, &s);
    }

    // If shipment policy, also bundle active shipping providers with zones and rates!
    if slug == "shipment-policy" {
        let providers = fetch_all_active_providers_with_rates(&pool).await?;
        return Ok(Json(json!({
            "page": page,
            "providers": providers
        })));
    }

    Ok(Json(json!({ "page": page })))
}

// 9. Public Shipping Providers & Rates
async fn public_get_shipping_providers(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let providers = fetch_all_active_providers_with_rates(&pool).await?;
    Ok(Json(providers))
}

async fn fetch_all_active_providers_with_rates(pool: &PgPool) -> Result<Vec<ShippingProviderWithZones>, (StatusCode, String)> {
    let providers = sqlx::query_as::<_, ShippingProvider>(
        "SELECT id, name, code, tracking_url_template, is_active, sort_order, created_at FROM shipping_providers WHERE is_active = true ORDER BY sort_order ASC, name ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for p in providers {
        let zones = sqlx::query_as::<_, ShippingZone>(
            "SELECT id, provider_id, zone_name, country_codes, is_default, created_at FROM shipping_zones WHERE provider_id = $1 ORDER BY is_default DESC, zone_name ASC"
        )
        .bind(p.id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut zones_with_rates = Vec::new();
        for z in zones {
            let rates = sqlx::query_as::<_, ShippingRate>(
                "SELECT id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days, max_length_mm, max_width_mm, max_height_mm FROM shipping_rates WHERE zone_id = $1 ORDER BY price_cents ASC"
            )
            .bind(z.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            zones_with_rates.push(ShippingZoneWithRates { zone: z, rates });
        }

        result.push(ShippingProviderWithZones { provider: p, zones: zones_with_rates });
    }

    Ok(result)
}

// 10. Public Product Parts / BOM
async fn public_get_product_parts(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let product_id: Uuid = sqlx::query_scalar("SELECT id FROM products WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let parts = sqlx::query_as::<_, ProductPart>(
        "SELECT id, product_id, variant_id, NULL::uuid AS part_id, part_name, part_sku, quantity, notes, NULL::varchar AS storage_location, created_at, 0 AS stock_quantity, 0 AS low_stock_threshold FROM product_parts WHERE product_id = $1 AND COALESCE(part_sku, '') <> 'DIGITAL_FILE' ORDER BY created_at ASC"
    )
    .bind(product_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(parts))
}

// 11. Customer Authentication & Account Actions
fn extract_customer_email(headers: &HeaderMap) -> Option<String> {
    let token = auth::bearer_token(headers)?;
    auth::verify_token(token, auth::ROLE_CUSTOMER_TOKEN).map(|c| c.sub)
}

fn customer_token(email: &str) -> Result<String, (StatusCode, String)> {
    auth::issue_token(email, auth::ROLE_CUSTOMER_TOKEN, Duration::days(30))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

fn normalize_email(email: &str) -> Result<String, (StatusCode, String)> {
    let email = email.trim().to_lowercase();
    if email.len() < 3 || email.len() > 254 || !email.contains('@') || email.contains(char::is_whitespace) {
        return Err((StatusCode::BAD_REQUEST, "Please enter a valid email address".to_string()));
    }
    Ok(email)
}

const MIN_CUSTOMER_PASSWORD: usize = 8;

async fn send_verification(pool: &PgPool, email: &str, name: &str, token: &str) -> Result<(), String> {
    let settings = sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    crate::services::email::send_verification_email(&settings, email, name, token, &crate::services::email::shop_public_url()).await
}

#[derive(Debug, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

/// Sends a new verification link to an unverified account. The answer never reveals whether the
/// account exists — except that email sending as a whole is unavailable.
async fn public_resend_verification(
    State(pool): State<PgPool>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = payload.email.trim().to_lowercase();

    // 1-minute cooldown per email address
    if let Err(remaining) = auth::check_resend_allowed(&email) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            format!("Please wait {} seconds before requesting another verification email.", remaining),
        ));
    }

    let limiter_key = format!("verify:{}", email);
    if auth::check_login_allowed(&limiter_key).is_err() {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please wait a few minutes before trying again.".to_string(),
        ));
    }
    auth::record_login_failure(&limiter_key); // caps resend emails per address

    let token = auth::random_token(32);
    let row = sqlx::query(
        "UPDATE customers SET verification_token = $1, updated_at = NOW() WHERE LOWER(email) = $2 AND is_verified = FALSE RETURNING email, display_name"
    )
    .bind(&token)
    .bind(&email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(r) = row {
        let to: String = r.get("email");
        let name: String = r.get("display_name");
        if let Err(e) = send_verification(&pool, &to, &name, &token).await {
            tracing::error!("Resending a verification email failed: {}", e);
            return Err((StatusCode::SERVICE_UNAVAILABLE, "Emails cannot be sent at the moment. Please try again later or contact the shop.".to_string()));
        }
    }
    // Cooldown and answer are the same whether or not the account exists (no account probing)
    auth::record_resend_sent(&email);

    Ok(Json(json!({
        "message": "If this address belongs to an unverified account, a new verification link is on its way. Please check your inbox and spam folder.",
        "cooldown_seconds": 60
    })))
}

async fn public_customer_register(
    State(pool): State<PgPool>,
    Json(payload): Json<CustomerRegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = normalize_email(&payload.email)?;
    if payload.password.len() < MIN_CUSTOMER_PASSWORD {
        return Err((StatusCode::BAD_REQUEST, format!("Password must be at least {} characters", MIN_CUSTOMER_PASSWORD)));
    }

    let full_name = payload.full_name.clone().unwrap_or_else(|| {
        let first = payload.first_name.clone().unwrap_or_default();
        let last = payload.last_name.clone().unwrap_or_default();
        format!("{} {}", first, last).trim().to_string()
    });
    let first_name = payload.first_name.clone().unwrap_or_else(|| full_name.split_whitespace().next().unwrap_or("").to_string());
    let last_name = payload.last_name.clone().unwrap_or_else(|| {
        let parts: Vec<&str> = full_name.split_whitespace().collect();
        if parts.len() > 1 { parts[1..].join(" ") } else { String::new() }
    });
    let display_name = if !full_name.is_empty() { full_name.clone() } else { email.split('@').next().unwrap_or("Customer").to_string() };

    let require_verification: bool = sqlx::query_scalar("SELECT require_email_verification FROM store_settings WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
    let verification_token = require_verification.then(|| auth::random_token(32));

    let hashed = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Never overwrite an existing account: registering with a taken email must not reset its password.
    let inserted = sqlx::query(
        r#"
        INSERT INTO customers (id, email, password_hash, first_name, last_name, display_name, preferred_currency, is_verified, verification_token)
        SELECT $1, $2, $3, $4, $5, $6, 'EUR', $7, $8
        WHERE NOT EXISTS (SELECT 1 FROM customers WHERE LOWER(email) = $2)
        ON CONFLICT (email) DO NOTHING
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&email)
    .bind(&hashed)
    .bind(first_name.trim())
    .bind(last_name.trim())
    .bind(display_name.trim())
    .bind(!require_verification)
    .bind(&verification_token)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .rows_affected();

    if inserted == 0 {
        return Err((StatusCode::CONFLICT, "An account with this email already exists. Please log in or reset your password.".to_string()));
    }

    if let Some(tok) = verification_token {
        // Sent before answering so the customer learns immediately if the email could not go out
        let sent = send_verification(&pool, &email, &display_name, &tok).await;
        if let Err(e) = &sent {
            tracing::error!("Verification email for a new customer could not be sent: {}", e);
        } else {
            auth::record_resend_sent(&email);
        }
        // No session until the email address is confirmed
        return Ok((StatusCode::CREATED, Json(json!({
            "email": email,
            "full_name": display_name,
            "is_verified": false,
            "verification_pending": true,
            "verification_email_sent": sent.is_ok()
        }))));
    }

    Ok((StatusCode::CREATED, Json(json!({
        "token": customer_token(&email)?,
        "email": email,
        "full_name": display_name,
        "first_name": first_name,
        "last_name": last_name,
        "preferred_currency": "EUR",
        "is_verified": true,
        "verification_pending": false
    }))))
}

async fn public_customer_login(
    State(pool): State<PgPool>,
    Json(payload): Json<CustomerLoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = payload.email.trim().to_lowercase();
    let limiter_key = format!("customer:{}", email);
    auth::check_login_allowed(&limiter_key).map_err(|m| (StatusCode::TOO_MANY_REQUESTS, m))?;

    let row = sqlx::query(
        "SELECT email, password_hash, first_name, last_name, display_name, preferred_currency, is_verified FROM customers WHERE LOWER(email) = $1"
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let hash = match &row {
        Some(r) => r.get::<String, _>("password_hash"),
        None => dummy_password_hash().to_string(), // equalise timing for unknown accounts
    };
    let valid = bcrypt::verify(&payload.password, &hash).unwrap_or(false);
    let Some(r) = row.filter(|_| valid) else {
        auth::record_login_failure(&limiter_key);
        return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
    };
    auth::clear_login_failures(&limiter_key);

    let require_verification: bool = sqlx::query_scalar("SELECT require_email_verification FROM store_settings WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
    if require_verification && !r.get::<bool, _>("is_verified") {
        return Err((
            StatusCode::FORBIDDEN,
            "Please verify your email address before logging in. Check your inbox for the activation link.".to_string(),
        ));
    }

    let stored_email: String = r.get("email");
    let display_name: String = r.get("display_name");
    Ok(Json(json!({
        "token": customer_token(&stored_email)?,
        "email": stored_email,
        "full_name": if !display_name.is_empty() { display_name } else { stored_email.split('@').next().unwrap_or("Customer").to_string() },
        "first_name": r.get::<String, _>("first_name"),
        "last_name": r.get::<String, _>("last_name"),
        "preferred_currency": r.get::<String, _>("preferred_currency")
    })))
}

fn dummy_password_hash() -> &'static str {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| bcrypt::hash(auth::random_token(16), bcrypt::DEFAULT_COST).unwrap_or_default())
}

/// Token-based password reset. The response never reveals whether an account exists.
async fn public_customer_reset_password(
    State(pool): State<PgPool>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Step 2: set a new password with the emailed token
    if let (Some(token), Some(new_pass)) = (payload.token.as_deref(), payload.new_password.as_deref()) {
        if new_pass.len() < MIN_CUSTOMER_PASSWORD {
            return Err((StatusCode::BAD_REQUEST, format!("Password must be at least {} characters", MIN_CUSTOMER_PASSWORD)));
        }
        let hashed = bcrypt::hash(new_pass, bcrypt::DEFAULT_COST).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let updated = sqlx::query(
            r#"
            UPDATE customers
            SET password_hash = $1, reset_token_hash = NULL, reset_token_expires_at = NULL, is_verified = TRUE, updated_at = NOW()
            WHERE reset_token_hash = $2 AND reset_token_expires_at > NOW()
            "#
        )
        .bind(&hashed)
        .bind(auth::sha256_hex(token.trim()))
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .rows_affected();

        if updated == 0 {
            return Err((StatusCode::BAD_REQUEST, "This reset link is invalid or has expired. Please request a new one.".to_string()));
        }
        return Ok(Json(json!({ "message": "Password successfully reset. You may now log in." })));
    }

    // Step 1: email a reset link (only the token's hash is stored)
    if let Some(email) = payload.email.as_deref().map(|e| e.trim().to_lowercase()).filter(|e| e.contains('@')) {
        let limiter_key = format!("reset:{}", email);
        if auth::check_login_allowed(&limiter_key).is_ok() {
            auth::record_login_failure(&limiter_key); // also caps reset emails per address
            let token = auth::random_token(32);
            let found = sqlx::query(
                "UPDATE customers SET reset_token_hash = $1, reset_token_expires_at = NOW() + INTERVAL '1 hour' WHERE LOWER(email) = $2 RETURNING email"
            )
            .bind(auth::sha256_hex(&token))
            .bind(&email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            if let Some(row) = found {
                let to: String = row.get("email");
                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    if let Ok(settings) = sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(&pool_clone).await {
                        let _ = crate::services::email::send_password_reset_email(&settings, &to, &token).await;
                    }
                });
            }
        }
    }

    Ok(Json(json!({ "message": "If an account exists for this email, a password reset link has been sent." })))
}

async fn public_customer_change_password(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<CustomerChangePasswordRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    if payload.new_password != payload.confirm_password {
        return Err((StatusCode::BAD_REQUEST, "New passwords do not match".to_string()));
    }
    if payload.new_password.len() < 6 {
        return Err((StatusCode::BAD_REQUEST, "New password must be at least 6 characters".to_string()));
    }

    // Fetch existing password hash from customers or users
    let hash: Option<String> = sqlx::query_scalar("SELECT password_hash FROM customers WHERE email = $1")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let existing_hash = match hash {
        Some(h) => h,
        None => {
            sqlx::query_scalar("SELECT password_hash FROM users WHERE email = $1")
                .bind(&email)
                .fetch_optional(&pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?
        }
    };

    let is_valid = bcrypt::verify(&payload.current_password, &existing_hash).unwrap_or(false);
    if !is_valid {
        return Err((StatusCode::BAD_REQUEST, "Current password is incorrect".to_string()));
    }

    let new_hash = bcrypt::hash(&payload.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE customers SET password_hash = $1, updated_at = NOW() WHERE email = $2")
        .bind(&new_hash)
        .bind(&email)
        .execute(&pool)
        .await
        .ok();

    sqlx::query("UPDATE users SET password_hash = $1 WHERE email = $2")
        .bind(&new_hash)
        .bind(&email)
        .execute(&pool)
        .await
        .ok();

    Ok(Json(json!({ "success": true, "message": "Password changed successfully" })))
}

async fn public_get_customer_profile(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let cust = sqlx::query(
        "SELECT id, email, first_name, last_name, display_name, preferred_currency, phone FROM customers WHERE email = $1"
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(r) = cust {
        return Ok(Json(json!({
            "id": r.get::<Uuid, _>("id"),
            "email": r.get::<String, _>("email"),
            "first_name": r.get::<String, _>("first_name"),
            "last_name": r.get::<String, _>("last_name"),
            "display_name": r.get::<String, _>("display_name"),
            "preferred_currency": r.get::<String, _>("preferred_currency"),
            "phone": r.get::<String, _>("phone"),
        })));
    }

    let user_row = sqlx::query("SELECT id, username, email FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(json!({
        "id": user_row.get::<Uuid, _>("id"),
        "email": user_row.get::<String, _>("email"),
        "first_name": "",
        "last_name": "",
        "display_name": user_row.get::<String, _>("username"),
        "preferred_currency": "EUR",
        "phone": "",
    })))
}

async fn public_update_customer_profile(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateCustomerProfileRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO customers (id, email, password_hash, first_name, last_name, display_name, preferred_currency, phone)
        VALUES ($1, $2, 'placeholder', $3, $4, $5, $6, $7)
        ON CONFLICT (email) DO UPDATE
        SET first_name = $3, last_name = $4, display_name = $5, preferred_currency = $6, phone = $7, updated_at = NOW()
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&email)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.display_name)
    .bind(&payload.preferred_currency)
    .bind(payload.phone.unwrap_or_default())
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn public_get_customer_orders(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let orders = sqlx::query(
        r#"
        SELECT 
            id, order_number, customer_name, customer_email, shipping_address,
            shipping_cost_cents, subtotal_cents, tax_cents, total_cents, 
            payment_provider, payment_status, order_status, tracking_number, notes, created_at 
        FROM orders 
        WHERE customer_email = $1 
        ORDER BY created_at DESC
        "#
    )
    .bind(&email)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut list = Vec::new();
    for r in orders {
        let order_id: Uuid = r.get("id");
        let items = sqlx::query_as::<_, OrderItem>(
            r#"
            SELECT 
                id, order_id, product_id, variant_id, product_title, variant_title, 
                sku, unit_price_cents, quantity, total_price_cents, is_digital, download_url
            FROM order_items
            WHERE order_id = $1
            ORDER BY id ASC
            "#
        )
        .bind(order_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        let shipping_addr: serde_json::Value = r.try_get("shipping_address").unwrap_or(json!({}));

        let mut items_with_files = Vec::new();
        let paid = r.get::<String, _>("payment_status") == "paid";
        for mut item in items {
            // Personal, counted download links for paid orders only; the storage address stays hidden
            let files = match (item.is_digital && paid, item.product_id) {
                (true, Some(pid)) => crate::routes::shop_features::order_download_files(&pool, order_id, pid, &item.product_title, item.download_url.as_deref()).await,
                _ => Vec::new(),
            };
            item.download_url = None;
            let mut item_json = serde_json::to_value(&item).unwrap_or(json!({}));
            if let Some(obj) = item_json.as_object_mut() {
                obj.insert("files".to_string(), json!(files));
            }
            items_with_files.push(item_json);
        }

        list.push(json!({
            "id": order_id,
            "order_number": r.get::<String, _>("order_number"),
            "customer_name": r.get::<String, _>("customer_name"),
            "customer_email": r.get::<String, _>("customer_email"),
            "shipping_address": shipping_addr,
            "shipping_cost_cents": r.get::<i32, _>("shipping_cost_cents"),
            "subtotal_cents": r.get::<i32, _>("subtotal_cents"),
            "tax_cents": r.get::<i32, _>("tax_cents"),
            "total_cents": r.get::<i32, _>("total_cents"),
            "payment_provider": r.get::<String, _>("payment_provider"),
            "payment_status": r.get::<String, _>("payment_status"),
            "order_status": r.get::<String, _>("order_status"),
            "tracking_number": r.get::<Option<String>, _>("tracking_number"),
            "notes": r.get::<Option<String>, _>("notes"),
            "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "items": items_with_files
        }));
    }

    Ok(Json(list))
}

async fn public_get_customer_wishlist(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers).unwrap_or_else(|| "guest@rustwebshop.local".to_string());

    let items = sqlx::query(
        r#"
        SELECT p.id, p.title, p.slug, p.category, p.subcategory, p.base_price_cents, p.image_url, p.product_type
        FROM customer_wishlist w
        JOIN products p ON w.product_id = p.id
        WHERE w.customer_email = $1
        ORDER BY w.created_at DESC
        "#
    )
    .bind(&email)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let list = items.into_iter().map(|r| {
        json!({
            "id": r.get::<Uuid, _>("id"),
            "title": r.get::<String, _>("title"),
            "slug": r.get::<String, _>("slug"),
            "category": r.get::<String, _>("category"),
            "subcategory": r.get::<String, _>("subcategory"),
            "base_price_cents": r.get::<i32, _>("base_price_cents"),
            "image_url": r.get::<String, _>("image_url"),
            "product_type": r.get::<String, _>("product_type"),
        })
    }).collect::<Vec<_>>();

    Ok(Json(list))
}

async fn public_toggle_wishlist(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<ToggleWishlistRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers).unwrap_or_else(|| "guest@rustwebshop.local".to_string());

    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM customer_wishlist WHERE customer_email = $1 AND product_id = $2"
    )
    .bind(&email)
    .bind(payload.product_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(id) = existing {
        sqlx::query("DELETE FROM customer_wishlist WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(json!({ "in_wishlist": false })))
    } else {
        sqlx::query("INSERT INTO customer_wishlist (id, customer_email, product_id) VALUES ($1, $2, $3)")
            .bind(Uuid::new_v4())
            .bind(&email)
            .bind(payload.product_id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(json!({ "in_wishlist": true })))
    }
}

async fn public_get_customer_addresses(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let addrs = sqlx::query_as::<_, CustomerAddress>(
        "SELECT id, customer_email, address_type, full_name, street_address, apartment_suite, city, state_province, postal_code, country_code, is_default, created_at FROM customer_addresses WHERE customer_email = $1 ORDER BY is_default DESC, created_at DESC"
    )
    .bind(&email)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(addrs))
}

async fn public_save_customer_address(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<SaveAddressRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO customer_addresses (id, customer_email, address_type, full_name, street_address, apartment_suite, city, state_province, postal_code, country_code, is_default)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#
    )
    .bind(id)
    .bind(&email)
    .bind(&payload.address_type)
    .bind(&payload.full_name)
    .bind(&payload.street_address)
    .bind(payload.apartment_suite)
    .bind(&payload.city)
    .bind(&payload.state_province)
    .bind(&payload.postal_code)
    .bind(&payload.country_code)
    .bind(payload.is_default.unwrap_or(true))
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn public_delete_customer_address(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    sqlx::query("DELETE FROM customer_addresses WHERE id = $1 AND customer_email = $2")
        .bind(id)
        .bind(&email)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn public_list_categories(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Category>(
        "SELECT id, parent_id, name, slug, description, display_order, image_url, created_at FROM categories ORDER BY display_order ASC, name ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

async fn public_verify_customer_email(
    State(pool): State<PgPool>,
    Query(query): Query<VerifyQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query(
        "UPDATE customers SET is_verified = true, verification_token = NULL, updated_at = NOW() WHERE verification_token = $1"
    )
    .bind(&query.token)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::BAD_REQUEST, "Invalid or expired verification token".to_string()));
    }

    Ok(Json(json!({ "success": true, "message": "Email verified successfully! You can now log in and place orders." })))
}

async fn public_subscribe_stock_notification(
    State(pool): State<PgPool>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<CreateStockNotificationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Valid email address is required".to_string()));
    }
    sqlx::query(
        r#"
        INSERT INTO stock_notifications (id, product_id, variant_id, customer_email)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(product_id)
    .bind(payload.variant_id)
    .bind(&payload.email.trim().to_lowercase())
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true, "message": "You will be notified when this item is back in stock!" })))
}

async fn public_get_related_products(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let current_product = sqlx::query_as::<_, Product>(
        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent, weight_grams, delivery_time, sale_starts_at, sale_ends_at, gpsr_manufacturer, gpsr_responsible_person, gpsr_safety_info, unit_content, unit_content_unit, length_mm, width_mm, height_mm FROM products WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let related = sqlx::query_as::<_, Product>(
        r#"
        SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent, weight_grams, delivery_time, sale_starts_at, sale_ends_at, gpsr_manufacturer, gpsr_responsible_person, gpsr_safety_info, unit_content, unit_content_unit, length_mm, width_mm, height_mm
        FROM products 
        WHERE is_active = true AND id != $1
        ORDER BY (category = $2) DESC, (subcategory = $3) DESC, created_at DESC
        LIMIT 5
        "#
    )
    .bind(id)
    .bind(&current_product.category)
    .bind(&current_product.subcategory)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut result = Vec::new();
    for p in related {
        let variants = sqlx::query_as::<_, ProductVariant>(
            "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at, images, weight_grams, sale_price_cents, length_mm, width_mm, height_mm FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
        )
        .bind(p.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        result.push(ProductWithVariants { product: public_product(p), variants });
    }

    attach_pricing(&pool, &mut result).await;
    Ok(Json(result))
}

async fn public_get_carousels(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let config_val: serde_json::Value = sqlx::query_scalar("SELECT carousels_config FROM store_settings WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| json!({
            "sections": [
                { "id": "featured", "title": "Featured Products", "enabled": true, "product_ids": [] },
                { "id": "new", "title": "New Arrivals", "enabled": true, "days": 30 },
                { "id": "bestsellers", "title": "Best Sellers", "enabled": true, "limit": 10 },
                { "id": "catalog", "title": "In Stock Hardware & Gear", "enabled": true }
            ]
        }));

    let sections = config_val.get("sections").and_then(|s| s.as_array()).cloned().unwrap_or_default();
    let mut carousels = Vec::new();

    for sec in sections {
        let enabled = sec.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        if !enabled {
            continue;
        }
        let sec_id = sec.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let title = sec.get("title").and_then(|v| v.as_str()).unwrap_or("Products");

        let mut products: Vec<Product> = Vec::new();

        match sec_id {
            "featured" => {
                let ids: Vec<Uuid> = sec.get("product_ids")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).filter_map(|s| Uuid::parse_str(s).ok()).collect())
                    .unwrap_or_default();
                if !ids.is_empty() {
                    products = sqlx::query_as::<_, Product>(
                        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent, weight_grams, delivery_time, sale_starts_at, sale_ends_at, gpsr_manufacturer, gpsr_responsible_person, gpsr_safety_info, unit_content, unit_content_unit, length_mm, width_mm, height_mm FROM products WHERE is_active = true AND id = ANY($1)"
                    )
                    .bind(&ids)
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();
                } else {
                    products = sqlx::query_as::<_, Product>(
                        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent, weight_grams, delivery_time, sale_starts_at, sale_ends_at, gpsr_manufacturer, gpsr_responsible_person, gpsr_safety_info, unit_content, unit_content_unit, length_mm, width_mm, height_mm FROM products WHERE is_active = true ORDER BY created_at DESC LIMIT 10"
                    )
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();
                }
            }
            "new" => {
                let days = sec.get("days").and_then(|v| v.as_i64()).unwrap_or(30);
                let limit = sec.get("limit").and_then(|v| v.as_i64()).unwrap_or(12);
                products = sqlx::query_as::<_, Product>(
                    "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent, weight_grams, delivery_time, sale_starts_at, sale_ends_at, gpsr_manufacturer, gpsr_responsible_person, gpsr_safety_info, unit_content, unit_content_unit, length_mm, width_mm, height_mm FROM products WHERE is_active = true AND created_at >= NOW() - ($1 || ' days')::INTERVAL ORDER BY created_at DESC LIMIT $2"
                )
                .bind(days.to_string())
                .bind(limit)
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
            }
            "bestsellers" => {
                let limit = sec.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
                products = sqlx::query_as::<_, Product>(
                    r#"
                    SELECT p.id, p.title, p.slug, p.description, p.product_type, p.category, p.subcategory,
                           p.base_price_cents, p.digital_download_url, p.image_url, p.is_active, p.created_at, p.updated_at,
                           p.subtitle, p.variant_selector_label, p.short_description, p.long_description, p.images, p.has_multiple_variants,
                           p.tax_rate_percent, p.weight_grams, p.delivery_time, p.sale_starts_at, p.sale_ends_at, p.gpsr_manufacturer,
                           p.gpsr_responsible_person, p.gpsr_safety_info, p.unit_content, p.unit_content_unit, p.length_mm, p.width_mm, p.height_mm
                    FROM products p
                    LEFT JOIN order_items oi ON oi.product_id = p.id
                    WHERE p.is_active = true
                    GROUP BY p.id
                    ORDER BY COALESCE(SUM(oi.quantity), 0) DESC, p.created_at DESC
                    LIMIT $1
                    "#
                )
                .bind(limit)
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
            }
            "catalog" => {
                let limit = sec.get("limit").and_then(|v| v.as_i64()).unwrap_or(24);
                products = sqlx::query_as::<_, Product>(
                    r#"
                    SELECT DISTINCT p.id, p.title, p.slug, p.description, p.product_type, p.category, p.subcategory,
                           p.base_price_cents, p.digital_download_url, p.image_url, p.is_active, p.created_at, p.updated_at,
                           p.subtitle, p.variant_selector_label, p.short_description, p.long_description, p.images, p.has_multiple_variants,
                           p.tax_rate_percent, p.weight_grams, p.delivery_time, p.sale_starts_at, p.sale_ends_at, p.gpsr_manufacturer,
                           p.gpsr_responsible_person, p.gpsr_safety_info, p.unit_content, p.unit_content_unit, p.length_mm, p.width_mm, p.height_mm
                    FROM products p
                    JOIN product_variants pv ON pv.product_id = p.id
                    WHERE p.is_active = true AND (p.product_type = 'digital' OR pv.stock_quantity > 0)
                    ORDER BY p.created_at DESC
                    LIMIT $1
                    "#
                )
                .bind(limit)
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
            }
            _ => {}
        }

        let mut items_with_variants = Vec::new();
        for product in products {
            let variants = sqlx::query_as::<_, ProductVariant>(
                "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at, images, weight_grams, sale_price_cents, length_mm, width_mm, height_mm FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
            )
            .bind(product.id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            items_with_variants.push(ProductWithVariants { product: public_product(product), variants });
        }
        attach_pricing(&pool, &mut items_with_variants).await;

        carousels.push(json!({
            "id": sec_id,
            "title": title,
            "items": items_with_variants
        }));
    }

    Ok(Json(carousels))
}

// ==========================================
// Coupon Code Validation
// ==========================================

async fn public_validate_coupon(
    State(pool): State<PgPool>,
    Json(payload): Json<ValidateCouponRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let clean_code = payload.code.trim().to_uppercase();
    if clean_code.is_empty() {
        return Ok(Json(ValidateCouponResponse {
            valid: false,
            code: String::new(),
            discount_type: String::new(),
            discount_cents: 0,
            message: "Please enter a promo code".to_string(),
        }));
    }

    let coupon_opt = sqlx::query_as::<_, Coupon>(
        "SELECT * FROM coupons WHERE UPPER(code) = UPPER($1) AND is_active = true"
    )
    .bind(&clean_code)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let coupon = match coupon_opt {
        Some(c) => c,
        None => {
            return Ok(Json(ValidateCouponResponse {
                valid: false,
                code: clean_code,
                discount_type: String::new(),
                discount_cents: 0,
                message: "Coupon code is invalid or not active".to_string(),
            }));
        }
    };

    if let Some(exp) = coupon.expires_at {
        if exp < chrono::Utc::now() {
            return Ok(Json(ValidateCouponResponse {
                valid: false,
                code: clean_code,
                discount_type: coupon.discount_type,
                discount_cents: 0,
                message: "This coupon code has expired".to_string(),
            }));
        }
    }

    if let Some(max_u) = coupon.max_uses {
        if coupon.used_count >= max_u {
            return Ok(Json(ValidateCouponResponse {
                valid: false,
                code: clean_code,
                discount_type: coupon.discount_type,
                discount_cents: 0,
                message: "This coupon code has reached its maximum usage limit".to_string(),
            }));
        }
    }

    if payload.subtotal_cents < coupon.min_order_cents {
        return Ok(Json(ValidateCouponResponse {
            valid: false,
            code: clean_code,
            discount_type: coupon.discount_type,
            discount_cents: 0,
            message: format!(
                "Order subtotal of {:.2} € is below the minimum {:.2} € required for this code",
                (payload.subtotal_cents as f64) / 100.0,
                (coupon.min_order_cents as f64) / 100.0
            ),
        }));
    }

    let shipping_cents = payload.shipping_cost_cents.unwrap_or(0);
    let (discount_cents, msg) = match coupon.discount_type.as_str() {
        "free_shipping" => {
            (shipping_cents, "Free shipping coupon applied!".to_string())
        }
        "fixed_amount" => {
            let disc = std::cmp::min(coupon.value_cents, payload.subtotal_cents);
            (disc, format!("{:.2} € discount applied!", (disc as f64) / 100.0))
        }
        "percentage" => {
            let pct = std::cmp::min(100, std::cmp::max(0, coupon.value_cents));
            let disc = ((payload.subtotal_cents as f64) * (pct as f64 / 100.0)).round() as i32;
            (disc, format!("{}% discount applied!", pct))
        }
        _ => (0, "Coupon applied".to_string()),
    };

    Ok(Json(ValidateCouponResponse {
        valid: true,
        code: coupon.code,
        discount_type: coupon.discount_type,
        discount_cents,
        message: msg,
    }))
}

// ==========================================
// Customer Digital Downloads Overview
// Dynamic resolution from latest product records and BOM
// ==========================================

#[derive(Serialize)]
struct CustomerDownloadProduct {
    product_id: Option<Uuid>,
    product_title: String,
    image_url: Option<String>,
    order_id: Uuid,
    order_number: String,
    purchase_date: chrono::DateTime<chrono::Utc>,
    files: Vec<serde_json::Value>,
}

async fn public_get_customer_downloads(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // Query all digital order items from paid/completed orders for this customer
    let rows = sqlx::query(
        r#"
        SELECT 
            oi.id as order_item_id, oi.product_id, oi.product_title, oi.download_url as original_download_url,
            o.id as order_id, o.order_number, o.created_at as purchase_date,
            p.title as live_product_title, p.digital_download_url as live_download_url, p.image_url as live_image_url
        FROM order_items oi
        JOIN orders o ON o.id = oi.order_id
        LEFT JOIN products p ON p.id = oi.product_id
        WHERE o.customer_email = $1 
          AND o.payment_status = 'paid'
          AND oi.is_digital = true
        ORDER BY o.created_at DESC
        "#
    )
    .bind(&email)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();

    for r in rows {
        let product_id: Option<Uuid> = r.get("product_id");
        let order_id: Uuid = r.get("order_id");
        let order_number: String = r.get("order_number");
        let purchase_date: chrono::DateTime<chrono::Utc> = r.get("purchase_date");

        // Prefer live product title and image if product still exists
        let live_title: Option<String> = r.try_get("live_product_title").ok().flatten();
        let fallback_title: String = r.get("product_title");
        let product_title = live_title.unwrap_or(fallback_title);
        let image_url: Option<String> = r.try_get("live_image_url").ok().flatten();

        // Latest files of the product, each behind a personal, counted download link
        let orig_url: Option<String> = r.try_get("original_download_url").ok().flatten();
        let files = match product_id {
            Some(pid) => crate::routes::shop_features::order_download_files(&pool, order_id, pid, &product_title, orig_url.as_deref()).await,
            None => Vec::new(),
        };

        result.push(CustomerDownloadProduct {
            product_id,
            product_title,
            image_url,
            order_id,
            order_number,
            purchase_date,
            files,
        });
    }

    Ok(Json(result))
}


/// Sale, regular and unit prices of every variant (see `services::pricing`).
async fn attach_pricing(pool: &PgPool, products: &mut [ProductWithVariants]) {
    if let Ok(settings) = sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await {
        crate::services::pricing::attach(pool, &settings, products).await;
    }
}

/// Privacy-policy sections for the features this shop actually uses (`{{FEATURE_PRIVACY_SECTIONS}}`).
async fn feature_privacy_sections(pool: &PgPool, s: &StoreSettings) -> String {
    let mut out: Vec<String> = Vec::new();
    let providers: Vec<String> = sqlx::query_scalar("SELECT provider FROM payment_configs WHERE is_enabled = true ORDER BY provider")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    if !providers.is_empty() {
        let mut lines = vec!["### Payment Providers".to_string(),
            "To process your payment we pass the data required for it (name, email address, billing address, order amount and order number) to the payment provider you choose (Art. 6 Para. 1 lit. b GDPR):".to_string(), String::new()];
        for p in &providers {
            match p.as_str() {
                "stripe" => lines.push("- **Stripe** (Stripe Payments Europe, Ltd., 1 Grand Canal Street Lower, Dublin 2, Ireland) for card, wallet and bank payments.".to_string()),
                "paypal" => lines.push("- **PayPal** (PayPal (Europe) S.à r.l. et Cie, S.C.A., 22–24 Boulevard Royal, 2449 Luxembourg).".to_string()),
                other => lines.push(format!("- {}", other)),
            }
        }
        out.push(lines.join("\n"));
    }
    out.push("### Emails\nWe send order confirmations, invoices, shipping and account emails to the email address you provide (Art. 6 Para. 1 lit. b GDPR). The emails are delivered through our email provider.".to_string());
    out.push("### Withdrawal Function\nIf you withdraw from a contract using the withdrawal function of this shop, we process the details you enter (name, email address, order number, message) to handle your withdrawal and confirm its receipt (Art. 6 Para. 1 lit. c GDPR).".to_string());
    if s.newsletter_enabled {
        out.push("### Newsletter\nIf you subscribe to our newsletter, we only send it after you have confirmed your email address with the link in our confirmation email (double opt-in). The legal basis is your consent (Art. 6 Para. 1 lit. a GDPR), which you can withdraw at any time with the unsubscribe link in every newsletter. To prove your consent we store the time and IP address of your sign-up and of your confirmation (Art. 6 Para. 1 lit. f GDPR). Sign-ups that are not confirmed are deleted after 30 days; after you unsubscribe we keep only the proof of consent and of the unsubscription for up to three years.".to_string());
    }
    if s.reviews_enabled {
        out.push("### Product Reviews\nOnly customers who bought a product can review it. With your review we publish your rating, the text and the name you choose (by default your first name and the initial of your last name). Reviews are checked before publication. The legal basis is your consent (Art. 6 Para. 1 lit. a GDPR); you can ask us to delete your review at any time.".to_string());
    }
    if crate::services::vies::available(s) {
        out.push("### VAT ID Check\nIf you order as a business with a VAT identification number, we check it with the VAT Information Exchange System (VIES) of the European Commission and keep the result as proof required by tax law (Art. 6 Para. 1 lit. c GDPR).".to_string());
    }
    let backups: Option<bool> = sqlx::query_scalar("SELECT enabled FROM backup_settings WHERE id = 1").fetch_optional(pool).await.ok().flatten();
    if backups == Some(true) {
        out.push("### Backups\nTo protect your data against loss we store backups of our shop database in a private repository at GitHub (GitHub B.V., Netherlands / GitHub, Inc., USA). GitHub is certified under the EU-U.S. Data Privacy Framework. The legal basis is our legitimate interest in data security (Art. 6 Para. 1 lit. f, Art. 32 GDPR).".to_string());
    }
    out.push("### Storage Duration\nWe store order and invoice data for as long as commercial and tax law requires (invoices and accounting records 8 years, business letters 6 years, § 147 AO, § 257 HGB). Customer account data is stored until you delete your account or ask us to delete it; data without a retention obligation is deleted as soon as it is no longer needed.".to_string());
    out.push("### Right to Lodge a Complaint\nYou have the right to lodge a complaint with a data protection supervisory authority, in particular in the EU member state of your residence, your place of work or the place of the alleged infringement (Art. 77 GDPR).".to_string());
    out.join("\n\n")
}
