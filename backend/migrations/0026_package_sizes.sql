-- Migration 0026: package sizes. Products/variants get dimensions, shipping rates a maximum package
-- size, so checkout only offers packages the cart fits into (millimetres; empty/0 = no limit).

ALTER TABLE products ADD COLUMN IF NOT EXISTS length_mm INT;
ALTER TABLE products ADD COLUMN IF NOT EXISTS width_mm INT;
ALTER TABLE products ADD COLUMN IF NOT EXISTS height_mm INT;

ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS length_mm INT;
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS width_mm INT;
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS height_mm INT;

ALTER TABLE shipping_rates ADD COLUMN IF NOT EXISTS max_length_mm INT NOT NULL DEFAULT 0;
ALTER TABLE shipping_rates ADD COLUMN IF NOT EXISTS max_width_mm INT NOT NULL DEFAULT 0;
ALTER TABLE shipping_rates ADD COLUMN IF NOT EXISTS max_height_mm INT NOT NULL DEFAULT 0;
