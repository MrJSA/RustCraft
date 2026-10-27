pub mod admin;
pub mod payments;
pub mod public;
pub mod shop_features;

use axum::Router;
use sqlx::PgPool;

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/v1", public::public_router().merge(payments::payments_router()).merge(shop_features::public_router()))
        .nest("/api/v1/admin", admin::admin_router(pool.clone()))
        .with_state(pool)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn router_has_no_conflicting_routes() {
        // Building the router panics on overlapping paths; no database connection is needed
        let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy("postgres://user:pass@localhost/db").unwrap();
        let _ = super::create_router(pool);
    }
}
