-- Migration 0027: email to the shop team for every new order. The shop decides which roles get it
-- (off, superadmin, admin = admin + superadmin, all = also editors); every admin can switch it off for themselves.

ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS order_notification_roles VARCHAR(20) NOT NULL DEFAULT 'superadmin';
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS order_notifications BOOLEAN NOT NULL DEFAULT TRUE;
