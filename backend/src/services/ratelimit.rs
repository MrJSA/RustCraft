//! Request limits per client IP, plus bounded bcrypt work.
//!
//! The client IP comes from `x-shop-client-ip`, which only the storefront and admin servers set (they take it
//! from the reverse proxy or the TCP connection and strip any value a browser sends). The backend trusts the
//! header only on its internal port; the optional debug port removes it (see `main.rs`).

use axum::http::HeaderMap;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

pub const CLIENT_IP_HEADER: &str = "x-shop-client-ip";

/// Fixed-window counter per key (client IP or e-mail address).
pub struct Limiter {
    max: u32,
    window: Duration,
    buckets: Mutex<HashMap<String, (u32, Instant)>>,
}

impl Limiter {
    pub fn new(max: u32, window: Duration) -> Self {
        Limiter { max, window, buckets: Mutex::new(HashMap::new()) }
    }

    /// `Err(seconds)` until the window of a key that reached the limit has passed.
    pub fn check(&self, key: &str) -> Result<(), u64> {
        let map = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        match map.get(key) {
            Some((count, start)) if *count >= self.max && start.elapsed() < self.window => {
                Err((self.window - start.elapsed()).as_secs().max(1))
            }
            _ => Ok(()),
        }
    }

    /// Counts one event for the key.
    pub fn hit(&self, key: &str) {
        let mut map = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        // Keep memory bounded: expired windows are dropped once the map grows
        if map.len() > 4096 {
            let window = self.window;
            map.retain(|_, (_, start)| start.elapsed() < window);
        }
        let entry = map.entry(key.to_string()).or_insert((0, Instant::now()));
        if entry.1.elapsed() >= self.window {
            *entry = (0, Instant::now());
        }
        entry.0 = entry.0.saturating_add(1);
    }

    /// For actions that always count (sign-ups, e-mails): check and count in one step.
    pub fn check_and_hit(&self, key: &str) -> Result<(), u64> {
        self.check(key)?;
        self.hit(key);
        Ok(())
    }
}

const FIFTEEN_MINUTES: Duration = Duration::from_secs(15 * 60);
const HOUR: Duration = Duration::from_secs(3600);

/// Failed admin logins per client IP.
pub static ADMIN_LOGIN_FAILURES: LazyLock<Limiter> = LazyLock::new(|| Limiter::new(10, FIFTEEN_MINUTES));
/// Failed customer logins per client IP (credential stuffing across many accounts).
pub static CUSTOMER_LOGIN_FAILURES: LazyLock<Limiter> = LazyLock::new(|| Limiter::new(10, FIFTEEN_MINUTES));
/// Account registrations per client IP.
pub static REGISTRATIONS: LazyLock<Limiter> = LazyLock::new(|| Limiter::new(5, HOUR));
/// Password-reset requests and new-password submissions per client IP.
pub static PASSWORD_RESETS: LazyLock<Limiter> = LazyLock::new(|| Limiter::new(10, HOUR));
/// Verification e-mail resends per client IP.
pub static VERIFICATION_RESENDS: LazyLock<Limiter> = LazyLock::new(|| Limiter::new(5, HOUR));
/// Newsletter sign-ups per client IP.
pub static NEWSLETTER_SIGNUPS: LazyLock<Limiter> = LazyLock::new(|| Limiter::new(5, HOUR));
/// E-mails (reset links, verification links) per recipient address — no mailbox gets flooded.
pub static EMAILS_PER_ADDRESS: LazyLock<Limiter> = LazyLock::new(|| Limiter::new(5, HOUR));

/// Limiter key for the client: the IPv4 address, or the /64 network of an IPv6 address
/// (one IPv6 connection usually owns a whole /64, so single addresses would be free to rotate).
pub fn client_key(headers: &HeaderMap) -> String {
    let ip = headers
        .get(CLIENT_IP_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<IpAddr>().ok());
    match ip {
        Some(IpAddr::V4(v4)) => v4.to_string(),
        Some(IpAddr::V6(v6)) => match v6.to_ipv4_mapped() {
            Some(v4) => v4.to_string(),
            None => {
                let s = v6.segments();
                format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
            }
        },
        // Direct calls without the storefront/admin in front (local debugging) share one bucket
        None => "direct".to_string(),
    }
}

pub fn too_many(seconds: u64) -> (axum::http::StatusCode, String) {
    let wait = if seconds >= 120 { format!("{} minutes", seconds.div_ceil(60)) } else { format!("{} seconds", seconds) };
    (axum::http::StatusCode::TOO_MANY_REQUESTS, format!("Too many attempts from your network. Please try again in {}.", wait))
}

// ---------------------------------------------------------------- bcrypt

/// bcrypt is deliberately slow (~0.1–0.3 s CPU each). Only a few run at once, on blocking threads,
/// so a login flood can neither stall the async runtime nor take every CPU core of a shared server.
static BCRYPT_SLOTS: LazyLock<Semaphore> = LazyLock::new(|| {
    let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
    Semaphore::new(cpus.div_ceil(2).clamp(1, 4))
});
/// Requests that cannot get a bcrypt slot within this time are turned away instead of queueing forever.
const BCRYPT_QUEUE_TIMEOUT: Duration = Duration::from_secs(10);

fn busy() -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::SERVICE_UNAVAILABLE, "The shop is busy right now. Please try again in a moment.".to_string())
}

/// Checks a password against a bcrypt hash with bounded concurrency.
pub async fn verify_password(password: &str, hash: &str) -> Result<bool, (axum::http::StatusCode, String)> {
    let _slot = tokio::time::timeout(BCRYPT_QUEUE_TIMEOUT, BCRYPT_SLOTS.acquire()).await.map_err(|_| busy())?.map_err(|_| busy())?;
    let (password, hash) = (zeroize::Zeroizing::new(password.to_string()), hash.to_string());
    tokio::task::spawn_blocking(move || bcrypt::verify(password.as_str(), &hash).unwrap_or(false)).await.map_err(|_| busy())
}

/// Hashes a password with bounded concurrency.
pub async fn hash_password(password: &str, cost: u32) -> Result<String, (axum::http::StatusCode, String)> {
    let _slot = tokio::time::timeout(BCRYPT_QUEUE_TIMEOUT, BCRYPT_SLOTS.acquire()).await.map_err(|_| busy())?.map_err(|_| busy())?;
    let password = zeroize::Zeroizing::new(password.to_string());
    tokio::task::spawn_blocking(move || bcrypt::hash(password.as_str(), cost))
        .await
        .map_err(|_| busy())?
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(ip: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(CLIENT_IP_HEADER, ip.parse().unwrap());
        h
    }

    #[test]
    fn limits_per_key_until_the_window_passes() {
        let l = Limiter::new(3, Duration::from_millis(200));
        for _ in 0..3 {
            assert!(l.check_and_hit("1.2.3.4").is_ok());
        }
        assert!(l.check("1.2.3.4").is_err());
        assert!(l.check("5.6.7.8").is_ok(), "other clients are not affected");
        std::thread::sleep(Duration::from_millis(220));
        assert!(l.check_and_hit("1.2.3.4").is_ok(), "a new window starts");
    }

    #[test]
    fn ipv6_clients_are_grouped_per_64_and_garbage_is_ignored() {
        assert_eq!(client_key(&headers("203.0.113.7")), "203.0.113.7");
        assert_eq!(client_key(&headers("2001:db8:1:2::1")), client_key(&headers("2001:db8:1:2:ffff::9")));
        assert_ne!(client_key(&headers("2001:db8:1:2::1")), client_key(&headers("2001:db8:1:3::1")));
        assert_eq!(client_key(&headers("::ffff:203.0.113.7")), "203.0.113.7");
        assert_eq!(client_key(&headers("not-an-ip")), "direct");
        assert_eq!(client_key(&HeaderMap::new()), "direct");
    }

    #[tokio::test]
    async fn bcrypt_runs_bounded_and_correct() {
        let hash = hash_password("correct horse", 4).await.unwrap();
        assert!(verify_password("correct horse", &hash).await.unwrap());
        assert!(!verify_password("wrong", &hash).await.unwrap());
        // Many at once all finish (queued, not rejected)
        let checks = (0..12).map(|_| {
            let h = hash.clone();
            tokio::spawn(async move { verify_password("correct horse", &h).await.unwrap() })
        });
        for c in checks {
            assert!(c.await.unwrap());
        }
    }
}
