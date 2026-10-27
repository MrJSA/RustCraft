-- Payment gateway overhaul:
--   * One Stripe provider (on-site Payment Element or hosted redirect) with wallet / method toggles
--   * Real PayPal Orders v2 integration
--   * Server-side pending checkouts so a paid order can always be finalized (client return or webhook)
--   * Removal of the simulated Apple Pay / Google Pay / Amazon Pay providers (now offered through Stripe)

ALTER TABLE payment_configs ADD COLUMN IF NOT EXISTS webhook_secret TEXT NOT NULL DEFAULT '';

-- Carry over keys from a legacy 'stripe_elements' row if one was ever created
UPDATE payment_configs s
SET public_client_id = CASE
        WHEN s.public_client_id NOT LIKE 'pk\_%' AND e.public_client_id LIKE 'pk\_%' THEN e.public_client_id
        ELSE s.public_client_id
    END,
    secret_key = CASE
        WHEN s.secret_key NOT LIKE 'sk\_%' AND s.secret_key NOT LIKE 'rk\_%'
             AND (e.secret_key LIKE 'sk\_%' OR e.secret_key LIKE 'rk\_%') THEN e.secret_key
        ELSE s.secret_key
    END
FROM payment_configs e
WHERE s.provider = 'stripe' AND e.provider = 'stripe_elements';

DELETE FROM payment_configs WHERE provider IN ('stripe_elements', 'stripe_hosted', 'apple_pay', 'google_pay', 'amazon_pay');

-- Clear the placeholder demo credentials seeded in 0001 (they can never authenticate)
UPDATE payment_configs SET public_client_id = '' WHERE public_client_id IN ('pk_test_sample_stripe_key_rustwebshop', 'sb_client_id_rustwebshop_paypal');
UPDATE payment_configs SET secret_key = '' WHERE secret_key IN ('sk_test_sample_stripe_secret', 'sb_secret_rustwebshop_paypal');

INSERT INTO payment_configs (provider, display_name, is_enabled, is_sandbox, public_client_id, secret_key, config_data)
VALUES
    ('stripe', 'Credit / Debit Card', false, true, '', '', '{}'),
    ('paypal', 'PayPal', false, true, '', '', '{}')
ON CONFLICT (provider) DO NOTHING;

UPDATE payment_configs
SET display_name = CASE
        WHEN display_name IN ('Stripe', 'Stripe Credit & Debit Cards', 'Stripe Hosted Checkout (Redirect to Stripe)') THEN 'Credit / Debit Card'
        ELSE display_name
    END,
    config_data = '{"checkout_mode": "elements", "methods": {"apple_pay": true, "google_pay": true, "link": false, "amazon_pay": false, "paypal": false}}'::jsonb
                  || (config_data - 'collect_billing_address')
WHERE provider = 'stripe';

UPDATE payment_configs
SET display_name = CASE WHEN display_name = 'PayPal Express Checkout' THEN 'PayPal' ELSE display_name END,
    config_data = '{"allow_pay_later": true}'::jsonb || config_data
WHERE provider = 'paypal';

-- A checkout whose payment has been started but not yet turned into an order
CREATE TABLE IF NOT EXISTS pending_checkouts (
    id UUID PRIMARY KEY,
    provider VARCHAR(50) NOT NULL,
    provider_reference TEXT UNIQUE,
    checkout_payload JSONB NOT NULL,
    amount_cents INT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'EUR',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    order_number VARCHAR(100),
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Provider transaction reference (Stripe PaymentIntent / PayPal capture id) used for refunds and idempotency
ALTER TABLE orders ADD COLUMN IF NOT EXISTS payment_reference TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_orders_payment_reference ON orders(payment_reference) WHERE payment_reference IS NOT NULL;
