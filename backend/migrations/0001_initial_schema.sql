-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 1. Store Settings
CREATE TABLE IF NOT EXISTS store_settings (
    id INTEGER PRIMARY KEY DEFAULT 1,
    store_name VARCHAR(255) NOT NULL DEFAULT 'Rust E-Commerce Store',
    currency VARCHAR(10) NOT NULL DEFAULT 'EUR',
    currency_symbol VARCHAR(5) NOT NULL DEFAULT '€',
    tax_rate_percent DOUBLE PRECISION NOT NULL DEFAULT 19.00,
    deployment_mode VARCHAR(50) NOT NULL DEFAULT 'development',
    debug_mode BOOLEAN NOT NULL DEFAULT true,
    support_email VARCHAR(255) NOT NULL DEFAULT 'support@rustwebshop.local',
    company_address TEXT NOT NULL DEFAULT 'Rustacean Way 42, 10115 Berlin, Germany',
    vat_id VARCHAR(50) NOT NULL DEFAULT 'DE314159265',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT single_store_settings CHECK (id = 1)
);

-- 2. Users Table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'admin',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Products Table
CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    product_type VARCHAR(50) NOT NULL DEFAULT 'physical', -- 'physical' | 'digital'
    category VARCHAR(100) NOT NULL,
    subcategory VARCHAR(100) NOT NULL DEFAULT '',
    base_price_cents INTEGER NOT NULL,
    digital_download_url TEXT,
    image_url TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);
CREATE INDEX IF NOT EXISTS idx_products_slug ON products(slug);

-- 4. Product Variants Table (With SKU, JSONB attributes, and GIN Index)
CREATE TABLE IF NOT EXISTS product_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sku VARCHAR(100) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    price_override_cents INTEGER,
    attributes JSONB NOT NULL DEFAULT '{}',
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    low_stock_threshold INTEGER NOT NULL DEFAULT 5,
    image_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_variants_product_id ON product_variants(product_id);
CREATE INDEX IF NOT EXISTS idx_variants_sku ON product_variants(sku);
CREATE INDEX IF NOT EXISTS idx_variants_attributes_gin ON product_variants USING gin (attributes);

-- 5. Shipping Zones & Rates Table
CREATE TABLE IF NOT EXISTS shipping_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_name VARCHAR(100) NOT NULL,
    country_codes JSONB NOT NULL DEFAULT '[]',
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS shipping_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id UUID NOT NULL REFERENCES shipping_zones(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    package_type VARCHAR(50) NOT NULL DEFAULT 'standard', -- standard, express, fragile, heavy
    min_weight_g INTEGER NOT NULL DEFAULT 0,
    max_weight_g INTEGER NOT NULL DEFAULT 5000,
    price_cents INTEGER NOT NULL,
    estimated_delivery_days VARCHAR(50) NOT NULL DEFAULT '2-4 business days'
);

-- 6. Payment Configurations Table
CREATE TABLE IF NOT EXISTS payment_configs (
    provider VARCHAR(50) PRIMARY KEY, -- 'stripe', 'paypal', 'apple_pay', 'google_pay', 'amazon_pay'
    display_name VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    is_sandbox BOOLEAN NOT NULL DEFAULT true,
    public_client_id TEXT NOT NULL DEFAULT '',
    secret_key TEXT NOT NULL DEFAULT '',
    config_data JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 7. Orders Table
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_number VARCHAR(50) NOT NULL UNIQUE,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255) NOT NULL,
    shipping_address JSONB NOT NULL,
    billing_address JSONB NOT NULL,
    shipping_rate_id UUID REFERENCES shipping_rates(id),
    shipping_cost_cents INTEGER NOT NULL DEFAULT 0,
    subtotal_cents INTEGER NOT NULL,
    tax_cents INTEGER NOT NULL DEFAULT 0,
    total_cents INTEGER NOT NULL,
    payment_provider VARCHAR(50) NOT NULL,
    payment_status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, paid, failed, refunded
    order_status VARCHAR(50) NOT NULL DEFAULT 'processing', -- processing, shipped, delivered, cancelled
    tracking_number VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_order_number ON orders(order_number);
CREATE INDEX IF NOT EXISTS idx_orders_customer_email ON orders(customer_email);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(order_status);

-- 8. Order Items Table
CREATE TABLE IF NOT EXISTS order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    variant_id UUID NOT NULL REFERENCES product_variants(id),
    product_title VARCHAR(255) NOT NULL,
    variant_title VARCHAR(255) NOT NULL,
    sku VARCHAR(100) NOT NULL,
    unit_price_cents INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    total_price_cents INTEGER NOT NULL,
    is_digital BOOLEAN NOT NULL DEFAULT false,
    download_url TEXT
);

CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);

-- SEED DATA --

-- Initial Store Settings
INSERT INTO store_settings (id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode)
VALUES (1, 'RustCraft Gear & Software', 'EUR', '€', 19.00, 'development', true)
ON CONFLICT (id) DO NOTHING;

-- Initial Admin Account: username "admin", password "admin123" (bcrypt hash)
INSERT INTO users (username, email, password_hash, role)
VALUES ('admin', 'admin@rustwebshop.local', '$2b$12$Kk5zG5m9c4E6H/vIu8B2l.iV4X8vj6gEa7I6oR5P6K9O1Y0W1a2b3', 'admin')
ON CONFLICT (username) DO NOTHING;

-- Initial Payment Configurations
INSERT INTO payment_configs (provider, display_name, is_enabled, is_sandbox, public_client_id, secret_key, config_data)
VALUES 
('stripe', 'Stripe Credit & Debit Cards', true, true, 'pk_test_sample_stripe_key_rustwebshop', 'sk_test_sample_stripe_secret', '{"collect_billing_address": true}'),
('paypal', 'PayPal Express Checkout', true, true, 'sb_client_id_rustwebshop_paypal', 'sb_secret_rustwebshop_paypal', '{"allow_pay_later": true}'),
('apple_pay', 'Apple Pay', true, true, 'merchant.com.rustwebshop.apple', '', '{"merchant_name": "RustCraft Store"}'),
('google_pay', 'Google Pay', true, true, 'gpay_merchant_id_rustwebshop', '', '{"allowed_auth_methods": ["PAN_ONLY", "CRYPTOGRAM_3DS"]}'),
('amazon_pay', 'Amazon Pay', true, true, 'amzn_merchant_rustwebshop', '', '{"sandbox_mode": true}')
ON CONFLICT (provider) DO NOTHING;

-- Initial Shipping Zones & Rates
INSERT INTO shipping_zones (id, zone_name, country_codes, is_default)
VALUES 
('11111111-1111-1111-1111-111111111111', 'Germany (Domestic)', '["DE"]', true),
('22222222-2222-2222-2222-222222222222', 'European Union (Zone 1)', '["FR", "NL", "BE", "AT", "IT", "ES", "PL", "DK", "SE"]', false),
('33333333-3333-3333-3333-333333333333', 'International (Rest of World)', '["US", "CA", "GB", "CH", "JP", "AU"]', false)
ON CONFLICT DO NOTHING;

INSERT INTO shipping_rates (id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days)
VALUES
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', 'DHL Standard Paket', 'standard', 0, 5000, 499, '1-2 business days'),
('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '11111111-1111-1111-1111-111111111111', 'DHL Express 24h', 'express', 0, 5000, 1199, 'Next day guaranteed'),
('cccccccc-cccc-cccc-cccc-cccccccccccc', '22222222-2222-2222-2222-222222222222', 'EU Standard Courier', 'standard', 0, 5000, 999, '3-5 business days'),
('dddddddd-dddd-dddd-dddd-dddddddddddd', '22222222-2222-2222-2222-222222222222', 'EU Express Priority', 'express', 0, 5000, 1899, '1-2 business days'),
('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', '33333333-3333-3333-3333-333333333333', 'Global Airmail Tracked', 'standard', 0, 5000, 2499, '5-10 business days')
ON CONFLICT DO NOTHING;

-- Initial Products (Physical & Digital) - only seeded if products table is completely empty
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM products) THEN
        INSERT INTO products (id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active)
        VALUES
        ('a1000000-0000-0000-0000-000000000001', 'Rust Mechanical Keyboard (Ferris Edition)', 'rust-mechanical-keyboard', 'High-end hot-swappable mechanical keyboard engineered with CNC aluminum chassis, customized Rust orange keycaps, and ultra-low latency response.', 'physical', 'Hardware', 'Keyboards', 14999, NULL, 'https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=800&q=80', true),
        ('a2000000-0000-0000-0000-000000000002', 'Rustacean Heavyweight Organic Hoodie', 'rustacean-heavyweight-hoodie', 'Ultra-comfortable 450 GSM organic cotton hoodie with embroidered Ferris the Crab mascot and minimal Rust code sleeve accents.', 'physical', 'Apparel', 'Hoodies', 7999, NULL, 'https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=800&q=80', true),
        ('a3000000-0000-0000-0000-000000000003', 'SpeedDesk Extended Precision Mousepad', 'speeddesk-extended-mousepad', 'Micro-textured low-friction cloth pad with stitched anti-fray borders and water-resistant nano coating.', 'physical', 'Accessories', 'Desk Mats', 2999, NULL, 'https://images.unsplash.com/photo-1616440347437-b1c73416efc2?auto=format&fit=crop&w=800&q=80', true),
        ('a4000000-0000-0000-0000-000000000004', 'Mastering High-Concurrency Rust Architecture', 'mastering-high-concurrency-rust-guide', 'Complete 400-page interactive eBook, architecture blueprints, and 12 production-ready Tokio & Axum microservices project templates.', 'digital', 'Software & Books', 'eBooks', 4999, 'https://cdn.rustwebshop.local/downloads/mastering-rust-concurrency-v2.pdf', 'https://images.unsplash.com/photo-1532012164546-f432f2e3edd4?auto=format&fit=crop&w=800&q=80', true)
        ON CONFLICT DO NOTHING;

        -- Variants for Keyboard
        INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold)
        VALUES
        ('b1000000-0000-0000-0000-000000000001', 'a1000000-0000-0000-0000-000000000001', 'KB-RUST-TACT-BLK', 'Space Black / Tactile Brown Switches', 14999, '{"color": "Space Black", "switch": "Tactile Brown", "layout": "ANSI 75%"}', 38, 5),
        ('b1000000-0000-0000-0000-000000000002', 'a1000000-0000-0000-0000-000000000001', 'KB-RUST-LINR-BLK', 'Space Black / Linear Red Switches', 14999, '{"color": "Space Black", "switch": "Linear Red", "layout": "ANSI 75%"}', 4, 5),
        ('b1000000-0000-0000-0000-000000000003', 'a1000000-0000-0000-0000-000000000001', 'KB-RUST-CLK-ORG', 'Ferris Orange / Clicky Blue Switches', 15999, '{"color": "Ferris Orange", "switch": "Clicky Blue", "layout": "ANSI 75%"}', 19, 5)
        ON CONFLICT DO NOTHING;

        -- Variants for Hoodie
        INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold)
        VALUES
        ('b2000000-0000-0000-0000-000000000001', 'a2000000-0000-0000-0000-000000000002', 'HD-RUST-CHRC-M', 'Charcoal Heather / Medium', 7999, '{"color": "Charcoal Heather", "size": "M"}', 25, 5),
        ('b2000000-0000-0000-0000-000000000002', 'a2000000-0000-0000-0000-000000000002', 'HD-RUST-CHRC-L', 'Charcoal Heather / Large', 7999, '{"color": "Charcoal Heather", "size": "L"}', 42, 5),
        ('b2000000-0000-0000-0000-000000000003', 'a2000000-0000-0000-0000-000000000002', 'HD-RUST-CHRC-XL', 'Charcoal Heather / Extra Large', 7999, '{"color": "Charcoal Heather", "size": "XL"}', 15, 5),
        ('b2000000-0000-0000-0000-000000000004', 'a2000000-0000-0000-0000-000000000002', 'HD-RUST-NVY-L', 'Deep Navy / Large', 7999, '{"color": "Deep Navy", "size": "L"}', 2, 5)
        ON CONFLICT DO NOTHING;

        -- Variants for Deskmat
        INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold)
        VALUES
        ('b3000000-0000-0000-0000-000000000001', 'a3000000-0000-0000-0000-000000000003', 'MP-STEALTH-900', 'Stealth Dark 900x400mm', 2999, '{"color": "Stealth Dark", "size": "900x400mm"}', 85, 10),
        ('b3000000-0000-0000-0000-000000000002', 'a3000000-0000-0000-0000-000000000003', 'MP-TOPOGRAPHY-900', 'Topography Edition 900x400mm', 3499, '{"color": "Topography White/Orange", "size": "900x400mm"}', 3, 10)
        ON CONFLICT DO NOTHING;

        -- Variant for Digital Product
        INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold)
        VALUES
        ('b4000000-0000-0000-0000-000000000001', 'a4000000-0000-0000-0000-000000000004', 'DIG-RUST-ARCH-EBOOK', 'Standard Digital License (PDF + ePub + Code)', 4999, '{"format": "PDF / ePub / GitHub Repository Access"}', 999999, 10)
        ON CONFLICT DO NOTHING;
    END IF;
END $$;
