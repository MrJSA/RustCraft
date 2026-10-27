use crate::services::auth::{bearer_token, verify_token, ROLE_ADMIN_TOKEN};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value as JsonValue};
use sqlx::{PgPool, Row};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Admin areas that can be granted per user (matches the admin navigation groups).
pub const SECTIONS: [&str; 5] = ["overview", "products", "orders", "storefront", "settings"];

/// Defaults for a role: settings only for superadmins, overview & storefront not for editors.
pub fn default_permission(role: &str, section: &str) -> bool {
    match (role, section) {
        ("superadmin", _) => true,
        (_, "settings") => false,
        ("editor", "overview" | "storefront") => false,
        _ => true,
    }
}

/// Stored permissions merged over the role defaults. Superadmins always have everything.
pub fn effective_permissions(role: &str, stored: &JsonValue) -> BTreeMap<String, bool> {
    SECTIONS
        .iter()
        .map(|s| {
            let allowed = role == "superadmin" || stored[*s].as_bool().unwrap_or_else(|| default_permission(role, s));
            (s.to_string(), allowed)
        })
        .collect()
}

/// The authenticated admin, available to handlers via `Extension<CurrentAdmin>`.
#[derive(Debug, Clone)]
pub struct CurrentAdmin {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub is_default: bool,
    pub permissions: BTreeMap<String, bool>,
}

impl CurrentAdmin {
    pub fn is_superadmin(&self) -> bool {
        self.role == "superadmin"
    }
    pub fn can(&self, section: &str) -> bool {
        self.permissions.get(section).copied().unwrap_or(false)
    }
}

fn deny(status: StatusCode, code: &str, message: &str) -> Response {
    (status, Json(json!({ "error": message, "code": code }))).into_response()
}

/// Paths an account that still has its initial password may use.
fn allowed_with_default_password(path: &str) -> bool {
    matches!(path, "/auth/status" | "/auth/me" | "/auth/change-credentials")
}

#[derive(Debug, PartialEq)]
pub enum Access {
    /// Any logged-in admin
    Always,
    /// At least one of these sections
    AnyOf(&'static [&'static str]),
    /// Checked field-by-field inside the handler (shared store settings)
    Handler,
}

/// Which section(s) an admin API request needs. Unknown paths require settings (deny by default).
pub fn required_access(method: &Method, path: &str) -> Access {
    let starts = |p: &str| path == p || path.starts_with(&format!("{}/", p));
    let read = method == Method::GET;

    if starts("/auth") {
        return Access::Always;
    }
    if path == "/settings/system" {
        return if read { Access::Always } else { Access::Handler };
    }
    if starts("/dashboard") || starts("/analytics") {
        return Access::AnyOf(&["overview"]);
    }
    if (starts("/products") || starts("/categories")) && read {
        // Storefront pages pick products/categories for carousels and menus
        return Access::AnyOf(&["products", "storefront"]);
    }
    if starts("/products") || starts("/variants") || starts("/parts") || starts("/bom-parts") || starts("/categories") || starts("/coupons") || starts("/logistics") {
        return Access::AnyOf(&["products"]);
    }
    if starts("/orders") || starts("/withdrawals") {
        return Access::AnyOf(&["orders"]);
    }
    if starts("/reviews") {
        return Access::AnyOf(&["products"]);
    }
    if starts("/newsletter") {
        return Access::AnyOf(&["storefront"]);
    }
    if starts("/menu") || starts("/pages") {
        return Access::AnyOf(&["storefront"]);
    }
    if starts("/media") {
        return Access::AnyOf(&["products", "storefront", "settings"]);
    }
    Access::AnyOf(&["settings"])
}

pub async fn admin_auth_middleware(State(pool): State<PgPool>, mut req: Request<Body>, next: Next) -> Result<Response, Response> {
    let claims = bearer_token(req.headers())
        .and_then(|t| verify_token(t, ROLE_ADMIN_TOKEN))
        .ok_or_else(|| deny(StatusCode::UNAUTHORIZED, "unauthenticated", "Please log in"))?;

    let admin_id = Uuid::parse_str(&claims.sub).map_err(|_| deny(StatusCode::UNAUTHORIZED, "unauthenticated", "Please log in again"))?;
    let row = sqlx::query("SELECT id, username, email, role, is_default, permissions FROM admin_users WHERE id = $1")
        .bind(admin_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| deny(StatusCode::SERVICE_UNAVAILABLE, "unavailable", "Database unavailable"))?
        .ok_or_else(|| deny(StatusCode::UNAUTHORIZED, "unauthenticated", "This admin account no longer exists"))?;

    let role: String = row.get("role");
    let admin = CurrentAdmin {
        id: row.get("id"),
        username: row.get("username"),
        email: row.try_get::<Option<String>, _>("email").ok().flatten(),
        permissions: effective_permissions(&role, &row.get::<JsonValue, _>("permissions")),
        role,
        is_default: row.get("is_default"),
    };

    // Nested routers see the path without the /api/v1/admin prefix
    let path = req.uri().path().trim_start_matches("/api/v1/admin").to_string();

    if admin.is_default && !allowed_with_default_password(&path) {
        return Err(deny(StatusCode::FORBIDDEN, "password_change_required", "Please change the initial password before using the admin area"));
    }
    match required_access(req.method(), &path) {
        Access::AnyOf(sections) if !sections.iter().any(|s| admin.can(s)) => {
            return Err(deny(StatusCode::FORBIDDEN, "forbidden", "You do not have permission for this area"));
        }
        Access::Handler if !admin.can("storefront") && !admin.can("settings") => {
            return Err(deny(StatusCode::FORBIDDEN, "forbidden", "You do not have permission for this area"));
        }
        _ => {}
    }

    req.extensions_mut().insert(admin);
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_defaults_match_policy() {
        let editor = effective_permissions("editor", &json!({}));
        assert_eq!(editor["products"], true);
        assert_eq!(editor["orders"], true);
        assert_eq!(editor["overview"], false);
        assert_eq!(editor["storefront"], false);
        assert_eq!(editor["settings"], false);
        let admin = effective_permissions("admin", &json!({}));
        assert!(admin["overview"] && admin["storefront"] && !admin["settings"]);
        // Superadmins cannot be restricted
        let sa = effective_permissions("superadmin", &json!({ "settings": false }));
        assert!(sa.values().all(|v| *v));
        // Stored values override defaults
        let custom = effective_permissions("editor", &json!({ "settings": true, "orders": false }));
        assert!(custom["settings"] && !custom["orders"]);
    }

    #[test]
    fn api_paths_map_to_sections() {
        assert_eq!(required_access(&Method::GET, "/users"), Access::AnyOf(&["settings"]));
        assert_eq!(required_access(&Method::PUT, "/settings/payments/stripe"), Access::AnyOf(&["settings"]));
        assert_eq!(required_access(&Method::GET, "/export/store-data"), Access::AnyOf(&["settings"]));
        assert_eq!(required_access(&Method::PUT, "/backup/settings"), Access::AnyOf(&["settings"]));
        assert_eq!(required_access(&Method::PUT, "/reviews/abc"), Access::AnyOf(&["products"]));
        assert_eq!(required_access(&Method::POST, "/newsletter/campaigns"), Access::AnyOf(&["storefront"]));
        assert_eq!(required_access(&Method::GET, "/withdrawals"), Access::AnyOf(&["orders"]));
        assert_eq!(required_access(&Method::GET, "/settings/system"), Access::Always);
        assert_eq!(required_access(&Method::PUT, "/settings/system"), Access::Handler);
        assert_eq!(required_access(&Method::GET, "/orders"), Access::AnyOf(&["orders"]));
        assert_eq!(required_access(&Method::POST, "/products"), Access::AnyOf(&["products"]));
        assert_eq!(required_access(&Method::GET, "/products"), Access::AnyOf(&["products", "storefront"]));
        assert_eq!(required_access(&Method::PUT, "/menu/abc"), Access::AnyOf(&["storefront"]));
        assert_eq!(required_access(&Method::GET, "/dashboard/stats"), Access::AnyOf(&["overview"]));
        assert_eq!(required_access(&Method::GET, "/something-new"), Access::AnyOf(&["settings"]));
        assert_eq!(required_access(&Method::GET, "/productsx"), Access::AnyOf(&["settings"]));
    }

    #[test]
    fn default_password_accounts_are_limited() {
        assert!(allowed_with_default_password("/auth/change-credentials"));
        assert!(!allowed_with_default_password("/orders"));
    }
}
