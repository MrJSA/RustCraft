-- Migration 0021: Structured shop address + legal pages that always show the shop's own data.
-- Idempotent — the backend runs it again after a data import (older backups only have the
-- single-line address and may contain the demo texts).

-- 1. Structured address; company_address stays as the composed single line for documents & emails
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS address_street VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS address_house_number VARCHAR(50) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS address_extra VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS address_postal_code VARCHAR(20) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS address_city VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS address_country VARCHAR(100) NOT NULL DEFAULT '';

-- 2. Split an existing single-line address ("Street 1 a, Addition, 12345 City, Country") once
DO $$
DECLARE
    s RECORD;
    parts TEXT[];
    p TEXT;
    i INT;
    zip_idx INT := 0;
    m TEXT[];
    v_street TEXT := '';
    v_number TEXT := '';
    v_extra TEXT := '';
    v_zip TEXT := '';
    v_city TEXT := '';
    v_country TEXT := '';
BEGIN
    SELECT * INTO s FROM store_settings WHERE id = 1;
    IF NOT FOUND OR TRIM(s.company_address) = ''
       OR (s.address_street <> '' OR s.address_postal_code <> '' OR s.address_city <> '') THEN
        RETURN;
    END IF;

    SELECT array_agg(TRIM(x)) INTO parts FROM unnest(string_to_array(s.company_address, ',')) AS x WHERE TRIM(x) <> '';
    FOR i IN 1 .. COALESCE(array_length(parts, 1), 0) LOOP
        IF zip_idx = 0 AND i > 1 AND parts[i] ~ '^[0-9]{4,5}\s+\S' THEN
            zip_idx := i;
        END IF;
    END LOOP;

    IF zip_idx = 0 THEN
        -- Unknown format: keep everything in the street field, nothing is lost
        v_street := s.company_address;
    ELSE
        m := regexp_match(parts[1], '^(.*\S)\s+([0-9]+\s*[A-Za-z]?(\s*[-/]\s*[0-9]+\s*[A-Za-z]?)?)$');
        IF m IS NULL THEN
            v_street := parts[1];
        ELSE
            v_street := m[1];
            v_number := m[2];
        END IF;
        IF zip_idx > 2 THEN
            v_extra := array_to_string(parts[2:zip_idx - 1], ', ');
        END IF;
        m := regexp_match(parts[zip_idx], '^([0-9]{4,5})\s+(.+)$');
        v_zip := m[1];
        v_city := m[2];
        IF array_length(parts, 1) > zip_idx THEN
            v_country := array_to_string(parts[zip_idx + 1:], ', ');
        END IF;
    END IF;

    UPDATE store_settings
    SET address_street = v_street, address_house_number = v_number, address_extra = v_extra,
        address_postal_code = v_zip, address_city = v_city, address_country = v_country
    WHERE id = 1;
END $$;

-- 3. Default legal texts: demo identity → placeholders, outdated law references, wrong button name.
--    Only these exact default strings are touched; texts the shop wrote itself stay as they are.
UPDATE pages SET content_markdown =
    REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(
        content_markdown,
        'RustCraft Gear & Software GmbH', '{{LEGAL_NAME}}'),
        'RustCraft Gear & Software', '{{STORE_NAME}}'),
        'Rustacean Way 42, 10115 Berlin, Germany', '{{COMPANY_ADDRESS}}'),
        'privacy@rustwebshop.local', '{{SUPPORT_EMAIL}}'),
        'support@rustwebshop.local', '{{SUPPORT_EMAIL}}'),
        '+49 (0) 30 123456-78', '{{PHONE}}'),
        'DE314159265', '{{VAT_ID}}'),
        'our central warehouse in Berlin', 'our warehouse'),
        'According to § 5 TMG / § 55 RStV:', 'According to § 5 DDG / § 18 MStV:'),
        'Section 7, paragraph 1 of the TMG (German Telemedia Act)', 'Section 7, paragraph 1 of the DDG (German Digital Services Act)'),
        'Sections 8 to 10 of the TMG', 'Sections 8 to 10 of the DDG'),
        '- By clicking the final "Pay & Complete Order" button, you accept the offer for the items in your shopping cart.',
        '- By clicking the "Order with Obligation to Pay" button, you place a binding order for the items in your shopping cart. The contract is concluded with our order confirmation.')
WHERE content_markdown ~ '(RustCraft Gear & Software|Rustacean Way 42|rustwebshop\.local|123456-78|DE314159265|warehouse in Berlin|TMG|RStV|Pay & Complete Order)';

UPDATE pages SET content_markdown = REPLACE(content_markdown,
    '- Product presentations in our online shop constitute a binding offer to enter into a sales contract.',
    '- The presentation of products in our online shop is not a binding offer but an invitation to place an order.')
WHERE content_markdown LIKE '%constitute a binding offer to enter into a sales contract%';
