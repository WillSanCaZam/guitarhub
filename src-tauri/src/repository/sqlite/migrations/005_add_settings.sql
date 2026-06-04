-- Migration 005: Create settings table
-- Key-value store for app configuration (alert channel, export prefs, etc.)

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
