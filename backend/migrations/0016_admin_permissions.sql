-- Per-user access to admin sections. Superadmins always have every section.
-- Defaults: settings only for superadmins; storefront & overview for admins (not editors);
-- products and orders for everyone.
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS permissions JSONB NOT NULL DEFAULT '{}';

UPDATE admin_users
SET permissions = CASE role
    WHEN 'superadmin' THEN '{"overview": true, "products": true, "orders": true, "storefront": true, "settings": true}'::jsonb
    WHEN 'admin'      THEN '{"overview": true, "products": true, "orders": true, "storefront": true, "settings": false}'::jsonb
    ELSE                   '{"overview": false, "products": true, "orders": true, "storefront": false, "settings": false}'::jsonb
END
WHERE permissions = '{}'::jsonb;

-- The seeded sender addresses use domains nobody owns, so real mail servers reject them.
-- Empty = send from the SMTP login.
ALTER TABLE store_settings ALTER COLUMN smtp_from_email SET DEFAULT '';
UPDATE store_settings SET smtp_from_email = '' WHERE smtp_from_email IN ('shop@rustcraft.local', 'noreply@rustcraft.com');
