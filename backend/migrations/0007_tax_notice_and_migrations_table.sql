-- Migration 0007: Tax notice in store settings and Carousel 'All Products' normalization
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS tax_notice TEXT NOT NULL DEFAULT 'Gemäß § 19 UStG wird keine Umsatzsteuer berechnet.';

UPDATE store_settings
SET carousels_config = regexp_replace(regexp_replace(carousels_config::text, 'In Stock Hardware & Gear', 'All Products', 'g'), 'Public Key / Merchant Client ID', 'All Products', 'g')::jsonb
WHERE id = 1 AND carousels_config IS NOT NULL;

