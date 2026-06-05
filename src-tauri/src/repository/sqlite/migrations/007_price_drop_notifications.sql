-- Migration 007: Price drop notification cooldown table
--
-- Per-SKU cooldown state for the price-drop notifier. One row per SKU that
-- has ever produced a notification; absence of a row means "never notified",
-- which is NOT the same as "in cooldown".
--
-- Columns:
--   sku           TEXT PRIMARY KEY  — single source of truth for cooldown
--   last_notified INTEGER NOT NULL  — unix epoch seconds of last successful dispatch
--   last_price    REAL    NOT NULL  — price at the time of last successful dispatch
--                                      (kept for future "we notified you at $X" UI)
--   channel       TEXT    NOT NULL  — which dispatcher was used ("app" | "ntfy" | "webhook")
--
-- Anti-spam semantics: a SKU is in cooldown when (now - last_notified) < COOLDOWN_SECS
-- (COOLDOWN_SECS = 86_400, defined in services/price_drop.rs).

CREATE TABLE IF NOT EXISTS price_drop_notifications (
    sku           TEXT    PRIMARY KEY,
    last_notified INTEGER NOT NULL,
    last_price    REAL    NOT NULL,
    channel       TEXT    NOT NULL
);

-- Lookup index: dispatch loop scans "is this SKU in cooldown?" once per drop.
CREATE INDEX IF NOT EXISTS idx_price_drop_notifications_notified
    ON price_drop_notifications(last_notified);
