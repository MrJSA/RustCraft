//! Optional shop features and consumer-law functions:
//! product reviews (verified buyers, approval required), double opt-in newsletter, the withdrawal
//! button, protected digital downloads and the EU VAT ID check.

use crate::middleware::CurrentAdmin;
use crate::models::StoreSettings;
use crate::services::auth;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

type ApiResult<T> = Result<T, (StatusCode, String)>;

fn db_err(e: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn bad(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.to_string())
}

async fn settings(pool: &PgPool) -> ApiResult<StoreSettings> {
    sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await.map_err(db_err)
}

fn customer_email(headers: &HeaderMap) -> Option<String> {
    auth::bearer_token(headers).and_then(|t| auth::verify_token(t, auth::ROLE_CUSTOMER_TOKEN)).map(|c| c.sub.to_lowercase())
}

/// Client address as forwarded by the storefront (proof for the double opt-in)
fn client_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-shop-client-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(64).collect())
}

fn valid_email(email: &str) -> bool {
    let e = email.trim();
    e.len() <= 254 && !e.contains(char::is_whitespace) && e.split_once('@').is_some_and(|(l, d)| !l.is_empty() && d.contains('.'))
}

fn sha256_hex(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

pub fn public_router() -> Router<PgPool> {
    Router::new()
        .route("/products/:id/reviews", get(list_reviews).post(create_review))
        .route("/products/:id/reviews/eligibility", get(review_eligibility))
        .route("/newsletter/subscribe", post(newsletter_subscribe))
        .route("/newsletter/confirm", post(newsletter_confirm))
        .route("/newsletter/unsubscribe", post(newsletter_unsubscribe))
        .route("/withdrawals", post(create_withdrawal))
        .route("/downloads/:token", get(download_file))
        .route("/checkout/vat-check", post(vat_check))
        .route("/checkout/shipping-options", post(shipping_options))
}

pub fn admin_routes() -> Router<PgPool> {
    Router::new()
        .route("/reviews", get(admin_list_reviews))
        .route("/reviews/:id", put(admin_moderate_review).delete(admin_delete_review))
        .route("/newsletter/subscribers", get(admin_list_subscribers))
        .route("/newsletter/subscribers/:id", delete(admin_delete_subscriber))
        .route("/newsletter/campaigns", get(admin_list_campaigns).post(admin_create_campaign))
        .route("/newsletter/campaigns/:id/send", post(admin_send_campaign))
        .route("/newsletter/test", post(admin_test_campaign))
        .route("/withdrawals", get(admin_list_withdrawals))
        .route("/withdrawals/:id", put(admin_update_withdrawal))
        .route("/orders/:id/downloads", get(admin_order_downloads))
        .route("/orders/:id/downloads/reset", post(admin_reset_downloads))
}

// ================================================================== reviews

#[derive(Deserialize)]
struct ReviewInput {
    rating: i16,
    #[serde(default)]
    title: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    author_name: String,
}

async fn list_reviews(State(pool): State<PgPool>, Path(product_id): Path<Uuid>) -> ApiResult<impl IntoResponse> {
    let s = settings(&pool).await?;
    if !s.reviews_enabled {
        return Ok(Json(json!({ "enabled": false, "reviews": [], "count": 0, "average": null })));
    }
    let rows = sqlx::query(
        "SELECT author_name, rating, title, body, created_at FROM product_reviews WHERE product_id = $1 AND status = 'approved' ORDER BY created_at DESC LIMIT 200",
    )
    .bind(product_id)
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;
    let reviews: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "author_name": r.get::<String, _>("author_name"),
                "rating": r.get::<i16, _>("rating"),
                "title": r.get::<String, _>("title"),
                "body": r.get::<String, _>("body"),
                "created_at": r.get::<DateTime<Utc>, _>("created_at"),
            })
        })
        .collect();
    let count = reviews.len();
    let average = (count > 0).then(|| rows.iter().map(|r| r.get::<i16, _>("rating") as f64).sum::<f64>() / count as f64);
    Ok(Json(json!({ "enabled": true, "reviews": reviews, "count": count, "average": average })))
}

/// The customer's paid order containing this product (only verified buyers may review).
async fn purchase_for_review(pool: &PgPool, email: &str, product_id: Uuid) -> ApiResult<Option<Uuid>> {
    sqlx::query_scalar(
        "SELECT o.id FROM orders o JOIN order_items oi ON oi.order_id = o.id
         WHERE LOWER(o.customer_email) = $1 AND oi.product_id = $2 AND o.payment_status = 'paid'
         ORDER BY o.created_at DESC LIMIT 1",
    )
    .bind(email)
    .bind(product_id)
    .fetch_optional(pool)
    .await
    .map_err(db_err)
}

async fn review_eligibility(State(pool): State<PgPool>, headers: HeaderMap, Path(product_id): Path<Uuid>) -> ApiResult<impl IntoResponse> {
    let Some(email) = customer_email(&headers) else {
        return Ok(Json(json!({ "can_review": false, "reason": "login" })));
    };
    let existing: Option<String> = sqlx::query_scalar("SELECT status FROM product_reviews WHERE product_id = $1 AND customer_email = $2")
        .bind(product_id)
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(db_err)?;
    if let Some(status) = existing {
        return Ok(Json(json!({ "can_review": false, "reason": "already_reviewed", "status": status })));
    }
    let bought = purchase_for_review(&pool, &email, product_id).await?.is_some();
    Ok(Json(json!({ "can_review": bought, "reason": if bought { "ok" } else { "not_purchased" } })))
}

async fn create_review(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(product_id): Path<Uuid>,
    Json(input): Json<ReviewInput>,
) -> ApiResult<impl IntoResponse> {
    let s = settings(&pool).await?;
    if !s.reviews_enabled {
        return Err((StatusCode::NOT_FOUND, "Reviews are not enabled".to_string()));
    }
    let email = customer_email(&headers).ok_or((StatusCode::UNAUTHORIZED, "Please log in to write a review".to_string()))?;
    if !(1..=5).contains(&input.rating) {
        return Err(bad("Please choose 1 to 5 stars"));
    }
    let order_id = purchase_for_review(&pool, &email, product_id)
        .await?
        .ok_or((StatusCode::FORBIDDEN, "Only customers who bought this product can review it".to_string()))?;
    let author: String = input.author_name.trim().chars().take(80).collect();
    let author = if author.is_empty() {
        // First name + initial of the last name from the account
        let names: Option<(String, String)> = sqlx::query_as("SELECT COALESCE(first_name, ''), COALESCE(last_name, '') FROM customers WHERE LOWER(email) = $1")
            .bind(&email)
            .fetch_optional(&pool)
            .await
            .map_err(db_err)?;
        let (first, last) = names.unwrap_or_default();
        match (first.trim(), last.trim().chars().next()) {
            ("", _) => "Verified buyer".to_string(),
            (first, Some(initial)) => format!("{} {}.", first, initial),
            (first, None) => first.to_string(),
        }
    } else {
        author
    };
    sqlx::query(
        "INSERT INTO product_reviews (product_id, order_id, customer_email, author_name, rating, title, body)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(product_id)
    .bind(order_id)
    .bind(&email)
    .bind(&author)
    .bind(input.rating)
    .bind(input.title.trim().chars().take(150).collect::<String>())
    .bind(input.body.trim().chars().take(5000).collect::<String>())
    .execute(&pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(d) if d.code().as_deref() == Some("23505") => (StatusCode::CONFLICT, "You have already reviewed this product".to_string()),
        _ => db_err(e),
    })?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "message": "Thank you! Your review will be published after a short check." }))))
}

#[derive(Deserialize)]
struct ReviewFilter {
    status: Option<String>,
}

async fn admin_list_reviews(State(pool): State<PgPool>, Query(f): Query<ReviewFilter>) -> ApiResult<impl IntoResponse> {
    let rows = sqlx::query(
        "SELECT r.id, r.product_id, p.title AS product_title, r.customer_email, r.author_name, r.rating, r.title, r.body, r.status, r.created_at
         FROM product_reviews r LEFT JOIN products p ON p.id = r.product_id
         WHERE ($1::text IS NULL OR r.status = $1) ORDER BY (r.status = 'pending') DESC, r.created_at DESC LIMIT 500",
    )
    .bind(f.status.filter(|s| !s.is_empty()))
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;
    let list: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "product_id": r.get::<Uuid, _>("product_id"),
                "product_title": r.get::<Option<String>, _>("product_title"),
                "customer_email": r.get::<String, _>("customer_email"),
                "author_name": r.get::<String, _>("author_name"),
                "rating": r.get::<i16, _>("rating"),
                "title": r.get::<String, _>("title"),
                "body": r.get::<String, _>("body"),
                "status": r.get::<String, _>("status"),
                "created_at": r.get::<DateTime<Utc>, _>("created_at"),
            })
        })
        .collect();
    Ok(Json(list))
}

#[derive(Deserialize)]
struct ModerateInput {
    status: String,
}

async fn admin_moderate_review(State(pool): State<PgPool>, Path(id): Path<Uuid>, Json(input): Json<ModerateInput>) -> ApiResult<impl IntoResponse> {
    if !["approved", "rejected", "pending"].contains(&input.status.as_str()) {
        return Err(bad("Status must be approved, rejected or pending"));
    }
    sqlx::query("UPDATE product_reviews SET status = $1, moderated_at = NOW() WHERE id = $2")
        .bind(&input.status)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db_err)?;
    Ok(Json(json!({ "success": true })))
}

async fn admin_delete_review(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> ApiResult<impl IntoResponse> {
    sqlx::query("DELETE FROM product_reviews WHERE id = $1").bind(id).execute(&pool).await.map_err(db_err)?;
    Ok(Json(json!({ "success": true })))
}

// ================================================================== newsletter (double opt-in)

pub const NEWSLETTER_CONSENT_TEXT: &str =
    "I would like to receive the newsletter with offers and news by email. I can unsubscribe at any time with the link in every newsletter.";

#[derive(Deserialize)]
struct SubscribeInput {
    email: String,
    consent: bool,
    #[serde(default)]
    source: Option<String>,
}

async fn newsletter_subscribe(State(pool): State<PgPool>, headers: HeaderMap, Json(input): Json<SubscribeInput>) -> ApiResult<impl IntoResponse> {
    let s = settings(&pool).await?;
    if !s.newsletter_enabled {
        return Err((StatusCode::NOT_FOUND, "The newsletter is not available".to_string()));
    }
    if !input.consent {
        return Err(bad("Please confirm that you want to receive the newsletter"));
    }
    let email = input.email.trim().to_lowercase();
    if !valid_email(&email) {
        return Err(bad("Please enter a valid email address"));
    }
    let generic = Json(json!({
        "success": true,
        "message": "Almost done! Please check your inbox and click the confirmation link to activate your subscription."
    }));

    let existing = sqlx::query("SELECT status, requested_at FROM newsletter_subscribers WHERE email = $1")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(db_err)?;
    if let Some(row) = &existing {
        let status: String = row.get("status");
        let requested: DateTime<Utc> = row.get("requested_at");
        // Already subscribed, or a confirmation was sent very recently: same answer, no extra email
        if status == "confirmed" || (status == "pending" && Utc::now() - requested < Duration::minutes(10)) {
            return Ok(generic);
        }
    }

    let token = auth::random_token(32);
    let source: String = input.source.unwrap_or_else(|| "footer".to_string()).chars().take(30).collect();
    sqlx::query(
        "INSERT INTO newsletter_subscribers (email, status, confirm_token_hash, unsubscribe_token, consent_text, source, requested_at, requested_ip)
         VALUES ($1, 'pending', $2, $3, $4, $5, NOW(), $6)
         ON CONFLICT (email) DO UPDATE SET status = 'pending', confirm_token_hash = $2, consent_text = $4, source = $5,
             requested_at = NOW(), requested_ip = $6, confirmed_at = NULL, confirmed_ip = NULL, unsubscribed_at = NULL",
    )
    .bind(&email)
    .bind(sha256_hex(&token))
    .bind(auth::random_token(32))
    .bind(NEWSLETTER_CONSENT_TEXT)
    .bind(&source)
    .bind(client_ip(&headers))
    .execute(&pool)
    .await
    .map_err(db_err)?;

    crate::services::email::send_newsletter_confirmation_email(&s, &email, &token)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("The confirmation email could not be sent: {}", e)))?;
    Ok(generic)
}

#[derive(Deserialize)]
struct TokenInput {
    token: String,
}

async fn newsletter_confirm(State(pool): State<PgPool>, headers: HeaderMap, Json(input): Json<TokenInput>) -> ApiResult<impl IntoResponse> {
    // Confirmation links are valid for 7 days
    let updated = sqlx::query(
        "UPDATE newsletter_subscribers SET status = 'confirmed', confirmed_at = NOW(), confirmed_ip = $2, confirm_token_hash = NULL
         WHERE confirm_token_hash = $1 AND status = 'pending' AND requested_at > NOW() - INTERVAL '7 days'",
    )
    .bind(sha256_hex(input.token.trim()))
    .bind(client_ip(&headers))
    .execute(&pool)
    .await
    .map_err(db_err)?;
    if updated.rows_affected() == 0 {
        return Err(bad("This confirmation link is invalid, already used or expired. Please subscribe again."));
    }
    Ok(Json(json!({ "success": true, "message": "Thank you — your newsletter subscription is confirmed." })))
}

async fn newsletter_unsubscribe(State(pool): State<PgPool>, Json(input): Json<TokenInput>) -> ApiResult<impl IntoResponse> {
    sqlx::query(
        "UPDATE newsletter_subscribers SET status = 'unsubscribed', unsubscribed_at = NOW(), confirm_token_hash = NULL
         WHERE unsubscribe_token = $1 AND status <> 'unsubscribed'",
    )
    .bind(input.token.trim())
    .execute(&pool)
    .await
    .map_err(db_err)?;
    // Same answer for unknown tokens: nothing to reveal
    Ok(Json(json!({ "success": true, "message": "You have been unsubscribed. You will not receive any more newsletters." })))
}

async fn admin_list_subscribers(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    // Unconfirmed sign-ups older than 30 days are deleted (no consent → no reason to keep them)
    let _ = sqlx::query("DELETE FROM newsletter_subscribers WHERE status = 'pending' AND requested_at < NOW() - INTERVAL '30 days'")
        .execute(&pool)
        .await;
    let rows = sqlx::query(
        "SELECT id, email, status, source, consent_text, requested_at, requested_ip, confirmed_at, confirmed_ip, unsubscribed_at
         FROM newsletter_subscribers ORDER BY requested_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(db_err)?;
    let list: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "email": r.get::<String, _>("email"),
                "status": r.get::<String, _>("status"),
                "source": r.get::<String, _>("source"),
                "consent_text": r.get::<String, _>("consent_text"),
                "requested_at": r.get::<DateTime<Utc>, _>("requested_at"),
                "requested_ip": r.get::<Option<String>, _>("requested_ip"),
                "confirmed_at": r.get::<Option<DateTime<Utc>>, _>("confirmed_at"),
                "confirmed_ip": r.get::<Option<String>, _>("confirmed_ip"),
                "unsubscribed_at": r.get::<Option<DateTime<Utc>>, _>("unsubscribed_at"),
            })
        })
        .collect();
    Ok(Json(list))
}

async fn admin_delete_subscriber(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> ApiResult<impl IntoResponse> {
    sqlx::query("DELETE FROM newsletter_subscribers WHERE id = $1").bind(id).execute(&pool).await.map_err(db_err)?;
    Ok(Json(json!({ "success": true })))
}

#[derive(Deserialize)]
struct CampaignInput {
    subject: String,
    body_markdown: String,
    #[serde(default)]
    to: Option<String>,
}

async fn admin_list_campaigns(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    let rows = sqlx::query("SELECT * FROM newsletter_campaigns ORDER BY created_at DESC LIMIT 100").fetch_all(&pool).await.map_err(db_err)?;
    let list: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "subject": r.get::<String, _>("subject"),
                "body_markdown": r.get::<String, _>("body_markdown"),
                "status": r.get::<String, _>("status"),
                "recipients": r.get::<i32, _>("recipients"),
                "sent_count": r.get::<i32, _>("sent_count"),
                "failed_count": r.get::<i32, _>("failed_count"),
                "created_by": r.get::<Option<String>, _>("created_by"),
                "created_at": r.get::<DateTime<Utc>, _>("created_at"),
                "sent_at": r.get::<Option<DateTime<Utc>>, _>("sent_at"),
            })
        })
        .collect();
    Ok(Json(list))
}

fn check_campaign(input: &CampaignInput) -> ApiResult<()> {
    if input.subject.trim().is_empty() || input.body_markdown.trim().is_empty() {
        return Err(bad("Subject and text are required"));
    }
    Ok(())
}

async fn admin_create_campaign(
    State(pool): State<PgPool>,
    Extension(admin): Extension<CurrentAdmin>,
    Json(input): Json<CampaignInput>,
) -> ApiResult<impl IntoResponse> {
    check_campaign(&input)?;
    let id: Uuid = sqlx::query_scalar("INSERT INTO newsletter_campaigns (subject, body_markdown, created_by) VALUES ($1, $2, $3) RETURNING id")
        .bind(input.subject.trim())
        .bind(&input.body_markdown)
        .bind(&admin.username)
        .fetch_one(&pool)
        .await
        .map_err(db_err)?;
    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn admin_test_campaign(State(pool): State<PgPool>, Extension(admin): Extension<CurrentAdmin>, Json(input): Json<CampaignInput>) -> ApiResult<impl IntoResponse> {
    check_campaign(&input)?;
    let s = settings(&pool).await?;
    let to = input.to.clone().or(admin.email.clone()).filter(|e| valid_email(e)).ok_or(bad("Enter an email address for the test"))?;
    crate::services::email::send_newsletter_email(&s, &to, input.subject.trim(), &input.body_markdown, "test-unsubscribe-token")
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(json!({ "success": true, "message": format!("Test sent to {}", to) })))
}

async fn admin_send_campaign(State(pool): State<PgPool>, Extension(admin): Extension<CurrentAdmin>, Path(id): Path<Uuid>) -> ApiResult<impl IntoResponse> {
    let s = settings(&pool).await?;
    if !s.newsletter_enabled {
        return Err(bad("Switch the newsletter on in Settings → Shop Features & Legal first"));
    }
    // Only drafts can be sent, and only once
    let campaign = sqlx::query("UPDATE newsletter_campaigns SET status = 'sending' WHERE id = $1 AND status = 'draft' RETURNING subject, body_markdown")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(db_err)?
        .ok_or((StatusCode::CONFLICT, "This newsletter was already sent".to_string()))?;
    let subject: String = campaign.get("subject");
    let body: String = campaign.get("body_markdown");
    let recipients: Vec<(String, String)> = sqlx::query_as("SELECT email, unsubscribe_token FROM newsletter_subscribers WHERE status = 'confirmed' ORDER BY confirmed_at")
        .fetch_all(&pool)
        .await
        .map_err(db_err)?;
    sqlx::query("UPDATE newsletter_campaigns SET recipients = $1 WHERE id = $2")
        .bind(recipients.len() as i32)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db_err)?;
    tracing::info!("Newsletter '{}' sent by '{}' to {} subscribers", subject, admin.username, recipients.len());
    let count = recipients.len();
    tokio::spawn(async move {
        for (email, unsubscribe_token) in recipients {
            let ok = crate::services::email::send_newsletter_email(&s, &email, &subject, &body, &unsubscribe_token).await.is_ok();
            let column = if ok { "sent_count" } else { "failed_count" };
            let _ = sqlx::query(&format!("UPDATE newsletter_campaigns SET {0} = {0} + 1 WHERE id = $1", column)).bind(id).execute(&pool).await;
            // Gentle pace so mail servers do not throttle the shop
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        }
        let _ = sqlx::query("UPDATE newsletter_campaigns SET status = 'sent', sent_at = NOW() WHERE id = $1").bind(id).execute(&pool).await;
    });
    Ok(Json(json!({ "success": true, "recipients": count })))
}

// ================================================================== withdrawal button

#[derive(Deserialize)]
struct WithdrawalInput {
    order_number: String,
    customer_name: String,
    customer_email: String,
    #[serde(default)]
    items: String,
    #[serde(default)]
    message: String,
}

async fn create_withdrawal(State(pool): State<PgPool>, Json(input): Json<WithdrawalInput>) -> ApiResult<impl IntoResponse> {
    let order_number = input.order_number.trim();
    let name = input.customer_name.trim();
    let email = input.customer_email.trim().to_lowercase();
    if order_number.is_empty() || name.is_empty() {
        return Err(bad("Please enter your name and the order number"));
    }
    if !valid_email(&email) {
        return Err(bad("Please enter a valid email address so we can confirm the receipt of your withdrawal"));
    }
    let order_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM orders WHERE order_number = $1 AND LOWER(customer_email) = $2")
        .bind(order_number)
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(db_err)?;
    // A withdrawal is valid even if the details do not match an order exactly — it is recorded either way
    let row = sqlx::query(
        "INSERT INTO withdrawal_requests (order_id, order_number, customer_name, customer_email, items, message)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, received_at",
    )
    .bind(order_id)
    .bind(order_number.chars().take(100).collect::<String>())
    .bind(name.chars().take(255).collect::<String>())
    .bind(&email)
    .bind(input.items.trim().chars().take(5000).collect::<String>())
    .bind(input.message.trim().chars().take(5000).collect::<String>())
    .fetch_one(&pool)
    .await
    .map_err(db_err)?;
    let id: Uuid = row.get("id");
    let received_at: DateTime<Utc> = row.get("received_at");
    let reference = format!("W-{}", id.to_string()[..8].to_uppercase());

    let s = settings(&pool).await?;
    let details = crate::services::email::WithdrawalDetails {
        reference: &reference,
        received_at,
        order_number,
        customer_name: name,
        customer_email: &email,
        items: input.items.trim(),
        message: input.message.trim(),
    };
    let confirmation_sent = crate::services::email::send_withdrawal_confirmation_email(&s, &details).await;
    if confirmation_sent.is_ok() {
        let _ = sqlx::query("UPDATE withdrawal_requests SET confirmation_sent_at = NOW() WHERE id = $1").bind(id).execute(&pool).await;
    }
    // The shop is told right away
    // Everyone who handles orders (with an email address), plus the support address
    let admins: Vec<(Option<String>, String, Value)> = sqlx::query_as("SELECT email, role, permissions FROM admin_users")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    let mut recipients: Vec<String> = admins
        .into_iter()
        .filter(|(_, role, perms)| crate::middleware::effective_permissions(role, perms).get("orders").copied().unwrap_or(false))
        .filter_map(|(email, _, _)| email.map(|e| e.trim().to_string()).filter(|e| e.contains('@')))
        .collect();
    if s.support_email.contains('@') {
        recipients.push(s.support_email.trim().to_string());
    }
    recipients.sort();
    recipients.dedup();
    for to in recipients {
        let _ = crate::services::email::send_withdrawal_notification_email(&s, &to, &details).await;
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "reference": reference,
            "received_at": received_at,
            "confirmation_email_sent": confirmation_sent.is_ok(),
            "message": "Your withdrawal has been received. We have sent a confirmation to your email address."
        })),
    ))
}

async fn admin_list_withdrawals(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    let rows = sqlx::query("SELECT * FROM withdrawal_requests ORDER BY received_at DESC LIMIT 500").fetch_all(&pool).await.map_err(db_err)?;
    let list: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "order_id": r.get::<Option<Uuid>, _>("order_id"),
                "order_number": r.get::<String, _>("order_number"),
                "customer_name": r.get::<String, _>("customer_name"),
                "customer_email": r.get::<String, _>("customer_email"),
                "items": r.get::<String, _>("items"),
                "message": r.get::<String, _>("message"),
                "status": r.get::<String, _>("status"),
                "received_at": r.get::<DateTime<Utc>, _>("received_at"),
                "confirmation_sent_at": r.get::<Option<DateTime<Utc>>, _>("confirmation_sent_at"),
                "processed_at": r.get::<Option<DateTime<Utc>>, _>("processed_at"),
                "admin_note": r.get::<String, _>("admin_note"),
            })
        })
        .collect();
    Ok(Json(list))
}

#[derive(Deserialize)]
struct WithdrawalUpdate {
    status: Option<String>,
    admin_note: Option<String>,
}

async fn admin_update_withdrawal(State(pool): State<PgPool>, Path(id): Path<Uuid>, Json(input): Json<WithdrawalUpdate>) -> ApiResult<impl IntoResponse> {
    if input.status.as_deref().is_some_and(|s| s != "received" && s != "processed") {
        return Err(bad("Status must be received or processed"));
    }
    sqlx::query(
        "UPDATE withdrawal_requests SET status = COALESCE($1, status), admin_note = COALESCE($2, admin_note),
         processed_at = CASE WHEN $1 = 'processed' THEN NOW() WHEN $1 = 'received' THEN NULL ELSE processed_at END WHERE id = $3",
    )
    .bind(input.status)
    .bind(input.admin_note)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(db_err)?;
    Ok(Json(json!({ "success": true })))
}

// ================================================================== protected digital downloads

/// Digital files of a paid order, each with its own download link (token), remaining downloads
/// and expiry. Files come from the product as it is now, so customers always get the latest version.
pub async fn order_download_files(pool: &PgPool, order_id: Uuid, product_id: Uuid, product_title: &str, fallback_url: Option<&str>) -> Vec<Value> {
    let mut files: Vec<(String, String)> = Vec::new();
    let live: Option<Option<String>> = sqlx::query_scalar("SELECT digital_download_url FROM products WHERE id = $1")
        .bind(product_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let source = live.flatten().or_else(|| fallback_url.map(str::to_string));
    if let Some(u) = source {
        match serde_json::from_str::<Vec<Value>>(&u) {
            Ok(list) => {
                for f in list {
                    let url = f.get("url").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                    if !url.is_empty() {
                        files.push((f.get("name").and_then(|v| v.as_str()).unwrap_or("File").to_string(), url));
                    }
                }
            }
            Err(_) if !u.trim().is_empty() => files.push((format!("{} Package", product_title), u.trim().to_string())),
            Err(_) => {}
        }
    }
    let parts: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT part_name, notes FROM product_parts WHERE product_id = $1 AND part_sku = 'DIGITAL_FILE'")
            .bind(product_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    for (name, url) in parts {
        if let Some(url) = url.map(|u| u.trim().to_string()).filter(|u| !u.is_empty()) {
            if !files.iter().any(|(_, u)| *u == url) {
                files.push((name, url));
            }
        }
    }

    let s = match settings(pool).await {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for (name, url) in files {
        let grant = sqlx::query(
            "INSERT INTO download_grants (order_id, product_id, file_url, file_name, token) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (order_id, file_url) DO UPDATE SET file_name = EXCLUDED.file_name
             RETURNING token, download_count, created_at",
        )
        .bind(order_id)
        .bind(product_id)
        .bind(&url)
        .bind(&name)
        .bind(auth::random_token(32))
        .fetch_one(pool)
        .await;
        let Ok(grant) = grant else { continue };
        let token: String = grant.get("token");
        let count: i32 = grant.get("download_count");
        let created: DateTime<Utc> = grant.get("created_at");
        let expires_at = (s.download_days > 0).then(|| created + Duration::days(s.download_days as i64));
        out.push(json!({
            "name": name,
            "url": format!("/api/v1/downloads/{}", token),
            "downloads_left": (s.download_limit > 0).then(|| (s.download_limit - count).max(0)),
            "expires_at": expires_at,
        }));
    }
    out
}

async fn download_file(State(pool): State<PgPool>, Path(token): Path<String>) -> ApiResult<Response> {
    let not_found = || (StatusCode::NOT_FOUND, "This download link is invalid.".to_string());
    if token.len() != 64 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(not_found());
    }
    let row = sqlx::query(
        "SELECT g.id, g.file_url, g.file_name, g.download_count, g.created_at, o.payment_status
         FROM download_grants g JOIN orders o ON o.id = g.order_id WHERE g.token = $1",
    )
    .bind(&token)
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .ok_or_else(not_found)?;
    if row.get::<String, _>("payment_status") != "paid" {
        return Err((StatusCode::FORBIDDEN, "Downloads are only available for paid orders.".to_string()));
    }
    let s = settings(&pool).await?;
    let created: DateTime<Utc> = row.get("created_at");
    if s.download_days > 0 && Utc::now() > created + Duration::days(s.download_days as i64) {
        return Err((StatusCode::GONE, "This download link has expired. Please contact us for a new one.".to_string()));
    }
    // Count atomically so parallel requests cannot exceed the limit
    let counted = sqlx::query(
        "UPDATE download_grants SET download_count = download_count + 1, last_download_at = NOW()
         WHERE id = $1 AND ($2 <= 0 OR download_count < $2)",
    )
    .bind(row.get::<Uuid, _>("id"))
    .bind(s.download_limit)
    .execute(&pool)
    .await
    .map_err(db_err)?;
    if counted.rows_affected() == 0 {
        return Err((StatusCode::GONE, "The download limit for this file has been reached. Please contact us if you need it again.".to_string()));
    }

    let url: String = row.get("file_url");
    let local = url.split("/uploads/").nth(1).filter(|_| url.starts_with("/uploads/") || url.contains("/uploads/"));
    match local {
        Some(name) if !name.is_empty() && !name.contains(['/', '\\']) && !name.contains("..") => {
            let path = std::path::Path::new("uploads").join(name);
            let file = tokio::fs::File::open(&path).await.map_err(|_| not_found())?;
            let len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
            let ext = name.rsplit('.').next().unwrap_or("");
            let download_name = {
                let base: String = row.get::<String, _>("file_name").chars().filter(|c| !c.is_control() && *c != '"' && *c != '\\').collect();
                if base.to_lowercase().ends_with(&format!(".{}", ext.to_lowercase())) || ext.is_empty() { base } else { format!("{}.{}", base, ext) }
            };
            let body = Body::from_stream(tokio_util::io::ReaderStream::new(file));
            Ok((
                [
                    (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                    (header::CONTENT_LENGTH, len.to_string()),
                    (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", download_name)),
                    (header::CACHE_CONTROL, "private, no-store".to_string()),
                ],
                body,
            )
                .into_response())
        }
        // Files hosted elsewhere: forward after the checks
        _ if url.starts_with("https://") || url.starts_with("http://") => Ok(Redirect::temporary(&url).into_response()),
        _ => Err(not_found()),
    }
}

async fn admin_order_downloads(State(pool): State<PgPool>, Path(order_id): Path<Uuid>) -> ApiResult<impl IntoResponse> {
    let rows = sqlx::query("SELECT file_name, download_count, created_at, last_download_at FROM download_grants WHERE order_id = $1 ORDER BY file_name")
        .bind(order_id)
        .fetch_all(&pool)
        .await
        .map_err(db_err)?;
    let list: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "file_name": r.get::<String, _>("file_name"),
                "download_count": r.get::<i32, _>("download_count"),
                "created_at": r.get::<DateTime<Utc>, _>("created_at"),
                "last_download_at": r.get::<Option<DateTime<Utc>>, _>("last_download_at"),
            })
        })
        .collect();
    Ok(Json(list))
}

/// Gives the customer fresh downloads (counter reset, validity restarts today).
async fn admin_reset_downloads(State(pool): State<PgPool>, Path(order_id): Path<Uuid>) -> ApiResult<impl IntoResponse> {
    let r = sqlx::query("UPDATE download_grants SET download_count = 0, created_at = NOW() WHERE order_id = $1")
        .bind(order_id)
        .execute(&pool)
        .await
        .map_err(db_err)?;
    Ok(Json(json!({ "success": true, "files": r.rows_affected() })))
}

/// Media files that are sold as digital products — never served publicly under /uploads.
pub async fn protected_upload_names(pool: &PgPool) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    let mut add = |url: &str| {
        if let Some(name) = url.split("/uploads/").nth(1) {
            names.insert(name.split(['?', '#']).next().unwrap_or("").to_string());
        }
    };
    let products: Vec<Option<String>> = sqlx::query_scalar("SELECT digital_download_url FROM products WHERE digital_download_url IS NOT NULL")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    for u in products.into_iter().flatten() {
        match serde_json::from_str::<Vec<Value>>(&u) {
            Ok(list) => list.iter().filter_map(|f| f.get("url").and_then(|v| v.as_str())).for_each(&mut add),
            Err(_) => add(&u),
        }
    }
    let parts: Vec<Option<String>> = sqlx::query_scalar("SELECT notes FROM product_parts WHERE part_sku = 'DIGITAL_FILE'")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    for u in parts.into_iter().flatten() {
        add(&u);
    }
    names
}

// ================================================================== EU VAT ID check

#[derive(Deserialize)]
struct VatCheckInput {
    vat_id: String,
}

async fn vat_check(State(pool): State<PgPool>, Json(input): Json<VatCheckInput>) -> ApiResult<impl IntoResponse> {
    let s = settings(&pool).await?;
    if !crate::services::vies::available(&s) {
        return Err((StatusCode::NOT_FOUND, "VAT-free business orders are not available in this shop".to_string()));
    }
    let check = crate::services::vies::lookup(&pool, &s, &input.vat_id).await.map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "vat_id": check.vat_id, "valid": check.valid, "name": check.name, "address": check.address })))
}

/// Guards the public /uploads folder: files sold as digital products are only served to logged-in
/// admins (customers get them through their personal download links).
pub async fn guard_uploads(
    State(pool): State<PgPool>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration as StdDuration, Instant};
    static CACHE: OnceLock<Mutex<(Option<Instant>, std::collections::HashSet<String>)>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new((None, Default::default())));

    let name = req.uri().path().split("/uploads/").nth(1).unwrap_or("").to_string();
    let name = percent_decode(&name);
    let fresh = cache.lock().ok().and_then(|c| c.0.map(|t| t.elapsed() < StdDuration::from_secs(30))).unwrap_or(false);
    if !fresh {
        let names = protected_upload_names(&pool).await;
        if let Ok(mut c) = cache.lock() {
            *c = (Some(Instant::now()), names);
        }
    }
    let protected = cache.lock().map(|c| c.1.contains(&name)).unwrap_or(true);
    if protected {
        let is_admin = auth::bearer_token(req.headers()).and_then(|t| auth::verify_token(t, auth::ROLE_ADMIN_TOKEN)).is_some();
        if !is_admin {
            return (StatusCode::NOT_FOUND, "Not Found").into_response();
        }
    }
    next.run(req).await
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(v) = std::str::from_utf8(&bytes[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_names_are_decoded() {
        assert_eq!(percent_decode("my%20file.zip"), "my file.zip");
        assert_eq!(percent_decode("a%2"), "a%2");
        assert_eq!(percent_decode("%C3%A4.pdf"), "ä.pdf");
        assert_eq!(percent_decode("plain.png"), "plain.png");
    }

    #[test]
    fn emails_are_checked() {
        assert!(valid_email("a@b.de"));
        assert!(!valid_email("a b@c.de"));
        assert!(!valid_email("nobody"));
    }
}

#[derive(Deserialize)]
struct ShippingOptionsInput {
    country_code: String,
    items: Vec<crate::models::CartItemInput>,
}

/// Packages (rates) the given products fit into for a country, cheapest first.
async fn shipping_options(State(pool): State<PgPool>, Json(input): Json<ShippingOptionsInput>) -> ApiResult<impl IntoResponse> {
    if input.items.len() > 100 {
        return Err(bad("Too many items"));
    }
    let (rates, any) = crate::services::checkout::CheckoutService::shipping_options(&pool, &input.country_code, &input.items)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let options: Vec<Value> = rates
        .iter()
        .map(|r| json!({ "id": r.id, "name": r.name, "price_cents": r.price_cents, "estimated_delivery_days": r.estimated_delivery_days }))
        .collect();
    // ships = false: no zone serves this country; fits = false: served, but no package is big enough
    Ok(Json(json!({ "ships": any, "fits": !options.is_empty() || !any, "options": options })))
}
