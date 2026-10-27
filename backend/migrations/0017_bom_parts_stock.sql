-- Migration 0017: BOM Parts Stock Tracking & Multi-Product Part Logistics
-- Adds inventory stock tracking and replenishment thresholds to product_parts.

ALTER TABLE product_parts ADD COLUMN IF NOT EXISTS stock_quantity INTEGER NOT NULL DEFAULT 0;
ALTER TABLE product_parts ADD COLUMN IF NOT EXISTS low_stock_threshold INTEGER NOT NULL DEFAULT 5;

-- Seed sensible stock quantities for existing default BOM parts
UPDATE product_parts SET stock_quantity = 15, low_stock_threshold = 5 WHERE part_sku = 'PRT-CASE-ALUM';
UPDATE product_parts SET stock_quantity = 18, low_stock_threshold = 5 WHERE part_sku = 'PRT-PCB-75-RGB';
UPDATE product_parts SET stock_quantity = 25, low_stock_threshold = 5 WHERE part_sku = 'PRT-STAB-V2';
UPDATE product_parts SET stock_quantity = 12, low_stock_threshold = 5 WHERE part_sku = 'PRT-CBL-AVIATOR';
UPDATE product_parts SET stock_quantity = 8, low_stock_threshold = 10 WHERE part_sku = 'PRT-SW-BROWN-84';
UPDATE product_parts SET stock_quantity = 0, low_stock_threshold = 5 WHERE part_sku = 'PRT-KC-PBT-BLK';
UPDATE product_parts SET stock_quantity = 4, low_stock_threshold = 10 WHERE part_sku = 'PRT-SW-RED-84';
