-- Migration 0018: Centralized BOM Parts & Storage Locations
-- Centralizes Bill of Materials (BOM) components with SKU, name, storage location, and inventory tracking.

CREATE TABLE IF NOT EXISTS bom_parts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    storage_location VARCHAR(255) DEFAULT 'Warehouse Main, Bin 01',
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    low_stock_threshold INTEGER NOT NULL DEFAULT 5,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add storage_location and part_id foreign key to product_parts
ALTER TABLE product_parts ADD COLUMN IF NOT EXISTS storage_location VARCHAR(255);
ALTER TABLE product_parts ADD COLUMN IF NOT EXISTS part_id UUID REFERENCES bom_parts(id) ON DELETE SET NULL;

-- Populate bom_parts from existing physical product_parts (where part_sku <> 'DIGITAL_FILE')
INSERT INTO bom_parts (sku, name, storage_location, stock_quantity, low_stock_threshold, notes)
SELECT DISTINCT ON (COALESCE(NULLIF(part_sku, ''), id::text))
    COALESCE(NULLIF(part_sku, ''), 'PRT-' || SUBSTRING(id::text, 1, 8)) as sku,
    part_name as name,
    CASE 
        WHEN part_sku = 'PRT-CASE-ALUM' THEN 'Aisle 1, Shelf A (Chassis)'
        WHEN part_sku = 'PRT-PCB-75-RGB' THEN 'Aisle 2, Bin 14 (Electronics)'
        WHEN part_sku = 'PRT-STAB-V2' THEN 'Aisle 2, Bin 05 (Hardware)'
        WHEN part_sku = 'PRT-CBL-AVIATOR' THEN 'Aisle 3, Rack C (Cables)'
        WHEN part_sku = 'PRT-SW-BROWN-84' THEN 'Aisle 4, Bin 21 (Switches)'
        WHEN part_sku = 'PRT-KC-PBT-BLK' THEN 'Aisle 4, Bin 09 (Keycaps)'
        WHEN part_sku = 'PRT-SW-RED-84' THEN 'Aisle 4, Bin 22 (Switches)'
        ELSE 'Warehouse Main, Bin 01'
    END as storage_location,
    COALESCE(stock_quantity, 0) as stock_quantity,
    COALESCE(low_stock_threshold, 5) as low_stock_threshold,
    notes
FROM product_parts
WHERE COALESCE(part_sku, '') <> 'DIGITAL_FILE'
ON CONFLICT (sku) DO UPDATE SET
    storage_location = EXCLUDED.storage_location,
    stock_quantity = EXCLUDED.stock_quantity;

-- Seed BOM components for the demo Hoodie and Deskmat — only where those demo products exist,
-- so real shops never get demo parts in their inventory
INSERT INTO bom_parts (sku, name, storage_location, stock_quantity, low_stock_threshold, notes)
SELECT v.sku, v.name, v.storage_location, v.stock_quantity, v.low_stock_threshold, v.notes
FROM (VALUES
('PRT-FABRIC-TERRY', '480 GSM French Terry Organic Cotton (per meter)', 'Warehouse 1, Fabric Rack 2', 40, 10, 'Heavyweight unbrushed loopback fleece', 'hoodie'),
('PRT-THREAD-EMB', 'Embroidered Ferris Insignia Thread Spool', 'Warehouse 1, Bin 18', 35, 5, 'Rust-orange high-sheen Madeira polyester thread', 'hoodie'),
('PRT-CORD-HOOD', 'Braided Drawstring Cord with Metal Aglets', 'Warehouse 1, Bin 07', 25, 5, 'Heavy-duty natural woven cord with gunmetal tips', 'hoodie'),
('PRT-RUBBER-BASE', 'High-Density Non-Slip Natural Rubber Base (900x400)', 'Warehouse 1, Rack 05', 30, 5, '4mm textured rubber non-slip mat backing', 'deskmat'),
('PRT-MICRO-CLOTH', 'Micro-Weave Low-Friction Speed Cloth Surface', 'Warehouse 1, Rack 06', 28, 5, 'Spill-resistant micro-woven polyester glide fabric', 'deskmat')
) AS v(sku, name, storage_location, stock_quantity, low_stock_threshold, notes, demo)
WHERE EXISTS (
    SELECT 1 FROM products p
    WHERE (v.demo = 'hoodie' AND (p.id = 'a2000000-0000-0000-0000-000000000002' OR p.slug = 'ferris-heavyweight-hoodie'))
       OR (v.demo = 'deskmat' AND (p.id = 'a3000000-0000-0000-0000-000000000003' OR p.slug = 'ferris-circuit-deskmat'))
)
ON CONFLICT (sku) DO NOTHING;

-- Link existing product_parts to the centralized bom_parts entries
UPDATE product_parts pp
SET part_id = bp.id,
    storage_location = bp.storage_location
FROM bom_parts bp
WHERE pp.part_sku = bp.sku;

-- Attach BOM parts for Hoodie if product exists
INSERT INTO product_parts (product_id, variant_id, part_id, part_name, part_sku, quantity, notes, storage_location, stock_quantity, low_stock_threshold)
SELECT 
    p.id,
    NULL,
    bp.id,
    bp.name,
    bp.sku,
    1,
    bp.notes,
    bp.storage_location,
    bp.stock_quantity,
    bp.low_stock_threshold
FROM bom_parts bp
JOIN products p ON (p.id = 'a2000000-0000-0000-0000-000000000002' OR p.slug = 'ferris-heavyweight-hoodie')
WHERE bp.sku IN ('PRT-FABRIC-TERRY', 'PRT-THREAD-EMB', 'PRT-CORD-HOOD')
AND NOT EXISTS (
    SELECT 1 FROM product_parts WHERE product_id = p.id AND part_sku = bp.sku
);

-- Attach BOM parts for Deskmat if product exists
INSERT INTO product_parts (product_id, variant_id, part_id, part_name, part_sku, quantity, notes, storage_location, stock_quantity, low_stock_threshold)
SELECT 
    p.id,
    NULL,
    bp.id,
    bp.name,
    bp.sku,
    1,
    bp.notes,
    bp.storage_location,
    bp.stock_quantity,
    bp.low_stock_threshold
FROM bom_parts bp
JOIN products p ON (p.id = 'a3000000-0000-0000-0000-000000000003' OR p.slug = 'ferris-circuit-deskmat')
WHERE bp.sku IN ('PRT-RUBBER-BASE', 'PRT-MICRO-CLOTH')
AND NOT EXISTS (
    SELECT 1 FROM product_parts WHERE product_id = p.id AND part_sku = bp.sku
);
