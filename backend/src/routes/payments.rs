//! Checkout & payment endpoints.
//!
//! Flow for every provider:
//!   1. `POST /checkout/quote` — server-computed totals for display.
//!   2. `POST /checkout/{provider}/...` — server prices the cart, stores a pending checkout and
//!      creates the provider payment object (Stripe PaymentIntent / Checkout Session, PayPal order).
//!   3. The browser authorizes the payment directly with the provider.
//!   4. `POST /checkout/{provider}/complete|capture` (and the Stripe webhook) verify the payment with
//!      the provider and turn the pending checkout into an order exactly once.

use crate::models::CheckoutRequest;
use crate::services::checkout::{CheckoutService, FinalizeError, FinalizeOutcome, VerifiedPayment};
use crate::services::payments::{self, paypal, stripe, ProviderConfig};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use sqlx::PgPool;
use tracing::{error, info, warn};

type ApiError = (StatusCode, String);

fn bad_request(e: impl std::fmt::Display) -> ApiError {
    (StatusCode::BAD_REQUEST, e.to_string())
}

pub fn payments_router() -> Router<PgPool> {
    Router::new()
        .route("/checkout/quote", post(quote))
        .route("/checkout/free", post(free_order))
        .route("/checkout/stripe/methods", get(stripe_checkout_methods))
        .route("/checkout/stripe/intent", post(stripe_create_intent))
        .route("/checkout/stripe/session", post(stripe_create_session))
        .route("/checkout/stripe/complete", post(stripe_complete))
        .route("/checkout/paypal/order", post(paypal_create_order))
        .route("/checkout/paypal/capture", post(paypal_capture))
        .route("/payments/stripe/webhook", post(stripe_webhook))
}

fn order_description(pool_store_name: &str, req: &CheckoutRequest) -> String {
    let count: i32 = req.items.iter().map(|i| i.quantity).sum();
    format!("{} order ({} item{})", pool_store_name, count, if count == 1 { "" } else { "s" })
}

async fn store_name(pool: &PgPool) -> String {
    sqlx::query_scalar::<_, String>("SELECT store_name FROM store_settings WHERE id = 1")
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| "Shop".to_string())
}

/// Validates the policy and prices the cart for a paid checkout.
async fn prepare_paid_checkout(pool: &PgPool, req: &CheckoutRequest, min_cents: i32) -> Result<i32, ApiError> {
    CheckoutService::enforce_checkout_policy(pool, req.customer_email.trim())
        .await
        .map_err(|e| (StatusCode::FORBIDDEN, e.to_string()))?;
    let quote = CheckoutService::quote(pool, req, true).await.map_err(bad_request)?;
    if quote.total_cents <= 0 {
        return Err(bad_request("This order is free — no payment is required."));
    }
    if quote.total_cents < min_cents {
        return Err(bad_request(format!(
            "The minimum amount for this payment method is {}.",
            payments::format_amount(min_cents)
        )));
    }
    Ok(quote.total_cents)
}

fn send_order_emails(pool: &PgPool, order_number: &str, is_paid: bool) {
    let pool = pool.clone();
    let order_number = order_number.to_string();
    tokio::spawn(async move {
        crate::services::email::send_order_created_notification(&pool, &order_number).await;
        if is_paid {
            crate::services::email::send_payment_received_notification(&pool, &order_number).await;
        }
        let _ = crate::services::inventory::recalculate_bom_stock(&pool).await;
        crate::services::email::check_and_send_low_stock_alerts(&pool).await;
    });
}

/// Finalizes a verified payment; refunds it automatically if the order cannot be created
/// (e.g. the last item sold out while the customer was paying).
async fn finalize_or_refund(pool: &PgPool, provider_reference: &str, payment: VerifiedPayment) -> Result<FinalizeOutcome, ApiError> {
    let provider = payment.provider.clone();
    let refund_reference = payment.reference.clone();
    match CheckoutService::finalize_pending(pool, provider_reference, payment).await {
        Ok(outcome) => {
            if outcome.newly_created {
                send_order_emails(pool, &outcome.response.order_number, outcome.response.payment_status == "paid");
            }
            Ok(outcome)
        }
        Err(FinalizeError::OrderRejected(err)) => {
            warn!("Order rejected after payment {} ({}): {}", refund_reference, provider, err);
            match payments::refund(pool, &provider, &refund_reference).await {
                Ok(()) => Err(bad_request(format!(
                    "Your order could not be completed: {}. Your payment has been refunded automatically.",
                    err
                ))),
                Err(refund_err) => {
                    error!("AUTOMATIC REFUND FAILED for {} ({}): {}", refund_reference, provider, refund_err);
                    Err(bad_request(format!(
                        "Your order could not be completed: {}. Please contact us with payment reference {} for a refund.",
                        err, refund_reference
                    )))
                }
            }
        }
        Err(FinalizeError::Invalid(err)) => Err(bad_request(err)),
        Err(FinalizeError::Internal(err)) => {
            error!("Finalizing payment {} failed: {}", refund_reference, err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Temporary error while confirming your order. Please retry in a moment.".to_string()))
        }
    }
}

async fn quote(State(pool): State<PgPool>, Json(req): Json<CheckoutRequest>) -> Result<impl IntoResponse, ApiError> {
    CheckoutService::quote(&pool, &req, false).await.map(Json).map_err(bad_request)
}

async fn free_order(State(pool): State<PgPool>, Json(req): Json<CheckoutRequest>) -> Result<impl IntoResponse, ApiError> {
    CheckoutService::enforce_checkout_policy(&pool, req.customer_email.trim())
        .await
        .map_err(|e| (StatusCode::FORBIDDEN, e.to_string()))?;
    let quote = CheckoutService::quote(&pool, &req, true).await.map_err(bad_request)?;
    if quote.total_cents != 0 {
        return Err(bad_request("This order requires payment."));
    }
    let response = CheckoutService::create_free_order(&pool, &req).await.map_err(bad_request)?;
    send_order_emails(&pool, &response.order_number, true);
    Ok(Json(response))
}

// ---------------------------------------------------------------- Stripe

#[derive(Deserialize)]
struct StripeIntentRequest {
    #[serde(flatten)]
    checkout: CheckoutRequest,
    /// Method chosen in the checkout list (card, apple_pay, klarna, …); all enabled methods if absent
    payment_method: Option<String>,
}

async fn stripe_create_intent(State(pool): State<PgPool>, Json(body): Json<StripeIntentRequest>) -> Result<impl IntoResponse, ApiError> {
    let req = body.checkout;
    let cfg = ProviderConfig::load_enabled(&pool, "stripe").await.map_err(bad_request)?;
    stripe::secret_key(&cfg).map_err(bad_request)?;
    if let Some(m) = body.payment_method.as_deref() {
        stripe::intent_types_for(&cfg, m).map_err(bad_request)?;
    }
    let amount = prepare_paid_checkout(&pool, &req, stripe::MIN_AMOUNT_CENTS).await?;

    let pending_id = CheckoutService::create_pending(&pool, "stripe", &req, amount).await.map_err(bad_request)?;
    let description = order_description(&store_name(&pool).await, &req);
    let intent = stripe::create_payment_intent(&cfg, amount, &pending_id.to_string(), req.customer_email.trim(), &description, body.payment_method.as_deref())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let intent_id = intent["id"].as_str().unwrap_or_default();
    CheckoutService::attach_reference(&pool, pending_id, intent_id).await.map_err(bad_request)?;

    Ok(Json(json!({
        "client_secret": intent["client_secret"],
        "payment_intent_id": intent_id,
        "amount_cents": amount
    })))
}

#[derive(Deserialize)]
struct StripeSessionRequest {
    #[serde(flatten)]
    checkout: CheckoutRequest,
    /// Storefront origin used for the success / cancel URLs (e.g. https://shop.example)
    return_origin: String,
}

async fn stripe_create_session(State(pool): State<PgPool>, Json(body): Json<StripeSessionRequest>) -> Result<impl IntoResponse, ApiError> {
    let origin = body.return_origin.trim_end_matches('/');
    if !(origin.starts_with("https://") || origin.starts_with("http://")) || origin.contains(['?', '#']) {
        return Err(bad_request("Invalid return origin"));
    }
    let req = body.checkout;
    let cfg = ProviderConfig::load_enabled(&pool, "stripe").await.map_err(bad_request)?;
    stripe::secret_key(&cfg).map_err(bad_request)?;
    let amount = prepare_paid_checkout(&pool, &req, stripe::MIN_AMOUNT_CENTS).await?;

    let pending_id = CheckoutService::create_pending(&pool, "stripe", &req, amount).await.map_err(bad_request)?;
    let description = order_description(&store_name(&pool).await, &req);
    let session = stripe::create_checkout_session(
        &cfg,
        amount,
        &pending_id.to_string(),
        req.customer_email.trim(),
        &description,
        &format!("{}/checkout/complete?session_id={{CHECKOUT_SESSION_ID}}", origin),
        &format!("{}/checkout", origin),
    )
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let session_id = session["id"].as_str().unwrap_or_default();
    CheckoutService::attach_reference(&pool, pending_id, session_id).await.map_err(bad_request)?;

    Ok(Json(json!({ "url": session["url"], "session_id": session_id })))
}

#[derive(Deserialize)]
struct StripeCompleteRequest {
    payment_intent_id: Option<String>,
    session_id: Option<String>,
}

async fn stripe_complete(State(pool): State<PgPool>, Json(body): Json<StripeCompleteRequest>) -> Result<impl IntoResponse, ApiError> {
    let cfg = ProviderConfig::load(&pool, "stripe").await.map_err(bad_request)?;

    if let Some(pi_id) = body.payment_intent_id.as_deref() {
        let intent = stripe::retrieve_payment_intent(&cfg, pi_id).await.map_err(bad_request)?;
        let payment = verified_from_intent(&intent).map_err(bad_request)?;
        let outcome = finalize_or_refund(&pool, pi_id, payment).await?;
        return Ok(Json(outcome.response));
    }

    if let Some(session_id) = body.session_id.as_deref() {
        let session = stripe::retrieve_checkout_session(&cfg, session_id).await.map_err(bad_request)?;
        let payment = verified_from_session(&session).map_err(bad_request)?;
        let outcome = finalize_or_refund(&pool, session_id, payment).await?;
        return Ok(Json(outcome.response));
    }

    Err(bad_request("Missing payment reference"))
}

fn verified_from_intent(intent: &JsonValue) -> Result<VerifiedPayment, String> {
    let id = intent["id"].as_str().unwrap_or_default();
    if intent["currency"].as_str() != Some("eur") {
        return Err("Unexpected payment currency".to_string());
    }
    let (status, amount) = match intent["status"].as_str().unwrap_or_default() {
        "succeeded" => ("paid", intent["amount_received"].as_i64()),
        // Delayed methods (e.g. SEPA Direct Debit): create the order now, mark it paid via webhook later
        "processing" => ("pending", intent["amount"].as_i64()),
        "requires_payment_method" => return Err("The payment was declined. Please try another payment method.".to_string()),
        "canceled" => return Err("The payment was canceled.".to_string()),
        other => return Err(format!("The payment has not been completed (status: {}).", other)),
    };
    Ok(VerifiedPayment {
        provider: "stripe".to_string(),
        reference: id.to_string(),
        amount_cents: amount.unwrap_or(0) as i32,
        status: status.to_string(),
        is_test: intent["livemode"].as_bool() == Some(false),
    })
}

fn session_payment_intent(session: &JsonValue) -> Option<&str> {
    // payment_intent is an id string unless expanded
    session["payment_intent"].as_str().or_else(|| session["payment_intent"]["id"].as_str())
}

fn verified_from_session(session: &JsonValue) -> Result<VerifiedPayment, String> {
    let status = match (session["status"].as_str(), session["payment_status"].as_str()) {
        (_, Some("paid")) => "paid",
        // Hosted checkout with a delayed method: completed, money arrives later
        (Some("complete"), Some("unpaid")) => "pending",
        _ => return Err("The Stripe payment has not been completed.".to_string()),
    };
    if session["currency"].as_str() != Some("eur") {
        return Err("Unexpected payment currency".to_string());
    }
    let pi = session_payment_intent(session).ok_or_else(|| "Stripe session has no PaymentIntent".to_string())?;
    Ok(VerifiedPayment {
        provider: "stripe".to_string(),
        reference: pi.to_string(),
        amount_cents: session["amount_total"].as_i64().unwrap_or(0) as i32,
        status: status.to_string(),
        is_test: session["livemode"].as_bool() == Some(false),
    })
}

async fn on_delayed_payment_succeeded(pool: &PgPool, payment_intent_id: &str) -> Result<bool, ApiError> {
    match CheckoutService::mark_payment_succeeded(pool, payment_intent_id).await {
        Ok(Some(order_number)) => {
            info!("Delayed payment confirmed for order {}", order_number);
            let pool = pool.clone();
            tokio::spawn(async move {
                crate::services::email::send_payment_received_notification(&pool, &order_number).await;
            });
            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn on_payment_failed(pool: &PgPool, payment_intent_id: &str) -> Result<(), ApiError> {
    match CheckoutService::mark_payment_failed(pool, payment_intent_id).await {
        Ok(Some(order_number)) => {
            warn!("Payment failed — order {} cancelled and stock returned", order_number);
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// Finalizes a pending checkout from a webhook event; transient errors make Stripe retry.
async fn finalize_from_webhook(pool: &PgPool, reference: &str, payment: Result<VerifiedPayment, String>) -> Result<(), ApiError> {
    let Ok(payment) = payment else { return Ok(()) };
    if CheckoutService::pending_exists(pool, "stripe", reference).await.map_err(bad_request)?.is_none() {
        return Ok(());
    }
    match finalize_or_refund(pool, reference, payment).await {
        Ok(outcome) => {
            if outcome.newly_created {
                info!("Order {} created from Stripe webhook", outcome.response.order_number);
            }
            Ok(())
        }
        Err((StatusCode::INTERNAL_SERVER_ERROR, msg)) => Err((StatusCode::INTERNAL_SERVER_ERROR, msg)),
        Err(_) => Ok(()),
    }
}

/// Stripe webhook: finalizes orders even if the customer never returns to the shop after paying,
/// and settles delayed payments. Subscribe to `payment_intent.succeeded`, `payment_intent.payment_failed`,
/// `checkout.session.completed`, `checkout.session.async_payment_succeeded` and
/// `checkout.session.async_payment_failed`.
async fn stripe_webhook(State(pool): State<PgPool>, headers: HeaderMap, body: Bytes) -> Result<impl IntoResponse, ApiError> {
    let cfg = ProviderConfig::load(&pool, "stripe").await.map_err(bad_request)?;
    if cfg.webhook_secret.is_empty() {
        return Err(bad_request("Stripe webhook signing secret is not configured"));
    }
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| bad_request("Missing Stripe-Signature header"))?;
    stripe::verify_webhook_signature(&body, signature, &cfg.webhook_secret, 300).map_err(|e| {
        warn!("Rejected Stripe webhook: {}", e);
        bad_request("Invalid signature")
    })?;

    let event: JsonValue = serde_json::from_slice(&body).map_err(bad_request)?;
    let object = &event["data"]["object"];
    let object_id = object["id"].as_str().unwrap_or_default();

    match event["type"].as_str().unwrap_or_default() {
        "payment_intent.succeeded" => {
            // Either a delayed payment of an existing order, or a checkout the customer never returned from
            if !on_delayed_payment_succeeded(&pool, object_id).await? {
                finalize_from_webhook(&pool, object_id, verified_from_intent(object)).await?;
            }
        }
        "payment_intent.payment_failed" | "payment_intent.canceled" => on_payment_failed(&pool, object_id).await?,
        "checkout.session.completed" => finalize_from_webhook(&pool, object_id, verified_from_session(object)).await?,
        "checkout.session.async_payment_succeeded" => {
            let settled = match session_payment_intent(object) {
                Some(pi) => on_delayed_payment_succeeded(&pool, pi).await?,
                None => false,
            };
            if !settled {
                finalize_from_webhook(&pool, object_id, verified_from_session(object)).await?;
            }
        }
        "checkout.session.async_payment_failed" => {
            if let Some(pi) = session_payment_intent(object) {
                on_payment_failed(&pool, pi).await?;
            }
        }
        _ => {}
    }
    Ok(Json(json!({ "received": true })))
}



// ---------------------------------------------------------------- PayPal

async fn paypal_create_order(State(pool): State<PgPool>, Json(req): Json<CheckoutRequest>) -> Result<impl IntoResponse, ApiError> {
    let cfg = ProviderConfig::load_enabled(&pool, "paypal").await.map_err(bad_request)?;
    paypal::ensure_configured(&cfg).map_err(bad_request)?;
    let amount = prepare_paid_checkout(&pool, &req, 1).await?;

    let pending_id = CheckoutService::create_pending(&pool, "paypal", &req, amount).await.map_err(bad_request)?;
    let description = order_description(&store_name(&pool).await, &req);
    let order_id = paypal::create_order(&cfg, amount, &pending_id.to_string(), &description)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    CheckoutService::attach_reference(&pool, pending_id, &order_id).await.map_err(bad_request)?;

    Ok(Json(json!({ "id": order_id, "amount_cents": amount })))
}

#[derive(Deserialize)]
struct PaypalCaptureRequest {
    order_id: String,
}

async fn paypal_capture(State(pool): State<PgPool>, Json(body): Json<PaypalCaptureRequest>) -> Result<impl IntoResponse, ApiError> {
    let cfg = ProviderConfig::load(&pool, "paypal").await.map_err(bad_request)?;
    let pending_id = CheckoutService::pending_exists(&pool, "paypal", &body.order_id)
        .await
        .map_err(bad_request)?
        .ok_or_else(|| bad_request("Unknown PayPal order"))?;

    let captured = paypal::capture_order(&cfg, &body.order_id, &pending_id.to_string())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let capture = &captured["purchase_units"][0]["payments"]["captures"][0];
    let status = match capture["status"].as_str() {
        Some("COMPLETED") => "paid",
        Some("PENDING") => "pending",
        _ => return Err(bad_request("PayPal did not complete the payment. Please try again or choose another payment method.")),
    };
    if capture["amount"]["currency_code"].as_str() != Some("EUR") {
        return Err(bad_request("Unexpected payment currency"));
    }
    let amount_cents = parse_decimal_cents(capture["amount"]["value"].as_str().unwrap_or("0"));
    let capture_id = capture["id"].as_str().unwrap_or_default().to_string();

    let payment = VerifiedPayment {
        provider: "paypal".to_string(),
        reference: capture_id,
        amount_cents,
        status: status.to_string(),
        is_test: cfg.is_sandbox,
    };
    let outcome = finalize_or_refund(&pool, &body.order_id, payment).await?;
    Ok(Json(outcome.response))
}

fn parse_decimal_cents(value: &str) -> i32 {
    let (whole, frac) = value.split_once('.').unwrap_or((value, "0"));
    let frac = format!("{:0<2}", frac);
    whole.parse::<i32>().unwrap_or(0) * 100 + frac[..2].parse::<i32>().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::parse_decimal_cents;

    #[test]
    fn parses_paypal_decimal_amounts() {
        assert_eq!(parse_decimal_cents("19.99"), 1999);
        assert_eq!(parse_decimal_cents("19.9"), 1990);
        assert_eq!(parse_decimal_cents("20"), 2000);
        assert_eq!(parse_decimal_cents("0.05"), 5);
    }
}

/// Payment methods the checkout may list: enabled in the admin *and* activated in the Stripe account,
/// so customers never see a method that would fail at payment.
async fn stripe_checkout_methods(State(pool): State<PgPool>) -> Result<impl IntoResponse, ApiError> {
    let cfg = ProviderConfig::load_enabled(&pool, "stripe").await.map_err(bad_request)?;
    let (active, _) = stripe::active_checkout_methods(&cfg).await;
    Ok(Json(json!({ "methods": active })))
}
