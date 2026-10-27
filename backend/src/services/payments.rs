//! Thin clients for the payment providers (Stripe REST API, PayPal Orders v2 API).
//!
//! Card and wallet credentials never touch this server: the browser authorizes the payment
//! directly with Stripe Elements / PayPal Buttons, and the backend only creates the payment
//! object with a server-computed amount and later verifies its final state by reference.

use anyhow::{anyhow, bail, Result};
use hmac::{Hmac, Mac};
use serde_json::{json, Value as JsonValue};
use sha2::Sha256;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub is_enabled: bool,
    pub is_sandbox: bool,
    pub public_client_id: String,
    pub secret_key: String,
    pub webhook_secret: String,
    pub config_data: JsonValue,
}

impl ProviderConfig {
    pub async fn load(pool: &PgPool, provider: &str) -> Result<Self> {
        let row = sqlx::query(
            "SELECT is_enabled, is_sandbox, public_client_id, secret_key, webhook_secret, config_data FROM payment_configs WHERE provider = $1",
        )
        .bind(provider)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Payment provider '{}' is not configured", provider))?;

        Ok(Self {
            is_enabled: row.get("is_enabled"),
            is_sandbox: row.get("is_sandbox"),
            public_client_id: row.get::<String, _>("public_client_id").trim().to_string(),
            secret_key: row.get::<String, _>("secret_key").trim().to_string(),
            webhook_secret: row.get::<String, _>("webhook_secret").trim().to_string(),
            config_data: row.get("config_data"),
        })
    }

    pub async fn load_enabled(pool: &PgPool, provider: &str) -> Result<Self> {
        let cfg = Self::load(pool, provider).await?;
        if !cfg.is_enabled {
            bail!("This payment method is currently disabled");
        }
        Ok(cfg)
    }
}

pub fn format_amount(cents: i32) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

pub mod stripe {
    use super::*;

    const API: &str = "https://api.stripe.com/v1";

    /// Minimum chargeable amount for EUR on Stripe.
    pub const MIN_AMOUNT_CENTS: i32 = 50;

    pub fn secret_key(cfg: &ProviderConfig) -> Result<&str> {
        let key = cfg.secret_key.as_str();
        if !(key.starts_with("sk_") || key.starts_with("rk_")) {
            bail!("Stripe secret key (sk_... or rk_...) is not configured in Admin → Payment Providers");
        }
        let pk = cfg.public_client_id.as_str();
        if pk.starts_with("pk_") && (pk.contains("_live_") != key.contains("_live_")) {
            bail!("Stripe keys are mixed: publishable and secret key must both be test keys or both be live keys");
        }
        Ok(key)
    }

    /// Payment method types enabled in the admin toggles. Apple Pay and Google Pay are wallets of
    /// the `card` type and are switched on/off in the Payment Element instead.
    /// Optional Stripe payment method types that support EUR charges. The admin toggles use the
    /// same keys (`config_data.methods.<type>`); the storefront mirrors this list.
    pub const OPTIONAL_METHODS: &[&str] = &[
        "link",
        "amazon_pay",
        "paypal",
        "klarna",
        "sepa_debit",
        "ideal",
        "bancontact",
        "eps",
        "p24",
        "revolut_pay",
        "mobilepay",
        "alipay",
        "wechat_pay",
    ];

    /// PaymentIntent types for the method chosen at checkout (Apple Pay / Google Pay are card
    /// payments; Link must be offered together with card). Rejects methods not enabled in the admin.
    pub fn intent_types_for(cfg: &ProviderConfig, method: &str) -> Result<Vec<&'static str>> {
        let methods = &cfg.config_data["methods"];
        match method {
            "card" => Ok(vec!["card"]),
            "apple_pay" | "google_pay" if methods[method].as_bool().unwrap_or(false) => Ok(vec!["card"]),
            "link" if methods["link"].as_bool().unwrap_or(false) => Ok(vec!["link", "card"]),
            other => OPTIONAL_METHODS
                .iter()
                .copied()
                .find(|m| *m == other && *m != "link" && methods[*m].as_bool().unwrap_or(false))
                .map(|m| vec![m])
                .ok_or_else(|| anyhow!("This payment method is not available")),
        }
    }

    /// Stripe account capability that must be active for a checkout method.
    pub fn capability_for(method: &str) -> &'static str {
        match method {
            "card" | "apple_pay" | "google_pay" => "card_payments",
            "link" => "link_payments",
            "amazon_pay" => "amazon_pay_payments",
            "paypal" => "paypal_payments",
            "klarna" => "klarna_payments",
            "sepa_debit" => "sepa_debit_payments",
            "ideal" => "ideal_payments",
            "bancontact" => "bancontact_payments",
            "eps" => "eps_payments",
            "p24" => "p24_payments",
            "revolut_pay" => "revolut_pay_payments",
            "mobilepay" => "mobilepay_payments",
            "alipay" => "alipay_payments",
            "wechat_pay" => "wechat_pay_payments",
            _ => "",
        }
    }

    /// Methods enabled in the admin (incl. card and wallets), in admin order.
    pub fn enabled_checkout_methods(cfg: &ProviderConfig) -> Vec<&'static str> {
        let methods = &cfg.config_data["methods"];
        let mut out = vec!["card"];
        for wallet in ["apple_pay", "google_pay"] {
            if methods[wallet].as_bool().unwrap_or(false) {
                out.push(wallet);
            }
        }
        out.extend(OPTIONAL_METHODS.iter().copied().filter(|m| methods[*m].as_bool().unwrap_or(false)));
        out
    }

    /// Activation status of the account's payment capabilities (cached for 5 minutes).
    /// `None` if Stripe could not be asked (e.g. a restricted key without "Accounts: read").
    pub async fn account_capabilities(cfg: &ProviderConfig) -> Option<JsonValue> {
        use std::sync::Mutex;
        use std::time::{Duration, Instant};
        static CACHE: Mutex<Option<(String, Instant, JsonValue)>> = Mutex::new(None);
        let key = cfg.secret_key.clone();
        if let Some((k, at, caps)) = CACHE.lock().unwrap().as_ref() {
            if *k == key && at.elapsed() < Duration::from_secs(300) {
                return Some(caps.clone());
            }
        }
        let account = get(secret_key(cfg).ok()?, "/account").await.ok()?;
        let caps = account["capabilities"].clone();
        if !caps.is_object() {
            return None;
        }
        *CACHE.lock().unwrap() = Some((key, Instant::now(), caps.clone()));
        Some(caps)
    }

    /// Enabled methods whose Stripe capability is active; all enabled methods if Stripe cannot be asked.
    pub async fn active_checkout_methods(cfg: &ProviderConfig) -> (Vec<&'static str>, Vec<&'static str>) {
        let enabled = enabled_checkout_methods(cfg);
        match account_capabilities(cfg).await {
            Some(caps) => enabled.into_iter().partition(|m| caps[capability_for(m)].as_str() == Some("active")),
            None => (enabled, Vec::new()),
        }
    }

    pub fn payment_method_types(cfg: &ProviderConfig) -> Vec<&'static str> {
        let methods = &cfg.config_data["methods"];
        let mut types = vec!["card"];
        types.extend(OPTIONAL_METHODS.iter().copied().filter(|m| methods[*m].as_bool().unwrap_or(false)));
        types
    }

    /// Registers the shop domain for Apple Pay / Google Pay / Link (Stripe "payment method domains").
    pub async fn register_domain(cfg: &ProviderConfig, domain: &str) -> Result<JsonValue> {
        let form = vec![("domain_name".to_string(), domain.to_string())];
        post(secret_key(cfg)?, "/payment_method_domains", &form, None).await
    }

    pub async fn list_domains(cfg: &ProviderConfig) -> Result<JsonValue> {
        get(secret_key(cfg)?, "/payment_method_domains?limit=20").await
    }

    async fn parse(resp: reqwest::Response) -> Result<JsonValue> {
        let status = resp.status();
        let body: JsonValue = resp.json().await.unwrap_or_default();
        if !status.is_success() {
            let msg = body["error"]["message"].as_str().unwrap_or("Unknown Stripe API error");
            bail!("Stripe: {}", msg);
        }
        Ok(body)
    }

    async fn post(secret: &str, path: &str, form: &[(String, String)], idempotency_key: Option<&str>) -> Result<JsonValue> {
        let mut req = reqwest::Client::new()
            .post(format!("{}{}", API, path))
            .basic_auth(secret, Some(""))
            .form(form);
        if let Some(key) = idempotency_key {
            req = req.header("Idempotency-Key", key);
        }
        let resp = req.send().await.map_err(|e| anyhow!("Could not reach Stripe: {}", e))?;
        parse(resp).await
    }

    async fn get(secret: &str, path: &str) -> Result<JsonValue> {
        let resp = reqwest::Client::new()
            .get(format!("{}{}", API, path))
            .basic_auth(secret, Some(""))
            .send()
            .await
            .map_err(|e| anyhow!("Could not reach Stripe: {}", e))?;
        parse(resp).await
    }

    fn method_types_form(types: &[&str]) -> Vec<(String, String)> {
        types
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("payment_method_types[{}]", i), t.to_string()))
            .collect()
    }

    pub async fn create_payment_intent(
        cfg: &ProviderConfig,
        amount_cents: i32,
        pending_id: &str,
        customer_email: &str,
        description: &str,
        selected_method: Option<&str>,
    ) -> Result<JsonValue> {
        let secret = secret_key(cfg)?;
        let mut form = vec![
            ("amount".to_string(), amount_cents.to_string()),
            ("currency".to_string(), "eur".to_string()),
            ("description".to_string(), description.to_string()),
            ("receipt_email".to_string(), customer_email.to_string()),
            ("metadata[pending_checkout_id]".to_string(), pending_id.to_string()),
        ];
        let types = match selected_method {
            Some(m) => intent_types_for(cfg, m)?,
            None => payment_method_types(cfg),
        };
        if types.contains(&"wechat_pay") {
            form.push(("payment_method_options[wechat_pay][client]".to_string(), "web".to_string()));
        }
        form.extend(method_types_form(&types));
        post(secret, "/payment_intents", &form, Some(&format!("pi-{}", pending_id))).await
    }

    pub async fn create_checkout_session(
        cfg: &ProviderConfig,
        amount_cents: i32,
        pending_id: &str,
        customer_email: &str,
        description: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<JsonValue> {
        let secret = secret_key(cfg)?;
        let mut form = vec![
            ("mode".to_string(), "payment".to_string()),
            ("success_url".to_string(), success_url.to_string()),
            ("cancel_url".to_string(), cancel_url.to_string()),
            ("customer_email".to_string(), customer_email.to_string()),
            ("client_reference_id".to_string(), pending_id.to_string()),
            ("metadata[pending_checkout_id]".to_string(), pending_id.to_string()),
            ("payment_intent_data[metadata][pending_checkout_id]".to_string(), pending_id.to_string()),
            ("line_items[0][quantity]".to_string(), "1".to_string()),
            ("line_items[0][price_data][currency]".to_string(), "eur".to_string()),
            ("line_items[0][price_data][unit_amount]".to_string(), amount_cents.to_string()),
            ("line_items[0][price_data][product_data][name]".to_string(), description.to_string()),
        ];
        form.extend(method_types_form(&payment_method_types(cfg)));
        post(secret, "/checkout/sessions", &form, Some(&format!("cs-{}", pending_id))).await
    }

    pub async fn retrieve_payment_intent(cfg: &ProviderConfig, id: &str) -> Result<JsonValue> {
        if !id.starts_with("pi_") || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            bail!("Invalid PaymentIntent reference");
        }
        get(secret_key(cfg)?, &format!("/payment_intents/{}", id)).await
    }

    pub async fn retrieve_checkout_session(cfg: &ProviderConfig, id: &str) -> Result<JsonValue> {
        if !id.starts_with("cs_") || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            bail!("Invalid Checkout Session reference");
        }
        get(secret_key(cfg)?, &format!("/checkout/sessions/{}", id)).await
    }

    pub async fn refund(cfg: &ProviderConfig, payment_intent_id: &str) -> Result<JsonValue> {
        let form = vec![("payment_intent".to_string(), payment_intent_id.to_string())];
        post(secret_key(cfg)?, "/refunds", &form, Some(&format!("refund-{}", payment_intent_id))).await
    }

    /// Verifies the `Stripe-Signature` header (HMAC-SHA256 over "{timestamp}.{payload}").
    pub fn verify_webhook_signature(payload: &[u8], header: &str, secret: &str, tolerance_secs: i64) -> Result<()> {
        let mut timestamp: Option<i64> = None;
        let mut signatures: Vec<Vec<u8>> = Vec::new();
        for part in header.split(',') {
            match part.trim().split_once('=') {
                Some(("t", v)) => timestamp = v.parse().ok(),
                Some(("v1", v)) => {
                    if let Ok(sig) = hex::decode(v) {
                        signatures.push(sig);
                    }
                }
                _ => {}
            }
        }
        let timestamp = timestamp.ok_or_else(|| anyhow!("Missing webhook timestamp"))?;
        if (chrono::Utc::now().timestamp() - timestamp).abs() > tolerance_secs {
            bail!("Webhook timestamp outside of tolerance");
        }
        for sig in signatures {
            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|e| anyhow!(e.to_string()))?;
            mac.update(timestamp.to_string().as_bytes());
            mac.update(b".");
            mac.update(payload);
            if mac.verify_slice(&sig).is_ok() {
                return Ok(());
            }
        }
        bail!("Webhook signature mismatch")
    }
}

pub mod paypal {
    use super::*;

    fn base_url(cfg: &ProviderConfig) -> &'static str {
        if cfg.is_sandbox {
            "https://api-m.sandbox.paypal.com"
        } else {
            "https://api-m.paypal.com"
        }
    }

    pub fn ensure_configured(cfg: &ProviderConfig) -> Result<()> {
        if cfg.public_client_id.is_empty() || cfg.secret_key.is_empty() {
            bail!("PayPal Client ID and Secret are not configured in Admin → Payment Providers");
        }
        Ok(())
    }

    async fn access_token(cfg: &ProviderConfig) -> Result<String> {
        ensure_configured(cfg)?;
        let resp = reqwest::Client::new()
            .post(format!("{}/v1/oauth2/token", base_url(cfg)))
            .basic_auth(&cfg.public_client_id, Some(&cfg.secret_key))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|e| anyhow!("Could not reach PayPal: {}", e))?;
        let status = resp.status();
        let body: JsonValue = resp.json().await.unwrap_or_default();
        if !status.is_success() {
            bail!(
                "PayPal authentication failed ({}). Check Client ID / Secret and the Sandbox toggle.",
                body["error_description"].as_str().unwrap_or("unknown error")
            );
        }
        body["access_token"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| anyhow!("PayPal returned no access token"))
    }

    fn error_message(body: &JsonValue) -> String {
        let issue = body["details"][0]["issue"].as_str();
        let msg = body["message"].as_str().unwrap_or("Unknown PayPal API error");
        match issue {
            Some(i) => format!("PayPal: {} ({})", msg, i),
            None => format!("PayPal: {}", msg),
        }
    }

    pub async fn create_order(cfg: &ProviderConfig, amount_cents: i32, pending_id: &str, description: &str) -> Result<String> {
        let token = access_token(cfg).await?;
        let body = json!({
            "intent": "CAPTURE",
            "purchase_units": [{
                "reference_id": pending_id,
                "custom_id": pending_id,
                "description": description.chars().take(127).collect::<String>(),
                "amount": { "currency_code": "EUR", "value": format_amount(amount_cents) }
            }],
            "application_context": { "shipping_preference": "NO_SHIPPING", "user_action": "PAY_NOW" }
        });
        let resp = reqwest::Client::new()
            .post(format!("{}/v2/checkout/orders", base_url(cfg)))
            .bearer_auth(token)
            .header("PayPal-Request-Id", format!("order-{}", pending_id))
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Could not reach PayPal: {}", e))?;
        let status = resp.status();
        let body: JsonValue = resp.json().await.unwrap_or_default();
        if !status.is_success() {
            bail!(error_message(&body));
        }
        body["id"].as_str().map(str::to_string).ok_or_else(|| anyhow!("PayPal returned no order id"))
    }

    fn valid_id(order_id: &str) -> Result<()> {
        if order_id.is_empty() || !order_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            bail!("Invalid PayPal order reference");
        }
        Ok(())
    }

    /// Captures an approved order. Already-captured orders are read back instead, so retries are safe.
    pub async fn capture_order(cfg: &ProviderConfig, order_id: &str, pending_id: &str) -> Result<JsonValue> {
        valid_id(order_id)?;
        let token = access_token(cfg).await?;
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/v2/checkout/orders/{}/capture", base_url(cfg), order_id))
            .bearer_auth(&token)
            .header("PayPal-Request-Id", format!("capture-{}", pending_id))
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await
            .map_err(|e| anyhow!("Could not reach PayPal: {}", e))?;
        let status = resp.status();
        let body: JsonValue = resp.json().await.unwrap_or_default();
        if status.is_success() {
            return Ok(body);
        }
        if body["details"][0]["issue"].as_str() == Some("ORDER_ALREADY_CAPTURED") {
            let resp = client
                .get(format!("{}/v2/checkout/orders/{}", base_url(cfg), order_id))
                .bearer_auth(&token)
                .send()
                .await
                .map_err(|e| anyhow!("Could not reach PayPal: {}", e))?;
            return Ok(resp.json().await.unwrap_or_default());
        }
        bail!(error_message(&body))
    }

    pub async fn refund_capture(cfg: &ProviderConfig, capture_id: &str) -> Result<JsonValue> {
        valid_id(capture_id)?;
        let token = access_token(cfg).await?;
        let resp = reqwest::Client::new()
            .post(format!("{}/v2/payments/captures/{}/refund", base_url(cfg), capture_id))
            .bearer_auth(token)
            .header("PayPal-Request-Id", format!("refund-{}", capture_id))
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await
            .map_err(|e| anyhow!("Could not reach PayPal: {}", e))?;
        let status = resp.status();
        let body: JsonValue = resp.json().await.unwrap_or_default();
        if !status.is_success() {
            bail!(error_message(&body));
        }
        Ok(body)
    }
}

/// Fully refunds a captured payment by its provider reference (Stripe PaymentIntent id / PayPal capture id).
pub async fn refund(pool: &PgPool, provider: &str, reference: &str) -> Result<()> {
    let cfg = ProviderConfig::load(pool, provider).await?;
    match provider {
        "stripe" => stripe::refund(&cfg, reference).await.map(|_| ()),
        "paypal" => paypal::refund_capture(&cfg, reference).await.map(|_| ()),
        other => bail!("Automatic refunds are not supported for provider '{}'", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sign(secret: &str, timestamp: i64, payload: &[u8]) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(format!("{}.", timestamp).as_bytes());
        mac.update(payload);
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn webhook_signature_accepts_valid_and_rejects_tampered() {
        let secret = "whsec_test_secret";
        let payload = br#"{"type":"payment_intent.succeeded"}"#;
        let now = chrono::Utc::now().timestamp();
        let header = format!("t={},v1={}", now, sign(secret, now, payload));

        assert!(stripe::verify_webhook_signature(payload, &header, secret, 300).is_ok());
        assert!(stripe::verify_webhook_signature(br#"{"type":"tampered"}"#, &header, secret, 300).is_err());
        assert!(stripe::verify_webhook_signature(payload, &header, "whsec_other", 300).is_err());

        let old = now - 3600;
        let stale = format!("t={},v1={}", old, sign(secret, old, payload));
        assert!(stripe::verify_webhook_signature(payload, &stale, secret, 300).is_err());
    }

    #[test]
    fn formats_amounts_for_paypal() {
        assert_eq!(format_amount(1999), "19.99");
        assert_eq!(format_amount(5), "0.05");
        assert_eq!(format_amount(100), "1.00");
    }
}

#[cfg(test)]
mod method_tests {
    use super::*;

    fn cfg(methods: JsonValue) -> ProviderConfig {
        ProviderConfig {
            is_enabled: true,
            is_sandbox: true,
            public_client_id: String::new(),
            secret_key: String::new(),
            webhook_secret: String::new(),
            config_data: json!({ "methods": methods }),
        }
    }

    #[test]
    fn selected_methods_must_be_enabled() {
        let c = cfg(json!({ "apple_pay": true, "klarna": true, "link": false }));
        assert_eq!(stripe::intent_types_for(&c, "card").unwrap(), vec!["card"]);
        assert_eq!(stripe::intent_types_for(&c, "apple_pay").unwrap(), vec!["card"]);
        assert_eq!(stripe::intent_types_for(&c, "klarna").unwrap(), vec!["klarna"]);
        assert!(stripe::intent_types_for(&c, "google_pay").is_err());
        assert!(stripe::intent_types_for(&c, "link").is_err());
        assert!(stripe::intent_types_for(&c, "bitcoin").is_err());
        let with_link = cfg(json!({ "link": true, "amazon_pay": true }));
        assert_eq!(stripe::intent_types_for(&with_link, "link").unwrap(), vec!["link", "card"]);
        assert_eq!(stripe::intent_types_for(&with_link, "amazon_pay").unwrap(), vec!["amazon_pay"]);
        assert_eq!(stripe::enabled_checkout_methods(&with_link), vec!["card", "link", "amazon_pay"]);
        assert_eq!(stripe::capability_for("google_pay"), "card_payments");
    }
}
