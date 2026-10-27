-- Migration 0011: Custom Order Number Format & Sequential Sequence (GoBD Compliant)

-- 1. Create atomic ascending sequence starting at 10000
CREATE SEQUENCE IF NOT EXISTS order_number_seq START WITH 10000 INCREMENT BY 1;

-- 2. Add order number customization columns to store_settings
ALTER TABLE store_settings
    ADD COLUMN IF NOT EXISTS order_prefix_enabled BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS order_prefix VARCHAR(10) NOT NULL DEFAULT 'ORD',
    ADD COLUMN IF NOT EXISTS order_date_enabled BOOLEAN NOT NULL DEFAULT true;
