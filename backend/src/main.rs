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
    let app = routes::create_router(pool)
        .merge(uploads)
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Axum Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

