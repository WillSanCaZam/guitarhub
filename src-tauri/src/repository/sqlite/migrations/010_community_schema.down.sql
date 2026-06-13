-- Rollback community schema — drop all community tables in reverse dependency order.

DROP TABLE IF EXISTS community_cache;
DROP TABLE IF EXISTS community_follows;
DROP TABLE IF EXISTS community_streaks;
DROP TABLE IF EXISTS community_comments;
DROP TABLE IF EXISTS community_riffs;
DROP TABLE IF EXISTS community_lessons;
DROP TABLE IF EXISTS community_profiles;
DROP TABLE IF EXISTS community_users;
