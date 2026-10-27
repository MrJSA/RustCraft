-- Migration 0023: Orders paid in test mode (Stripe test keys, PayPal sandbox) are flagged
-- and never counted in the dashboard or analytics.

ALTER TABLE orders ADD COLUMN IF NOT EXISTS is_test BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX IF NOT EXISTS idx_orders_live_created_at ON orders (created_at) WHERE is_test = FALSE;
