-- Migration 0022: Automatic off-site backups (full database dump + uploads) to a private GitHub repository.
-- One row; the access token is write-only (never returned by the API).

CREATE TABLE IF NOT EXISTS backup_settings (
    id INT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    github_repo VARCHAR(200) NOT NULL DEFAULT '',
    github_token TEXT NOT NULL DEFAULT '',
    -- daily | weekly (Sundays) | monthly (1st of the month)
    frequency VARCHAR(20) NOT NULL DEFAULT 'daily',
    -- Hour of day in UTC at which a due backup starts
    backup_hour_utc SMALLINT NOT NULL DEFAULT 2 CHECK (backup_hour_utc BETWEEN 0 AND 23),
    keep_count INT NOT NULL DEFAULT 30 CHECK (keep_count BETWEEN 1 AND 365),
    include_media BOOLEAN NOT NULL DEFAULT TRUE,
    token_expires_at VARCHAR(100),
    last_attempt_at TIMESTAMPTZ,
    last_success_at TIMESTAMPTZ,
    last_status VARCHAR(20),
    last_message TEXT,
    last_file VARCHAR(255),
    last_size_bytes BIGINT,
    last_url TEXT,
    failure_notified BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO backup_settings (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
