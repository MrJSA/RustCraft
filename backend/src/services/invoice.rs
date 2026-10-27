//! The invoice of an order as one calculated model, rendered as PDF (`invoice_pdf`) and as an
//! EN 16931 compliant UN/CEFACT CII XML (ZUGFeRD 2 / Factur-X, profile EN 16931).
//!
//! All amounts are integer cents. Lines, discounts and shipping are converted to net amounts per VAT
//! rate, the VAT is calculated per rate, and the difference between that calculation and the amount
//! the customer actually paid (at most a few cents from rounding) is stated as rounding amount
//! (BT-114) — so PDF, XML and payment always agree.

use crate::models::{Order, OrderItem, StoreSettings};
use chrono::NaiveDate;
use std::fmt::Write;

#[derive(Debug, Clone, Default)]
pub struct Party {
    pub name: String,
    /// Contact person when the party is a company
    pub contact: String,
    pub vat_id: String,
    pub street: String,
    pub extra: String,
    pub postal_code: String,
    pub city: String,
    pub country_code: String,
    pub email: String,
    pub phone: String,
}

impl Party {
    /// Name to greet: the contact person of a company, otherwise the name
    pub fn person(&self) -> &str {
        if self.contact.is_empty() { &self.name } else { &self.contact }
    }
}

#[derive(Debug, Clone)]
pub struct Seller {
    pub party: Party,
    pub trade_name: String,
    pub owner: String,
    pub vat_id: String,
    pub tax_number: String,
    pub register: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VatRate {
    /// S = standard, Z = zero rated, E = exempt (§ 19 UStG small business)
    pub category: &'static str,
    pub percent: f64,
}

impl VatRate {
    fn key(&self) -> (&'static str, i64) {
        (self.category, (self.percent * 100.0).round() as i64)
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceLine {
    pub position: usize,
    pub sku: String,
    pub name: String,
    pub variant: Option<String>,
    pub is_digital: bool,
    pub quantity: i64,
    pub vat: VatRate,
    /// Price as shown to the customer (gross for B2C/small business, net for B2B pricing)
    pub unit_display_cents: i64,
    pub line_display_cents: i64,
    /// Net unit price in 1/10000 of the currency (EN 16931 allows 4 decimals for prices)
    pub unit_net_e4: i64,
    pub line_net_cents: i64,
}

#[derive(Debug, Clone)]
pub struct DocumentAmount {
    pub vat: VatRate,
    pub net_cents: i64,
}

#[derive(Debug, Clone)]
pub struct TaxGroup {
    pub vat: VatRate,
    pub basis_cents: i64,
    pub tax_cents: i64,
}

#[derive(Debug, Clone)]
pub struct Invoice {
    pub number: String,
    /// 380 = invoice, 381 = credit note
    pub type_code: u16,
    /// Credit notes: number of the corrected invoice (BT-25)
    pub referenced_invoice: Option<String>,
    /// VAT-free EU business order
    pub reverse_charge: bool,
    /// Country the goods were shipped to (BT-80)
    pub ship_to_country: Option<String>,
    pub order_number: String,
    pub issue_date: NaiveDate,
    pub delivery_date: NaiveDate,
    pub currency: String,
    pub currency_symbol: String,
    /// kleingewerbe | included | excluded
    pub tax_mode: String,
    pub seller: Seller,
    pub buyer: Party,
    pub lines: Vec<InvoiceLine>,
    /// Coupon discount on the products, as shown (gross or net like the lines)
    pub discount_display_cents: i64,
    pub discount_code: Option<String>,
    pub allowances: Vec<DocumentAmount>,
    pub shipping_display_cents: i64,
    pub shipping: Option<DocumentAmount>,
    pub has_physical_items: bool,
    pub groups: Vec<TaxGroup>,
    pub line_total_cents: i64,
    pub allowance_total_cents: i64,
    pub charge_total_cents: i64,
    pub tax_basis_cents: i64,
    pub tax_total_cents: i64,
    pub rounding_cents: i64,
    pub grand_total_cents: i64,
    /// What the customer pays/paid: grand total + rounding
    pub payable_cents: i64,
    pub paid: bool,
    pub payment_label: String,
    pub payment_means_code: Option<&'static str>,
    pub notes: Vec<String>,
}

/// Index of the VAT rate with the largest amount (first one on a tie). Shipping is taxed with this rate —
/// checkout and invoice must pick the same one.
pub fn main_rate_index(amounts: &[i64]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, a) in amounts.iter().enumerate() {
        if best.map_or(true, |b| *a > amounts[b]) {
            best = Some(i);
        }
    }
    best
}

pub const SMALL_BUSINESS_REASON: &str = "Kleinunternehmer gemäß § 19 UStG – es wird keine Umsatzsteuer berechnet.";
pub const INTRA_COMMUNITY_REASON: &str = "Tax-free intra-Community supply – steuerfreie innergemeinschaftliche Lieferung (§ 4 Nr. 1b, § 6a UStG; Art. 138 VAT Directive).";
pub const REVERSE_CHARGE_REASON: &str = "Reverse charge – Steuerschuldnerschaft des Leistungsempfängers (§ 13b UStG; Art. 196 VAT Directive).";

fn exemption_reason(category: &str) -> Option<(&'static str, &'static str)> {
    match category {
        "E" => Some((SMALL_BUSINESS_REASON, "VATEX-EU-132")),
        "K" => Some((INTRA_COMMUNITY_REASON, "VATEX-EU-IC")),
        "AE" => Some((REVERSE_CHARGE_REASON, "VATEX-EU-AE")),
        _ => None,
    }
}

fn net_of(gross: i64, percent: f64) -> i64 {
    if percent <= 0.0 {
        gross
    } else {
        (gross as f64 / (1.0 + percent / 100.0)).round() as i64
    }
}

fn tax_of(net: i64, percent: f64) -> i64 {
    (net as f64 * percent / 100.0).round() as i64
}

/// ISO 3166-1 alpha-2 code for a country name or code (defaults to DE).
pub fn country_code(input: &str) -> String {
    let s = input.trim();
    if s.len() == 2 && s.chars().all(|c| c.is_ascii_alphabetic()) {
        return s.to_ascii_uppercase();
    }
    let known: &[(&[&str], &str)] = &[
        (&["germany", "deutschland"], "DE"),
        (&["austria", "österreich", "oesterreich"], "AT"),
        (&["switzerland", "schweiz", "suisse"], "CH"),
        (&["netherlands", "niederlande", "the netherlands"], "NL"),
        (&["belgium", "belgien"], "BE"),
        (&["luxembourg", "luxemburg"], "LU"),
        (&["france", "frankreich"], "FR"),
        (&["italy", "italien"], "IT"),
        (&["spain", "spanien"], "ES"),
        (&["poland", "polen"], "PL"),
        (&["denmark", "dänemark", "daenemark"], "DK"),
        (&["czech republic", "czechia", "tschechien"], "CZ"),
        (&["sweden", "schweden"], "SE"),
        (&["united kingdom", "great britain", "großbritannien", "uk"], "GB"),
        (&["united states", "usa", "vereinigte staaten"], "US"),
    ];
    let lower = s.to_lowercase();
    known
        .iter()
        .find(|(names, _)| names.contains(&lower.as_str()))
        .map(|(_, code)| code.to_string())
        .unwrap_or_else(|| "DE".to_string())
}

pub fn country_name(code: &str) -> &'static str {
    match code {
        "DE" => "Deutschland",
        "AT" => "Österreich",
        "CH" => "Schweiz",
        "NL" => "Niederlande",
        "BE" => "Belgien",
        "LU" => "Luxemburg",
        "FR" => "Frankreich",
        "IT" => "Italien",
        "ES" => "Spanien",
        "PL" => "Polen",
        "DK" => "Dänemark",
        "CZ" => "Tschechien",
        "SE" => "Schweden",
        "GB" => "Vereinigtes Königreich",
        "US" => "USA",
        _ => "",
    }
}

pub fn seller_from_settings(s: &StoreSettings) -> Seller {
    let street = [s.address_street.trim(), s.address_house_number.trim()]
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    let mut party = Party {
        contact: String::new(),
        vat_id: s.vat_id.trim().to_string(),
        name: if s.legal_name.trim().is_empty() { s.store_name.trim().to_string() } else { s.legal_name.trim().to_string() },
        street,
        extra: s.address_extra.trim().to_string(),
        postal_code: s.address_postal_code.trim().to_string(),
        city: s.address_city.trim().to_string(),
        country_code: country_code(&s.address_country),
        email: if s.support_email.trim().is_empty() { s.smtp_from_email.trim().to_string() } else { s.support_email.trim().to_string() },
        phone: s.phone.trim().to_string(),
    };
    // Installations from before the structured address: "Street 1, 12345 City, Country"
    if party.street.is_empty() && !s.company_address.trim().is_empty() {
        let parts: Vec<&str> = s.company_address.split(',').map(str::trim).filter(|p| !p.is_empty()).collect();
        party.street = parts.first().copied().unwrap_or_default().to_string();
        if let Some(city_part) = parts.iter().find(|p| p.chars().next().is_some_and(|c| c.is_ascii_digit())) {
            let (zip, city) = city_part.split_once(' ').unwrap_or((city_part, ""));
            party.postal_code = zip.to_string();
            party.city = city.to_string();
        }
        if let Some(last) = parts.last().filter(|_| parts.len() > 2) {
            party.country_code = country_code(last);
        }
    }
    Seller {
        trade_name: s.store_name.trim().to_string(),
        owner: s.store_owner.trim().to_string(),
        vat_id: s.vat_id.trim().to_string(),
        tax_number: s.tax_number.trim().to_string(),
        register: s.commercial_register.trim().to_string(),
        party,
    }
}

fn buyer_from_order(order: &Order) -> Party {
    let addr = if order.billing_address.is_object() { &order.billing_address } else { &order.shipping_address };
    let get = |k: &str| addr.get(k).and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let person = Some(get("full_name")).filter(|n| !n.is_empty()).unwrap_or_else(|| order.customer_name.trim().to_string());
    let company = order.buyer_company.as_deref().map(str::trim).filter(|c| !c.is_empty()).map(str::to_string);
    Party {
        name: company.clone().unwrap_or_else(|| person.clone()),
        contact: if company.is_some() { person } else { String::new() },
        vat_id: order.buyer_vat_id.clone().unwrap_or_default(),
        street: get("street_address"),
        extra: get("apartment_suite"),
        postal_code: get("postal_code"),
        city: get("city"),
        country_code: country_code(&get("country_code")),
        email: order.customer_email.trim().to_string(),
        phone: get("phone"),
    }
}

impl Invoice {
    pub fn build(order: &Order, items: &[OrderItem], settings: &StoreSettings) -> Invoice {
        let tax_mode = order.tax_mode.clone().filter(|m| !m.is_empty()).unwrap_or_else(|| settings.tax_mode.clone());
        let small_business = tax_mode == "kleingewerbe";
        let gross_prices = tax_mode != "excluded";
        // VAT-free EU business order: net amounts, category K (goods) / AE (digital, reverse charge)
        let reverse_charge = order.reverse_charge && !small_business;
        // Amounts on the invoice include VAT
        let shown_gross = gross_prices && !reverse_charge;
        let product_rate = |item: &OrderItem| if small_business { 0.0 } else { item.tax_rate_percent.unwrap_or(settings.tax_rate_percent).max(0.0) };

        let vat_for = |percent: Option<f64>| -> VatRate {
            if small_business {
                return VatRate { category: "E", percent: 0.0 };
            }
            let p = percent.unwrap_or(settings.tax_rate_percent).max(0.0);
            VatRate { category: if p > 0.0 { "S" } else { "Z" }, percent: p }
        };

        let lines: Vec<InvoiceLine> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let vat = if reverse_charge {
                    VatRate { category: if item.is_digital { "AE" } else { "K" }, percent: 0.0 }
                } else {
                    vat_for(item.tax_rate_percent)
                };
                let qty = item.quantity.max(1) as i64;
                let charged = item.total_price_cents as i64;
                let line_net = if gross_prices { net_of(charged, product_rate(item)) } else { charged };
                let display = if shown_gross { charged } else { line_net };
                let unit_display = if display == charged { item.unit_price_cents as i64 } else { (line_net as f64 / qty as f64).round() as i64 };
                let variant = Some(item.variant_title.trim().to_string())
                    .filter(|v| !v.is_empty() && !["standard", "default"].contains(&v.to_lowercase().as_str()));
                InvoiceLine {
                    position: i + 1,
                    sku: item.sku.clone(),
                    name: item.product_title.clone(),
                    variant,
                    is_digital: item.is_digital,
                    quantity: qty,
                    vat,
                    unit_display_cents: unit_display,
                    line_display_cents: display,
                    unit_net_e4: ((line_net as f64) * 100.0 / qty as f64).round() as i64,
                    line_net_cents: line_net,
                }
            })
            .collect();

        // Display amounts per VAT rate, in order of first appearance
        let mut rates: Vec<(VatRate, i64)> = Vec::new();
        for l in &lines {
            match rates.iter_mut().find(|(v, _)| v.key() == l.vat.key()) {
                Some((_, sum)) => *sum += l.line_display_cents,
                None => rates.push((l.vat, l.line_display_cents)),
            }
        }
        let items_sum: i64 = lines.iter().map(|l| l.line_display_cents).sum();

        // Product discounts lower the subtotal; a free-shipping coupon only zeroes the shipping.
        // Business orders without VAT state the discount net (scaled like the lines).
        let charged_sum: i64 = items.iter().map(|i| i.total_price_cents as i64).sum();
        let charged_discount = (charged_sum - order.subtotal_cents as i64).max(0);
        let discount = if charged_sum > 0 && items_sum != charged_sum {
            (charged_discount as f64 * items_sum as f64 / charged_sum as f64).round() as i64
        } else {
            charged_discount
        };
        let mut allowances = Vec::new();
        if discount > 0 && items_sum > 0 {
            let mut left = discount;
            for (i, (vat, amount)) in rates.iter().enumerate() {
                let share = if i + 1 == rates.len() { left } else { (discount as f64 * *amount as f64 / items_sum as f64).round() as i64 };
                left -= share;
                let net = if shown_gross { net_of(share, vat.percent) } else { share };
                if net > 0 {
                    allowances.push(DocumentAmount { vat: *vat, net_cents: net });
                }
            }
        }

        // Shipping follows the VAT rate of the main delivery (largest share): gross with gross prices, net with net prices
        let shipping_charged = order.shipping_cost_cents.max(0) as i64;
        let amounts: Vec<i64> = rates.iter().map(|(_, a)| *a).collect();
        let main_rate = main_rate_index(&amounts).map(|i| rates[i].0).unwrap_or_else(|| vat_for(None));
        // The main delivery's product VAT rate (same rule as the checkout) for the net shipping price
        let mut product_rates: Vec<(f64, i64)> = Vec::new();
        for item in items {
            let r = product_rate(item);
            match product_rates.iter_mut().find(|(x, _)| (*x - r).abs() < 1e-9) {
                Some((_, a)) => *a += item.total_price_cents as i64,
                None => product_rates.push((r, item.total_price_cents as i64)),
            }
        }
        let product_amounts: Vec<i64> = product_rates.iter().map(|(_, a)| *a).collect();
        let main_product_rate = main_rate_index(&product_amounts).map(|i| product_rates[i].0).unwrap_or(main_rate.percent);
        let shipping_vat = if reverse_charge {
            VatRate { category: if lines.iter().any(|l| !l.is_digital) { "K" } else { "AE" }, percent: 0.0 }
        } else {
            main_rate
        };
        let shipping = (shipping_charged > 0).then(|| DocumentAmount {
            vat: shipping_vat,
            net_cents: if gross_prices { net_of(shipping_charged, main_product_rate) } else { shipping_charged },
        });
        let shipping_display_cents = match (&shipping, shown_gross) {
            (Some(s), false) => s.net_cents,
            _ => shipping_charged,
        };

        let mut groups: Vec<TaxGroup> = rates.iter().map(|(vat, _)| TaxGroup { vat: *vat, basis_cents: 0, tax_cents: 0 }).collect();
        if let Some(s) = &shipping {
            if !groups.iter().any(|g| g.vat.key() == s.vat.key()) {
                groups.push(TaxGroup { vat: s.vat, basis_cents: 0, tax_cents: 0 });
            }
        }
        for g in groups.iter_mut() {
            let lines_net: i64 = lines.iter().filter(|l| l.vat.key() == g.vat.key()).map(|l| l.line_net_cents).sum();
            let allow: i64 = allowances.iter().filter(|a| a.vat.key() == g.vat.key()).map(|a| a.net_cents).sum();
            let charge: i64 = shipping.iter().filter(|s| s.vat.key() == g.vat.key()).map(|s| s.net_cents).sum();
            g.basis_cents = lines_net - allow + charge;
            g.tax_cents = tax_of(g.basis_cents, g.vat.percent);
        }
        if groups.is_empty() {
            groups.push(TaxGroup { vat: main_rate, basis_cents: 0, tax_cents: 0 });
        }

        let line_total_cents: i64 = lines.iter().map(|l| l.line_net_cents).sum();
        let allowance_total_cents: i64 = allowances.iter().map(|a| a.net_cents).sum();
        let charge_total_cents: i64 = shipping.iter().map(|s| s.net_cents).sum();
        let tax_basis_cents = line_total_cents - allowance_total_cents + charge_total_cents;
        let tax_total_cents: i64 = groups.iter().map(|g| g.tax_cents).sum();
        let grand_total_cents = tax_basis_cents + tax_total_cents;
        let payable_cents = order.total_cents as i64;

        let (payment_label, payment_means_code) = match order.payment_provider.as_str() {
            "stripe" => ("Online payment (Stripe)".to_string(), Some("68")),
            "paypal" => ("PayPal".to_string(), Some("68")),
            "free" => ("No payment required".to_string(), None),
            other => (other.to_string(), Some("ZZZ")),
        };

        let mut notes = Vec::new();
        if small_business {
            let notice = settings.tax_notice.trim();
            notes.push(if notice.is_empty() || !notice.contains("19") { SMALL_BUSINESS_REASON.to_string() } else { notice.to_string() });
        }
        if reverse_charge {
            if lines.iter().any(|l| !l.is_digital) {
                notes.push(INTRA_COMMUNITY_REASON.to_string());
            }
            if lines.iter().any(|l| l.is_digital) {
                notes.push(REVERSE_CHARGE_REASON.to_string());
            }
        }
        // Issued when the payment was confirmed; goods/services are delivered from that day on
        let date = order.paid_at.unwrap_or(order.created_at).date_naive();

        Invoice {
            number: order.order_number.clone(),
            type_code: 380,
            referenced_invoice: None,
            reverse_charge,
            ship_to_country: lines
                .iter()
                .any(|l| !l.is_digital)
                .then(|| order.shipping_address.get("country_code").and_then(|v| v.as_str()).map(country_code))
                .flatten(),
            order_number: order.order_number.clone(),
            issue_date: date,
            delivery_date: date,
            currency: if settings.currency.trim().is_empty() { "EUR".to_string() } else { settings.currency.trim().to_uppercase() },
            currency_symbol: if settings.currency_symbol.trim().is_empty() { "€".to_string() } else { settings.currency_symbol.trim().to_string() },
            tax_mode,
            seller: seller_from_settings(settings),
            buyer: buyer_from_order(order),
            has_physical_items: lines.iter().any(|l| !l.is_digital),
            lines,
            discount_display_cents: discount,
            discount_code: order.coupon_code.clone().filter(|_| discount > 0),
            allowances,
            shipping_display_cents,
            shipping,
            groups,
            line_total_cents,
            allowance_total_cents,
            charge_total_cents,
            tax_basis_cents,
            tax_total_cents,
            rounding_cents: payable_cents - grand_total_cents,
            grand_total_cents,
            payable_cents,
            paid: order.payment_status == "paid",
            payment_label,
            payment_means_code,
            notes,
        }
    }

    pub fn is_small_business(&self) -> bool {
        self.tax_mode == "kleingewerbe"
    }

    pub fn shows_gross_prices(&self) -> bool {
        self.tax_mode != "excluded" && !self.reverse_charge
    }

    pub fn is_credit_note(&self) -> bool {
        self.type_code == 381
    }

    /// Credit note (Stornorechnung) cancelling this invoice in full, e.g. after a refund.
    pub fn credit_note(&self, number: String, issue_date: NaiveDate) -> Invoice {
        let mut cn = self.clone();
        cn.type_code = 381;
        cn.referenced_invoice = Some(self.number.clone());
        cn.number = number;
        cn.issue_date = issue_date;
        cn.paid = true;
        cn
    }

    /// File name used for attachments and downloads
    pub fn file_stem(&self) -> String {
        let safe: String = self.number.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
        format!("{}-{}", if self.is_credit_note() { "Credit-Note" } else { "Invoice" }, safe)
    }

    /// UN/CEFACT Cross Industry Invoice D16B, guideline EN 16931 (ZUGFeRD 2.x "EN 16931" / Factur-X "EN 16931").
    pub fn to_cii_xml(&self) -> String {
        let c = &self.currency;
        let mut x = String::with_capacity(8 * 1024);
        x.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100" xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100" xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
  <rsm:ExchangedDocumentContext>
    <ram:GuidelineSpecifiedDocumentContextParameter>
      <ram:ID>urn:cen.eu:en16931:2017</ram:ID>
    </ram:GuidelineSpecifiedDocumentContextParameter>
  </rsm:ExchangedDocumentContext>
  <rsm:ExchangedDocument>
"#);
        let _ = writeln!(x, "    <ram:ID>{}</ram:ID>", esc(&self.number));
        x.push_str("    <ram:TypeCode>380</ram:TypeCode>\n");
        let _ = writeln!(x, "    <ram:IssueDateTime>{}</ram:IssueDateTime>", date102(self.issue_date));
        for note in &self.notes {
            let _ = writeln!(x, "    <ram:IncludedNote>\n      <ram:Content>{}</ram:Content>\n    </ram:IncludedNote>", esc(note));
        }
        x.push_str("  </rsm:ExchangedDocument>\n  <rsm:SupplyChainTradeTransaction>\n");

        for l in &self.lines {
            let _ = write!(
                x,
                r#"    <ram:IncludedSupplyChainTradeLineItem>
      <ram:AssociatedDocumentLineDocument>
        <ram:LineID>{}</ram:LineID>
      </ram:AssociatedDocumentLineDocument>
      <ram:SpecifiedTradeProduct>
        <ram:SellerAssignedID>{}</ram:SellerAssignedID>
        <ram:Name>{}</ram:Name>
{}      </ram:SpecifiedTradeProduct>
      <ram:SpecifiedLineTradeAgreement>
        <ram:NetPriceProductTradePrice>
          <ram:ChargeAmount>{}</ram:ChargeAmount>
        </ram:NetPriceProductTradePrice>
      </ram:SpecifiedLineTradeAgreement>
      <ram:SpecifiedLineTradeDelivery>
        <ram:BilledQuantity unitCode="C62">{}</ram:BilledQuantity>
      </ram:SpecifiedLineTradeDelivery>
      <ram:SpecifiedLineTradeSettlement>
{}        <ram:SpecifiedTradeSettlementLineMonetarySummation>
          <ram:LineTotalAmount>{}</ram:LineTotalAmount>
        </ram:SpecifiedTradeSettlementLineMonetarySummation>
      </ram:SpecifiedLineTradeSettlement>
    </ram:IncludedSupplyChainTradeLineItem>
"#,
                l.position,
                esc(&l.sku),
                esc(&l.name),
                l.variant.as_ref().map(|v| format!("        <ram:Description>{}</ram:Description>\n", esc(v))).unwrap_or_default(),
                fmt4(l.unit_net_e4),
                l.quantity,
                trade_tax(&l.vat, "        "),
                fmt2(l.line_net_cents),
            );
        }

        // Agreement: seller and buyer
        let s = &self.seller;
        x.push_str("    <ram:ApplicableHeaderTradeAgreement>\n      <ram:SellerTradeParty>\n");
        // BR-CO-26: a seller identifier is required when there is neither a VAT ID nor a register number
        if s.vat_id.is_empty() && s.register.is_empty() && !s.tax_number.is_empty() {
            let _ = writeln!(x, "        <ram:ID>{}</ram:ID>", esc(&s.tax_number));
        }
        let _ = writeln!(x, "        <ram:Name>{}</ram:Name>", esc(&s.party.name));
        if !s.register.is_empty() {
            let _ = writeln!(
                x,
                "        <ram:SpecifiedLegalOrganization>\n          <ram:ID>{}</ram:ID>\n          <ram:TradingBusinessName>{}</ram:TradingBusinessName>\n        </ram:SpecifiedLegalOrganization>",
                esc(&s.register),
                esc(&s.trade_name)
            );
        } else if !s.trade_name.is_empty() && s.trade_name != s.party.name {
            let _ = writeln!(
                x,
                "        <ram:SpecifiedLegalOrganization>\n          <ram:TradingBusinessName>{}</ram:TradingBusinessName>\n        </ram:SpecifiedLegalOrganization>",
                esc(&s.trade_name)
            );
        }
        if !s.party.phone.is_empty() || !s.party.email.is_empty() {
            x.push_str("        <ram:DefinedTradeContact>\n");
            let _ = writeln!(x, "          <ram:PersonName>{}</ram:PersonName>", esc(if s.owner.is_empty() { &s.party.name } else { &s.owner }));
            if !s.party.phone.is_empty() {
                let _ = writeln!(x, "          <ram:TelephoneUniversalCommunication>\n            <ram:CompleteNumber>{}</ram:CompleteNumber>\n          </ram:TelephoneUniversalCommunication>", esc(&s.party.phone));
            }
            if !s.party.email.is_empty() {
                let _ = writeln!(x, "          <ram:EmailURIUniversalCommunication>\n            <ram:URIID>{}</ram:URIID>\n          </ram:EmailURIUniversalCommunication>", esc(&s.party.email));
            }
            x.push_str("        </ram:DefinedTradeContact>\n");
        }
        address_xml(&mut x, &s.party);
        if !s.party.email.is_empty() {
            let _ = writeln!(x, "        <ram:URIUniversalCommunication>\n          <ram:URIID schemeID=\"EM\">{}</ram:URIID>\n        </ram:URIUniversalCommunication>", esc(&s.party.email));
        }
        if !s.tax_number.is_empty() {
            let _ = writeln!(x, "        <ram:SpecifiedTaxRegistration>\n          <ram:ID schemeID=\"FC\">{}</ram:ID>\n        </ram:SpecifiedTaxRegistration>", esc(&s.tax_number));
        }
        if !s.vat_id.is_empty() {
            let _ = writeln!(x, "        <ram:SpecifiedTaxRegistration>\n          <ram:ID schemeID=\"VA\">{}</ram:ID>\n        </ram:SpecifiedTaxRegistration>", esc(&s.vat_id));
        }
        x.push_str("      </ram:SellerTradeParty>\n      <ram:BuyerTradeParty>\n");
        let _ = writeln!(x, "        <ram:Name>{}</ram:Name>", esc(&self.buyer.name));
        if !self.buyer.contact.is_empty() {
            let _ = writeln!(x, "        <ram:DefinedTradeContact>\n          <ram:PersonName>{}</ram:PersonName>\n        </ram:DefinedTradeContact>", esc(&self.buyer.contact));
        }
        address_xml(&mut x, &self.buyer);
        if !self.buyer.email.is_empty() {
            let _ = writeln!(x, "        <ram:URIUniversalCommunication>\n          <ram:URIID schemeID=\"EM\">{}</ram:URIID>\n        </ram:URIUniversalCommunication>", esc(&self.buyer.email));
        }
        if !self.buyer.vat_id.is_empty() {
            let _ = writeln!(x, "        <ram:SpecifiedTaxRegistration>\n          <ram:ID schemeID=\"VA\">{}</ram:ID>\n        </ram:SpecifiedTaxRegistration>", esc(&self.buyer.vat_id));
        }
        x.push_str("      </ram:BuyerTradeParty>\n");
        let _ = writeln!(
            x,
            "      <ram:BuyerOrderReferencedDocument>\n        <ram:IssuerAssignedID>{}</ram:IssuerAssignedID>\n      </ram:BuyerOrderReferencedDocument>",
            esc(&self.order_number)
        );
        x.push_str("    </ram:ApplicableHeaderTradeAgreement>\n");

        // Delivery (the destination country is required for intra-Community supplies, BR-IC-12)
        x.push_str("    <ram:ApplicableHeaderTradeDelivery>\n");
        if let Some(country) = &self.ship_to_country {
            let _ = writeln!(
                x,
                "      <ram:ShipToTradeParty>\n        <ram:Name>{}</ram:Name>\n        <ram:PostalTradeAddress>\n          <ram:CountryID>{}</ram:CountryID>\n        </ram:PostalTradeAddress>\n      </ram:ShipToTradeParty>",
                esc(&self.buyer.name),
                esc(country)
            );
        }
        let _ = writeln!(
            x,
            "      <ram:ActualDeliverySupplyChainEvent>\n        <ram:OccurrenceDateTime>{}</ram:OccurrenceDateTime>\n      </ram:ActualDeliverySupplyChainEvent>\n    </ram:ApplicableHeaderTradeDelivery>",
            date102(self.delivery_date)
        );

        // Settlement
        x.push_str("    <ram:ApplicableHeaderTradeSettlement>\n");
        let _ = writeln!(x, "      <ram:PaymentReference>{}</ram:PaymentReference>", esc(&self.number));
        let _ = writeln!(x, "      <ram:InvoiceCurrencyCode>{}</ram:InvoiceCurrencyCode>", esc(c));
        if let Some(code) = self.payment_means_code {
            let _ = writeln!(
                x,
                "      <ram:SpecifiedTradeSettlementPaymentMeans>\n        <ram:TypeCode>{}</ram:TypeCode>\n        <ram:Information>{}</ram:Information>\n      </ram:SpecifiedTradeSettlementPaymentMeans>",
                code,
                esc(&self.payment_label)
            );
        }
        for g in &self.groups {
            let _ = write!(x, "      <ram:ApplicableTradeTax>\n        <ram:CalculatedAmount>{}</ram:CalculatedAmount>\n        <ram:TypeCode>VAT</ram:TypeCode>\n", fmt2(g.tax_cents));
            let exemption = exemption_reason(g.vat.category);
            if let Some((reason, _)) = exemption {
                let _ = writeln!(x, "        <ram:ExemptionReason>{}</ram:ExemptionReason>", esc(reason));
            }
            let _ = write!(x, "        <ram:BasisAmount>{}</ram:BasisAmount>\n        <ram:CategoryCode>{}</ram:CategoryCode>\n", fmt2(g.basis_cents), g.vat.category);
            if let Some((_, code)) = exemption {
                let _ = writeln!(x, "        <ram:ExemptionReasonCode>{}</ram:ExemptionReasonCode>", code);
            }
            let _ = write!(x, "        <ram:RateApplicablePercent>{}</ram:RateApplicablePercent>\n      </ram:ApplicableTradeTax>\n", fmt_rate(g.vat.percent));
        }
        for a in &self.allowances {
            let reason = match &self.discount_code {
                Some(code) => format!("Discount ({})", code),
                None => "Discount".to_string(),
            };
            let _ = write!(
                x,
                "      <ram:SpecifiedTradeAllowanceCharge>\n        <ram:ChargeIndicator>\n          <udt:Indicator>false</udt:Indicator>\n        </ram:ChargeIndicator>\n        <ram:ActualAmount>{}</ram:ActualAmount>\n        <ram:ReasonCode>95</ram:ReasonCode>\n        <ram:Reason>{}</ram:Reason>\n{}      </ram:SpecifiedTradeAllowanceCharge>\n",
                fmt2(a.net_cents),
                esc(&reason),
                category_trade_tax(&a.vat)
            );
        }
        if let Some(sh) = &self.shipping {
            let _ = write!(
                x,
                "      <ram:SpecifiedTradeAllowanceCharge>\n        <ram:ChargeIndicator>\n          <udt:Indicator>true</udt:Indicator>\n        </ram:ChargeIndicator>\n        <ram:ActualAmount>{}</ram:ActualAmount>\n        <ram:ReasonCode>FC</ram:ReasonCode>\n        <ram:Reason>Shipping</ram:Reason>\n{}      </ram:SpecifiedTradeAllowanceCharge>\n",
                fmt2(sh.net_cents),
                category_trade_tax(&sh.vat)
            );
        }
        let terms = if self.is_credit_note() {
            format!("Refunded via {}.", self.payment_label)
        } else if self.paid {
            format!("Paid via {}.", self.payment_label)
        } else {
            format!("Payment via {} is being processed.", self.payment_label)
        };
        let _ = write!(x, "      <ram:SpecifiedTradePaymentTerms>\n        <ram:Description>{}</ram:Description>\n", esc(&terms));
        if !self.paid {
            let _ = writeln!(x, "        <ram:DueDateDateTime>{}</ram:DueDateDateTime>", date102(self.issue_date));
        }
        x.push_str("      </ram:SpecifiedTradePaymentTerms>\n");

        let prepaid = if self.paid { self.payable_cents } else { 0 };
        let _ = writeln!(x, "      <ram:SpecifiedTradeSettlementHeaderMonetarySummation>");
        let _ = writeln!(x, "        <ram:LineTotalAmount>{}</ram:LineTotalAmount>", fmt2(self.line_total_cents));
        let _ = writeln!(x, "        <ram:ChargeTotalAmount>{}</ram:ChargeTotalAmount>", fmt2(self.charge_total_cents));
        let _ = writeln!(x, "        <ram:AllowanceTotalAmount>{}</ram:AllowanceTotalAmount>", fmt2(self.allowance_total_cents));
        let _ = writeln!(x, "        <ram:TaxBasisTotalAmount>{}</ram:TaxBasisTotalAmount>", fmt2(self.tax_basis_cents));
        let _ = writeln!(x, "        <ram:TaxTotalAmount currencyID=\"{}\">{}</ram:TaxTotalAmount>", esc(c), fmt2(self.tax_total_cents));
        if self.rounding_cents != 0 {
            let _ = writeln!(x, "        <ram:RoundingAmount>{}</ram:RoundingAmount>", fmt2(self.rounding_cents));
        }
        let _ = writeln!(x, "        <ram:GrandTotalAmount>{}</ram:GrandTotalAmount>", fmt2(self.grand_total_cents));
        let _ = writeln!(x, "        <ram:TotalPrepaidAmount>{}</ram:TotalPrepaidAmount>", fmt2(prepaid));
        let _ = writeln!(x, "        <ram:DuePayableAmount>{}</ram:DuePayableAmount>", fmt2(self.payable_cents - prepaid));
        x.push_str("      </ram:SpecifiedTradeSettlementHeaderMonetarySummation>\n");
        if let Some(original) = &self.referenced_invoice {
            let _ = writeln!(x, "      <ram:InvoiceReferencedDocument>\n        <ram:IssuerAssignedID>{}</ram:IssuerAssignedID>\n      </ram:InvoiceReferencedDocument>", esc(original));
        }
        x.push_str("    </ram:ApplicableHeaderTradeSettlement>\n  </rsm:SupplyChainTradeTransaction>\n</rsm:CrossIndustryInvoice>\n");
        x
    }
}

fn address_xml(x: &mut String, p: &Party) {
    x.push_str("        <ram:PostalTradeAddress>\n");
    if !p.postal_code.is_empty() {
        let _ = writeln!(x, "          <ram:PostcodeCode>{}</ram:PostcodeCode>", esc(&p.postal_code));
    }
    if !p.street.is_empty() {
        let _ = writeln!(x, "          <ram:LineOne>{}</ram:LineOne>", esc(&p.street));
    }
    if !p.extra.is_empty() {
        let _ = writeln!(x, "          <ram:LineTwo>{}</ram:LineTwo>", esc(&p.extra));
    }
    if !p.city.is_empty() {
        let _ = writeln!(x, "          <ram:CityName>{}</ram:CityName>", esc(&p.city));
    }
    let _ = writeln!(x, "          <ram:CountryID>{}</ram:CountryID>", esc(&p.country_code));
    x.push_str("        </ram:PostalTradeAddress>\n");
}

fn trade_tax(vat: &VatRate, indent: &str) -> String {
    format!(
        "{i}<ram:ApplicableTradeTax>\n{i}  <ram:TypeCode>VAT</ram:TypeCode>\n{i}  <ram:CategoryCode>{}</ram:CategoryCode>\n{i}  <ram:RateApplicablePercent>{}</ram:RateApplicablePercent>\n{i}</ram:ApplicableTradeTax>\n",
        vat.category,
        fmt_rate(vat.percent),
        i = indent
    )
}

fn category_trade_tax(vat: &VatRate) -> String {
    format!(
        "        <ram:CategoryTradeTax>\n          <ram:TypeCode>VAT</ram:TypeCode>\n          <ram:CategoryCode>{}</ram:CategoryCode>\n          <ram:RateApplicablePercent>{}</ram:RateApplicablePercent>\n        </ram:CategoryTradeTax>\n",
        vat.category,
        fmt_rate(vat.percent)
    )
}

fn date102(d: NaiveDate) -> String {
    format!("<udt:DateTimeString format=\"102\">{}</udt:DateTimeString>", d.format("%Y%m%d"))
}

pub fn fmt2(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    format!("{}{}.{:02}", sign, cents.abs() / 100, cents.abs() % 100)
}

fn fmt4(e4: i64) -> String {
    let sign = if e4 < 0 { "-" } else { "" };
    format!("{}{}.{:04}", sign, e4.abs() / 10000, e4.abs() % 10000)
}

fn fmt_rate(p: f64) -> String {
    format!("{:.2}", p)
}

fn esc(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use uuid::Uuid;

    pub fn sample_settings(tax_mode: &str) -> StoreSettings {
        let raw = r#"{
            "id": 1, "store_name": "Example Store", "currency": "EUR", "currency_symbol": "€", "tax_rate_percent": 19.0,
            "deployment_mode": "production", "debug_mode": false, "support_email": "shop@example.com",
            "company_address": "Musterstraße 1, 10115 Berlin, Germany", "vat_id": "", "logo_url": "", "phone": "+49 30 123456",
            "hero_config": {}, "smtp_host": "", "smtp_password": "", "smtp_port": 587, "smtp_username": "", "smtp_encryption": "starttls",
            "smtp_from_email": "", "smtp_from_name": "", "smtp_enabled": false, "require_registered_checkout": false,
            "require_email_verification": false, "store_subtitle": "", "show_store_title": true, "show_store_subtitle": true,
            "carousels_config": [], "cookie_banner_enabled": false, "cookie_banner_title": "", "cookie_banner_description": "",
            "cookie_banner_policy_url": "", "cookie_accept_label": "", "cookie_deny_label": "", "cookie_preferences_label": "",
            "tax_notice": "", "tax_mode": "TAX_MODE", "legal_name": "Max Mustermann", "store_owner": "Max Mustermann",
            "commercial_register": "", "dispute_resolution_notice": "", "odr_url": "", "footer_config": {},
            "order_prefix_enabled": true, "order_prefix": "ORD", "order_date_enabled": false, "stock_display_template": "",
            "address_street": "Musterstraße", "address_house_number": "1", "address_postal_code": "10115",
            "address_city": "Berlin", "address_country": "Deutschland", "tax_number": "12/345/67890",
            "updated_at": "2026-10-25T00:00:00Z"
        }"#;
        serde_json::from_str(&raw.replace("TAX_MODE", tax_mode)).expect("sample settings")
    }

    pub fn sample_order(tax_mode: &str, subtotal: i32, shipping: i32, total: i32) -> Order {
        serde_json::from_value(json!({
            "id": Uuid::nil(), "order_number": "ORD-10000", "customer_name": "Erika Musterfrau", "customer_email": "erika@example.com",
            "shipping_address": {}, "billing_address": { "full_name": "Erika Musterfrau", "street_address": "Beispielweg 5",
                "postal_code": "80331", "city": "München", "country_code": "DE" },
            "shipping_rate_id": null, "shipping_cost_cents": shipping, "subtotal_cents": subtotal, "tax_cents": 0, "total_cents": total,
            "payment_provider": "stripe", "payment_status": "paid", "order_status": "processing", "tracking_number": null, "notes": null,
            "coupon_code": null, "discount_cents": 0, "payment_reference": "pi_test", "is_test": true, "tax_mode": tax_mode,
            "created_at": Utc.with_ymd_and_hms(2026, 10, 25, 12, 0, 0).unwrap(), "updated_at": Utc.with_ymd_and_hms(2026, 10, 25, 12, 0, 0).unwrap()
        }))
        .expect("sample order")
    }

    pub fn item(title: &str, sku: &str, unit: i32, qty: i32, rate: f64, digital: bool) -> OrderItem {
        OrderItem {
            id: Uuid::nil(),
            order_id: Uuid::nil(),
            product_id: None,
            variant_id: None,
            product_title: title.into(),
            variant_title: "Standard".into(),
            sku: sku.into(),
            unit_price_cents: unit,
            quantity: qty,
            total_price_cents: unit * qty,
            is_digital: digital,
            download_url: None,
            tax_rate_percent: Some(rate),
        }
    }

    #[test]
    fn small_business_invoice_is_exempt_and_exact() {
        let items = vec![item("Widget", "W-1", 1250, 2, 19.0, false)];
        let inv = Invoice::build(&sample_order("kleingewerbe", 2500, 490, 2990), &items, &sample_settings("kleingewerbe"));
        assert_eq!(inv.groups.len(), 1);
        assert_eq!(inv.groups[0].vat.category, "E");
        assert_eq!(inv.tax_total_cents, 0);
        assert_eq!(inv.tax_basis_cents, 2990);
        assert_eq!(inv.rounding_cents, 0);
        let xml = inv.to_cii_xml();
        assert!(xml.contains("<ram:CategoryCode>E</ram:CategoryCode>"));
        assert!(xml.contains("<ram:ExemptionReason>"));
        assert!(xml.contains("<ram:ID schemeID=\"FC\">12/345/67890</ram:ID>"));
        assert!(xml.contains("<ram:GrandTotalAmount>29.90</ram:GrandTotalAmount>"));
        assert!(xml.contains("<ram:DuePayableAmount>0.00</ram:DuePayableAmount>"));
    }

    #[test]
    fn gross_prices_with_two_rates_discount_and_shipping_add_up() {
        // 2 × 11.90 € (19 %) + 1 × 10.70 € (7 %), 10 % discount, 4.90 € shipping
        let items = vec![item("Gadget", "G-1", 1190, 2, 19.0, false), item("Book", "B-1", 1070, 1, 7.0, false)];
        let items_sum = 2380 + 1070;
        let discount = 345;
        let subtotal = items_sum - discount;
        let total = subtotal + 490;
        let inv = Invoice::build(&sample_order("included", subtotal, 490, total), &items, &sample_settings("included"));
        assert_eq!(inv.groups.len(), 2);
        assert_eq!(inv.discount_display_cents, 345);
        // Net amounts and VAT add up to what was paid, up to a few cents of rounding
        assert_eq!(inv.tax_basis_cents + inv.tax_total_cents + inv.rounding_cents, total as i64);
        assert!(inv.rounding_cents.abs() <= 3, "rounding {}", inv.rounding_cents);
        for g in &inv.groups {
            assert_eq!(g.tax_cents, tax_of(g.basis_cents, g.vat.percent));
        }
        // Shipping follows the main (19 %) rate
        assert_eq!(inv.shipping.as_ref().unwrap().vat.percent, 19.0);
    }

    #[test]
    fn net_prices_keep_line_amounts() {
        let items = vec![item("Service", "S-1", 10000, 3, 19.0, true)];
        let inv = Invoice::build(&sample_order("excluded", 30000, 0, 35700), &items, &sample_settings("excluded"));
        assert_eq!(inv.line_total_cents, 30000);
        assert_eq!(inv.tax_total_cents, 5700);
        assert_eq!(inv.rounding_cents, 0);
        assert_eq!(inv.lines[0].unit_net_e4, 1_000_000);
        assert!(inv.to_cii_xml().contains("<ram:ChargeAmount>100.0000</ram:ChargeAmount>"));
    }

    #[test]
    fn countries_and_escaping() {
        assert_eq!(country_code("Deutschland"), "DE");
        assert_eq!(country_code("at"), "AT");
        assert_eq!(country_code("Österreich"), "AT");
        assert_eq!(esc("A & B <C>"), "A &amp; B &lt;C&gt;");
        assert_eq!(fmt2(-5), "-0.05");
        assert_eq!(fmt4(119000), "11.9000");
    }
}
