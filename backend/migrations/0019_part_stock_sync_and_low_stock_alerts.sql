-- Migration 0019: Part Stock Synchronization & Low Stock Email Alerts
-- 1. Synchronize any existing stock discrepancies across all parts sharing the same SKU
UPDATE product_parts pp
SET stock_quantity = bp.stock_quantity
FROM bom_parts bp
WHERE (pp.part_id = bp.id OR (pp.part_sku IS NOT NULL AND pp.part_sku = bp.sku))
  AND pp.stock_quantity IS DISTINCT FROM bp.stock_quantity;

-- 2. Trigger on bom_parts to sync stock_quantity to product_parts
CREATE OR REPLACE FUNCTION sync_bom_parts_stock_to_product_parts()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.stock_quantity IS DISTINCT FROM OLD.stock_quantity THEN
        UPDATE product_parts
        SET stock_quantity = NEW.stock_quantity
        WHERE (part_id = NEW.id OR (part_sku IS NOT NULL AND part_sku = NEW.sku))
          AND stock_quantity IS DISTINCT FROM NEW.stock_quantity;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_sync_bom_parts_stock ON bom_parts;
CREATE TRIGGER trg_sync_bom_parts_stock
AFTER UPDATE OF stock_quantity ON bom_parts
FOR EACH ROW
EXECUTE FUNCTION sync_bom_parts_stock_to_product_parts();

-- 3. Trigger on product_parts to sync stock_quantity to bom_parts and sister product_parts
CREATE OR REPLACE FUNCTION sync_product_parts_stock_to_bom_parts()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.stock_quantity IS DISTINCT FROM OLD.stock_quantity THEN
        IF NEW.part_sku IS NOT NULL AND NEW.part_sku <> 'DIGITAL_FILE' AND NEW.part_sku <> '' THEN
            UPDATE bom_parts
            SET stock_quantity = NEW.stock_quantity
            WHERE (id = NEW.part_id OR sku = NEW.part_sku)
              AND stock_quantity IS DISTINCT FROM NEW.stock_quantity;

            UPDATE product_parts
            SET stock_quantity = NEW.stock_quantity
            WHERE part_sku = NEW.part_sku
              AND id <> NEW.id
              AND stock_quantity IS DISTINCT FROM NEW.stock_quantity;
        ELSIF NEW.part_id IS NOT NULL THEN
            UPDATE bom_parts
            SET stock_quantity = NEW.stock_quantity
            WHERE id = NEW.part_id
              AND stock_quantity IS DISTINCT FROM NEW.stock_quantity;

            UPDATE product_parts
            SET stock_quantity = NEW.stock_quantity
            WHERE part_id = NEW.part_id
              AND id <> NEW.id
              AND stock_quantity IS DISTINCT FROM NEW.stock_quantity;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_sync_product_parts_stock ON product_parts;
CREATE TRIGGER trg_sync_product_parts_stock
AFTER UPDATE OF stock_quantity ON product_parts
FOR EACH ROW
EXECUTE FUNCTION sync_product_parts_stock_to_bom_parts();

-- 4. Initial stock inheritance on product_parts INSERT
CREATE OR REPLACE FUNCTION set_initial_product_part_stock_from_sku()
RETURNS TRIGGER AS $$
DECLARE
    matched_stock INTEGER;
BEGIN
    IF NEW.part_sku IS NOT NULL AND NEW.part_sku <> 'DIGITAL_FILE' AND NEW.part_sku <> '' THEN
        SELECT stock_quantity INTO matched_stock FROM bom_parts WHERE sku = NEW.part_sku OR id = NEW.part_id LIMIT 1;
        IF matched_stock IS NOT NULL THEN
            NEW.stock_quantity := matched_stock;
        ELSE
            SELECT stock_quantity INTO matched_stock FROM product_parts WHERE part_sku = NEW.part_sku LIMIT 1;
            IF matched_stock IS NOT NULL THEN
                NEW.stock_quantity := matched_stock;
            END IF;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_set_initial_product_part_stock ON product_parts;
CREATE TRIGGER trg_set_initial_product_part_stock
BEFORE INSERT ON product_parts
FOR EACH ROW
EXECUTE FUNCTION set_initial_product_part_stock_from_sku();

-- 5. Alert tracking columns on bom_parts
ALTER TABLE bom_parts ADD COLUMN IF NOT EXISTS last_low_stock_alert_at TIMESTAMPTZ;
ALTER TABLE bom_parts ADD COLUMN IF NOT EXISTS last_alert_stock_quantity INTEGER;

-- 6. Low stock alert configuration on store_settings
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS low_stock_alerts_enabled BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS low_stock_alert_recipients_mode VARCHAR(50) NOT NULL DEFAULT 'stock_managers';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS low_stock_alert_custom_emails TEXT NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS low_stock_alert_selected_user_ids JSONB NOT NULL DEFAULT '[]';

-- 7. Reset alert cooldown on bom_parts when stock is replenished above threshold
CREATE OR REPLACE FUNCTION reset_bom_part_alert_on_restock()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.stock_quantity > NEW.low_stock_threshold THEN
        NEW.last_low_stock_alert_at := NULL;
        NEW.last_alert_stock_quantity := NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_reset_bom_part_alert ON bom_parts;
CREATE TRIGGER trg_reset_bom_part_alert
BEFORE UPDATE OF stock_quantity, low_stock_threshold ON bom_parts
FOR EACH ROW
EXECUTE FUNCTION reset_bom_part_alert_on_restock();

