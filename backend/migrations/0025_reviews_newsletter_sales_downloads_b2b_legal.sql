-- Migration 0025: optional product reviews, double opt-in newsletter, sale prices with the
-- lowest price of the last 30 days (§ 11 PAngV), protected digital downloads, product weights for
-- shipping, EU VAT ID check with reverse charge, withdrawal button, delivery times, product safety
-- information (GPSR), unit prices (§ 4 PAngV), stored invoices / credit notes (GoBD) and removal of
-- the discontinued EU ODR platform.

-- ---------------------------------------------------------------- feature switches & defaults
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS reviews_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS newsletter_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS sale_prices_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS reverse_charge_enabled BOOLEAN NOT NULL DEFAULT FALSE;
-- 0 = unlimited
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS download_limit INT NOT NULL DEFAULT 10;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS download_days INT NOT NULL DEFAULT 0;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS default_delivery_time VARCHAR(120) NOT NULL DEFAULT '2–4 business days';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS gpsr_default_manufacturer TEXT NOT NULL DEFAULT '';

-- ---------------------------------------------------------------- products
ALTER TABLE products ADD COLUMN IF NOT EXISTS weight_grams INT;
ALTER TABLE products ADD COLUMN IF NOT EXISTS delivery_time VARCHAR(120);
ALTER TABLE products ADD COLUMN IF NOT EXISTS sale_starts_at TIMESTAMPTZ;
ALTER TABLE products ADD COLUMN IF NOT EXISTS sale_ends_at TIMESTAMPTZ;
ALTER TABLE products ADD COLUMN IF NOT EXISTS gpsr_manufacturer TEXT;
ALTER TABLE products ADD COLUMN IF NOT EXISTS gpsr_responsible_person TEXT;
ALTER TABLE products ADD COLUMN IF NOT EXISTS gpsr_safety_info TEXT;
-- Net content for the unit price, e.g. 0.25 + 'kg' → price per kg
ALTER TABLE products ADD COLUMN IF NOT EXISTS unit_content DOUBLE PRECISION;
ALTER TABLE products ADD COLUMN IF NOT EXISTS unit_content_unit VARCHAR(10);

ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS weight_grams INT;
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS sale_price_cents INT;

-- Every price a variant was actually sold at (regular or sale), for the lowest price of the
-- 30 days before a price reduction (§ 11 PAngV). Recorded by triggers on every price change and by
-- the backend every few minutes so scheduled sale starts and ends are captured.
CREATE TABLE IF NOT EXISTS variant_price_history (
    id BIGSERIAL PRIMARY KEY,
    variant_id UUID NOT NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    price_cents INT NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_variant_price_history ON variant_price_history (variant_id, valid_from);

CREATE OR REPLACE FUNCTION record_variant_prices(p_product UUID) RETURNS void AS $$
    INSERT INTO variant_price_history (variant_id, price_cents)
    SELECT x.id, x.effective
    FROM (
        SELECT pv.id,
               CASE WHEN COALESCE((SELECT s.sale_prices_enabled FROM store_settings s WHERE s.id = 1), FALSE)
                     AND pv.sale_price_cents IS NOT NULL
                     AND pv.sale_price_cents < COALESCE(pv.price_override_cents, p.base_price_cents)
                     AND (p.sale_starts_at IS NULL OR p.sale_starts_at <= NOW())
                     AND (p.sale_ends_at IS NULL OR p.sale_ends_at > NOW())
                    THEN pv.sale_price_cents
                    ELSE COALESCE(pv.price_override_cents, p.base_price_cents)
               END AS effective
        FROM product_variants pv
        JOIN products p ON p.id = pv.product_id
        WHERE p_product IS NULL OR pv.product_id = p_product
    ) x
    WHERE x.effective IS DISTINCT FROM (
        SELECT h.price_cents FROM variant_price_history h
        WHERE h.variant_id = x.id
        ORDER BY h.valid_from DESC, h.id DESC
        LIMIT 1
    );
$$ LANGUAGE sql;

CREATE OR REPLACE FUNCTION trg_record_product_prices() RETURNS trigger AS $$
BEGIN
    PERFORM record_variant_prices(NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION trg_record_variant_prices() RETURNS trigger AS $$
BEGIN
    PERFORM record_variant_prices(NEW.product_id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS record_product_prices ON products;
CREATE TRIGGER record_product_prices AFTER UPDATE OF base_price_cents, sale_starts_at, sale_ends_at ON products
    FOR EACH ROW EXECUTE FUNCTION trg_record_product_prices();
DROP TRIGGER IF EXISTS record_variant_prices ON product_variants;
CREATE TRIGGER record_variant_prices AFTER INSERT OR UPDATE OF price_override_cents, sale_price_cents ON product_variants
    FOR EACH ROW EXECUTE FUNCTION trg_record_variant_prices();

SELECT record_variant_prices(NULL);

-- VAT ID checks (VIES), cached so checkout, payment and order always use the same answer
CREATE TABLE IF NOT EXISTS vat_checks (
    vat_id VARCHAR(30) PRIMARY KEY,
    valid BOOLEAN NOT NULL,
    name TEXT,
    address TEXT,
    request_identifier VARCHAR(100),
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ---------------------------------------------------------------- reviews (verified buyers only)
CREATE TABLE IF NOT EXISTS product_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    customer_email VARCHAR(255) NOT NULL,
    author_name VARCHAR(80) NOT NULL,
    rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    title VARCHAR(150) NOT NULL DEFAULT '',
    body TEXT NOT NULL DEFAULT '',
    -- pending | approved | rejected
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    moderated_at TIMESTAMPTZ,
    UNIQUE (product_id, customer_email)
);
CREATE INDEX IF NOT EXISTS idx_product_reviews_product ON product_reviews (product_id, status);

-- ---------------------------------------------------------------- newsletter (double opt-in)
CREATE TABLE IF NOT EXISTS newsletter_subscribers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    -- pending | confirmed | unsubscribed
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    confirm_token_hash VARCHAR(64),
    unsubscribe_token VARCHAR(64) NOT NULL,
    consent_text TEXT NOT NULL,
    source VARCHAR(30) NOT NULL DEFAULT 'footer',
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    requested_ip VARCHAR(64),
    confirmed_at TIMESTAMPTZ,
    confirmed_ip VARCHAR(64),
    unsubscribed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS newsletter_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject VARCHAR(200) NOT NULL,
    body_markdown TEXT NOT NULL,
    -- draft | sending | sent
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    recipients INT NOT NULL DEFAULT 0,
    sent_count INT NOT NULL DEFAULT 0,
    failed_count INT NOT NULL DEFAULT 0,
    created_by VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at TIMESTAMPTZ
);

-- ---------------------------------------------------------------- protected digital downloads
CREATE TABLE IF NOT EXISTS download_grants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID,
    file_url TEXT NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    token VARCHAR(64) NOT NULL UNIQUE,
    download_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_download_at TIMESTAMPTZ,
    UNIQUE (order_id, file_url)
);

-- ---------------------------------------------------------------- B2B / reverse charge
ALTER TABLE orders ADD COLUMN IF NOT EXISTS buyer_company VARCHAR(255);
ALTER TABLE orders ADD COLUMN IF NOT EXISTS buyer_vat_id VARCHAR(30);
ALTER TABLE orders ADD COLUMN IF NOT EXISTS reverse_charge BOOLEAN NOT NULL DEFAULT FALSE;
-- VIES answer kept as proof (valid, name, address, request date, consultation number)
ALTER TABLE orders ADD COLUMN IF NOT EXISTS vat_check JSONB;
ALTER TABLE orders ADD COLUMN IF NOT EXISTS paid_at TIMESTAMPTZ;
UPDATE orders SET paid_at = created_at WHERE paid_at IS NULL AND payment_status IN ('paid', 'refunded');

-- ---------------------------------------------------------------- withdrawal button (Art. 11a Directive 2011/83/EU, since 19 June 2026)
CREATE TABLE IF NOT EXISTS withdrawal_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    order_number VARCHAR(100) NOT NULL,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255) NOT NULL,
    items TEXT NOT NULL DEFAULT '',
    message TEXT NOT NULL DEFAULT '',
    -- received | processed
    status VARCHAR(20) NOT NULL DEFAULT 'received',
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmation_sent_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    admin_note TEXT NOT NULL DEFAULT ''
);

-- ---------------------------------------------------------------- stored invoices (GoBD)
CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    -- invoice | credit_note
    kind VARCHAR(20) NOT NULL,
    number VARCHAR(120) NOT NULL UNIQUE,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_cents BIGINT NOT NULL,
    pdf BYTEA NOT NULL,
    xml TEXT,
    UNIQUE (order_id, kind)
);

-- ---------------------------------------------------------------- legal texts
-- The EU ODR platform was shut down on 20 July 2025; the link must no longer be shown.
UPDATE pages
SET content_markdown = regexp_replace(
        content_markdown,
        E'### Online Dispute Resolution:\\s*\\nThe European Commission provides a platform for out-of-court resolution of disputes \\(ODR platform\\): \\[\\{\\{ODR_URL\\}\\}\\]\\(\\{\\{ODR_URL\\}\\}\\)\\.\\s*\\n',
        E'### Consumer Dispute Resolution:\n'),
    updated_at = NOW()
WHERE slug = 'legal-notice' AND content_markdown LIKE '%{{ODR_URL}}%';

UPDATE store_settings
SET odr_url = '',
    dispute_resolution_notice = 'We are neither willing nor obliged to participate in dispute resolution proceedings before a consumer arbitration board (§ 36 VSBG).'
WHERE dispute_resolution_notice LIKE '%ec.europa.eu/odr%' OR dispute_resolution_notice LIKE '%ODR platform%';
UPDATE store_settings SET odr_url = '' WHERE odr_url LIKE '%ec.europa.eu/odr%';

-- Privacy policy: sections for the features that are switched on are filled in automatically
UPDATE pages
SET content_markdown = content_markdown || E'\n\n{{FEATURE_PRIVACY_SECTIONS}}\n',
    updated_at = NOW()
WHERE slug = 'privacy-policy' AND content_markdown NOT LIKE '%{{FEATURE_PRIVACY_SECTIONS}}%';
