//! EU VAT ID check (VIES) and the decision whether a business order is VAT-free:
//! goods shipped to a business in another EU country are an intra-Community supply
//! (§ 4 Nr. 1b / § 6a UStG, Art. 138 VAT Directive), digital products are reverse charge
//! (§ 13b UStG, Art. 196 VAT Directive). Both need a valid VAT ID of the buyer from another EU
//! country, confirmed by VIES; the answer (incl. the consultation number) is kept as proof.

use crate::models::{CheckoutRequest, StoreSettings};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};

const VIES_URL: &str = "https://ec.europa.eu/taxation_customs/vies/rest-api/check-vat-number";
const CACHE_HOURS: i64 = 24;

/// EU member states: ISO country code → VAT ID prefix (Greece uses EL).
pub const EU_COUNTRIES: [(&str, &str); 27] = [
    ("AT", "AT"), ("BE", "BE"), ("BG", "BG"), ("CY", "CY"), ("CZ", "CZ"), ("DE", "DE"), ("DK", "DK"),
    ("EE", "EE"), ("GR", "EL"), ("ES", "ES"), ("FI", "FI"), ("FR", "FR"), ("HR", "HR"), ("HU", "HU"),
    ("IE", "IE"), ("IT", "IT"), ("LT", "LT"), ("LU", "LU"), ("LV", "LV"), ("MT", "MT"), ("NL", "NL"),
    ("PL", "PL"), ("PT", "PT"), ("RO", "RO"), ("SE", "SE"), ("SI", "SI"), ("SK", "SK"),
];

pub fn vat_prefix(country: &str) -> Option<&'static str> {
    EU_COUNTRIES.iter().find(|(c, _)| *c == country).map(|(_, p)| *p)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VatCheck {
    pub vat_id: String,
    pub valid: bool,
    pub name: Option<String>,
    pub address: Option<String>,
    /// VIES consultation number (proof of the check)
    pub request_identifier: Option<String>,
    pub checked_at: DateTime<Utc>,
}

/// "de 123 456 789" → "DE123456789"
pub fn normalize(vat_id: &str) -> String {
    vat_id.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_uppercase()
}

fn split(vat_id: &str) -> Option<(&str, &str)> {
    if vat_id.len() < 4 || !vat_id.is_char_boundary(2) {
        return None;
    }
    let (cc, number) = vat_id.split_at(2);
    (cc.chars().all(|c| c.is_ascii_alphabetic()) && !number.is_empty()).then_some((cc, number))
}

/// Checks a VAT ID with VIES (answers are cached for a day).
pub async fn lookup(pool: &PgPool, settings: &StoreSettings, vat_id: &str) -> Result<VatCheck, String> {
    let vat_id = normalize(vat_id);
    let (cc, number) = split(&vat_id).ok_or_else(|| "Please enter the VAT ID with its country code, e.g. FR12345678901.".to_string())?;
    if !EU_COUNTRIES.iter().any(|(_, p)| *p == cc) {
        return Err("Only VAT IDs of EU member states can be checked.".to_string());
    }

    if let Ok(Some(row)) = sqlx::query("SELECT * FROM vat_checks WHERE vat_id = $1 AND checked_at > NOW() - make_interval(hours => $2)")
        .bind(&vat_id)
        .bind(CACHE_HOURS as i32)
        .fetch_optional(pool)
        .await
    {
        return Ok(VatCheck {
            vat_id: vat_id.clone(),
            valid: row.get("valid"),
            name: row.get("name"),
            address: row.get("address"),
            request_identifier: row.get("request_identifier"),
            checked_at: row.get("checked_at"),
        });
    }

    let mut body = json!({ "countryCode": cc, "vatNumber": number });
    // With the shop's own VAT ID, VIES returns a consultation number that proves the check
    if let Some((own_cc, own_number)) = split(&normalize(&settings.vat_id)) {
        body["requesterMemberStateCode"] = json!(own_cc);
        body["requesterNumber"] = json!(own_number);
    }
    let resp = reqwest::Client::new()
        .post(VIES_URL)
        .timeout(std::time::Duration::from_secs(15))
        .json(&body)
        .send()
        .await
        .map_err(|_| "The EU VAT ID service (VIES) is not reachable right now. Please try again later or order without VAT ID.".to_string())?;
    let data: Value = resp.json().await.unwrap_or(Value::Null);
    if let Some(err) = data["errorWrappers"][0]["error"].as_str().or_else(|| data["userError"].as_str().filter(|e| *e != "VALID" && *e != "INVALID")) {
        return Err(format!(
            "The VAT ID could not be checked right now (VIES: {}). Please try again later or order without VAT ID.",
            err
        ));
    }
    let valid = data["valid"].as_bool().or_else(|| data["isValid"].as_bool()).unwrap_or(false);
    let clean = |v: &Value| v.as_str().map(str::trim).filter(|s| !s.is_empty() && *s != "---").map(str::to_string);
    let check = VatCheck {
        vat_id: vat_id.clone(),
        valid,
        name: clean(&data["name"]),
        address: clean(&data["address"]),
        request_identifier: clean(&data["requestIdentifier"]),
        checked_at: Utc::now(),
    };
    let _ = sqlx::query(
        "INSERT INTO vat_checks (vat_id, valid, name, address, request_identifier, checked_at) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (vat_id) DO UPDATE SET valid = $2, name = $3, address = $4, request_identifier = $5, checked_at = $6",
    )
    .bind(&check.vat_id)
    .bind(check.valid)
    .bind(&check.name)
    .bind(&check.address)
    .bind(&check.request_identifier)
    .bind(check.checked_at)
    .execute(pool)
    .await;
    Ok(check)
}

/// Shop country (ISO code) for the intra-EU check.
pub fn shop_country(settings: &StoreSettings) -> String {
    crate::services::invoice::country_code(&settings.address_country)
}

/// Whether the shop can sell VAT-free to EU businesses at all.
pub fn available(settings: &StoreSettings) -> bool {
    settings.reverse_charge_enabled && settings.tax_mode != "kleingewerbe" && !settings.vat_id.trim().is_empty()
}

/// The VIES-confirmed VAT ID if this order is a VAT-free EU business order, `None` for a normal
/// order, or an error the customer has to fix (invalid VAT ID, wrong country, VIES unavailable).
pub async fn reverse_charge_for(pool: &PgPool, settings: &StoreSettings, req: &CheckoutRequest) -> Result<Option<VatCheck>, String> {
    let Some(vat_id) = req.vat_id.as_deref().map(normalize).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    if !available(settings) {
        return Ok(None);
    }
    let billing = req.billing_address.as_ref().unwrap_or(&req.shipping_address);
    let country = billing.country_code.trim().to_uppercase();
    // Same country as the shop: a normal order with VAT (the VAT ID is printed on the invoice)
    if country == shop_country(settings) {
        return Ok(None);
    }
    let Some(prefix) = vat_prefix(&country) else {
        return Ok(None);
    };
    if !vat_id.starts_with(prefix) {
        return Err(format!("The VAT ID must be from the billing country ({}): it has to start with {}.", country, prefix));
    }
    if req.buyer_company.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err("Please enter the company name for a VAT-free business order.".to_string());
    }
    let check = lookup(pool, settings, &vat_id).await?;
    if !check.valid {
        return Err(format!("The VAT ID {} is not valid according to the EU VIES service.", vat_id));
    }
    Ok(Some(check))
}

/// Proof stored with the order.
pub fn proof_json(check: &VatCheck) -> Value {
    serde_json::to_value(check).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vat_ids_are_normalized_and_split() {
        assert_eq!(normalize(" fr 12 345.678-901 "), "FR12345678901");
        assert_eq!(split("FR12345678901"), Some(("FR", "12345678901")));
        assert_eq!(split("12"), None);
        assert_eq!(split("1234567"), None);
        assert_eq!(vat_prefix("GR"), Some("EL"));
        assert_eq!(vat_prefix("CH"), None);
    }
}
