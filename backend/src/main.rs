mod db;
mod middleware;
mod models;
mod routes;
mod services;

use std::net::SocketAddr;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Rust E-Commerce Backend Service...");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://shop_user:local_dev_password@127.0.0.1:5432/shop_db".to_string()
    });

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .unwrap_or(8000);

    if db::weak_database_password(&database_url) {
        tracing::warn!(
            "SECURITY: the database password is a published default or shorter than 16 characters. \
             Set a long random DB_PASSWORD (openssl rand -hex 24) and change it in PostgreSQL too (see HARDEN.md)."
        );
    }

    // Database connection with exponential retry loop for Docker orchestration
    let mut pool = None;
    let mut retry_count = 0;
    while pool.is_none() && retry_count < 15 {
        match db::init_db(&database_url).await {
            Ok(p) => {
                pool = Some(p);
                break;
            }
            Err(e) => {
                retry_count += 1;
                error!(
                    "Failed to connect to PostgreSQL (attempt {}/15): {}. Retrying in 2s...",
                    retry_count, e
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    let pool = pool.expect("Unable to establish PostgreSQL database connection after retries");

    // Ensure uploads directory exists
    tokio::fs::create_dir_all("uploads").await.ok();

    crate::services::auth::init(&pool).await.expect("Failed to initialise authentication secrets");

    // Automatic off-site backups (Admin → Settings → Export & Backups)
    crate::services::backup::spawn_scheduler(pool.clone());
    // Price history for the "lowest price of the last 30 days" (scheduled sale starts/ends)
    crate::services::pricing::spawn_price_recorder(pool.clone());

    // No CORS layer: browsers never call this API directly (only the storefront/admin servers do),
    // so cross-origin browser requests are refused by default.
    // Uploaded media is public, except files that are sold as digital products
    let uploads = axum::Router::new()
        .nest_service("/uploads", tower_http::services::ServeDir::new("uploads"))
        .layer(axum::middleware::from_fn_with_state(pool.clone(), crate::routes::shop_features::guard_uploads));
    // Readiness check for the updater (includes the database). Not reachable from outside: the
    // storefront and admin only forward /api/… paths and the backend port is bound to 127.0.0.1.
    let health = axum::Router::new()
        .route("/health", axum::routing::get(health))
        .with_state(pool.clone());
    let app = routes::create_router(pool)
        .merge(uploads)
        .merge(health)
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024));

    // Debug port (published only when BACKEND_DEBUG_PORT is set): same API, but the client-IP header is
    // dropped, so nobody on the host can pretend to be another client and dodge the per-IP limits.
    // Only the internal port, which only the storefront and admin reach, accepts the header.
    let debug_port: u16 = std::env::var("DEBUG_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8001);
    if debug_port != port {
        let debug_app = app.clone().layer(axum::middleware::map_request(strip_client_ip));
        let debug_addr = SocketAddr::from(([0, 0, 0, 0], debug_port));
        let debug_listener = tokio::net::TcpListener::bind(debug_addr).await?;
        info!("Debug listener on http://{} (client-IP header ignored)", debug_addr);
        tokio::spawn(async move {
            if let Err(e) = axum::serve(debug_listener, debug_app).await {
                error!("Debug listener stopped: {}", e);
            }
        });
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Axum Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn strip_client_ip(mut req: axum::extract::Request) -> axum::extract::Request {
    req.headers_mut().remove(crate::services::ratelimit::CLIENT_IP_HEADER);
    req
}


async fn health(axum::extract::State(pool): axum::extract::State<sqlx::PgPool>) -> axum::http::StatusCode {
    match sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&pool).await {
        Ok(_) => axum::http::StatusCode::OK,
        Err(_) => axum::http::StatusCode::SERVICE_UNAVAILABLE,
    }
}
