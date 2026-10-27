//! Authentication primitives shared by admin and customer routes.

use crate::models::Claims;
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration as StdDuration, Instant};
use tracing::{info, warn};

pub const ROLE_ADMIN_TOKEN: &str = "admin";
pub const ROLE_CUSTOMER_TOKEN: &str = "customer";

/// Values that were published in this repository and must never be used as a signing key.
const KNOWN_PUBLIC_SECRETS: &[&str] = &["super_secret_rustwebshop_jwt_token_2026"];

static JWT_SECRET: OnceLock<String> = OnceLock::new();

/// Cryptographically random hex string with `bytes` bytes of entropy.
pub fn random_token(bytes: usize) -> String {
    let mut out = Vec::with_capacity(bytes);
    while out.len() < bytes {
        // UUIDv4 is generated from the OS CSPRNG (getrandom)
        out.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    }
    out.truncate(bytes);
    hex::encode(out)
}

pub fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

/// Loads the JWT signing key: a strong `JWT_SECRET` env var, otherwise a random key persisted in the database.
pub async fn init(pool: &PgPool) -> Result<()> {
    let from_env = std::env::var("JWT_SECRET").ok().filter(|s| s.len() >= 32 && !KNOWN_PUBLIC_SECRETS.contains(&s.as_str()));
    let secret = match from_env {
        Some(s) => s,
        None => {
            if std::env::var("JWT_SECRET").map(|s| !s.is_empty()).unwrap_or(false) {
                warn!("JWT_SECRET is too short or a publicly known default — ignoring it and using a generated key");
            }
            let generated = random_token(48);
            sqlx::query("INSERT INTO server_secrets (name, value) VALUES ('jwt_secret', $1) ON CONFLICT (name) DO NOTHING")
                .bind(&generated)
                .execute(pool)
                .await?;
            let stored: String = sqlx::query_scalar("SELECT value FROM server_secrets WHERE name = 'jwt_secret'")
                .fetch_one(pool)
                .await?;
            info!("Using database-persisted JWT signing key");
            stored
        }
    };
    JWT_SECRET.set(secret).map_err(|_| anyhow!("auth already initialised"))?;
    Ok(())
}

fn secret() -> &'static [u8] {
    JWT_SECRET.get().expect("auth::init must run at startup").as_bytes()
}

pub fn issue_token(sub: &str, role: &str, ttl: Duration) -> Result<String> {
    let claims = Claims {
        sub: sub.to_string(),
        role: role.to_string(),
        exp: (Utc::now() + ttl).timestamp() as usize,
    };
    Ok(encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret()))?)
}

/// Decodes a token and checks that it was issued for `expected_role`.
pub fn verify_token(token: &str, expected_role: &str) -> Option<Claims> {
    let validation = Validation::new(Algorithm::HS256);
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret()), &validation).ok()?;
    (data.claims.role == expected_role).then_some(data.claims)
}

pub fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
}

// ---------------------------------------------------------------- brute-force protection

const MAX_FAILURES: u32 = 8;
const WINDOW: StdDuration = StdDuration::from_secs(15 * 60);

static FAILURES: OnceLock<Mutex<HashMap<String, (u32, Instant)>>> = OnceLock::new();

fn failures() -> &'static Mutex<HashMap<String, (u32, Instant)>> {
    FAILURES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Rejects further login attempts for an account after repeated failures (per account, 15 minutes).
pub fn check_login_allowed(key: &str) -> Result<(), String> {
    let mut map = failures().lock().unwrap();
    map.retain(|_, (_, since)| since.elapsed() < WINDOW);
    match map.get(key) {
        Some((count, _)) if *count >= MAX_FAILURES => Err("Too many failed login attempts. Please wait 15 minutes and try again.".to_string()),
        _ => Ok(()),
    }
}

pub fn record_login_failure(key: &str) {
    let mut map = failures().lock().unwrap();
    let entry = map.entry(key.to_string()).or_insert((0, Instant::now()));
    entry.0 += 1;
}

pub fn clear_login_failures(key: &str) {
    failures().lock().unwrap().remove(key);
}

const RESEND_COOLDOWN: StdDuration = StdDuration::from_secs(60);
static RESEND_TIMESTAMPS: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();

fn resend_timestamps() -> &'static Mutex<HashMap<String, Instant>> {
    RESEND_TIMESTAMPS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Enforces a 1-minute cooldown on verification email resends per address.
/// Returns Err(seconds_remaining) if too soon.
pub fn check_resend_allowed(email: &str) -> Result<(), u64> {
    let mut map = resend_timestamps().lock().unwrap();
    map.retain(|_, since| since.elapsed() < RESEND_COOLDOWN);
    if let Some(since) = map.get(&email.to_lowercase()) {
        let elapsed = since.elapsed();
        if elapsed < RESEND_COOLDOWN {
            let remaining = (RESEND_COOLDOWN - elapsed).as_secs();
            return Err(remaining.max(1));
        }
    }
    Ok(())
}

pub fn record_resend_sent(email: &str) {
    let mut map = resend_timestamps().lock().unwrap();
    map.insert(email.to_lowercase(), Instant::now());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_tokens_are_unique_hex() {
        let a = random_token(32);
        let b = random_token(32);
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn tokens_are_bound_to_their_role() {
        let _ = JWT_SECRET.set("test-secret-that-is-long-enough-0123456789".to_string());
        let admin = issue_token("id-1", ROLE_ADMIN_TOKEN, Duration::hours(1)).unwrap();
        let customer = issue_token("a@b.c", ROLE_CUSTOMER_TOKEN, Duration::hours(1)).unwrap();
        assert!(verify_token(&admin, ROLE_ADMIN_TOKEN).is_some());
        assert!(verify_token(&customer, ROLE_ADMIN_TOKEN).is_none());
        assert!(verify_token(&format!("{}x", admin), ROLE_ADMIN_TOKEN).is_none());
        let expired = issue_token("id-1", ROLE_ADMIN_TOKEN, Duration::hours(-2)).unwrap();
        assert!(verify_token(&expired, ROLE_ADMIN_TOKEN).is_none());
    }

    #[test]
    fn login_limiter_locks_after_repeated_failures() {
        let key = "test:limiter";
        clear_login_failures(key);
        for _ in 0..MAX_FAILURES {
            assert!(check_login_allowed(key).is_ok());
            record_login_failure(key);
        }
        assert!(check_login_allowed(key).is_err());
        clear_login_failures(key);
        assert!(check_login_allowed(key).is_ok());
    }

    #[test]
    fn resend_cooldown_enforced_for_one_minute() {
        let email = "cooldown-test@example.com";
        assert!(check_resend_allowed(email).is_ok());
        record_resend_sent(email);
        let err = check_resend_allowed(email);
        assert!(err.is_err());
        let remaining = err.unwrap_err();
        assert!(remaining <= 60 && remaining >= 1);
    }
}
