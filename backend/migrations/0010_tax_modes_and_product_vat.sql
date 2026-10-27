-- Migration 0010: Add tax mode to store settings and tax rate to products
ALTER TABLE store_settings 
ADD COLUMN IF NOT EXISTS tax_mode VARCHAR(30) NOT NULL DEFAULT 'kleingewerbe';

-- Add product-level tax rate percent (e.g., 19.00 for standard, 7.00 for reduced, 0.00 for exempt)
ALTER TABLE products 
ADD COLUMN IF NOT EXISTS tax_rate_percent DOUBLE PRECISION DEFAULT 19.00;

-- Update store settings default tax notice based on kleingewerbe
UPDATE store_settings
SET tax_mode = 'kleingewerbe',
    tax_notice = 'Gemäß § 19 UStG wird keine Umsatzsteuer berechnet.'
WHERE id = 1 AND (tax_mode IS NULL OR tax_mode = '');
