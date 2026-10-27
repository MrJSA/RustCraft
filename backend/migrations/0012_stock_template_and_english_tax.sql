-- Migration 0012: Stock availability text template & English default tax exemption notice
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS stock_display_template TEXT NOT NULL DEFAULT 'In Stock ({stock} units available in central warehouse)';

UPDATE store_settings
SET 
    stock_display_template = COALESCE(NULLIF(stock_display_template, ''), 'In Stock ({stock} units available in central warehouse)'),
    tax_notice = CASE 
        WHEN tax_notice = 'Gemäß § 19 UStG wird keine Umsatzsteuer berechnet.' 
             OR tax_notice = 'Value added tax is not collected, as small businesses according to §19 (1) UStG.' 
             OR tax_notice = '' 
             OR tax_notice IS NULL 
        THEN 'According to § 19 UStG, no value-added tax is charged (small business regulation).'
        ELSE tax_notice
    END
WHERE id = 1;
