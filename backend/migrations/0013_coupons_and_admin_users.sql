-- Migration 0013: Discount/Promo Codes, Order Coupons, and Admin User Access
CREATE TABLE IF NOT EXISTS coupons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,
    discount_type VARCHAR(30) NOT NULL, -- 'free_shipping', 'fixed_amount', 'percentage'
    value_cents INTEGER NOT NULL DEFAULT 0,
    min_order_cents INTEGER NOT NULL DEFAULT 0,
    max_uses INTEGER,
    used_count INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_coupons_code ON coupons(UPPER(code));

ALTER TABLE orders ADD COLUMN IF NOT EXISTS coupon_code VARCHAR(50);
ALTER TABLE orders ADD COLUMN IF NOT EXISTS discount_cents INTEGER NOT NULL DEFAULT 0;

ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS email VARCHAR(255);
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS role VARCHAR(50) NOT NULL DEFAULT 'admin';
