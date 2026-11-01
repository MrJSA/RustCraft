-- Migration 0029: off-site backups are encrypted with age to the owner's public key (age1…).
-- Only the public key is stored; the private key never reaches the server.
ALTER TABLE backup_settings ADD COLUMN IF NOT EXISTS encryption_recipient TEXT NOT NULL DEFAULT '';
