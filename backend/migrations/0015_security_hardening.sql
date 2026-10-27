-- Security hardening:
--   * server-generated secrets (JWT signing key) instead of a hard-coded default
--   * token-based customer password reset
--   * unguessable per-order access tokens for guest order pages
--   * role-based admin access (the oldest admin becomes superadmin)
--   * generic default order prefix

CREATE TABLE IF NOT EXISTS server_secrets (
    name VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE customers ADD COLUMN IF NOT EXISTS reset_token_hash TEXT;
ALTER TABLE customers ADD COLUMN IF NOT EXISTS reset_token_expires_at TIMESTAMPTZ;

ALTER TABLE orders ADD COLUMN IF NOT EXISTS access_token TEXT;
UPDATE orders SET access_token = replace(gen_random_uuid()::text, '-', '') || replace(gen_random_uuid()::text, '-', '')
WHERE access_token IS NULL;
CREATE INDEX IF NOT EXISTS idx_orders_customer_email_lower ON orders (LOWER(customer_email));

UPDATE admin_users SET role = 'admin' WHERE role NOT IN ('superadmin', 'admin', 'editor');
UPDATE admin_users SET role = 'superadmin'
WHERE id = (SELECT id FROM admin_users ORDER BY created_at ASC LIMIT 1)
  AND NOT EXISTS (SELECT 1 FROM admin_users WHERE role = 'superadmin');

ALTER TABLE store_settings ALTER COLUMN order_prefix SET DEFAULT 'ORD';
-- Replace the former hard-coded default prefix (compared by hash so it is not repeated here)
UPDATE store_settings SET order_prefix = 'ORD' WHERE md5(order_prefix) = '80d8927b2b6c9c3d8efa1e14e233b493';

-- Replace the former placeholder managing-director names in the seeded legal notice with a generic one
UPDATE pages
SET content_markdown = regexp_replace(content_markdown, '\mJ[a-z]+ Rust, Dr\. Ferris Crab', 'Max Mustermann', 'g')
WHERE content_markdown ~ '\mJ[a-z]+ Rust, Dr\. Ferris Crab';
