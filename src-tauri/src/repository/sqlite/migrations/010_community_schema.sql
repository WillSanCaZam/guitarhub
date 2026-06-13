-- Community Hub schema — additive only, no existing tables modified.
-- Creates tables for users, profiles, lessons, riffs, comments, streaks, follows, and cache.

CREATE TABLE IF NOT EXISTS community_users (
    id            TEXT PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS community_profiles (
    user_id       TEXT PRIMARY KEY REFERENCES community_users(id) ON DELETE CASCADE,
    display_name  TEXT NOT NULL,
    avatar_url    TEXT,
    bio           TEXT,
    gear_list     TEXT NOT NULL DEFAULT '[]',
    streak_days   INTEGER NOT NULL DEFAULT 0,
    joined_at     INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS community_lessons (
    id            TEXT PRIMARY KEY,
    author_id     TEXT NOT NULL REFERENCES community_users(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    description   TEXT NOT NULL DEFAULT '',
    content_url   TEXT NOT NULL DEFAULT '',
    difficulty    TEXT NOT NULL CHECK(difficulty IN ('beginner', 'intermediate', 'advanced')),
    tags          TEXT NOT NULL DEFAULT '[]',
    likes         INTEGER NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS community_riffs (
    id            TEXT PRIMARY KEY,
    author_id     TEXT NOT NULL REFERENCES community_users(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    tablature     TEXT NOT NULL DEFAULT '',
    bpm           INTEGER NOT NULL DEFAULT 120,
    tuning        TEXT NOT NULL DEFAULT 'E A D G B E',
    tags          TEXT NOT NULL DEFAULT '[]',
    likes         INTEGER NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS community_comments (
    id            TEXT PRIMARY KEY,
    author_id     TEXT NOT NULL REFERENCES community_users(id) ON DELETE CASCADE,
    content_type  TEXT NOT NULL CHECK(content_type IN ('lesson', 'riff')),
    content_id    TEXT NOT NULL,
    body          TEXT NOT NULL,
    created_at    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_community_comments_content
    ON community_comments(content_type, content_id);

CREATE TABLE IF NOT EXISTS community_streaks (
    user_id         TEXT PRIMARY KEY REFERENCES community_users(id) ON DELETE CASCADE,
    current_streak  INTEGER NOT NULL DEFAULT 0,
    longest_streak  INTEGER NOT NULL DEFAULT 0,
    last_practice_date TEXT
);

CREATE TABLE IF NOT EXISTS community_follows (
    follower_id  TEXT NOT NULL REFERENCES community_users(id) ON DELETE CASCADE,
    following_id TEXT NOT NULL REFERENCES community_users(id) ON DELETE CASCADE,
    created_at   INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (follower_id, following_id)
);

CREATE TABLE IF NOT EXISTS community_cache (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    expires_at INTEGER NOT NULL
);
