use crate::models::{Order, OrderItem, StoreSettings};
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::Address;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::{BTreeSet, HashSet};
use tracing::{error, info};
use uuid::Uuid;

/// A file sent along with an email (e.g. the invoice PDF).
pub struct EmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

/// Content-ID of the shop logo embedded in every email (shown even when remote images are blocked).
const LOGO_CID: &str = "shop-logo";
const LOGO_WIDTH_TOKEN: &str = "__LOGO_WIDTH__";
const LOGO_HEIGHT_TOKEN: &str = "__LOGO_HEIGHT__";
/// Logo box in the email header (CSS pixels); the embedded image has twice the resolution for sharp displays
const LOGO_MAX_H: u32 = 64;
const LOGO_MAX_W: u32 = 200;

pub async fn send_email_raw(
    settings: &StoreSettings,
    to_email: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), String> {
    send_email(settings, to_email, subject, html_body, Vec::new()).await
}

/// Sends an HTML email with a plain-text alternative, the embedded shop logo and optional attachments.
pub async fn send_email(
    settings: &StoreSettings,
    to_email: &str,
    subject: &str,
    html_body: &str,
    attachments: Vec<EmailAttachment>,
) -> Result<(), String> {
    let host = settings.smtp_host.trim();
    if !settings.smtp_enabled || host.is_empty() {
        info!("[SMTP disabled] Not sending \"{}\" to {}", subject, to_email);
        return Err("Email sending is switched off or no SMTP server is configured (Settings → Email).".to_string());
    }

    // Sender: explicit from-address, otherwise the SMTP login (most providers require them to match)
    let from_address = if !settings.smtp_from_email.trim().is_empty() {
        settings.smtp_from_email.trim()
    } else {
        settings.smtp_username.trim()
    };
    let from_address: Address = from_address
        .parse()
        .map_err(|_| format!("The sender address '{}' is not a valid email address.", from_address))?;
    let from_name = settings.smtp_from_name.trim();
    let from = Mailbox::new((!from_name.is_empty()).then(|| from_name.to_string()), from_address);
    let to: Mailbox = to_email
        .trim()
        .parse()
        .map_err(|_| format!("The recipient '{}' is not a valid email address.", to_email))?;

    // The logo travels inside the email; if it cannot be prepared (SVG, external URL) it is linked instead
    let mut html = html_body.to_string();
    let mut inline_logo = None;
    if html.contains(&format!("cid:{}", LOGO_CID)) {
        match crate::services::branding::logo_png_async(settings, LOGO_MAX_H * 4).await {
            Some(logo) => {
                let scale = (LOGO_MAX_H as f32 / logo.height as f32).min(LOGO_MAX_W as f32 / logo.width as f32);
                let (w, h) = ((logo.width as f32 * scale).round() as u32, (logo.height as f32 * scale).round() as u32);
                html = html.replace(LOGO_WIDTH_TOKEN, &w.to_string()).replace(LOGO_HEIGHT_TOKEN, &h.to_string());
                inline_logo = Some(logo);
            }
            None => {
                html = html
                    .replace(&format!("cid:{}", LOGO_CID), &html_escape(&absolute_logo_url(settings)))
                    .replace(&format!(" width=\"{}\"", LOGO_WIDTH_TOKEN), "")
                    .replace(LOGO_HEIGHT_TOKEN, &LOGO_MAX_H.to_string());
            }
        }
    }

    let html_part = SinglePart::builder().header(ContentType::TEXT_HTML).body(html.clone());
    let body = match inline_logo {
        Some(logo) => MultiPart::related().singlepart(html_part).singlepart(
            Attachment::new_inline(LOGO_CID.to_string()).body(logo.png.clone(), ContentType::parse("image/png").expect("static mime")),
        ),
        None => MultiPart::related().singlepart(html_part),
    };
    let alternative = MultiPart::alternative().singlepart(SinglePart::plain(html_to_text(&html))).multipart(body);
    let content = if attachments.is_empty() {
        alternative
    } else {
        let mut mixed = MultiPart::mixed().multipart(alternative);
        for a in attachments {
            let ct = ContentType::parse(&a.content_type).unwrap_or(ContentType::TEXT_PLAIN);
            mixed = mixed.singlepart(Attachment::new(a.filename).body(a.data, ct));
        }
        mixed
    };

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .multipart(content)
        .map_err(|e| format!("Failed to build email message: {}", e))?;

    let port = u16::try_from(settings.smtp_port).unwrap_or(587);
    // Well-known ports dictate the TLS mode; a mismatch would otherwise just hang or be rejected
    let mode = match port {
        465 => "tls",
        587 => "starttls",
        _ => settings.smtp_encryption.as_str(),
    };
    let mut transport_builder = match mode {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| format!("Invalid SMTP host '{}': {}", host, e))?,
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
            .map_err(|e| format!("Invalid SMTP host '{}': {}", host, e))?,
        _ => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host),
    }
    .port(port)
    .timeout(Some(std::time::Duration::from_secs(20)));

    if !settings.smtp_username.trim().is_empty() {
        let creds = Credentials::new(settings.smtp_username.trim().to_string(), settings.smtp_password.clone());
        transport_builder = transport_builder.credentials(creds);
    }

    let transport = transport_builder.build();

    transport.send(email).await.map_err(|e| {
        error!("SMTP send failure to {} via {}:{} ({}): {}", to_email, host, port, mode, e);
        format!("The mail server {}:{} ({}) rejected or did not answer: {}", host, port, mode, e)
    })?;

    info!("Email successfully dispatched to {}", to_email);
    Ok(())
}

fn absolute_logo_url(settings: &StoreSettings) -> String {
    let logo_url = settings.logo_url.trim();
    if logo_url.starts_with("http://") || logo_url.starts_with("https://") {
        logo_url.to_string()
    } else {
        format!("{}/{}", shop_public_url(), logo_url.trim_start_matches('/'))
    }
}

/// Plain-text version of an email for clients that do not show HTML (and for spam filters).
fn html_to_text(html: &str) -> String {
    let body = html.split_once("<body").and_then(|(_, b)| b.split_once('>')).map(|(_, b)| b).unwrap_or(html);
    // Hidden preheader text is not part of the readable message
    let body = match (body.find("<!--preheader-->"), body.find("<!--/preheader-->")) {
        (Some(a), Some(b)) if b > a => format!("{}{}", &body[..a], &body[b..]),
        _ => body.to_string(),
    };
    let mut out = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    let mut href: Option<(String, usize)> = None;
    for ch in body.chars() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
                let t = tag.trim().to_lowercase();
                let name = t.trim_start_matches('/').split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("");
                if t.starts_with("a ") {
                    href = tag.split("href=\"").nth(1).and_then(|r| r.split('"').next()).map(|h| (h.to_string(), out.len()));
                } else if t == "/a" {
                    // Links without text (e.g. around the logo) are left out
                    if let Some((h, start)) = href.take().filter(|(h, start)| h.starts_with("http") && !out[*start..].trim().is_empty()) {
                        if !out[start..].contains(&h) {
                            out.push_str(&format!(" ({})", h));
                        }
                    }
                }
                match name {
                    "br" | "p" | "div" | "tr" | "h1" | "h2" | "h3" | "table" | "li" => out.push('\n'),
                    "td" if !t.starts_with('/') => out.push(' '),
                    _ => {}
                }
                tag.clear();
            } else {
                tag.push(ch);
            }
        } else if ch == '<' {
            in_tag = true;
        } else {
            out.push(ch);
        }
    }
    let decoded = out
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&bull;", "·")
        .replace("&times;", "×")
        .replace("&#8199;", "")
        .replace("&#847;", "")
        .replace("&rarr;", "→")
        .replace("&copy;", "©");
    let mut lines: Vec<String> = Vec::new();
    for line in decoded.lines() {
        let l = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if l.is_empty() && lines.last().is_none_or(|p| p.is_empty()) {
            continue;
        }
        lines.push(l);
    }
    lines.join("\n").trim().to_string()
}

// ------------------------------------------------------------------ layout & building blocks

const FONT: &str = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif";
const C_TEXT: &str = "#374151";
const C_HEAD: &str = "#111827";
const C_MUTED: &str = "#6b7280";
const C_LINE: &str = "#e5e7eb";
const C_SOFT: &str = "#f9fafb";
const C_ACCENT: &str = "#ea580c";

/// Branded, responsive email: logo header, coloured accent, content card and the legal footer.
pub fn render_email_layout(
    settings: &StoreSettings,
    preheader: &str,
    heading_title: &str,
    heading_color: &str,
    content_body_html: &str,
) -> String {
    let store_name = html_escape(&settings.store_name);
    let pub_url = shop_public_url();
    let year = chrono::Utc::now().format("%Y");

    let brand = if !settings.logo_url.trim().is_empty() {
        format!(
            r#"<img src="cid:{cid}" width="{w}" height="{h}" alt="{name}" style="display: block; border: 0; outline: none; text-decoration: none; height: {h}px; width: auto; max-width: {mw}px;" />"#,
            cid = LOGO_CID,
            w = LOGO_WIDTH_TOKEN,
            h = LOGO_HEIGHT_TOKEN,
            name = store_name,
            mw = LOGO_MAX_W
        )
    } else {
        format!(r#"<span style="font-size: 20px; font-weight: 800; color: {}; letter-spacing: -0.3px;">{}</span>"#, C_HEAD, store_name)
    };

    // Legal / contact details for the footer
    let mut footer_lines = Vec::new();
    let legal = if settings.legal_name.trim().is_empty() { settings.store_name.trim() } else { settings.legal_name.trim() };
    footer_lines.push(format!("<strong style=\"color: {};\">{}</strong>", C_TEXT, html_escape(legal)));
    if !settings.company_address.trim().is_empty() {
        footer_lines.push(html_escape(settings.company_address.trim()));
    }
    let mut contact = Vec::new();
    let mail = if settings.support_email.trim().is_empty() { settings.smtp_from_email.trim() } else { settings.support_email.trim() };
    if !mail.is_empty() {
        contact.push(format!(r#"<a href="mailto:{0}" style="color: {1}; text-decoration: underline;">{0}</a>"#, html_escape(mail), C_MUTED));
    }
    if !settings.phone.trim().is_empty() {
        contact.push(html_escape(settings.phone.trim()));
    }
    if !contact.is_empty() {
        footer_lines.push(contact.join(" &bull; "));
    }
    let mut ids = Vec::new();
    if !settings.vat_id.trim().is_empty() {
        ids.push(format!("VAT ID {}", html_escape(settings.vat_id.trim())));
    }
    if !settings.tax_number.trim().is_empty() {
        ids.push(format!("Tax no. {}", html_escape(settings.tax_number.trim())));
    }
    if !settings.commercial_register.trim().is_empty() {
        ids.push(html_escape(settings.commercial_register.trim()));
    }
    if !ids.is_empty() {
        footer_lines.push(ids.join(" &bull; "));
    }
    let tax_notice = if settings.tax_mode == "kleingewerbe" && !settings.tax_notice.trim().is_empty() {
        format!(r#"<p style="margin: 10px 0 0; font-size: 11px; line-height: 1.5; color: {};">{}</p>"#, C_MUTED, html_escape(settings.tax_notice.trim()))
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en" xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="x-apple-disable-message-reformatting">
  <meta name="color-scheme" content="light">
  <meta name="supported-color-schemes" content="light">
  <title>{title}</title>
</head>
<body style="margin: 0; padding: 0; background-color: #f3f4f6; font-family: {font}; -webkit-font-smoothing: antialiased; color: {text};">
  <!--preheader--><div style="display: none; font-size: 1px; line-height: 1px; max-height: 0; max-width: 0; opacity: 0; overflow: hidden; mso-hide: all;">{preheader}&#8199;&#847;&#8199;&#847;&#8199;&#847;&#8199;&#847;&#8199;&#847;</div><!--/preheader-->
  <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="background-color: #f3f4f6;">
    <tr>
      <td align="center" style="padding: 32px 12px;">
        <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="max-width: 600px;">
          <tr>
            <td style="padding: 0 4px 18px 4px;">
              <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%">
                <tr>
                  <td align="left" style="vertical-align: middle;"><a href="{url}" target="_blank" style="text-decoration: none;">{brand}</a></td>
                  <td align="right" style="vertical-align: middle; font-size: 13px;"><a href="{url}" target="_blank" style="color: {accent}; font-weight: 600; text-decoration: none;">Visit shop &rarr;</a></td>
                </tr>
              </table>
            </td>
          </tr>
          <tr>
            <td style="background-color: #ffffff; border: 1px solid {line}; border-top: 4px solid {color}; border-radius: 12px; padding: 36px 36px 32px 36px;">
              <h1 style="margin: 0 0 18px 0; font-size: 22px; line-height: 1.3; font-weight: 700; color: {head}; letter-spacing: -0.3px;">{title}</h1>
              <div style="font-size: 15px; line-height: 1.65; color: {text};">
                {content}
              </div>
            </td>
          </tr>
          <tr>
            <td style="padding: 26px 16px 8px 16px; text-align: center; font-size: 12px; line-height: 1.7; color: {muted};">
              {footer}
              {tax_notice}
              <p style="margin: 14px 0 0; font-size: 11px; color: #9ca3af;">&copy; {year} {store} &bull; <a href="{url}" target="_blank" style="color: #9ca3af; text-decoration: underline;">{url_label}</a></p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#,
        title = heading_title,
        font = FONT,
        text = C_TEXT,
        preheader = html_escape(preheader),
        url = pub_url,
        url_label = html_escape(pub_url.trim_start_matches("https://").trim_start_matches("http://")),
        brand = brand,
        accent = C_ACCENT,
        line = C_LINE,
        color = heading_color,
        head = C_HEAD,
        content = content_body_html,
        muted = C_MUTED,
        footer = footer_lines.join("<br/>"),
        tax_notice = tax_notice,
        year = year,
        store = store_name,
    )
}

fn paragraph(html: &str) -> String {
    format!(r#"<p style="margin: 0 0 16px 0;">{}</p>"#, html)
}

fn small(html: &str) -> String {
    format!(r#"<p style="margin: 16px 0 0 0; font-size: 13px; line-height: 1.6; color: {};">{}</p>"#, C_MUTED, html)
}

/// "Bulletproof" button that renders in Outlook as well.
fn button(url: &str, label: &str, color: &str) -> String {
    format!(
        r#"<table role="presentation" cellpadding="0" cellspacing="0" border="0" style="margin: 26px 0;">
  <tr>
    <td align="center" bgcolor="{color}" style="border-radius: 8px; background-color: {color};">
      <a href="{url}" target="_blank" style="display: inline-block; padding: 13px 28px; font-size: 15px; font-weight: 700; color: #ffffff; text-decoration: none; border-radius: 8px;">{label}</a>
    </td>
  </tr>
</table>"#,
        color = color,
        url = html_escape(url),
        label = label
    )
}

fn link_fallback(url: &str) -> String {
    small(&format!(
        r#"If the button does not work, copy this link into your browser:<br/><a href="{0}" style="color: {1}; word-break: break-all;">{0}</a>"#,
        html_escape(url),
        C_ACCENT
    ))
}

/// Light grey box with label/value rows.
fn details_box(rows: &[(&str, String)]) -> String {
    let body = rows
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let border = if i > 0 { format!("border-top: 1px solid {};", C_LINE) } else { String::new() };
            format!(
                r#"<tr><td style="padding: 9px 0; {b} font-size: 14px; color: {m};">{l}</td><td align="right" style="padding: 9px 0; {b} font-size: 14px; color: {h}; font-weight: 600;">{v}</td></tr>"#,
                b = border,
                m = C_MUTED,
                h = C_HEAD,
                l = label,
                v = value
            )
        })
        .collect::<String>();
    format!(
        r#"<table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="margin: 22px 0; background-color: {}; border: 1px solid {}; border-radius: 10px;"><tr><td style="padding: 8px 20px;"><table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%">{}</table></td></tr></table>"#,
        C_SOFT, C_LINE, body
    )
}

fn money(cents: i64, symbol: &str) -> String {
    crate::services::invoice_pdf::money(cents, symbol)
}

/// What an order email shows: items and totals exactly as on the invoice.
pub struct OrderSummary {
    pub invoice: crate::services::invoice::Invoice,
}

impl OrderSummary {
    pub fn new(order: &Order, items: &[OrderItem], settings: &StoreSettings) -> Self {
        OrderSummary { invoice: crate::services::invoice::Invoice::build(order, items, settings) }
    }

    fn html(&self) -> String {
        let inv = &self.invoice;
        let sym = inv.currency_symbol.as_str();
        let mut rows = String::new();
        for line in &inv.lines {
            let mut sub = Vec::new();
            if let Some(v) = &line.variant {
                sub.push(html_escape(v));
            }
            if line.is_digital {
                sub.push("Digital download".to_string());
            }
            let sub_html = if sub.is_empty() {
                String::new()
            } else {
                format!(r#"<br/><span style="font-size: 12px; color: {};">{}</span>"#, C_MUTED, sub.join(" &bull; "))
            };
            rows.push_str(&format!(
                r#"<tr>
  <td style="padding: 12px 0; border-bottom: 1px solid {line}; font-size: 14px; color: {head};"><strong>{title}</strong>{sub}</td>
  <td align="center" style="padding: 12px 8px; border-bottom: 1px solid {line}; font-size: 14px; color: {muted}; white-space: nowrap;">&times; {qty}</td>
  <td align="right" style="padding: 12px 0; border-bottom: 1px solid {line}; font-size: 14px; color: {head}; white-space: nowrap;">{price}</td>
</tr>"#,
                line = C_LINE,
                head = C_HEAD,
                muted = C_MUTED,
                title = html_escape(&line.name),
                sub = sub_html,
                qty = line.quantity,
                price = money(line.line_display_cents, sym)
            ));
        }
        let total_row = |label: &str, value: String, strong: bool| {
            let (size, weight, color) = if strong { ("16px", "800", C_HEAD) } else { ("14px", "400", C_MUTED) };
            format!(
                r#"<tr><td colspan="2" style="padding: 5px 0; font-size: {s}; font-weight: {w}; color: {c};">{l}</td><td align="right" style="padding: 5px 0; font-size: {s}; font-weight: {w}; color: {c}; white-space: nowrap;">{v}</td></tr>"#,
                s = size,
                w = weight,
                c = color,
                l = label,
                v = value
            )
        };
        let items_display: i64 = inv.lines.iter().map(|l| l.line_display_cents).sum();
        let mut totals = String::new();
        totals.push_str(&total_row(if inv.shows_gross_prices() { "Subtotal" } else { "Subtotal (net)" }, money(items_display, sym), false));
        if inv.discount_display_cents > 0 {
            let label = match &inv.discount_code {
                Some(code) => format!("Discount ({})", html_escape(code)),
                None => "Discount".to_string(),
            };
            totals.push_str(&total_row(&label, money(-inv.discount_display_cents, sym), false));
        }
        if inv.has_physical_items || inv.shipping_display_cents > 0 {
            totals.push_str(&total_row("Shipping", money(inv.shipping_display_cents, sym), false));
        }
        if !inv.shows_gross_prices() {
            for g in &inv.groups {
                totals.push_str(&total_row(&format!("VAT {} %", g.vat.percent), money(g.tax_cents, sym), false));
            }
        }
        totals.push_str(&format!(r#"<tr><td colspan="3" style="padding-top: 6px; border-top: 2px solid {};"></td></tr>"#, C_HEAD));
        totals.push_str(&total_row("Total", money(inv.payable_cents, sym), true));
        if inv.tax_mode == "included" && inv.tax_total_cents > 0 {
            totals.push_str(&format!(
                r#"<tr><td colspan="3" align="right" style="font-size: 12px; color: {};">incl. {} VAT</td></tr>"#,
                C_MUTED,
                money(inv.tax_total_cents, sym)
            ));
        }
        format!(
            r#"<table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="margin: 24px 0 8px 0;">
<tr><td colspan="3" style="padding-bottom: 8px; border-bottom: 2px solid {head}; font-size: 12px; font-weight: 700; letter-spacing: 0.6px; text-transform: uppercase; color: {muted};">Order {number}</td></tr>
{rows}
<tr><td colspan="3" style="padding-top: 10px;"></td></tr>
{totals}
</table>"#,
            head = C_HEAD,
            muted = C_MUTED,
            number = html_escape(&inv.order_number),
            rows = rows,
            totals = totals
        )
    }
}

// ------------------------------------------------------------------ templates

pub async fn send_test_email(settings: &StoreSettings, to_email: &str) -> Result<(), String> {
    let subject = format!("Test email from {}", settings.store_name);
    let content = format!(
        "{}{}",
        paragraph(&format!(
            "This test confirms that <strong>{}</strong> can send emails. Your customers will receive order confirmations, invoices and shipping updates.",
            html_escape(&settings.store_name)
        )),
        details_box(&[
            ("Mail server", format!("{}:{}", html_escape(&settings.smtp_host), settings.smtp_port)),
            ("Encryption", html_escape(&settings.smtp_encryption)),
            ("Sender", format!("{} &lt;{}&gt;", html_escape(&settings.smtp_from_name), html_escape(&settings.smtp_from_email))),
        ])
    );
    let html = render_email_layout(settings, "Your shop can send emails.", "Email delivery works", "#10b981", &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_verification_email(
    settings: &StoreSettings,
    to_email: &str,
    customer_name: &str,
    token: &str,
    origin: &str,
) -> Result<(), String> {
    let verify_url = format!("{}/account/verify?token={}", origin.trim_end_matches('/'), token);
    let subject = format!("Please confirm your email address - {}", settings.store_name);
    let content = format!(
        "{}{}{}",
        paragraph(&format!(
            "Thank you for creating an account at <strong>{}</strong>. Please confirm your email address to activate it.",
            html_escape(&settings.store_name)
        )),
        button(&verify_url, "Confirm email address", C_ACCENT),
        link_fallback(&verify_url)
    );
    let heading = format!("Welcome, {}!", html_escape(customer_name));
    let html = render_email_layout(settings, "Confirm your email address to activate your account.", &heading, C_ACCENT, &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_order_created_email(settings: &StoreSettings, to_email: &str, summary: &OrderSummary) -> Result<(), String> {
    let inv = &summary.invoice;
    let subject = format!("Order confirmation {} - {}", inv.order_number, settings.store_name);
    let next = if inv.paid {
        "You will receive a payment confirmation with your invoice in a separate email."
    } else {
        "As soon as your payment has been confirmed, we will send you your invoice."
    };
    let content = format!(
        "{}{}{}",
        paragraph(&format!(
            "Hello {}, thank you for your order! We have received it and will take care of it right away.",
            html_escape(inv.buyer.person())
        )),
        summary.html(),
        small(next)
    );
    let heading = "Thank you for your order";
    let html = render_email_layout(settings, &format!("We have received your order {}.", inv.order_number), heading, C_ACCENT, &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_payment_received_email(
    settings: &StoreSettings,
    to_email: &str,
    summary: &OrderSummary,
    attachments: Vec<EmailAttachment>,
) -> Result<(), String> {
    let inv = &summary.invoice;
    let subject = format!("Payment received for order {} - {}", inv.order_number, settings.store_name);
    let next = if inv.has_physical_items {
        "We are now packing your order and will let you know as soon as it has been shipped."
    } else {
        "Your digital products are ready: you can download them right away in your customer account or via the link on your order page."
    };
    let invoice_note = match attachments.iter().any(|a| a.content_type == "application/pdf") {
        true => small("Your invoice is attached to this email as a PDF. Please keep it for your records."),
        false => String::new(),
    };
    let content = format!(
        "{}{}{}{}",
        paragraph(&format!(
            "Hello {}, we have received your payment of <strong>{}</strong>. {}",
            html_escape(inv.buyer.person()),
            money(inv.payable_cents, &inv.currency_symbol),
            next
        )),
        summary.html(),
        details_box(&[("Paid via", html_escape(&inv.payment_label)), ("Invoice number", html_escape(&inv.number))]),
        invoice_note
    );
    let html = render_email_layout(
        settings,
        &format!("Payment confirmed for order {} — your invoice is attached.", inv.order_number),
        "Payment received",
        "#10b981",
        &content,
    );
    send_email(settings, to_email, &subject, &html, attachments).await
}

pub async fn send_order_shipped_email(
    settings: &StoreSettings,
    to_email: &str,
    order_number: &str,
    tracking_number: &str,
) -> Result<(), String> {
    let tracking_url = if tracking_number.starts_with("DHL") || tracking_number.len() >= 10 {
        format!("https://www.dhl.de/de/privatkunden/pakete-empfangen/verfolgen.html?piececode={}", tracking_number)
    } else {
        format!("https://parcelsapp.com/en/tracking/{}", tracking_number)
    };

    let subject = format!("Your order {} has been shipped", order_number);
    let content = format!(
        "{}{}{}",
        paragraph(&format!("Good news: your order <strong>{}</strong> is on its way to you.", html_escape(order_number))),
        details_box(&[("Order", html_escape(order_number)), ("Tracking number", html_escape(tracking_number))]),
        button(&tracking_url, "Track your parcel", "#0284c7")
    );
    let html = render_email_layout(settings, &format!("Order {} has been shipped.", order_number), "Your order is on its way", "#0284c7", &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_back_in_stock_email(
    settings: &StoreSettings,
    to_email: &str,
    product_title: &str,
    product_url: &str,
) -> Result<(), String> {
    let subject = format!("Back in stock: {}", product_title);
    let content = format!(
        "{}{}{}",
        paragraph(&format!(
            "You asked us to let you know: <strong>{}</strong> is available again at {}.",
            html_escape(product_title),
            html_escape(&settings.store_name)
        )),
        button(product_url, "View product", C_ACCENT),
        small("Stock may be limited — order soon to secure yours.")
    );
    let html = render_email_layout(settings, &format!("{} is available again.", product_title), "It's back in stock", "#10b981", &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_password_reset_email(
    settings: &StoreSettings,
    to_email: &str,
    token: &str,
) -> Result<(), String> {
    let reset_url = format!("{}/account/reset-password?token={}", shop_public_url(), token);
    let subject = format!("Reset your password - {}", settings.store_name);
    let content = format!(
        "{}{}{}{}",
        paragraph(&format!(
            "Someone asked to reset the password of your account at <strong>{}</strong>. The link is valid for <strong>1 hour</strong>.",
            html_escape(&settings.store_name)
        )),
        button(&reset_url, "Choose a new password", C_ACCENT),
        link_fallback(&reset_url),
        small("If you did not request this, simply ignore this email — your password stays unchanged.")
    );
    let html = render_email_layout(settings, "Reset the password of your account.", "Reset your password", C_ACCENT, &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_part_low_stock_email(
    settings: &StoreSettings,
    to_email: &str,
    part_name: &str,
    part_sku: &str,
    stock_quantity: i32,
    low_stock_threshold: i32,
    storage_location: Option<&str>,
) -> Result<(), String> {
    let loc_str = storage_location.filter(|s| !s.trim().is_empty()).unwrap_or("Not specified");
    let subject = format!("Low stock: {} ({}) - {}", part_name, part_sku, settings.store_name);
    let admin_url = format!("{}/products?tab=stock", admin_public_url());
    let content = format!(
        "{}{}{}{}",
        paragraph(&format!("The stock of <strong>{}</strong> has reached its low-stock threshold.", html_escape(part_name))),
        details_box(&[
            ("Part", html_escape(part_name)),
            ("SKU", html_escape(part_sku)),
            ("In stock", format!(r#"<span style="color: #dc2626;">{} units</span>"#, stock_quantity)),
            ("Threshold", format!("{} units", low_stock_threshold)),
            ("Storage location", html_escape(loc_str)),
        ]),
        button(&admin_url, "Open stock overview", "#d97706"),
        small("You receive this notification because of your shop's stock alert settings.")
    );
    let html = render_email_layout(
        settings,
        &format!("{} ({}) is running low: {} units left.", part_name, part_sku, stock_quantity),
        "Stock is running low",
        "#f59e0b",
        &content,
    );
    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_backup_failed_email(
    settings: &StoreSettings,
    to_email: &str,
    repository: &str,
    error_message: &str,
) -> Result<(), String> {
    let subject = format!("Automatic backup failed - {}", settings.store_name);
    let admin_url = format!("{}/settings?tab=export", admin_public_url());
    let content = format!(
        "{}{}{}{}",
        paragraph(&format!(
            "The automatic backup of your shop to the GitHub repository <strong>{}</strong> could not be created. It is retried every hour; you will only get another email after a backup has succeeded again.",
            html_escape(repository)
        )),
        format!(
            r#"<div style="margin: 22px 0; padding: 16px 20px; background-color: #fef2f2; border: 1px solid #fecaca; border-radius: 10px; font-size: 14px; line-height: 1.55; color: #991b1b;">{}</div>"#,
            html_escape(error_message)
        ),
        button(&admin_url, "Open backup settings", "#dc2626"),
        small("You receive this notification because you are a superadmin of this shop.")
    );
    let html = render_email_layout(settings, "The automatic backup of your shop failed.", "Automatic backup failed", "#ef4444", &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

// ------------------------------------------------------------------ newsletter & withdrawal

/// Double opt-in: the subscription only becomes active after this link was clicked.
pub async fn send_newsletter_confirmation_email(settings: &StoreSettings, to_email: &str, token: &str) -> Result<(), String> {
    let url = format!("{}/newsletter/confirm?token={}", shop_public_url(), token);
    let subject = format!("Please confirm your newsletter subscription - {}", settings.store_name);
    let content = format!(
        "{}{}{}{}",
        paragraph(&format!(
            "Someone — hopefully you — signed up for the newsletter of <strong>{}</strong> with this email address. Please confirm that you want to receive it:",
            html_escape(&settings.store_name)
        )),
        button(&url, "Yes, subscribe me", C_ACCENT),
        link_fallback(&url),
        small("If you did not sign up, simply ignore this email — you will not be subscribed and will not hear from us again. You can unsubscribe at any time with the link in every newsletter.")
    );
    let html = render_email_layout(settings, "Confirm your newsletter subscription.", "Confirm your subscription", C_ACCENT, &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

/// Minimal, safe Markdown for newsletters: headings, paragraphs, lists, **bold**, *italic*, [links](https://…).
pub fn simple_markdown(md: &str) -> String {
    fn inline(text: &str) -> String {
        let mut s = html_escape(text);
        // Links: [label](https://url), scanned left to right
        let mut from = 0;
        while let Some(a) = s[from..].find('[').map(|i| from + i) {
            let Some(b) = s[a..].find("](").map(|i| a + i) else { break };
            let Some(c) = s[b..].find(')').map(|i| b + i) else { break };
            let label = s[a + 1..b].to_string();
            let url = s[b + 2..c].to_string();
            let link = if url.starts_with("https://") || url.starts_with("http://") || url.starts_with("mailto:") {
                format!(r#"<a href="{}" style="color: {}; text-decoration: underline;">{}</a>"#, url, C_ACCENT, label)
            } else {
                label
            };
            s = format!("{}{}{}", &s[..a], link, &s[c + 1..]);
            from = a + link.len();
        }
        for (marker, tag) in [("**", "strong"), ("*", "em")] {
            let mut out = String::new();
            let mut open = false;
            for (i, part) in s.split(marker).enumerate() {
                if i > 0 {
                    out.push_str(&if open { format!("</{}>", tag) } else { format!("<{}>", tag) });
                    open = !open;
                }
                out.push_str(part);
            }
            if open {
                out.push_str(&format!("</{}>", tag));
            }
            s = out;
        }
        s
    }
    let mut html = String::new();
    let mut list = false;
    let mut para: Vec<String> = Vec::new();
    let flush = |para: &mut Vec<String>, html: &mut String| {
        if !para.is_empty() {
            html.push_str(&paragraph(&para.join("<br/>")));
            para.clear();
        }
    };
    for line in md.lines() {
        let t = line.trim();
        if let Some(item) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
            flush(&mut para, &mut html);
            if !list {
                html.push_str(r#"<ul style="margin: 0 0 16px 0; padding-left: 22px;">"#);
                list = true;
            }
            html.push_str(&format!("<li style=\"margin-bottom: 6px;\">{}</li>", inline(item)));
            continue;
        }
        if list {
            html.push_str("</ul>");
            list = false;
        }
        if t.is_empty() {
            flush(&mut para, &mut html);
        } else if let Some(h) = t.strip_prefix("## ").or_else(|| t.strip_prefix("# ")) {
            flush(&mut para, &mut html);
            html.push_str(&format!(r#"<h2 style="margin: 24px 0 12px 0; font-size: 18px; color: {};">{}</h2>"#, C_HEAD, inline(h)));
        } else {
            para.push(inline(t));
        }
    }
    if list {
        html.push_str("</ul>");
    }
    flush(&mut para, &mut html);
    html
}

pub async fn send_newsletter_email(settings: &StoreSettings, to_email: &str, subject: &str, body_markdown: &str, unsubscribe_token: &str) -> Result<(), String> {
    let unsubscribe_url = format!("{}/newsletter/unsubscribe?token={}", shop_public_url(), unsubscribe_token);
    let content = format!(
        "{}{}",
        simple_markdown(body_markdown),
        small(&format!(
            "You receive this newsletter because you subscribed at {}. <a href=\"{}\" style=\"color: {};\">Unsubscribe</a> at any time with one click.",
            html_escape(&settings.store_name),
            html_escape(&unsubscribe_url),
            C_MUTED
        ))
    );
    let html = render_email_layout(settings, subject, &html_escape(subject), C_ACCENT, &content);
    send_email_raw(settings, to_email, subject, &html).await
}

pub struct WithdrawalDetails<'a> {
    pub reference: &'a str,
    pub received_at: chrono::DateTime<chrono::Utc>,
    pub order_number: &'a str,
    pub customer_name: &'a str,
    pub customer_email: &'a str,
    pub items: &'a str,
    pub message: &'a str,
}

fn withdrawal_rows(w: &WithdrawalDetails) -> Vec<(&'static str, String)> {
    let mut rows = vec![
        ("Received", w.received_at.format("%d.%m.%Y %H:%M UTC").to_string()),
        ("Reference", html_escape(w.reference)),
        ("Order number", html_escape(w.order_number)),
        ("Name", html_escape(w.customer_name)),
        ("Email", html_escape(w.customer_email)),
    ];
    rows.push(("Items", if w.items.trim().is_empty() { "All items of the order".to_string() } else { html_escape(w.items) }));
    if !w.message.trim().is_empty() {
        rows.push(("Message", html_escape(w.message)));
    }
    rows
}

/// Confirmation of receipt on a durable medium, required for the withdrawal function.
pub async fn send_withdrawal_confirmation_email(settings: &StoreSettings, w: &WithdrawalDetails<'_>) -> Result<(), String> {
    let subject = format!("We have received your withdrawal for order {} - {}", w.order_number, settings.store_name);
    let content = format!(
        "{}{}{}",
        paragraph(&format!(
            "Hello {}, we confirm that we have received your declaration of withdrawal from the contract for order <strong>{}</strong>.",
            html_escape(w.customer_name),
            html_escape(w.order_number)
        )),
        details_box(&withdrawal_rows(w)),
        small("Please send physical goods back to us within 14 days, unless we agreed otherwise. We will refund your payment within 14 days of receiving your withdrawal — for goods not before they have arrived back or you have proven that you sent them. Please keep this email as proof of your withdrawal.")
    );
    let html = render_email_layout(settings, "Confirmation of your withdrawal.", "Withdrawal received", "#0284c7", &content);
    send_email_raw(settings, w.customer_email, &subject, &html).await
}

pub async fn send_withdrawal_notification_email(settings: &StoreSettings, to_email: &str, w: &WithdrawalDetails<'_>) -> Result<(), String> {
    let subject = format!("Withdrawal received: order {}", w.order_number);
    let admin_url = format!("{}/orders?tab=withdrawals", admin_public_url());
    let content = format!(
        "{}{}{}",
        paragraph(&format!("A customer withdrew from order <strong>{}</strong> using the withdrawal button in the shop.", html_escape(w.order_number))),
        details_box(&withdrawal_rows(w)),
        button(&admin_url, "Open withdrawals", "#0284c7")
    );
    let html = render_email_layout(settings, &format!("Withdrawal for order {}.", w.order_number), "Withdrawal received", "#0284c7", &content);
    send_email_raw(settings, to_email, &subject, &html).await
}

// ------------------------------------------------------------------ invoices for emails

/// The invoice PDF (a ZUGFeRD / Factur-X e-invoice when enabled) for the payment confirmation.
pub async fn invoice_attachments(settings: &StoreSettings, order: &Order, items: &[OrderItem]) -> Vec<EmailAttachment> {
    if !settings.email_invoice_pdf {
        return Vec::new();
    }
    match render_invoice_pdf(settings, order, items).await {
        Ok((name, pdf)) => vec![EmailAttachment { filename: name, content_type: "application/pdf".to_string(), data: pdf }],
        Err(e) => {
            error!("Invoice PDF for order {} could not be created: {}", order.order_number, e);
            Vec::new()
        }
    }
}

/// Renders the invoice of an order; returns the file name and the PDF.
pub async fn render_invoice_pdf(settings: &StoreSettings, order: &Order, items: &[OrderItem]) -> Result<(String, Vec<u8>), String> {
    let invoice = crate::services::invoice::Invoice::build(order, items, settings);
    let settings = settings.clone();
    let shop_url = shop_public_url();
    tokio::task::spawn_blocking(move || {
        let logo = crate::services::branding::logo_png(&settings, 600);
        let xml = settings.einvoice_enabled.then(|| invoice.to_cii_xml());
        let pdf = crate::services::invoice_pdf::render(&invoice, logo.as_deref(), &shop_url, xml.as_deref())?;
        Ok((format!("{}.pdf", invoice.file_stem()), pdf))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ------------------------------------------------------------------ previews (test emails)

fn sample_order(settings: &StoreSettings) -> (Order, Vec<OrderItem>) {
    let now = chrono::Utc::now();
    let order: Order = serde_json::from_value(serde_json::json!({
        "id": Uuid::nil(), "order_number": "ORD-10042", "customer_name": "Erika Musterfrau", "customer_email": "erika@example.com",
        "shipping_address": {}, "billing_address": { "full_name": "Erika Musterfrau", "street_address": "Beispielweg 5",
            "postal_code": "80331", "city": "München", "country_code": "DE" },
        "shipping_rate_id": null, "shipping_cost_cents": 490, "subtotal_cents": 7470, "tax_cents": 0, "total_cents": 7960,
        "payment_provider": "stripe", "payment_status": "paid", "order_status": "processing", "tracking_number": null, "notes": null,
        "coupon_code": null, "discount_cents": 0, "payment_reference": null, "is_test": true, "tax_mode": settings.tax_mode,
        "created_at": now, "updated_at": now
    }))
    .expect("sample order");
    let item = |title: &str, variant: &str, sku: &str, unit: i32, qty: i32, digital: bool| OrderItem {
        id: Uuid::nil(),
        order_id: Uuid::nil(),
        product_id: None,
        variant_id: None,
        product_title: title.to_string(),
        variant_title: variant.to_string(),
        sku: sku.to_string(),
        unit_price_cents: unit,
        quantity: qty,
        total_price_cents: unit * qty,
        is_digital: digital,
        download_url: None,
        tax_rate_percent: Some(settings.tax_rate_percent),
    };
    let items = vec![
        item("Pro Mechanical Keyboard", "Brown switches", "KB-PRO-BRN", 5990, 1, false),
        item("USB-C Cable 2 m", "Standard", "CBL-USBC-2M", 990, 1, false),
        item("Setup Guide (PDF)", "Standard", "GUIDE-PDF", 490, 1, true),
    ];
    // Totals as the checkout would compute them for the current tax mode
    let mut order = order;
    let probe = crate::services::invoice::Invoice::build(&order, &items, settings);
    order.total_cents = if settings.tax_mode == "excluded" { probe.grand_total_cents as i32 } else { 7470 + 490 };
    (order, items)
}

/// Dispatches a test email according to the specified template type (or default test email) with sample data.
pub async fn send_test_email_by_type(
    settings: &StoreSettings,
    to_email: &str,
    email_type: Option<&str>,
) -> Result<(), String> {
    match email_type.unwrap_or("test") {
        "verification" => {
            send_verification_email(settings, to_email, "Erika Musterfrau", "sample-verification-token-12345", &shop_public_url()).await
        }
        "order_created" => {
            let (order, items) = sample_order(settings);
            send_order_created_email(settings, to_email, &OrderSummary::new(&order, &items, settings)).await
        }
        "payment_received" => {
            let (order, items) = sample_order(settings);
            let attachments = invoice_attachments(settings, &order, &items).await;
            send_payment_received_email(settings, to_email, &OrderSummary::new(&order, &items, settings), attachments).await
        }
        "refund" => {
            let (order, items) = sample_order(settings);
            let summary = OrderSummary::new(&order, &items, settings);
            let credit_note = summary.invoice.credit_note("ORD-10042-CN".to_string(), chrono::Utc::now().date_naive());
            let settings_c = settings.clone();
            let shop_url = shop_public_url();
            let rendered = tokio::task::spawn_blocking(move || {
                let logo = crate::services::branding::logo_png(&settings_c, 600);
                let xml = settings_c.einvoice_enabled.then(|| credit_note.to_cii_xml());
                crate::services::invoice_pdf::render(&credit_note, logo.as_deref(), &shop_url, xml.as_deref())
            })
            .await
            .map_err(|e| e.to_string())??;
            let attachment = EmailAttachment { filename: "Credit-Note-ORD-10042-CN.pdf".to_string(), content_type: "application/pdf".to_string(), data: rendered };
            send_refund_email(settings, to_email, &summary, vec![attachment]).await
        }
        "newsletter_confirm" => send_newsletter_confirmation_email(settings, to_email, "sample-confirmation-token").await,
        "newsletter" => {
            send_newsletter_email(
                settings,
                to_email,
                "New in the shop",
                "## Fresh arrivals\n\nWe have **new products** for you:\n\n- Pro Mechanical Keyboard\n- USB-C Cable 2 m\n\n[Visit the shop](https://example.com)",
                "sample-unsubscribe-token",
            )
            .await
        }
        "withdrawal_confirmation" | "withdrawal_notification" => {
            let w = WithdrawalDetails {
                reference: "W-1A2B3C",
                received_at: chrono::Utc::now(),
                order_number: "ORD-10042",
                customer_name: "Erika Musterfrau",
                customer_email: to_email,
                items: "",
                message: "The keyboard does not fit my desk.",
            };
            if email_type == Some("withdrawal_confirmation") {
                send_withdrawal_confirmation_email(settings, &w).await
            } else {
                send_withdrawal_notification_email(settings, to_email, &w).await
            }
        }
        "order_shipped" => send_order_shipped_email(settings, to_email, "ORD-10042", "DHL9876543210DE").await,
        "back_in_stock" => {
            let sample_url = format!("{}/products", shop_public_url());
            send_back_in_stock_email(settings, to_email, "Pro Mechanical Keyboard", &sample_url).await
        }
        "password_reset" => send_password_reset_email(settings, to_email, "sample-password-reset-token-67890").await,
        "low_stock_alert" => {
            send_part_low_stock_email(settings, to_email, "Cherry MX Red Switches", "PRT-SW-CHERRY-RED", 3, 10, Some("Warehouse A - Shelf 4B")).await
        }
        "backup_failed" => {
            send_backup_failed_email(
                settings,
                to_email,
                "my-account/shop-backups",
                "GitHub rejected the access token — it is wrong, expired or was deleted. Create a new token and save it.",
            )
            .await
        }
        "test" | _ => send_test_email(settings, to_email).await,
    }
}

// ------------------------------------------------------------------ order notifications

async fn load_order(pool: &sqlx::PgPool, order_number: &str) -> Option<(StoreSettings, Order, Vec<OrderItem>)> {
    let settings = match sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await {
        Ok(s) => s,
        Err(e) => {
            error!("Cannot load store settings for order email {}: {}", order_number, e);
            return None;
        }
    };
    let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE order_number = $1")
        .bind(order_number)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;
    let items = sqlx::query_as::<_, OrderItem>("SELECT * FROM order_items WHERE order_id = $1")
        .bind(order.id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    Some((settings, order, items))
}

pub async fn send_order_created_notification(pool: &sqlx::PgPool, order_number: &str) {
    let Some((settings, order, items)) = load_order(pool, order_number).await else { return };
    let summary = OrderSummary::new(&order, &items, &settings);
    if let Err(e) = send_order_created_email(&settings, &order.customer_email, &summary).await {
        error!("Order confirmation for {} not sent: {}", order_number, e);
    }
}

pub async fn send_payment_received_notification(pool: &sqlx::PgPool, order_number: &str) {
    let Some((settings, order, items)) = load_order(pool, order_number).await else { return };
    let summary = OrderSummary::new(&order, &items, &settings);
    // The invoice is issued (and stored unchangeable) now that the payment is confirmed
    let attachments = match crate::services::invoice_store::ensure_invoice(pool, order.id).await {
        Ok(Some(doc)) if settings.email_invoice_pdf => vec![pdf_attachment(doc)],
        Ok(_) => Vec::new(),
        Err(e) => {
            error!("Invoice for order {} could not be issued: {}", order_number, e);
            Vec::new()
        }
    };
    if let Err(e) = send_payment_received_email(&settings, &order.customer_email, &summary, attachments).await {
        error!("Payment confirmation for {} not sent: {}", order_number, e);
    }
}

fn pdf_attachment(doc: crate::services::invoice_store::StoredDocument) -> EmailAttachment {
    EmailAttachment { filename: doc.file_name, content_type: "application/pdf".to_string(), data: doc.pdf }
}

pub async fn send_refund_email(
    settings: &StoreSettings,
    to_email: &str,
    summary: &OrderSummary,
    attachments: Vec<EmailAttachment>,
) -> Result<(), String> {
    let inv = &summary.invoice;
    let subject = format!("Refund for order {} - {}", inv.order_number, settings.store_name);
    let note = if attachments.is_empty() {
        String::new()
    } else {
        small("The credit note cancelling your invoice is attached as a PDF. Please keep it for your records.")
    };
    let content = format!(
        "{}{}{}",
        paragraph(&format!(
            "Hello {}, we have refunded <strong>{}</strong> for your order <strong>{}</strong> via {}. Depending on your payment method it can take a few days until the amount shows up in your account.",
            html_escape(inv.buyer.person()),
            money(inv.payable_cents, &inv.currency_symbol),
            html_escape(&inv.order_number),
            html_escape(&inv.payment_label)
        )),
        summary.html(),
        note
    );
    let html = render_email_layout(settings, &format!("Your refund for order {} is on its way.", inv.order_number), "Refund issued", "#0284c7", &content);
    send_email(settings, to_email, &subject, &html, attachments).await
}

/// After a refund: credit note (stored) and email to the customer.
pub async fn send_refund_notification(pool: &sqlx::PgPool, order_id: Uuid) {
    let Ok((settings, order, items)) = crate::services::invoice_store::load_order(pool, order_id).await else { return };
    let attachments = match crate::services::invoice_store::ensure_credit_note(pool, order_id).await {
        Ok(Some(doc)) => vec![pdf_attachment(doc)],
        Ok(None) => Vec::new(),
        Err(e) => {
            error!("Credit note for order {} could not be issued: {}", order.order_number, e);
            Vec::new()
        }
    };
    let summary = OrderSummary::new(&order, &items, &settings);
    if let Err(e) = send_refund_email(&settings, &order.customer_email, &summary, attachments).await {
        error!("Refund email for {} not sent: {}", order.order_number, e);
    }
}

pub async fn check_and_send_low_stock_alerts(pool: &sqlx::PgPool) {
    let settings = match sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await {
        Ok(s) => s,
        Err(e) => {
            error!("Cannot load store settings for low stock alerts: {}", e);
            return;
        }
    };

    if !settings.low_stock_alerts_enabled || !settings.smtp_enabled || settings.smtp_host.trim().is_empty() {
        return;
    }

    // Determine target recipient emails
    let admin_users: Vec<(Uuid, String, Option<String>, String, serde_json::Value)> = sqlx::query_as(
        "SELECT id, username, email, role, permissions FROM admin_users"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mode = settings.low_stock_alert_recipients_mode.as_str();
    let selected_ids: HashSet<String> = settings
        .low_stock_alert_selected_user_ids
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let mut recipient_emails = BTreeSet::new();

    for (id, _username, email_opt, role, perms) in &admin_users {
        let email = match email_opt {
            Some(e) if !e.trim().is_empty() => e.trim().to_string(),
            _ => continue,
        };

        let has_stock_access = role == "superadmin" || {
            let eff = crate::middleware::effective_permissions(role, perms);
            eff.get("products").copied().unwrap_or(false)
        };

        let is_selected = selected_ids.contains(&id.to_string());

        let include = match mode {
            "stock_managers" => has_stock_access,
            "selected_users" => is_selected,
            "both" | "all" => has_stock_access || is_selected,
            _ => has_stock_access,
        };

        if include {
            recipient_emails.insert(email);
        }
    }

    // Parse custom emails
    for part in settings.low_stock_alert_custom_emails.split(&[',', ';', '\n', '\r'][..]) {
        let email = part.trim();
        if !email.is_empty() && email.contains('@') {
            recipient_emails.insert(email.to_string());
        }
    }

    // Fallback to support_email if no recipients found
    if recipient_emails.is_empty() {
        let sup = settings.support_email.trim();
        if !sup.is_empty() && sup.contains('@') {
            recipient_emails.insert(sup.to_string());
        }
    }

    if recipient_emails.is_empty() {
        info!("Low stock alert triggered but no valid recipient emails configured");
        return;
    }

    // Find parts that are at or below threshold and need an alert
    let low_parts: Vec<(Uuid, String, String, i32, i32, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, name, sku, stock_quantity, low_stock_threshold, storage_location
        FROM bom_parts
        WHERE low_stock_threshold > 0
          AND stock_quantity <= low_stock_threshold
          AND (
              last_low_stock_alert_at IS NULL
              OR last_alert_stock_quantity IS NULL
              OR stock_quantity < last_alert_stock_quantity
              OR last_low_stock_alert_at < NOW() - INTERVAL '24 hours'
          )
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (part_id, name, sku, stock_qty, threshold, loc) in low_parts {
        let mut any_sent = false;
        for email in &recipient_emails {
            match send_part_low_stock_email(&settings, email, &name, &sku, stock_qty, threshold, loc.as_deref()).await {
                Ok(_) => {
                    any_sent = true;
                }
                Err(e) => {
                    error!("Failed to dispatch low stock email for {} to {}: {}", sku, email, e);
                }
            }
        }

        if any_sent {
            let _ = sqlx::query(
                "UPDATE bom_parts SET last_low_stock_alert_at = NOW(), last_alert_stock_quantity = $1 WHERE id = $2"
            )
            .bind(stock_qty)
            .bind(part_id)
            .execute(pool)
            .await;
        }
    }
}

/// Escapes text inserted into HTML emails (customer-supplied names must never become markup).
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Public base URL of the storefront used in links inside emails (e.g. https://shop.example).
pub fn shop_public_url() -> String {
    std::env::var("SHOP_PUBLIC_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Public base URL of the admin dashboard used in links inside admin emails (e.g. https://admin.shop.example).
pub fn admin_public_url() -> String {
    std::env::var("ADMIN_PUBLIC_URL")
        .unwrap_or_else(|_| "http://localhost:4000".to_string())
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::invoice::tests::sample_settings;

    #[test]
    fn layout_embeds_logo_and_has_readable_text_version() {
        let mut settings = sample_settings("kleingewerbe");
        settings.logo_url = "/uploads/logo.png".into();
        let html = render_email_layout(&settings, "Hidden preheader", "Payment received", "#10b981", &button("https://shop.example/x", "Open", C_ACCENT));
        assert!(html.contains("src=\"cid:shop-logo\""));
        assert!(html.contains(LOGO_WIDTH_TOKEN) && html.contains(LOGO_HEIGHT_TOKEN));
        let text = html_to_text(&html);
        assert!(!text.contains("Hidden preheader"));
        assert!(text.contains("Payment received"));
        assert!(text.contains("Open (https://shop.example/x)"));
        assert!(!text.contains('<'));
    }

    #[test]
    fn order_summary_lists_items_and_total() {
        let settings = sample_settings("included");
        let (order, items) = sample_order(&settings);
        let html = OrderSummary::new(&order, &items, &settings).html();
        assert!(html.contains("Pro Mechanical Keyboard"));
        assert!(html.contains("79,60 €"));
        assert!(html.contains("incl."));
    }
}

#[cfg(test)]
mod markdown_tests {
    use super::simple_markdown;

    #[test]
    fn newsletter_markdown_is_safe_and_formatted() {
        let html = simple_markdown("# Hi\n\nSee [A](https://a.example) and [B](https://b.example) **now** *please*\n\n- one\n- two\n\n<script>x</script> [bad](javascript:alert(1))");
        assert!(html.contains(r#"href="https://a.example""#) && html.contains(r#"href="https://b.example""#));
        assert!(html.contains("<strong>now</strong>") && html.contains("<em>please</em>"));
        assert!(html.contains("<li") && html.contains("</ul>"));
        assert!(!html.contains("<script>") && !html.contains("javascript:"));
    }
}
