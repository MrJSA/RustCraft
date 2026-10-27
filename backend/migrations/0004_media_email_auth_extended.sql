-- Migration 0004: Media Library, Email Addon, Stock Notifications, Admin Authentication, Navigation Locations, and Storefront Carousels

-- 1. Admin Users Table
CREATE TABLE IF NOT EXISTS admin_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Media Library Table
CREATE TABLE IF NOT EXISTS media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    url VARCHAR(500) NOT NULL,
    mime_type VARCHAR(100) NOT NULL DEFAULT 'image/jpeg',
    size_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_media_created_at ON media(created_at DESC);

-- 3. Stock Notifications (Back-in-stock waitlist)
CREATE TABLE IF NOT EXISTS stock_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES product_variants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_stock_notification UNIQUE(product_id, variant_id, email)
);

CREATE INDEX IF NOT EXISTS idx_stock_notifications_prod ON stock_notifications(product_id);

-- 4. Store Settings Extensions
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS smtp_host VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS smtp_port INTEGER NOT NULL DEFAULT 587;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS smtp_username VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS smtp_password VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS smtp_encryption VARCHAR(50) NOT NULL DEFAULT 'starttls';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS smtp_from_email VARCHAR(255) NOT NULL DEFAULT 'shop@rustcraft.local';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS smtp_from_name VARCHAR(255) NOT NULL DEFAULT 'RustCraft Shop';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS smtp_enabled BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS require_registered_checkout BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS require_email_verification BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS store_subtitle VARCHAR(255) NOT NULL DEFAULT 'Rust-Powered ACID E-Commerce';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS show_store_title BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS show_store_subtitle BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS carousels_config JSONB NOT NULL DEFAULT '{
  "order": ["featured", "new", "bestsellers", "catalog"],
  "enabled": {
    "featured": true,
    "new": true,
    "bestsellers": true,
    "catalog": true
  },
  "new_products_days": 14,
  "featured_product_ids": [
    "a1000000-0000-0000-0000-000000000001",
    "a1000000-0000-0000-0000-000000000002",
    "a1000000-0000-0000-0000-000000000003",
    "a1000000-0000-0000-0000-000000000004"
  ]
}';

-- 5. Customer Email Verification
ALTER TABLE customers ADD COLUMN IF NOT EXISTS is_verified BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE customers ADD COLUMN IF NOT EXISTS verification_token VARCHAR(255);

-- 6. Navigation Items Location ('header' or 'footer')
ALTER TABLE navigation_items ADD COLUMN IF NOT EXISTS location VARCHAR(20) NOT NULL DEFAULT 'header';
CREATE INDEX IF NOT EXISTS idx_navigation_items_location ON navigation_items(location);
