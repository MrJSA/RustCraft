-- Migration 0020: Every physical BOM line points to exactly one shared part (bom_parts).
-- Parts are shared by SKU, so all products using the same SKU see the same name, storage
-- location, stock and threshold. Idempotent — the backend runs it again after a data import.

-- 1. Physical BOM lines without a SKU get a generated one
UPDATE product_parts
SET part_sku = 'PRT-' || UPPER(SUBSTRING(id::text, 1, 8))
WHERE part_id IS NULL AND NULLIF(TRIM(COALESCE(part_sku, '')), '') IS NULL;

-- 2. Create a shared part for every SKU that has none yet (case-insensitive)
INSERT INTO bom_parts (sku, name, storage_location, stock_quantity, low_stock_threshold, notes)
SELECT DISTINCT ON (UPPER(TRIM(pp.part_sku)))
    UPPER(TRIM(pp.part_sku)),
    pp.part_name,
    COALESCE(pp.storage_location, 'Warehouse Main, Bin 01'),
    GREATEST(COALESCE(pp.stock_quantity, 0), 0),
    GREATEST(COALESCE(pp.low_stock_threshold, 5), 0),
    pp.notes
FROM product_parts pp
WHERE pp.part_id IS NULL
  AND pp.part_sku <> 'DIGITAL_FILE'
  AND NOT EXISTS (SELECT 1 FROM bom_parts bp WHERE UPPER(bp.sku) = UPPER(TRIM(pp.part_sku)))
ORDER BY UPPER(TRIM(pp.part_sku)), pp.created_at
ON CONFLICT (sku) DO NOTHING;

-- 3. Link unlinked lines to their shared part
UPDATE product_parts pp
SET part_id = bp.id
FROM bom_parts bp
WHERE pp.part_id IS NULL
  AND pp.part_sku <> 'DIGITAL_FILE'
  AND UPPER(bp.sku) = UPPER(TRIM(pp.part_sku));

-- 4. Digital files never point to a physical part
UPDATE product_parts SET part_id = NULL WHERE part_sku = 'DIGITAL_FILE' AND part_id IS NOT NULL;

-- 5. Copy the shared values into the linked lines so every product shows the same data
UPDATE product_parts pp
SET part_name = bp.name,
    part_sku = bp.sku,
    storage_location = bp.storage_location,
    stock_quantity = bp.stock_quantity,
    low_stock_threshold = bp.low_stock_threshold
FROM bom_parts bp
WHERE pp.part_id = bp.id
  AND (pp.part_name, pp.part_sku, pp.storage_location, pp.stock_quantity, pp.low_stock_threshold)
      IS DISTINCT FROM (bp.name, bp.sku, bp.storage_location, bp.stock_quantity, bp.low_stock_threshold);
