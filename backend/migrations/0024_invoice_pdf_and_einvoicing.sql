-- Migration 0024: PDF invoices attached to payment emails and optional EN 16931 e-invoices (ZUGFeRD / Factur-X).
-- An invoice needs the VAT rate of every line and the tax mode at the time of purchase, so both are
-- now stored with the order instead of being read from the (changeable) product and shop settings.

ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS tax_number VARCHAR(50) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS email_invoice_pdf BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS einvoice_enabled BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE order_items ADD COLUMN IF NOT EXISTS tax_rate_percent DOUBLE PRECISION;
ALTER TABLE orders ADD COLUMN IF NOT EXISTS tax_mode VARCHAR(20);

-- Existing orders: best available values (product rate, otherwise the shop rate; the current tax mode)
UPDATE order_items oi SET tax_rate_percent = p.tax_rate_percent
FROM products p
WHERE p.id = oi.product_id AND oi.tax_rate_percent IS NULL AND p.tax_rate_percent IS NOT NULL;
UPDATE order_items SET tax_rate_percent = (SELECT tax_rate_percent FROM store_settings WHERE id = 1) WHERE tax_rate_percent IS NULL;
UPDATE orders SET tax_mode = (SELECT COALESCE(tax_mode, 'kleingewerbe') FROM store_settings WHERE id = 1) WHERE tax_mode IS NULL;
