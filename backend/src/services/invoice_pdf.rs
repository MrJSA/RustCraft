//! Renders an [`Invoice`] as an A4 PDF. With e-invoicing switched on the PDF is a PDF/A-3b file with
//! the EN 16931 XML embedded as `factur-x.xml` — a ZUGFeRD 2 / Factur-X invoice (profile EN 16931)
//! that people can read and accounting software can import.
//!
//! Layout happens first (positions of every text and box per page), drawing afterwards, so the
//! footer can say "Page x of y" and long orders flow over several pages.

use crate::services::branding::Logo;
use crate::services::invoice::{country_name, Invoice};
use krilla::color::rgb;
use krilla::configure::{Configuration, Validator};
use krilla::embed::{AssociationKind, EmbeddedFile, MimeType};
use krilla::geom::{PathBuilder, Point, Rect, Size, Transform};
use krilla::image::Image;
use krilla::metadata::{DateTime, Metadata};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::Fill;
use krilla::text::{Font, TextDirection};
use krilla::{Document, SerializeSettings};
use std::sync::OnceLock;

const FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/NotoSans-Bold.ttf");

const PAGE_W: f32 = 595.28;
const PAGE_H: f32 = 841.89;
const LEFT: f32 = 50.0;
const RIGHT: f32 = PAGE_W - 50.0;
const FOOTER_TOP: f32 = PAGE_H - 70.0;

type Rgb = (u8, u8, u8);
const INK: Rgb = (17, 24, 39);
const MUTED: Rgb = (107, 114, 128);
const RULE: Rgb = (229, 231, 235);
const SHADE: Rgb = (243, 244, 246);
const ACCENT: Rgb = (234, 88, 12);

pub const FACTURX_FILE_NAME: &str = "factur-x.xml";

#[derive(Clone, Copy, PartialEq)]
enum Align {
    Left,
    Right,
}

enum Op {
    Text { x: f32, y: f32, size: f32, bold: bool, color: Rgb, text: String },
    Rect { x: f32, y: f32, w: f32, h: f32, color: Rgb },
    Logo { x: f32, y: f32, w: f32, h: f32 },
}

struct Faces {
    regular: ttf_parser::Face<'static>,
    bold: ttf_parser::Face<'static>,
}

fn faces() -> &'static Faces {
    static FACES: OnceLock<Faces> = OnceLock::new();
    FACES.get_or_init(|| Faces {
        regular: ttf_parser::Face::parse(FONT_REGULAR, 0).expect("bundled regular font"),
        bold: ttf_parser::Face::parse(FONT_BOLD, 0).expect("bundled bold font"),
    })
}

fn text_width(text: &str, size: f32, bold: bool) -> f32 {
    let face = if bold { &faces().bold } else { &faces().regular };
    let units: u32 = text
        .chars()
        .map(|c| face.glyph_index(c).and_then(|g| face.glyph_hor_advance(g)).unwrap_or(0) as u32)
        .sum();
    units as f32 * size / face.units_per_em() as f32
}

/// Greedy word wrap; words longer than the width are broken.
fn wrap(text: &str, width: f32, size: f32, bold: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() { word.to_string() } else { format!("{} {}", current, word) };
        if text_width(&candidate, size, bold) <= width {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        let mut piece = String::new();
        for ch in word.chars() {
            piece.push(ch);
            if text_width(&piece, size, bold) > width && piece.chars().count() > 1 {
                piece.pop();
                lines.push(std::mem::take(&mut piece));
                piece.push(ch);
            }
        }
        current = piece;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// German number format: 1.234,56 €
pub fn money(cents: i64, symbol: &str) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    let euros = (abs / 100).to_string();
    let mut grouped = String::new();
    for (i, ch) in euros.chars().enumerate() {
        if i > 0 && (euros.len() - i) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(ch);
    }
    format!("{}{},{:02} {}", sign, grouped, abs % 100, symbol)
}

fn percent(p: f64) -> String {
    let s = format!("{:.2}", p);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    format!("{} %", s.replace('.', ","))
}

fn date(d: chrono::NaiveDate) -> String {
    d.format("%d.%m.%Y").to_string()
}

struct Layout {
    pages: Vec<Vec<Op>>,
    y: f32,
}

impl Layout {
    fn ops(&mut self) -> &mut Vec<Op> {
        self.pages.last_mut().expect("page")
    }
    fn text(&mut self, x: f32, y: f32, size: f32, bold: bool, color: Rgb, text: &str, align: Align) {
        if text.trim().is_empty() {
            return;
        }
        let x = if align == Align::Right { x - text_width(text, size, bold) } else { x };
        self.ops().push(Op::Text { x, y, size, bold, color, text: text.to_string() });
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
        self.ops().push(Op::Rect { x, y, w, h, color });
    }
    fn rule(&mut self, y: f32) {
        self.rect(LEFT, y, RIGHT - LEFT, 0.6, RULE);
    }
    fn new_page(&mut self) {
        self.pages.push(Vec::new());
        self.y = 60.0;
    }
    /// Starts a new page when `height` no longer fits above the footer.
    fn ensure(&mut self, height: f32) -> bool {
        if self.y + height > FOOTER_TOP - 15.0 {
            self.new_page();
            return true;
        }
        false
    }
}

struct Columns {
    pos: f32,
    desc: f32,
    desc_width: f32,
    qty: f32,
    unit: f32,
    vat: Option<f32>,
    amount: f32,
}

fn columns(show_vat: bool) -> Columns {
    if show_vat {
        Columns { pos: LEFT, desc: LEFT + 26.0, desc_width: 238.0, qty: 360.0, unit: 428.0, vat: Some(470.0), amount: RIGHT }
    } else {
        Columns { pos: LEFT, desc: LEFT + 26.0, desc_width: 270.0, qty: 385.0, unit: 460.0, vat: None, amount: RIGHT }
    }
}

fn table_header(l: &mut Layout, inv: &Invoice, c: &Columns) {
    let y = l.y;
    l.rect(LEFT, y, RIGHT - LEFT, 20.0, SHADE);
    let ty = y + 13.5;
    l.text(c.pos + 4.0, ty, 8.0, true, INK, "Pos.", Align::Left);
    l.text(c.desc, ty, 8.0, true, INK, "Description", Align::Left);
    l.text(c.qty, ty, 8.0, true, INK, "Qty", Align::Right);
    l.text(c.unit, ty, 8.0, true, INK, if inv.shows_gross_prices() { "Unit price" } else { "Unit price (net)" }, Align::Right);
    if let Some(x) = c.vat {
        l.text(x, ty, 8.0, true, INK, "VAT", Align::Right);
    }
    l.text(c.amount - 4.0, ty, 8.0, true, INK, "Amount", Align::Right);
    l.y = y + 26.0;
}

fn layout(inv: &Invoice, logo: Option<&Logo>, shop_url: &str) -> Layout {
    let sym = inv.currency_symbol.as_str();
    let mut l = Layout { pages: vec![Vec::new()], y: 0.0 };
    let seller = &inv.seller;

    // Header: logo left, seller right
    let mut logo_bottom: f32 = 45.0;
    match logo {
        Some(lg) => {
            let max_h = 62.0_f32;
            let max_w = 170.0_f32;
            let scale = (max_h / lg.height as f32).min(max_w / lg.width as f32);
            let (w, h) = (lg.width as f32 * scale, lg.height as f32 * scale);
            l.ops().push(Op::Logo { x: LEFT, y: 42.0, w, h });
            logo_bottom = 42.0 + h;
        }
        None => {
            l.text(LEFT, 66.0, 18.0, true, INK, &seller.trade_name, Align::Left);
            logo_bottom = 70.0_f32.max(logo_bottom);
        }
    }
    let mut sy = 52.0;
    l.text(RIGHT, sy, 10.5, true, INK, &seller.party.name, Align::Right);
    sy += 14.0;
    let p = &seller.party;
    let zip_city = format!("{} {}", p.postal_code, p.city);
    let country = country_name(&p.country_code);
    for line in [p.street.as_str(), p.extra.as_str(), zip_city.trim(), country] {
        if !line.trim().is_empty() {
            l.text(RIGHT, sy, 8.5, false, MUTED, line, Align::Right);
            sy += 11.5;
        }
    }
    for line in [p.email.as_str(), p.phone.as_str()] {
        if !line.is_empty() {
            l.text(RIGHT, sy, 8.5, false, MUTED, line, Align::Right);
            sy += 11.5;
        }
    }

    // Address window (DIN 5008 position) with sender line
    let top = logo_bottom.max(sy).max(130.0) + 18.0;
    let sender = [p.name.as_str(), p.street.as_str(), zip_city.trim()]
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" · ");
    l.text(LEFT, top, 6.5, false, MUTED, &sender, Align::Left);
    l.rect(LEFT, top + 3.0, text_width(&sender, 6.5, false).min(250.0), 0.4, MUTED);
    let b = &inv.buyer;
    let mut by = top + 18.0;
    let buyer_zip_city = format!("{} {}", b.postal_code, b.city);
    l.text(LEFT, by, 10.0, true, INK, &b.name, Align::Left);
    by += 13.5;
    for line in [b.contact.as_str(), b.street.as_str(), b.extra.as_str(), buyer_zip_city.trim(), country_name(&b.country_code)] {
        if !line.trim().is_empty() {
            l.text(LEFT, by, 10.0, false, INK, line, Align::Left);
            by += 13.5;
        }
    }
    if !b.vat_id.is_empty() {
        l.text(LEFT, by, 9.0, false, MUTED, &format!("VAT ID: {}", b.vat_id), Align::Left);
        by += 13.0;
    }

    // Invoice details on the right
    let mut my = top + 18.0;
    let credit = inv.is_credit_note();
    let mut details: Vec<(&str, String)> = vec![
        (if credit { "Credit note number" } else { "Invoice number" }, inv.number.clone()),
        (if credit { "Credit note date" } else { "Invoice date" }, date(inv.issue_date)),
        (if inv.has_physical_items { "Delivery date" } else { "Date of service" }, date(inv.delivery_date)),
        ("Order number", inv.order_number.clone()),
        ("Customer email", b.email.clone()),
    ];
    if let Some(original) = &inv.referenced_invoice {
        details.insert(2, ("Cancels invoice", original.clone()));
    }
    for (label, value) in &details {
        if value.is_empty() {
            continue;
        }
        l.text(345.0, my, 8.5, false, MUTED, label, Align::Left);
        l.text(RIGHT, my, 8.5, true, INK, value, Align::Right);
        my += 13.0;
    }

    // Title
    l.y = by.max(my) + 34.0;
    l.text(LEFT, l.y, 20.0, true, INK, if credit { "Credit note" } else { "Invoice" }, Align::Left);
    l.rect(LEFT, l.y + 7.0, 34.0, 2.2, ACCENT);
    l.y += 26.0;
    let intro = match &inv.referenced_invoice {
        Some(original) => format!("This credit note cancels invoice {} in full. We credit you the following items:", original),
        None => "Thank you for your order. We invoice you for the following items:".to_string(),
    };
    l.text(LEFT, l.y, 9.5, false, INK, &intro, Align::Left);
    l.y += 16.0;

    // Items
    let c = columns(!inv.is_small_business());
    table_header(&mut l, inv, &c);
    for line in &inv.lines {
        let name_lines = wrap(&line.name, c.desc_width, 9.5, true);
        let mut detail = vec![format!("SKU {}", line.sku)];
        if let Some(v) = &line.variant {
            detail.push(v.clone());
        }
        if line.is_digital {
            detail.push("Digital product".to_string());
        }
        let detail_lines = wrap(&detail.join(" · "), c.desc_width, 7.5, false);
        let height = name_lines.len() as f32 * 12.5 + detail_lines.len() as f32 * 10.0 + 10.0;
        if l.ensure(height) {
            table_header(&mut l, inv, &c);
        }
        let row_top = l.y;
        let mut ty = row_top + 4.0;
        for (i, n) in name_lines.iter().enumerate() {
            ty += 12.5;
            if i == 0 {
                l.text(c.pos + 4.0, ty, 9.0, false, MUTED, &line.position.to_string(), Align::Left);
                l.text(c.qty, ty, 9.5, false, INK, &line.quantity.to_string(), Align::Right);
                l.text(c.unit, ty, 9.5, false, INK, &money(line.unit_display_cents, sym), Align::Right);
                if let Some(x) = c.vat {
                    l.text(x, ty, 9.0, false, MUTED, &percent(line.vat.percent), Align::Right);
                }
                l.text(c.amount - 4.0, ty, 9.5, true, INK, &money(line.line_display_cents, sym), Align::Right);
            }
            l.text(c.desc, ty, 9.5, true, INK, n, Align::Left);
        }
        for d in &detail_lines {
            ty += 10.0;
            l.text(c.desc, ty, 7.5, false, MUTED, d, Align::Left);
        }
        l.y = row_top + height;
        l.rule(l.y);
    }

    // Totals
    let mut rows: Vec<(String, String, bool)> = Vec::new();
    let items_display: i64 = inv.lines.iter().map(|x| x.line_display_cents).sum();
    rows.push((if inv.shows_gross_prices() { "Subtotal".into() } else { "Subtotal (net)".into() }, money(items_display, sym), false));
    if inv.discount_display_cents > 0 {
        let label = match &inv.discount_code {
            Some(code) => format!("Discount ({})", code),
            None => "Discount".to_string(),
        };
        rows.push((label, money(-inv.discount_display_cents, sym), false));
    }
    if inv.has_physical_items || inv.shipping_display_cents > 0 {
        let label = if inv.shows_gross_prices() { "Shipping" } else { "Shipping (net)" };
        rows.push((label.into(), money(inv.shipping_display_cents, sym), false));
    }
    if !inv.shows_gross_prices() {
        rows.push(("Total net".into(), money(inv.tax_basis_cents, sym), false));
        for g in &inv.groups {
            let label = match g.vat.category {
                "K" => "VAT 0 % (intra-Community supply)".to_string(),
                "AE" => "VAT 0 % (reverse charge)".to_string(),
                _ => format!("VAT {}", percent(g.vat.percent)),
            };
            rows.push((label, money(g.tax_cents, sym), false));
        }
        if inv.rounding_cents != 0 {
            rows.push(("Rounding".into(), money(inv.rounding_cents, sym), false));
        }
    }
    rows.push(("Total".into(), money(inv.payable_cents, sym), true));
    let mut vat_rows: Vec<String> = Vec::new();
    if inv.tax_mode == "included" && inv.shows_gross_prices() {
        for g in &inv.groups {
            vat_rows.push(format!(
                "Includes {} VAT {} on {} net",
                money(g.tax_cents, sym),
                percent(g.vat.percent),
                money(g.basis_cents, sym)
            ));
        }
    }
    let paid_line = if credit {
        format!("The amount of {} has been refunded via {}.", money(inv.payable_cents, sym), inv.payment_label)
    } else if inv.paid {
        format!("Paid on {} via {}. Amount due: {}", date(inv.issue_date), inv.payment_label, money(0, sym))
    } else {
        format!("Payment via {} is being processed.", inv.payment_label)
    };

    let totals_height = rows.len() as f32 * 16.0 + vat_rows.len() as f32 * 12.0 + 40.0;
    l.ensure(totals_height);
    l.y += 10.0;
    let label_x = 330.0;
    for (label, value, strong) in &rows {
        if *strong {
            l.rect(label_x, l.y + 7.0, RIGHT - label_x, 0.8, INK);
            l.y += 22.0;
            l.text(label_x, l.y, 11.5, true, INK, label, Align::Left);
            l.text(RIGHT - 4.0, l.y, 11.5, true, INK, value, Align::Right);
            l.y += 6.0;
        } else {
            l.y += 15.0;
            l.text(label_x, l.y, 9.0, false, MUTED, label, Align::Left);
            l.text(RIGHT - 4.0, l.y, 9.0, false, INK, value, Align::Right);
        }
    }
    for v in &vat_rows {
        l.y += 13.0;
        l.text(RIGHT - 4.0, l.y, 8.0, false, MUTED, v, Align::Right);
    }

    // Notes
    let mut notes: Vec<String> = Vec::new();
    notes.push(paid_line);
    notes.extend(inv.notes.iter().cloned());
    notes.push(if inv.has_physical_items {
        "The delivery date corresponds to the invoice date.".to_string()
    } else {
        "The date of service corresponds to the invoice date.".to_string()
    });
    l.y += 26.0;
    for note in notes {
        let lines = wrap(&note, RIGHT - LEFT, 8.5, false);
        l.ensure(lines.len() as f32 * 12.0 + 4.0);
        for line in lines {
            l.text(LEFT, l.y, 8.5, false, INK, &line, Align::Left);
            l.y += 12.0;
        }
        l.y += 3.0;
    }
    l.y += 8.0;
    l.ensure(16.0);
    let thanks = format!("Thank you for shopping at {}!", seller.trade_name);
    l.text(LEFT, l.y, 9.5, true, INK, &thanks, Align::Left);

    // Footer on every page
    let total_pages = l.pages.len();
    let mut col1 = vec![seller.party.name.clone()];
    if !seller.owner.is_empty() && seller.owner != seller.party.name {
        col1.push(format!("Owner: {}", seller.owner));
    }
    col1.push([p.street.as_str(), zip_city.trim()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(", "));
    let mut col2 = Vec::new();
    for v in [p.email.as_str(), p.phone.as_str(), shop_url] {
        if !v.is_empty() {
            col2.push(v.to_string());
        }
    }
    let mut col3 = Vec::new();
    if !seller.vat_id.is_empty() {
        col3.push(format!("VAT ID: {}", seller.vat_id));
    }
    if !seller.tax_number.is_empty() {
        col3.push(format!("Tax number: {}", seller.tax_number));
    }
    if !seller.register.is_empty() {
        col3.push(seller.register.clone());
    }
    for page in 0..total_pages {
        let mut f = Layout { pages: vec![Vec::new()], y: 0.0 };
        f.rule(FOOTER_TOP);
        for (x, col) in [(LEFT, &col1), (LEFT + 175.0, &col2), (LEFT + 340.0, &col3)] {
            let mut fy = FOOTER_TOP + 14.0;
            for line in col.iter().take(4) {
                let shown = wrap(line, 160.0, 7.0, false).into_iter().next().unwrap_or_default();
                f.text(x, fy, 7.0, false, MUTED, &shown, Align::Left);
                fy += 9.5;
            }
        }
        f.text(RIGHT, PAGE_H - 22.0, 7.0, false, MUTED, &format!("Page {} of {}", page + 1, total_pages), Align::Right);
        let ops = f.pages.pop().unwrap_or_default();
        l.pages[page].extend(ops);
    }
    l
}

fn pdf_date(dt: chrono::DateTime<chrono::Utc>) -> DateTime {
    use chrono::{Datelike, Timelike};
    DateTime::new(dt.year() as u16)
        .month(dt.month() as u8)
        .day(dt.day() as u8)
        .hour(dt.hour() as u8)
        .minute(dt.minute() as u8)
        .second(dt.second() as u8)
        .utc_offset_hour(0)
        .utc_offset_minute(0)
}

/// Renders the invoice. With `xml` the result is a ZUGFeRD / Factur-X (EN 16931) PDF/A-3b.
pub fn render(inv: &Invoice, logo: Option<&Logo>, shop_url: &str, xml: Option<&str>) -> Result<Vec<u8>, String> {
    let regular = Font::new(FONT_REGULAR.into(), 0).ok_or("bundled font could not be loaded")?;
    let bold = Font::new(FONT_BOLD.into(), 0).ok_or("bundled font could not be loaded")?;
    let logo_image = match logo {
        // PDF/A forbids image interpolation
        Some(lg) => Some(Image::from_png(lg.png.clone().into(), false).map_err(|e| format!("logo: {}", e))?),
        None => None,
    };

    let settings = SerializeSettings {
        configuration: Configuration::new_with_validator(if xml.is_some() { Validator::A3_B } else { Validator::None }),
        ..Default::default()
    };
    let mut doc = Document::new_with(settings);
    let now = chrono::Utc::now();
    doc.set_metadata(
        Metadata::new()
            .title(format!("Invoice {}", inv.number))
            .authors(vec![inv.seller.party.name.clone()])
            .language("en".to_string())
            .creator("rustwebshop".to_string())
            .creation_date(pdf_date(now)),
    );

    let pages = layout(inv, logo, shop_url).pages;
    for ops in pages {
        let mut page = doc.start_page_with(PageSettings::from_wh(PAGE_W, PAGE_H).ok_or("page size")?);
        let mut surface = page.surface();
        for op in ops {
            match op {
                Op::Text { x, y, size, bold: is_bold, color, text } => {
                    surface.set_fill(Some(fill(color)));
                    let font = if is_bold { bold.clone() } else { regular.clone() };
                    surface.draw_text(Point::from_xy(x, y), font, size, &text, false, TextDirection::Auto);
                }
                Op::Rect { x, y, w, h, color } => {
                    if let Some(rect) = Rect::from_xywh(x, y, w, h) {
                        let mut pb = PathBuilder::new();
                        pb.push_rect(rect);
                        if let Some(path) = pb.finish() {
                            surface.set_fill(Some(fill(color)));
                            surface.draw_path(&path);
                        }
                    }
                }
                Op::Logo { x, y, w, h } => {
                    if let (Some(img), Some(size)) = (logo_image.clone(), Size::from_wh(w, h)) {
                        surface.push_transform(&Transform::from_translate(x, y));
                        surface.draw_image(img, size);
                        surface.pop();
                    }
                }
            }
        }
        surface.finish();
        page.finish();
    }

    if let Some(xml) = xml {
        doc.embed_file(EmbeddedFile {
            path: FACTURX_FILE_NAME.to_string(),
            mime_type: MimeType::new("text/xml"),
            description: Some("Factur-X / ZUGFeRD invoice (EN 16931)".to_string()),
            association_kind: AssociationKind::Alternative,
            data: xml.as_bytes().to_vec().into(),
            modification_date: Some(pdf_date(now)),
            compress: Some(true),
            location: None,
        })
        .ok_or("the XML invoice could not be embedded")?;
    }

    let pdf = doc.finish().map_err(|e| format!("PDF export failed: {:?}", e))?;
    match xml {
        Some(_) => add_facturx_xmp(pdf),
        None => Ok(pdf),
    }
}

fn fill(color: Rgb) -> Fill {
    Fill { paint: rgb::Color::new(color.0, color.1, color.2).into(), opacity: NormalizedF32::ONE, rule: Default::default() }
}

/// Factur-X / ZUGFeRD readers find the embedded invoice through these XMP properties.
const FACTURX_PROPERTIES: &str = r#"<rdf:Description rdf:about="" xmlns:fx="urn:factur-x:pdfa:CrossIndustryDocument:invoice:1p0#">
<fx:DocumentType>INVOICE</fx:DocumentType>
<fx:DocumentFileName>factur-x.xml</fx:DocumentFileName>
<fx:Version>1.0</fx:Version>
<fx:ConformanceLevel>EN 16931</fx:ConformanceLevel>
</rdf:Description>
"#;

/// PDF/A extension schema declaring the `fx` properties (joins the schemas krilla already declares).
const FACTURX_SCHEMA: &str = r#"<rdf:li rdf:parseType="Resource">
<pdfaSchema:schema>Factur-X PDFA Extension Schema</pdfaSchema:schema>
<pdfaSchema:namespaceURI>urn:factur-x:pdfa:CrossIndustryDocument:invoice:1p0#</pdfaSchema:namespaceURI>
<pdfaSchema:prefix>fx</pdfaSchema:prefix>
<pdfaSchema:property>
<rdf:Seq>
<rdf:li rdf:parseType="Resource"><pdfaProperty:name>DocumentFileName</pdfaProperty:name><pdfaProperty:valueType>Text</pdfaProperty:valueType><pdfaProperty:category>external</pdfaProperty:category><pdfaProperty:description>The name of the embedded XML document</pdfaProperty:description></rdf:li>
<rdf:li rdf:parseType="Resource"><pdfaProperty:name>DocumentType</pdfaProperty:name><pdfaProperty:valueType>Text</pdfaProperty:valueType><pdfaProperty:category>external</pdfaProperty:category><pdfaProperty:description>The type of the hybrid document in capital letters, e.g. INVOICE or ORDER</pdfaProperty:description></rdf:li>
<rdf:li rdf:parseType="Resource"><pdfaProperty:name>Version</pdfaProperty:name><pdfaProperty:valueType>Text</pdfaProperty:valueType><pdfaProperty:category>external</pdfaProperty:category><pdfaProperty:description>The actual version of the standard applying to the embedded XML document</pdfaProperty:description></rdf:li>
<rdf:li rdf:parseType="Resource"><pdfaProperty:name>ConformanceLevel</pdfaProperty:name><pdfaProperty:valueType>Text</pdfaProperty:valueType><pdfaProperty:category>external</pdfaProperty:category><pdfaProperty:description>The conformance level of the embedded XML document</pdfaProperty:description></rdf:li>
</rdf:Seq>
</pdfaSchema:property>
</rdf:li>
"#;

fn add_facturx_xmp(pdf: Vec<u8>) -> Result<Vec<u8>, String> {
    use lopdf::{Document as LoDoc, Object};
    let mut doc = LoDoc::load_mem(&pdf).map_err(|e| format!("reading the PDF failed: {}", e))?;
    let catalog_id = doc.trailer.get(b"Root").and_then(Object::as_reference).map_err(|_| "PDF has no catalog")?;
    let metadata_id = doc
        .get_object(catalog_id)
        .and_then(Object::as_dict)
        .and_then(|d| d.get(b"Metadata"))
        .and_then(Object::as_reference)
        .map_err(|_| "PDF has no XMP metadata")?;
    let stream = doc.get_object_mut(metadata_id).and_then(Object::as_stream_mut).map_err(|_| "XMP metadata is not a stream")?;
    let content = stream.decompressed_content().unwrap_or_else(|_| stream.content.clone());
    let xmp = String::from_utf8(content).map_err(|_| "XMP metadata is not UTF-8")?;
    // A property may appear only once, so the schema goes into the existing extension schema list
    const SCHEMAS: &str = "<pdfaExtension:schemas><rdf:Bag>";
    let with_schema = match xmp.find(SCHEMAS) {
        Some(i) => format!("{}{}{}", &xmp[..i + SCHEMAS.len()], FACTURX_SCHEMA, &xmp[i + SCHEMAS.len()..]),
        None => {
            let at = xmp.rfind("</rdf:RDF>").ok_or("XMP metadata has no RDF block")?;
            let block = format!(
                "<rdf:Description rdf:about=\"\" xmlns:pdfaExtension=\"http://www.aiim.org/pdfa/ns/extension/\" xmlns:pdfaSchema=\"http://www.aiim.org/pdfa/ns/schema#\" xmlns:pdfaProperty=\"http://www.aiim.org/pdfa/ns/property#\"><pdfaExtension:schemas><rdf:Bag>{}</rdf:Bag></pdfaExtension:schemas></rdf:Description>",
                FACTURX_SCHEMA
            );
            format!("{}{}{}", &xmp[..at], block, &xmp[at..])
        }
    };
    let at = with_schema.rfind("</rdf:RDF>").ok_or("XMP metadata has no RDF block")?;
    let patched = format!("{}{}{}", &with_schema[..at], FACTURX_PROPERTIES, &with_schema[at..]);
    stream.dict.remove(b"Filter");
    stream.dict.remove(b"DecodeParms");
    stream.set_content(patched.into_bytes());
    stream.allows_compression = false;

    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| format!("writing the PDF failed: {}", e))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::invoice::tests::{item, sample_order, sample_settings};

    #[test]
    fn money_uses_german_format() {
        assert_eq!(money(123456, "€"), "1.234,56 €");
        assert_eq!(money(-5, "€"), "-0,05 €");
        assert_eq!(money(100000000, "€"), "1.000.000,00 €");
        assert_eq!(percent(19.0), "19 %");
        assert_eq!(percent(7.5), "7,5 %");
    }

    #[test]
    fn wrapping_respects_width() {
        let lines = wrap("A fairly long product title that certainly needs more than one line", 120.0, 9.5, true);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| text_width(l, 9.5, true) <= 120.0));
    }

    #[test]
    fn renders_plain_and_facturx_pdfs() {
        let items: Vec<_> = (0..40).map(|i| item(&format!("Product number {}", i), &format!("SKU-{}", i), 1990, 1, 19.0, i % 2 == 0)).collect();
        let total: i32 = items.iter().map(|i| i.total_price_cents).sum::<i32>() + 490;
        let inv = Invoice::build(&sample_order("kleingewerbe", total - 490, 490, total), &items, &sample_settings("kleingewerbe"));
        assert!(layout(&inv, None, "https://shop.example").pages.len() >= 2, "40 lines need a second page");

        let plain = render(&inv, None, "https://shop.example", None).unwrap();
        assert!(plain.starts_with(b"%PDF-"));

        let xml = inv.to_cii_xml();
        let hybrid = render(&inv, None, "https://shop.example", Some(&xml)).unwrap();
        let text = String::from_utf8_lossy(&hybrid);
        assert!(text.contains("factur-x.xml"));
        assert!(text.contains("<fx:ConformanceLevel>EN 16931</fx:ConformanceLevel>"));
        assert!(text.contains("pdfaid:part"));
        if let Ok(dir) = std::env::var("INVOICE_TEST_OUT") {
            std::fs::write(format!("{}/invoice-plain.pdf", dir), &plain).unwrap();
            std::fs::write(format!("{}/invoice-facturx.pdf", dir), &hybrid).unwrap();
            std::fs::write(format!("{}/factur-x.xml", dir), &xml).unwrap();
        }
    }

    /// Writes sample invoices for every tax mode to $INVOICE_TEST_OUT (for visual checks and validators).
    #[test]
    fn sample_invoices_for_inspection() {
        let Ok(dir) = std::env::var("INVOICE_TEST_OUT") else { return };
        let logo = std::env::var("INVOICE_TEST_LOGO").ok().map(|p| {
            let img = image::open(p).unwrap().thumbnail(400, 400).to_rgb8();
            let mut png = Vec::new();
            image::DynamicImage::ImageRgb8(img.clone()).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
            Logo { png, width: img.width(), height: img.height() }
        });
        for mode in ["kleingewerbe", "included", "excluded"] {
            let mut items = vec![
                item("Mechanical Keyboard Pro with a rather long product name for wrapping", "KB-PRO-RGB", 8990, 1, 19.0, false),
                item("USB-C Cable", "CBL-USBC-2M", 1290, 3, 19.0, false),
                item("Handbook (PDF)", "BOOK-PDF", 1500, 1, 7.0, true),
            ];
            items[0].variant_title = "Brown switches / DE layout".into();
            let mut settings = sample_settings(mode);
            if mode != "kleingewerbe" {
                settings.vat_id = "DE123456789".into();
            }
            let items_sum: i32 = items.iter().map(|i| i.total_price_cents).sum();
            let discount = 1000;
            let subtotal = items_sum - discount;
            let probe = Invoice::build(&sample_order(mode, subtotal, 490, 0), &items, &settings);
            let total = (probe.grand_total_cents) as i32;
            let mut order = sample_order(mode, subtotal, 490, total);
            order.coupon_code = Some("WELCOME10".into());
            let inv = Invoice::build(&order, &items, &settings);
            let xml = inv.to_cii_xml();
            let pdf = render(&inv, logo.as_ref(), "https://shop.example", Some(&xml)).unwrap();
            std::fs::write(format!("{}/invoice-{}.pdf", dir, mode), &pdf).unwrap();
            std::fs::write(format!("{}/invoice-{}.xml", dir, mode), &xml).unwrap();
        }
        // Gross prices as actually paid: VAT per rate leaves a rounding difference (BT-114)
        let items = vec![item("Gadget", "G-1", 1999, 3, 19.0, false), item("Book", "B-1", 1099, 1, 7.0, false)];
        let paid = 3 * 1999 + 1099 + 395;
        let inv = Invoice::build(&sample_order("included", 3 * 1999 + 1099, 395, paid), &items, &sample_settings("included"));
        assert_ne!(inv.rounding_cents, 0, "sample should exercise the rounding amount");
        let xml = inv.to_cii_xml();
        let pdf = render(&inv, logo.as_ref(), "https://shop.example", Some(&xml)).unwrap();
        std::fs::write(format!("{}/invoice-rounding.pdf", dir), &pdf).unwrap();

        // Credit note cancelling that invoice
        let cn = inv.credit_note(format!("{}-CN", inv.number), inv.issue_date);
        let xml = cn.to_cii_xml();
        let pdf = render(&cn, logo.as_ref(), "https://shop.example", Some(&xml)).unwrap();
        std::fs::write(format!("{}/credit-note.pdf", dir), &pdf).unwrap();

        // VAT-free EU business orders: goods (K) + digital (AE), with gross and net shop prices
        for mode in ["included", "excluded"] {
            let mut settings = sample_settings(mode);
            settings.vat_id = "DE123456789".into();
            let items = vec![item("Gadget", "G-1", 11900, 2, 19.0, false), item("E-Book", "E-1", 1070, 1, 7.0, true)];
            let mut order = sample_order(mode, 2 * 11900 + 1070, 1190, 0);
            order.reverse_charge = true;
            order.buyer_company = Some("Exemple SARL".into());
            order.buyer_vat_id = Some("FR12345678901".into());
            order.billing_address = serde_json::json!({ "full_name": "Jean Dupont", "street_address": "1 Rue de Rivoli", "postal_code": "75001", "city": "Paris", "country_code": "FR" });
            order.shipping_address = order.billing_address.clone();
            let probe = Invoice::build(&order, &items, &settings);
            order.total_cents = probe.grand_total_cents as i32;
            let inv = Invoice::build(&order, &items, &settings);
            assert_eq!(inv.tax_total_cents, 0);
            let xml = inv.to_cii_xml();
            assert!(xml.contains("<ram:CategoryCode>K</ram:CategoryCode>") && xml.contains("<ram:CategoryCode>AE</ram:CategoryCode>"));
            let pdf = render(&inv, logo.as_ref(), "https://shop.example", Some(&xml)).unwrap();
            std::fs::write(format!("{}/invoice-reverse-charge-{}.pdf", dir, mode), &pdf).unwrap();
        }
    }
}
