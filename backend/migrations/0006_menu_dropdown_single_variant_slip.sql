-- Migration 0006: Menu Dropdowns, Single/Multi Variant Toggle, Order Items FK on delete, and Slip formatting

-- 1. Add parent_id to navigation_items for hierarchical dropdown menus
ALTER TABLE navigation_items ADD COLUMN IF NOT EXISTS parent_id UUID REFERENCES navigation_items(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_navigation_items_parent_id ON navigation_items(parent_id);

-- 2. Make order_items foreign keys nullable and ON DELETE SET NULL so products can be deleted without FK errors
ALTER TABLE order_items ALTER COLUMN product_id DROP NOT NULL;
ALTER TABLE order_items ALTER COLUMN variant_id DROP NOT NULL;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'order_items_product_id_fkey') THEN
        ALTER TABLE order_items DROP CONSTRAINT order_items_product_id_fkey;
    END IF;
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'order_items_variant_id_fkey') THEN
        ALTER TABLE order_items DROP CONSTRAINT order_items_variant_id_fkey;
    END IF;
END $$;

ALTER TABLE order_items ADD CONSTRAINT order_items_product_id_fkey FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE SET NULL;
ALTER TABLE order_items ADD CONSTRAINT order_items_variant_id_fkey FOREIGN KEY (variant_id) REFERENCES product_variants(id) ON DELETE SET NULL;

-- 3. Add has_multiple_variants flag to products (defaults to false for single-version simplicity)
ALTER TABLE products ADD COLUMN IF NOT EXISTS has_multiple_variants BOOLEAN NOT NULL DEFAULT false;

-- Update existing products with more than 1 variant to true
UPDATE products p
SET has_multiple_variants = true
WHERE (SELECT COUNT(*) FROM product_variants pv WHERE pv.product_id = p.id) > 1;

-- 4. Rename 'Featured Gear' to 'Featured Products' in carousels_config
UPDATE store_settings
SET carousels_config = regexp_replace(carousels_config::text, 'Featured Gear', 'Featured Products', 'g')::jsonb
WHERE id = 1 AND carousels_config IS NOT NULL;
