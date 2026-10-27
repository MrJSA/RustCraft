//! Issued invoices and credit notes are stored once and never re-rendered (GoBD: documents must
//! stay unchanged). The invoice is issued when the payment is confirmed; a refund issues a credit
//! note (Stornorechnung) that cancels it.

use crate::models::{Order, OrderItem, StoreSettings};
use crate::services::invoice::Invoice;
use anyhow::{anyhow, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct StoredDocument {
    pub number: String,
    pub file_name: String,
    pub pdf: Vec<u8>,
    pub xml: Option<String>,
}

pub const INVOICE: &str = "invoice";
pub const CREDIT_NOTE: &str = "credit_note";

pub async fn load_order(pool: &PgPool, order_id: Uuid) -> Result<(StoreSettings, Order, Vec<OrderItem>)> {
    let settings = sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await?;
    let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1")
        .bind(order_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Order not found"))?;
    let items = sqlx::query_as::<_, OrderItem>("SELECT * FROM order_items WHERE order_id = $1").bind(order_id).fetch_all(pool).await?;
    Ok((settings, order, items))
}

pub async fn get(pool: &PgPool, order_id: Uuid, kind: &str) -> Result<Option<StoredDocument>> {
    let row = sqlx::query("SELECT number, pdf, xml FROM invoices WHERE order_id = $1 AND kind = $2")
        .bind(order_id)
        .bind(kind)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| {
        let number: String = r.get("number");
        let prefix = if kind == CREDIT_NOTE { "Credit-Note" } else { "Invoice" };
        let safe: String = number.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
        StoredDocument { file_name: format!("{}-{}.pdf", prefix, safe), number, pdf: r.get("pdf"), xml: r.get("xml") }
    }))
}

async fn render_and_store(pool: &PgPool, settings: &StoreSettings, order_id: Uuid, kind: &str, invoice: Invoice) -> Result<StoredDocument> {
    let settings_c = settings.clone();
    let shop_url = crate::services::email::shop_public_url();
    let inv = invoice.clone();
    let (pdf, xml) = tokio::task::spawn_blocking(move || -> Result<(Vec<u8>, Option<String>), String> {
        let logo = crate::services::branding::logo_png(&settings_c, 600);
        let xml = settings_c.einvoice_enabled.then(|| inv.to_cii_xml());
        let pdf = crate::services::invoice_pdf::render(&inv, logo.as_deref(), &shop_url, xml.as_deref())?;
        Ok((pdf, xml))
    })
    .await?
    .map_err(|e| anyhow!(e))?;

    // Two concurrent requests may race: the unique (order_id, kind) keeps the first document
    sqlx::query(
        "INSERT INTO invoices (order_id, kind, number, issued_at, total_cents, pdf, xml) VALUES ($1, $2, $3, NOW(), $4, $5, $6)
         ON CONFLICT (order_id, kind) DO NOTHING",
    )
    .bind(order_id)
    .bind(kind)
    .bind(&invoice.number)
    .bind(invoice.payable_cents)
    .bind(&pdf)
    .bind(&xml)
    .execute(pool)
    .await?;
    get(pool, order_id, kind).await?.ok_or_else(|| anyhow!("The document could not be stored"))
}

/// The order's invoice; issued now if the order has been paid and has none yet.
/// `None` while the payment is still open (no invoice yet).
pub async fn ensure_invoice(pool: &PgPool, order_id: Uuid) -> Result<Option<StoredDocument>> {
    if let Some(doc) = get(pool, order_id, INVOICE).await? {
        return Ok(Some(doc));
    }
    let (settings, order, items) = load_order(pool, order_id).await?;
    if !matches!(order.payment_status.as_str(), "paid" | "refunded") {
        return Ok(None);
    }
    let invoice = Invoice::build(&order, &items, &settings);
    render_and_store(pool, &settings, order_id, INVOICE, invoice).await.map(Some)
}

/// Credit note cancelling the invoice in full (after a refund).
pub async fn ensure_credit_note(pool: &PgPool, order_id: Uuid) -> Result<Option<StoredDocument>> {
    if let Some(doc) = get(pool, order_id, CREDIT_NOTE).await? {
        return Ok(Some(doc));
    }
    let Some(original) = ensure_invoice(pool, order_id).await? else {
        return Ok(None);
    };
    let (settings, order, items) = load_order(pool, order_id).await?;
    let invoice = Invoice::build(&order, &items, &settings);
    let invoice = invoice.credit_note(format!("{}-CN", original.number), chrono::Utc::now().date_naive());
    render_and_store(pool, &settings, order_id, CREDIT_NOTE, invoice).await.map(Some)
}
