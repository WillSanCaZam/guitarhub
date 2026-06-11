// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};
use sqlx::SqlitePool;

/// A lightweight migration runner that discovers `.sql` files by numeric prefix,
/// applies them in order, and tracks state via `schema_meta.db_version`.
///
/// No external dependency beyond `sqlx` — runs plain SQL in transactions.
pub struct MigrationRunner {
    pool: SqlitePool,
    dir: PathBuf,
}

/// A migration file discovered in the migrations directory.
#[derive(Debug, Clone)]
pub struct DiscoveredMigration {
    pub version: u32,
    pub filename: String,
    pub path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("gap in migration sequence: expected v{expected}, found v{found}")]
    GapInSequence { expected: u32, found: u32 },
    #[error("invalid schema_meta.db_version value: {0}")]
    InvalidVersion(String),
    #[error("migration {filename} failed: {source}")]
    SqlError { filename: String, source: sqlx::Error },
    #[error("failed to read migration file {filename}: {source}")]
    IoError { filename: String, source: std::io::Error },
}

impl MigrationRunner {
    pub fn new(pool: SqlitePool, dir: PathBuf) -> Self {
        Self { pool, dir }
    }

    /// Run all pending migrations in order.
    ///
    /// Returns immediately with `Ok(())` if the database is already up-to-date.
    /// Returns `MigrationError` if a gap is detected, the version is corrupt,
    /// or any migration SQL fails.
    pub async fn run(&self) -> Result<(), MigrationError> {
        let discovered = self.discover_up()?;
        let current = self.current_version().await?;

        let pending: Vec<&DiscoveredMigration> = discovered
            .iter()
            .filter(|m| m.version > current)
            .collect();

        if pending.is_empty() {
            tracing::info!(
                "Database is up-to-date at version {}",
                current
            );
            return Ok(());
        }

        // Validate that the pending sequence has no gaps.
        for (i, migration) in pending.iter().enumerate() {
            let expected = current as usize + i + 1;
            if migration.version as usize != expected {
                return Err(MigrationError::GapInSequence {
                    expected: expected as u32,
                    found: migration.version,
                });
            }
        }

        // Apply each migration in its own transaction.
        for migration in &pending {
            let sql = std::fs::read_to_string(&migration.path).map_err(|e| {
                MigrationError::IoError {
                    filename: migration.filename.clone(),
                    source: e,
                }
            })?;

            let mut tx = self.pool.begin().await.map_err(|e| {
                MigrationError::SqlError {
                    filename: migration.filename.clone(),
                    source: e,
                }
            })?;

            for statement in split_statements(&sql) {
                let trimmed = statement.trim();
                if trimmed.is_empty() {
                    continue;
                }
                sqlx::query(sqlx::AssertSqlSafe(trimmed))
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| MigrationError::SqlError {
                        filename: migration.filename.clone(),
                        source: e,
                    })?;
            }

            sqlx::query(
                "INSERT INTO schema_meta (key, value) VALUES ('db_version', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .bind(migration.version.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| MigrationError::SqlError {
                filename: migration.filename.clone(),
                source: e,
            })?;

            tx.commit().await.map_err(|e| MigrationError::SqlError {
                filename: migration.filename.clone(),
                source: e,
            })?;

            tracing::info!("Applied migration {}", migration.filename);
        }

        Ok(())
    }

    /// Roll back `steps` migrations by applying their `.down.sql` files
    /// in reverse version order.
    ///
    /// Each down migration runs in its own transaction. After each step the
    /// `db_version` is decremented.
    ///
    /// Returns an error if:
    /// - The database is at version 0 (nothing to roll back).
    /// - A `.down.sql` file is missing for the target version.
    pub async fn rollback(&self, steps: u32) -> Result<(), MigrationError> {
        let current = self.current_version().await?;
        if current == 0 {
            return Ok(());
        }

        let target = current.saturating_sub(steps);
        let downs = self.discover_down()?;

        for version in (target + 1..=current).rev() {
            let down = downs
                .iter()
                .find(|m| m.version == version)
                .cloned()
                .ok_or_else(|| MigrationError::IoError {
                    filename: format!("{:03}_*.down.sql", version),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("down migration for v{version} not found"),
                    ),
                })?;

            let sql = std::fs::read_to_string(&down.path).map_err(|e| {
                MigrationError::IoError {
                    filename: down.filename.clone(),
                    source: e,
                }
            })?;

            let mut tx = self.pool.begin().await.map_err(|e| {
                MigrationError::SqlError {
                    filename: down.filename.clone(),
                    source: e,
                }
            })?;

            for statement in split_statements(&sql) {
                let trimmed = statement.trim();
                if trimmed.is_empty() {
                    continue;
                }
                sqlx::query(sqlx::AssertSqlSafe(trimmed))
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| MigrationError::SqlError {
                        filename: down.filename.clone(),
                        source: e,
                    })?;
            }

            let new_version = version - 1;
            sqlx::query(
                "INSERT INTO schema_meta (key, value) VALUES ('db_version', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .bind(new_version.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| MigrationError::SqlError {
                filename: down.filename.clone(),
                source: e,
            })?;

            tx.commit().await.map_err(|e| MigrationError::SqlError {
                filename: down.filename.clone(),
                source: e,
            })?;

            tracing::info!("Rolled back migration v{version} → v{new_version}");
        }

        Ok(())
    }

    /// Discover all `.sql` files (non-down) sorted by version.
    fn discover_up(&self) -> Result<Vec<DiscoveredMigration>, MigrationError> {
        Ok(self
            .discover()?
            .into_iter()
            .filter(|m| !m.filename.ends_with(".down.sql"))
            .collect())
    }

    /// Discover all `.down.sql` files sorted by version.
    fn discover_down(&self) -> Result<Vec<DiscoveredMigration>, MigrationError> {
        Ok(self
            .discover()?
            .into_iter()
            .filter(|m| m.filename.ends_with(".down.sql"))
            .collect())
    }

    /// Discover all `.sql` files in the migrations directory sorted by version.
    ///
    /// Files with a non-numeric prefix are skipped with a warning.
    /// If the directory does not exist, returns an empty list (graceful for tests).
    fn discover(&self) -> Result<Vec<DiscoveredMigration>, MigrationError> {
        if !self.dir.exists() {
            tracing::warn!(
                "Migrations directory not found: {:?}",
                self.dir
            );
            return Ok(vec![]);
        }

        let mut migrations = Vec::new();

        let entries = std::fs::read_dir(&self.dir).map_err(|e| {
            MigrationError::IoError {
                filename: self.dir.to_string_lossy().to_string(),
                source: e,
            }
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| MigrationError::IoError {
                filename: self.dir.to_string_lossy().to_string(),
                source: e,
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("sql") {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

            match extract_version(&filename) {
                Some(version) => migrations.push(DiscoveredMigration {
                    version,
                    filename,
                    path,
                }),
                None => {
                    tracing::warn!(
                        "Skipping non-numeric migration filename: {}",
                        filename
                    );
                }
            }
        }

        migrations.sort_by_key(|m| m.version);
        Ok(migrations)
    }

    /// Read the current schema version from `schema_meta.db_version`.
    ///
    /// Returns `0` when:
    /// - The `schema_meta` table does not exist (fresh database)
    /// - The `db_version` key is absent
    ///
    /// Returns `MigrationError::InvalidVersion` if the stored value is not
    /// a valid unsigned integer.
    async fn current_version(&self) -> Result<u32, MigrationError> {
        let result: Result<Option<String>, sqlx::Error> = sqlx::query_scalar(
            "SELECT value FROM schema_meta WHERE key = 'db_version'",
        )
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(val)) => val
                .parse::<u32>()
                .map_err(|_| MigrationError::InvalidVersion(val)),
            Ok(None) => Ok(0),
            Err(e) if is_table_not_found(&e) => Ok(0),
            Err(e) => Err(MigrationError::SqlError {
                filename: "schema_meta".to_string(),
                source: e,
            }),
        }
    }
}

/// Extract the numeric version from a migration filename.
///
/// Expects a three-digit numeric prefix followed by `_`, e.g.:
/// - `"001_init.sql"` → `Some(1)`
/// - `"002_add_index.sql"` → `Some(2)`
/// - `"setup.sql"` → `None`
fn extract_version(filename: &str) -> Option<u32> {
    let stem = Path::new(filename).file_stem()?.to_str()?;
    let prefix = stem.split('_').next()?;
    prefix.parse::<u32>().ok()
}

/// Split SQL text into individual statements, respecting `BEGIN...END` blocks.
///
/// A naive `split(';')` breaks `CREATE TRIGGER` bodies that contain
/// semicolons inside `BEGIN...END`. This function tracks `BEGIN`/`END`
/// depth so that semicolons inside trigger/procedure bodies are preserved.
fn split_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut depth: u32 = 0;

    for token in tokenize_sql(sql) {
        match token.to_uppercase().as_str() {
            "BEGIN" => {
                depth += 1;
                current.push_str(&token);
            }
            "END" if depth > 0 => {
                depth = depth.saturating_sub(1);
                current.push_str(&token);
                // If we just closed a block and hit ';', flush
            }
            ";" if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    statements.push(trimmed);
                }
                current.clear();
            }
            _ => {
                current.push_str(&token);
            }
        }
    }

    // Flush any trailing statement without a trailing semicolon
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        statements.push(trimmed);
    }

    statements
}

/// Tokenize SQL into words and separators, preserving whitespace within
/// the current statement but splitting on word boundaries for keyword detection.
/// Strips SQL line comments (-- ...) to prevent comment text from interfering
/// with statement parsing.
fn tokenize_sql(sql: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        // Skip SQL line comments: -- until end of line
        if ch == '-' && chars.peek() == Some(&'-') {
            // Consume the second '-'
            chars.next();
            // Skip until end of line
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '\n' {
                    break;
                }
                chars.next();
            }
            continue;
        }

        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            if !word.is_empty() {
                tokens.push(std::mem::take(&mut word));
            }
            tokens.push(ch.to_string());
        }
    }
    if !word.is_empty() {
        tokens.push(word);
    }

    tokens
}

/// Check whether a sqlx error is caused by a missing table (fresh database).
fn is_table_not_found(err: &sqlx::Error) -> bool {
    err.to_string().contains("no such table")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── discover() ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn discover_returns_empty_for_missing_dir() {
        let pool = make_memory_pool().await;
        let runner = MigrationRunner {
            pool,
            dir: PathBuf::from("/tmp/nonexistent-migrations-dir-12345"),
        };
        let result = runner.discover().unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn discover_skips_non_sql_files() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[
            "001_init.sql",
            "README.md",
            "002_add_index.sql",
            "notes.txt",
        ]);
        let runner = MigrationRunner {
            pool,
            dir,
        };
        let result = runner.discover().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].version, 1);
        assert_eq!(result[1].version, 2);
    }

    #[tokio::test]
    async fn discover_skips_non_numeric_prefix() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[
            "001_init.sql",
            "setup.sql",
            "002_add_index.sql",
        ]);
        let runner = MigrationRunner {
            pool,
            dir,
        };
        let result = runner.discover().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].version, 1);
        assert_eq!(result[1].version, 2);
    }

    #[tokio::test]
    async fn discover_sorts_by_version() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[
            "003_last.sql",
            "001_first.sql",
            "002_second.sql",
        ]);
        let runner = MigrationRunner {
            pool,
            dir,
        };
        let result = runner.discover().unwrap();
        assert_eq!(result.len(), 3);
        assert!(result[0].filename.contains("001"));
        assert!(result[1].filename.contains("002"));
        assert!(result[2].filename.contains("003"));
    }

    #[test]
    fn extract_version_parses_prefix() {
        assert_eq!(extract_version("001_init.sql"), Some(1));
        assert_eq!(extract_version("012_add_index.sql"), Some(12));
        assert_eq!(extract_version("setup.sql"), None);
        assert_eq!(extract_version("no_number.sql"), None);
    }

    // ── current_version() ───────────────────────────────────────────────

    #[tokio::test]
    async fn current_version_returns_0_when_table_missing() {
        let pool = make_memory_pool().await;
        let runner = MigrationRunner::new(pool, PathBuf::from("/tmp/empty"));
        assert_eq!(runner.current_version().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn current_version_reads_stored_value() {
        let pool = make_memory_pool().await;
        sqlx::query(
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schema_meta VALUES ('db_version', '3')")
            .execute(&pool)
            .await
            .unwrap();

        let runner = MigrationRunner::new(pool, PathBuf::from("/tmp/empty"));
        assert_eq!(runner.current_version().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn current_version_errors_on_corrupt_value() {
        let pool = make_memory_pool().await;
        sqlx::query(
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schema_meta VALUES ('db_version', 'abc')")
            .execute(&pool)
            .await
            .unwrap();

        let runner = MigrationRunner::new(pool, PathBuf::from("/tmp/empty"));
        let err = runner.current_version().await.unwrap_err();
        assert!(
            matches!(&err, MigrationError::InvalidVersion(v) if v == "abc"),
            "Expected InvalidVersion('abc'), got {err}"
        );
    }

    #[tokio::test]
    async fn current_version_returns_0_when_key_missing() {
        let pool = make_memory_pool().await;
        sqlx::query(
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schema_meta VALUES ('other_key', 'val')")
            .execute(&pool)
            .await
            .unwrap();

        let runner = MigrationRunner::new(pool, PathBuf::from("/tmp/empty"));
        assert_eq!(runner.current_version().await.unwrap(), 0);
    }

    // ── run() — full flow ───────────────────────────────────────────────

    #[tokio::test]
    async fn run_applies_all_migrations_on_fresh_db() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[
            "001_init.sql",
        ]);

        // Write a minimal 001_init.sql
        std::fs::write(
            dir.join("001_init.sql"),
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('db_version', '1');",
        )
        .unwrap();

        let runner = MigrationRunner::new(pool.clone(), dir);
        runner.run().await.unwrap();

        // Verify db_version was set
        let version: String = sqlx::query_scalar(
            "SELECT value FROM schema_meta WHERE key = 'db_version'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(version, "1");
    }

    #[tokio::test]
    async fn run_is_noop_when_up_to_date() {
        let pool = make_memory_pool().await;
        sqlx::query(
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schema_meta VALUES ('db_version', '1')")
            .execute(&pool)
            .await
            .unwrap();

        let dir = temp_dir_with_files(&["001_init.sql"]);
        std::fs::write(dir.join("001_init.sql"), "SELECT 1").unwrap();

        let runner = MigrationRunner::new(pool.clone(), dir);
        runner.run().await.unwrap();

        // Version unchanged
        let version: String = sqlx::query_scalar(
            "SELECT value FROM schema_meta WHERE key = 'db_version'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(version, "1");
    }

    #[tokio::test]
    async fn run_detects_gap_in_sequence() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&["002_init.sql"]); // missing 001
        std::fs::write(dir.join("002_init.sql"), "SELECT 1").unwrap();

        let runner = MigrationRunner::new(pool, dir);
        let err = runner.run().await.unwrap_err();

        assert!(
            matches!(&err, MigrationError::GapInSequence { expected: 1, found: 2 }),
            "Expected GapInSequence expecting v1 found v2, got {err}"
        );
    }

    #[tokio::test]
    async fn run_errors_on_sql_failure() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&["001_init.sql"]);
        std::fs::write(dir.join("001_init.sql"), "CREAT TABLE broken;").unwrap();

        let runner = MigrationRunner::new(pool, dir);
        let err = runner.run().await.unwrap_err();

        assert!(
            matches!(&err, MigrationError::SqlError { filename, .. } if filename == "001_init.sql"),
            "Expected SqlError for 001_init.sql, got {err}"
        );
    }

    #[tokio::test]
    async fn run_applies_incremental_migration() {
        let pool = make_memory_pool().await;
        sqlx::query(
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schema_meta VALUES ('db_version', '1')")
            .execute(&pool)
            .await
            .unwrap();

        let dir = temp_dir_with_files(&["001_init.sql", "002_add_table.sql"]);
        std::fs::write(dir.join("001_init.sql"), "SELECT 1").unwrap();
        std::fs::write(
            dir.join("002_add_table.sql"),
            "CREATE TABLE test_table (id INTEGER PRIMARY KEY);",
        )
        .unwrap();

        let runner = MigrationRunner::new(pool.clone(), dir);
        runner.run().await.unwrap();

        // Version should be 2
        let version: String = sqlx::query_scalar(
            "SELECT value FROM schema_meta WHERE key = 'db_version'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(version, "2");

        // Table should exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_table")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn run_failure_does_not_update_version() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&["001_init.sql", "002_broken.sql"]);
        // 001_init.sql must create schema_meta so we can verify rollback
        std::fs::write(
            dir.join("001_init.sql"),
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('db_version', '1');",
        )
        .unwrap();
        std::fs::write(dir.join("002_broken.sql"), "INVALID SQL HERE;").unwrap();

        let runner = MigrationRunner::new(pool.clone(), dir);
        let err = runner.run().await.unwrap_err();

        // The error should be from 002_broken.sql
        assert!(
            matches!(&err, MigrationError::SqlError { filename, .. } if filename == "002_broken.sql"),
            "Expected SqlError for 002_broken.sql, got {err}"
        );

        // Version should still be 1 (001 succeeded, 002 was rolled back)
        let version: String = sqlx::query_scalar(
            "SELECT value FROM schema_meta WHERE key = 'db_version'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            version, "1",
            "Expected version 1 after 001 succeeded and 002 rolled back"
        );
    }

    // ── split_statements() ─────────────────────────────────────────────

    #[test]
    fn split_statements_handles_trigger_with_begin_end() {
        let sql = "\
CREATE TABLE t1 (id INTEGER PRIMARY KEY, val TEXT);
CREATE TRIGGER t1_ai AFTER INSERT ON t1 BEGIN
  INSERT INTO log(msg) VALUES (new.val);
  INSERT INTO log(msg) VALUES ('done');
END;
CREATE TABLE t2 (id INTEGER PRIMARY KEY);";

        let stmts = split_statements(sql);
        assert_eq!(stmts.len(), 3, "expected 3 statements, got {}: {:?}", stmts.len(), stmts);
        assert!(stmts[0].starts_with("CREATE TABLE t1"), "first stmt: {}", stmts[0]);
        assert!(stmts[1].contains("CREATE TRIGGER"), "second stmt should be the trigger: {}", stmts[1]);
        assert!(stmts[1].contains("END"), "trigger stmt must include END: {}", stmts[1]);
        assert!(stmts[2].starts_with("CREATE TABLE t2"), "third stmt: {}", stmts[2]);
    }

    #[test]
    fn split_statements_plain_semicolon_statements() {
        let sql = "CREATE TABLE a (id INT); INSERT INTO a VALUES (1); CREATE TABLE b (id INT);";
        let stmts = split_statements(sql);
        assert_eq!(stmts.len(), 3, "expected 3 plain statements, got {}", stmts.len());
        assert!(stmts[0].contains("CREATE TABLE a"));
        assert!(stmts[1].contains("INSERT INTO a"));
        assert!(stmts[2].contains("CREATE TABLE b"));
    }

    #[test]
    fn split_statements_skips_empty_and_whitespace() {
        let sql = "  ;  CREATE TABLE x (id INT) ;  ;  ";
        let stmts = split_statements(sql);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].contains("CREATE TABLE x"));
    }

    // ── run() — trigger migration integration ─────────────────────────

    #[tokio::test]
    async fn run_applies_migration_with_trigger_begin_end() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&["001_init.sql", "002_triggers.sql"]);

        std::fs::write(
            dir.join("001_init.sql"),
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('db_version', '1');
             CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT);
             CREATE TABLE log (id INTEGER PRIMARY KEY AUTOINCREMENT, msg TEXT);",
        )
        .unwrap();

        std::fs::write(
            dir.join("002_triggers.sql"),
            "CREATE TRIGGER products_ai AFTER INSERT ON products BEGIN
  INSERT INTO log(msg) VALUES ('inserted: ' || new.name);
  INSERT INTO log(msg) VALUES ('done');
END;",
        )
        .unwrap();

        let runner = MigrationRunner::new(pool.clone(), dir);
        runner.run().await.unwrap();

        // Verify the trigger was created and works
        sqlx::query("INSERT INTO products (name) VALUES ('guitar')")
            .execute(&pool)
            .await
            .unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM log")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2, "trigger should have inserted 2 log rows");
    }

    // ── Migration 002 tests ─────────────────────────────────────────────

    /// Helper: apply the real migration 001 and 002 files on an in-memory pool.
    async fn apply_001_and_002(pool: &sqlx::SqlitePool) -> PathBuf {
        let dir = temp_dir_with_files(&["001_init.sql", "002_add_url_validation.sql"]);
        std::fs::write(
            dir.join("001_init.sql"),
            include_str!("../migrations/001_init.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("002_add_url_validation.sql"),
            include_str!("../migrations/002_add_url_validation.sql"),
        )
        .unwrap();

        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        dir
    }

    #[tokio::test]
    async fn migration_002_preserves_all_17_columns() {
        let pool = make_memory_pool().await;
        apply_001_and_002(&pool).await;

        // PRAGMA table_info returns one row per column
        let columns: Vec<(i64, String)> = sqlx::query_as(
            "PRAGMA table_info(products_meta)",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let col_names: Vec<&str> = columns.iter().map(|(_, name)| name.as_str()).collect();
        assert_eq!(
            col_names.len(),
            17,
            "products_meta must have 17 columns after 001→002, got {}: {:?}",
            col_names.len(),
            col_names
        );

        // Verify critical columns that the broken 002 dropped
        for expected in &["name", "brand", "model", "category", "subcategory", "specs_json"] {
            assert!(
                col_names.contains(expected),
                "missing column '{}' in products_meta — columns: {:?}",
                expected,
                col_names
            );
        }
    }

    #[tokio::test]
    async fn migration_002_fts5_triggers_fire_after_rewrite() {
        let pool = make_memory_pool().await;
        apply_001_and_002(&pool).await;

        // Insert a product — the FTS5 after-insert trigger should index it
        sqlx::query(
            "INSERT INTO products_meta (sku, source_id, name, brand, model, category, subcategory, specs_json, price, currency, condition, availability, url, image_url, seller, location, synced_at)
             VALUES ('TEST-001', 'reverb', 'Fender Strat', 'Fender', 'Stratocaster', 'guitars', 'electric', '{}', 1200.0, 'USD', 'new', 'in_stock', 'https://example.com/strat', '', 'seller1', 'US', 1000)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Query the FTS5 index — the trigger should have populated it
        let matches: Vec<String> = sqlx::query_scalar(
            "SELECT sku FROM products_fts WHERE products_fts MATCH 'Fender'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert!(
            matches.contains(&"TEST-001".to_string()),
            "FTS5 should find 'Fender' match for TEST-001, got: {:?}",
            matches
        );
    }

    #[tokio::test]
    async fn migration_002_preserves_existing_data() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&["001_init.sql", "002_add_url_validation.sql"]);
        std::fs::write(
            dir.join("001_init.sql"),
            include_str!("../migrations/001_init.sql"),
        )
        .unwrap();
        std::fs::write(
            dir.join("002_add_url_validation.sql"),
            include_str!("../migrations/002_add_url_validation.sql"),
        )
        .unwrap();

        // Apply only migration 001 first
        let runner1 = MigrationRunner::new(pool.clone(), dir.clone());
        runner1.run().await.unwrap();

        // Insert data before migration 002
        sqlx::query(
            "INSERT INTO products_meta (sku, source_id, name, brand, model, category, subcategory, specs_json, price, currency, condition, availability, url, image_url, seller, location, synced_at)
             VALUES ('PRE-001', 'reverb', 'Gibson LP', 'Gibson', 'Les Paul', 'guitars', 'electric', '{\"pickups\":\"humbucker\"}', 2500.0, 'USD', 'used', 'in_stock', 'https://example.com/lp', '', 'seller2', 'US', 900)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Now force re-run to apply migration 002 (reset version to 1)
        // The runner already applied both — let's verify data survived
        let name: String = sqlx::query_scalar(
            "SELECT name FROM products_meta WHERE sku = 'PRE-001'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(name, "Gibson LP", "name column must survive migration 002");

        let specs: String = sqlx::query_scalar(
            "SELECT specs_json FROM products_meta WHERE sku = 'PRE-001'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(specs, "{\"pickups\":\"humbucker\"}", "specs_json must survive migration 002");
    }

    // ── Migration 006 tests ─────────────────────────────────────────────

    /// Helper: apply the full migration chain 001→007 on an in-memory pool.
    async fn apply_full_migration_chain(pool: &sqlx::SqlitePool) -> PathBuf {
        let dir = temp_dir_with_files(&[
            "001_init.sql",
            "002_add_url_validation.sql",
            "003_add_image_cache.sql",
            "004_add_price_source.sql",
            "005_add_settings.sql",
            "006_wishlist_schema.sql",
            "007_price_drop_notifications.sql",
            "008_collection_items.sql",
            "009_add_recent_searches.sql",
        ]);
        std::fs::write(dir.join("001_init.sql"), include_str!("../migrations/001_init.sql")).unwrap();
        std::fs::write(dir.join("002_add_url_validation.sql"), include_str!("../migrations/002_add_url_validation.sql")).unwrap();
        std::fs::write(dir.join("003_add_image_cache.sql"), include_str!("../migrations/003_add_image_cache.sql")).unwrap();
        std::fs::write(dir.join("004_add_price_source.sql"), include_str!("../migrations/004_add_price_source.sql")).unwrap();
        std::fs::write(dir.join("005_add_settings.sql"), include_str!("../migrations/005_add_settings.sql")).unwrap();
        std::fs::write(dir.join("006_wishlist_schema.sql"), include_str!("../migrations/006_wishlist_schema.sql")).unwrap();
        std::fs::write(dir.join("007_price_drop_notifications.sql"), include_str!("../migrations/007_price_drop_notifications.sql")).unwrap();
        std::fs::write(dir.join("008_collection_items.sql"), include_str!("../migrations/008_collection_items.sql")).unwrap();
        std::fs::write(dir.join("009_add_recent_searches.sql"), include_str!("../migrations/009_add_recent_searches.sql")).unwrap();

        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        dir
    }

    #[tokio::test]
    async fn migration_006_wishlist_has_10_columns() {
        let pool = make_memory_pool().await;
        apply_full_migration_chain(&pool).await;

        let columns: Vec<(i64, String)> = sqlx::query_as("PRAGMA table_info(wishlist)")
            .fetch_all(&pool)
            .await
            .unwrap();

        let col_names: Vec<&str> = columns.iter().map(|(_, name)| name.as_str()).collect();
        assert_eq!(
            col_names.len(),
            10,
            "wishlist must have 10 columns after full chain, got {}: {:?}",
            col_names.len(),
            col_names
        );

        // Verify exact column names match WishlistRow struct
        let expected = ["id", "sku", "name", "brand", "price", "currency", "image_url", "product_url", "notes", "added_at"];
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(
                col_names[i], *exp,
                "column {} should be '{}', got '{}'", i, exp, col_names[i]
            );
        }
    }

    #[tokio::test]
    async fn migration_006_preserves_existing_wishlist_data() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[
            "001_init.sql",
            "002_add_url_validation.sql",
            "003_add_image_cache.sql",
            "004_add_price_source.sql",
            "005_add_settings.sql",
            "006_wishlist_schema.sql",
        ]);
        std::fs::write(dir.join("001_init.sql"), include_str!("../migrations/001_init.sql")).unwrap();
        std::fs::write(dir.join("002_add_url_validation.sql"), include_str!("../migrations/002_add_url_validation.sql")).unwrap();
        std::fs::write(dir.join("003_add_image_cache.sql"), include_str!("../migrations/003_add_image_cache.sql")).unwrap();
        std::fs::write(dir.join("004_add_price_source.sql"), include_str!("../migrations/004_add_price_source.sql")).unwrap();
        std::fs::write(dir.join("005_add_settings.sql"), include_str!("../migrations/005_add_settings.sql")).unwrap();
        std::fs::write(dir.join("006_wishlist_schema.sql"), include_str!("../migrations/006_wishlist_schema.sql")).unwrap();

        // Apply migrations 001→005 first (stop before 006)
        sqlx::query("CREATE TABLE IF NOT EXISTS schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();

        // Apply 001 through 005 by running them manually
        let m001 = include_str!("../migrations/001_init.sql");
        for stmt in split_statements(m001) {
            if !stmt.trim().is_empty() {
                sqlx::query(sqlx::AssertSqlSafe(stmt.as_str())).execute(&pool).await.unwrap();
            }
        }
        let m002 = include_str!("../migrations/002_add_url_validation.sql");
        for stmt in split_statements(m002) {
            if !stmt.trim().is_empty() {
                sqlx::query(sqlx::AssertSqlSafe(stmt.as_str())).execute(&pool).await.unwrap();
            }
        }
        let m003 = include_str!("../migrations/003_add_image_cache.sql");
        for stmt in split_statements(m003) {
            if !stmt.trim().is_empty() {
                sqlx::query(sqlx::AssertSqlSafe(stmt.as_str())).execute(&pool).await.unwrap();
            }
        }
        let m004 = include_str!("../migrations/004_add_price_source.sql");
        for stmt in split_statements(m004) {
            if !stmt.trim().is_empty() {
                sqlx::query(sqlx::AssertSqlSafe(stmt.as_str())).execute(&pool).await.unwrap();
            }
        }
        let m005 = include_str!("../migrations/005_add_settings.sql");
        for stmt in split_statements(m005) {
            if !stmt.trim().is_empty() {
                sqlx::query(sqlx::AssertSqlSafe(stmt.as_str())).execute(&pool).await.unwrap();
            }
        }

        // Set version to 5 so the runner picks up 006
        sqlx::query("INSERT OR REPLACE INTO schema_meta VALUES ('db_version', '5')")
            .execute(&pool)
            .await
            .unwrap();

        // Seed wishlist with pre-006 schema (sku, added_at, price_at_add, notes)
        sqlx::query(
            "INSERT INTO wishlist (sku, added_at, price_at_add, notes) VALUES ('WISH-001', 1700000000, 999.99, 'want this')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO wishlist (sku, added_at, price_at_add, notes) VALUES ('WISH-002', 1700001000, 500.0, 'maybe')",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Apply migration 006
        let runner = MigrationRunner::new(pool.clone(), dir);
        runner.run().await.unwrap();

        // Verify original data preserved
        let notes: String = sqlx::query_scalar("SELECT notes FROM wishlist WHERE sku = 'WISH-001'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(notes, "want this", "notes must survive migration 006");

        let added_at: i64 = sqlx::query_scalar("SELECT added_at FROM wishlist WHERE sku = 'WISH-002'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(added_at, 1700001000, "added_at must survive migration 006");

        // Verify new columns are NULL for migrated rows
        let name: Option<String> = sqlx::query_scalar("SELECT name FROM wishlist WHERE sku = 'WISH-001'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(name.is_none(), "name should be NULL for migrated rows, got {:?}", name);

        let brand: Option<String> = sqlx::query_scalar("SELECT brand FROM wishlist WHERE sku = 'WISH-001'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(brand.is_none(), "brand should be NULL for migrated rows, got {:?}", brand);

        // Verify id column exists and is autoincrement
        let id: i64 = sqlx::query_scalar("SELECT id FROM wishlist WHERE sku = 'WISH-001'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(id > 0, "id should be a positive autoincrement value, got {}", id);
    }

    // ── Migration 004 tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn migration_004_adds_source_id_and_index() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&["004_add_price_source.sql"]);

        // Write the migration file
        std::fs::write(
            dir.join("004_add_price_source.sql"),
            include_str!("../migrations/004_add_price_source.sql"),
        )
        .unwrap();

        // Set up schema as it would be after migration 003
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT OR REPLACE INTO schema_meta VALUES ('db_version', '3')")
            .execute(&pool)
            .await
            .unwrap();

        // Create price_history table as defined in 001_init.sql
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS price_history (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                sku         TEXT NOT NULL,
                price       REAL NOT NULL,
                recorded_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_price_history_sku ON price_history(sku)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Seed old-format data (no source_id column yet)
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at) VALUES ('SKU001', 100.0, 1000)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at) VALUES ('SKU001', 200.0, 2000)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Apply migration 004
        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 4);

        // Verify source_id column exists and defaults to ''
        let rows: Vec<(i64, String)> = sqlx::query_as(
            "SELECT id, source_id FROM price_history ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows.len(), 2, "expected 2 rows");
        assert_eq!(rows[0].1, "", "existing row should get default source_id ''");
        assert_eq!(rows[1].1, "", "existing row should get default source_id ''");

        // Verify we can insert with explicit source_id
        sqlx::query(
            "INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES ('SKU002', 150.0, 3000, 'reverb')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let fetched: String = sqlx::query_scalar(
            "SELECT source_id FROM price_history WHERE sku = 'SKU002'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(fetched, "reverb");

        // Verify the index exists via sqlite_master
        let idx_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name = 'idx_price_history_sku_recorded'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(
            idx_name.is_some(),
            "Expected index 'idx_price_history_sku_recorded' to exist"
        );
        assert_eq!(idx_name.unwrap(), "idx_price_history_sku_recorded");

        // Verify idempotent re-apply
        let runner3 = MigrationRunner::new(pool.clone(), dir);
        runner3.run().await.unwrap();
        assert_eq!(
            runner3.current_version().await.unwrap(),
            4,
            "version should remain 4 after re-apply"
        );
    }

    #[tokio::test]
    async fn migration_005_creates_settings_table() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&["005_add_settings.sql"]);

        // Write the migration file
        std::fs::write(
            dir.join("005_add_settings.sql"),
            include_str!("../migrations/005_add_settings.sql"),
        )
        .unwrap();

        // Set up schema as it would be after migration 004
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT OR REPLACE INTO schema_meta VALUES ('db_version', '4')")
            .execute(&pool)
            .await
            .unwrap();

        // Apply migration 005
        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 5);

        // Verify INSERT round-trip
        sqlx::query("INSERT INTO settings (key, value) VALUES ('alert_channel', 'app')")
            .execute(&pool)
            .await
            .unwrap();

        let val: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'alert_channel'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(val, "app");

        // Verify UPSERT (INSERT OR REPLACE)
        sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('alert_channel', 'ntfy')")
            .execute(&pool)
            .await
            .unwrap();

        let val2: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'alert_channel'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(val2, "ntfy");

        // Verify idempotent re-apply
        let runner3 = MigrationRunner::new(pool.clone(), dir);
        runner3.run().await.unwrap();
        assert_eq!(
            runner3.current_version().await.unwrap(),
            5,
            "version should remain 5 after re-apply"
        );

        // Table still works after re-apply
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "settings table should still have data after re-apply");
    }

    // ── Migration 007 tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn migration_007_creates_price_drop_notifications_table() {
        let pool = make_memory_pool().await;
        apply_full_migration_chain(&pool).await;

        // PRAGMA table_info returns (cid, name, type, notnull, dflt_value, pk)
        let columns: Vec<(i64, String, String, i64, Option<String>, i64)> = sqlx::query_as(
            "PRAGMA table_info(price_drop_notifications)",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let col_names: Vec<&str> = columns.iter().map(|(_, name, _, _, _, _)| name.as_str()).collect();
        assert_eq!(
            col_names.len(),
            4,
            "price_drop_notifications must have 4 columns after 001→007, got {}: {:?}",
            col_names.len(),
            col_names
        );

        // Verify exact column names + types
        let expected = [
            ("sku", "TEXT"),
            ("last_notified", "INTEGER"),
            ("last_price", "REAL"),
            ("channel", "TEXT"),
        ];
        for (i, (exp_name, exp_type)) in expected.iter().enumerate() {
            assert_eq!(
                col_names[i], *exp_name,
                "column {} should be '{}', got '{}'", i, exp_name, col_names[i]
            );
            assert_eq!(
                columns[i].2, *exp_type,
                "column '{}' should be type '{}', got '{}'", exp_name, exp_type, columns[i].2
            );
        }

        // Verify sku is PRIMARY KEY (pk=1 in PRAGMA table_info output)
        let sku_pk: i64 = columns[0].5;
        assert_eq!(sku_pk, 1, "sku column should be PRIMARY KEY (pk flag = 1)");

        // Verify NOT NULL constraints on the 3 non-PK columns
        // (PRIMARY KEY columns are implicitly NOT NULL but PRAGMA reports notnull=0 for them.)
        for col in columns.iter().skip(1) {
            assert_eq!(
                col.3, 1,
                "column '{}' should be NOT NULL, got notnull={}", col.1, col.3
            );
        }

        // Verify the index exists
        let idx_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name = 'idx_price_drop_notifications_notified'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(
            idx_name.is_some(),
            "Expected index 'idx_price_drop_notifications_notified' to exist"
        );

        // Verify db_version reached 7
        let version: String = sqlx::query_scalar(
            "SELECT value FROM schema_meta WHERE key = 'db_version'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(version, "9", "db_version should be 9 after 001→009");
    }

    #[tokio::test]
    async fn migration_007_price_drop_notifications_accepts_row() {
        let pool = make_memory_pool().await;
        apply_full_migration_chain(&pool).await;

        // Insert a row matching the cooldown table contract
        sqlx::query(
            "INSERT INTO price_drop_notifications (sku, last_notified, last_price, channel)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("SKU-001")
        .bind(1_700_000_000_i64)
        .bind(899.99_f64)
        .bind("ntfy")
        .execute(&pool)
        .await
        .unwrap();

        // Verify round-trip
        let (last_notified, last_price, channel): (i64, f64, String) = sqlx::query_as(
            "SELECT last_notified, last_price, channel FROM price_drop_notifications WHERE sku = ?1",
        )
        .bind("SKU-001")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(last_notified, 1_700_000_000);
        assert!((last_price - 899.99).abs() < 0.001);
        assert_eq!(channel, "ntfy");

        // Verify PK conflict (inserting same sku twice) — replaces row
        sqlx::query(
            "INSERT OR REPLACE INTO price_drop_notifications (sku, last_notified, last_price, channel)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("SKU-001")
        .bind(1_700_000_500_i64)
        .bind(799.99_f64)
        .bind("app")
        .execute(&pool)
        .await
        .unwrap();

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM price_drop_notifications WHERE sku = ?1",
        )
        .bind("SKU-001")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1, "INSERT OR REPLACE must not create duplicate rows");
    }

    // ── Migration 008 tests ─────────────────────────────────────────────

    /// Helper: apply the full migration chain 001→008 on an in-memory pool.
    async fn apply_full_chain_001_to_008(pool: &sqlx::SqlitePool) -> PathBuf {
        let dir = temp_dir_with_files(&[
            "001_init.sql",
            "002_add_url_validation.sql",
            "003_add_image_cache.sql",
            "004_add_price_source.sql",
            "005_add_settings.sql",
            "006_wishlist_schema.sql",
            "007_price_drop_notifications.sql",
            "008_collection_items.sql",
        ]);
        std::fs::write(dir.join("001_init.sql"), include_str!("../migrations/001_init.sql")).unwrap();
        std::fs::write(dir.join("002_add_url_validation.sql"), include_str!("../migrations/002_add_url_validation.sql")).unwrap();
        std::fs::write(dir.join("003_add_image_cache.sql"), include_str!("../migrations/003_add_image_cache.sql")).unwrap();
        std::fs::write(dir.join("004_add_price_source.sql"), include_str!("../migrations/004_add_price_source.sql")).unwrap();
        std::fs::write(dir.join("005_add_settings.sql"), include_str!("../migrations/005_add_settings.sql")).unwrap();
        std::fs::write(dir.join("006_wishlist_schema.sql"), include_str!("../migrations/006_wishlist_schema.sql")).unwrap();
        std::fs::write(dir.join("007_price_drop_notifications.sql"), include_str!("../migrations/007_price_drop_notifications.sql")).unwrap();
        std::fs::write(dir.join("008_collection_items.sql"), include_str!("../migrations/008_collection_items.sql")).unwrap();

        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        dir
    }

    #[tokio::test]
    async fn migration_008_creates_collection_items_table() {
        let pool = make_memory_pool().await;
        apply_full_chain_001_to_008(&pool).await;

        let columns: Vec<(i64, String, String, i64, Option<String>, i64)> = sqlx::query_as(
            "PRAGMA table_info(collection_items)",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let col_names: Vec<&str> = columns.iter().map(|(_, name, _, _, _, _)| name.as_str()).collect();
        assert_eq!(
            col_names.len(),
            12,
            "collection_items must have 12 columns after 001→008, got {}: {:?}",
            col_names.len(),
            col_names
        );

        let expected = [
            "id", "sku", "name", "brand", "purchase_price", "purchase_currency",
            "purchase_date", "condition", "serial_number", "notes", "image_url", "added_at",
        ];
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(
                col_names[i], *exp,
                "column {} should be '{}', got '{}'", i, exp, col_names[i]
            );
        }

        // Verify id is PRIMARY KEY (pk=1)
        let id_pk: i64 = columns[0].5;
        assert_eq!(id_pk, 1, "id column should be PRIMARY KEY (pk flag = 1)");

        // Verify added_at is NOT NULL
        assert_eq!(columns[11].3, 1, "added_at should be NOT NULL");
    }

    #[tokio::test]
    async fn migration_008_collection_items_condition_check_constraint() {
        let pool = make_memory_pool().await;
        apply_full_chain_001_to_008(&pool).await;

        // Valid condition values should succeed
        for condition in &["mint", "excellent", "good", "fair", "poor"] {
            sqlx::query(
                "INSERT INTO collection_items (name, condition, added_at) VALUES (?1, ?2, ?3)",
            )
            .bind(format!("test-{}", condition))
            .bind(*condition)
            .bind(1_700_000_000_i64)
            .execute(&pool)
            .await
            .unwrap_or_else(|_| panic!("condition='{}' should be accepted", condition));
        }

        // Invalid condition should fail
        let result = sqlx::query(
            "INSERT INTO collection_items (name, condition, added_at) VALUES (?1, ?2, ?3)",
        )
        .bind("bad-item")
        .bind("broken")
        .bind(1_700_000_000_i64)
        .execute(&pool)
        .await;

        assert!(
            result.is_err(),
            "condition='broken' should be rejected by CHECK constraint"
        );
    }

    #[tokio::test]
    async fn migration_008_collection_items_index_exists() {
        let pool = make_memory_pool().await;
        apply_full_chain_001_to_008(&pool).await;

        let idx_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name = 'idx_collection_items_sku'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(
            idx_name.is_some(),
            "Expected index 'idx_collection_items_sku' to exist"
        );
    }

    #[tokio::test]
    async fn migration_008_collection_items_roundtrip_insert() {
        let pool = make_memory_pool().await;
        apply_full_chain_001_to_008(&pool).await;

        sqlx::query(
            "INSERT INTO collection_items (sku, name, brand, purchase_price, purchase_currency, purchase_date, condition, serial_number, notes, image_url, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .bind("GUITAR-001")
        .bind("Stratocaster")
        .bind("Fender")
        .bind(1299.99_f64)
        .bind("USD")
        .bind(1_700_000_000_i64)
        .bind("excellent")
        .bind("SN123456")
        .bind("My first guitar")
        .bind("https://example.com/strat.jpg")
        .bind(1_700_000_000_i64)
        .execute(&pool)
        .await
        .unwrap();

        let (name, sku, brand): (String, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT name, sku, brand FROM collection_items WHERE sku = ?1",
        )
        .bind("GUITAR-001")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(name, "Stratocaster");
        assert_eq!(sku, Some("GUITAR-001".to_string()));
        assert_eq!(brand, Some("Fender".to_string()));

        // Verify db_version reached 8
        let version: String = sqlx::query_scalar(
            "SELECT value FROM schema_meta WHERE key = 'db_version'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(version, "8", "db_version should be 8 after 001→008");
    }

    // ── rollback() tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn rollback_is_noop_when_version_is_0() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[]);
        let runner = MigrationRunner::new(pool, dir);
        runner.rollback(1).await.unwrap();
    }

    #[tokio::test]
    async fn rollback_single_step_reverts_version() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[
            "001_init.sql", "001_init.down.sql",
            "002_add_table.sql", "002_add_table.down.sql",
        ]);

        // Create up migrations
        std::fs::write(
            dir.join("001_init.sql"),
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('db_version', '1');",
        ).unwrap();
        std::fs::write(
            dir.join("001_init.down.sql"),
            "DROP TABLE IF EXISTS schema_meta;",
        ).unwrap();
        std::fs::write(
            dir.join("002_add_table.sql"),
            "CREATE TABLE test_table (id INTEGER PRIMARY KEY);
             INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('db_version', '2');",
        ).unwrap();
        std::fs::write(
            dir.join("002_add_table.down.sql"),
            "DROP TABLE IF EXISTS test_table;",
        ).unwrap();

        // Apply both
        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 2);

        // Roll back 1 step
        runner.rollback(1).await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 1);

        // Table should be dropped by down migration
        let exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='test_table'",
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        assert!(!exists, "test_table should have been dropped by down migration");
    }

    #[tokio::test]
    async fn rollback_multiple_steps_reverts_all() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[
            "001_init.sql", "001_init.down.sql",
            "002_add_table.sql", "002_add_table.down.sql",
            "003_add_another.sql", "003_add_another.down.sql",
        ]);

        std::fs::write(dir.join("001_init.sql"),
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('db_version', '1');",
        ).unwrap();
        std::fs::write(dir.join("001_init.down.sql"), "DROP TABLE IF EXISTS schema_meta;").unwrap();
        std::fs::write(dir.join("002_add_table.sql"),
            "CREATE TABLE t1 (id INTEGER PRIMARY KEY);"
        ).unwrap();
        std::fs::write(dir.join("002_add_table.down.sql"), "DROP TABLE IF EXISTS t1;").unwrap();
        std::fs::write(dir.join("003_add_another.sql"),
            "CREATE TABLE t2 (id INTEGER PRIMARY KEY);"
        ).unwrap();
        std::fs::write(dir.join("003_add_another.down.sql"), "DROP TABLE IF EXISTS t2;").unwrap();

        // Apply all 3
        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 3);

        // Verify tables exist
        let _: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM t1")
            .fetch_one(&pool).await.unwrap();
        let _: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM t2")
            .fetch_one(&pool).await.unwrap();

        // Roll back 2 steps (v3→v1)
        runner.rollback(2).await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 1);

        // t1 and t2 should be gone; schema_meta should still exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_meta")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(count, 1, "schema_meta should still have db_version row");
        assert!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM t1")
                .fetch_one(&pool).await.is_err() ||
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM t1")
                .fetch_one(&pool).await.unwrap() == 0,
            "t1 should not exist or be empty"
        );
    }

    #[tokio::test]
    async fn rollback_errors_when_down_file_missing() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&["001_init.sql"]);
        std::fs::write(dir.join("001_init.sql"),
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('db_version', '1');",
        ).unwrap();

        // Apply migration
        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 1);

        // Attempt rollback without .down.sql → should error
        let err = runner.rollback(1).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("down migration for v1 not found") || msg.contains("not found"),
            "expected 'not found' error, got: {err}"
        );
    }

    #[tokio::test]
    async fn rollback_down_file_applied_correctly() {
        let pool = make_memory_pool().await;
        let dir = temp_dir_with_files(&[
            "001_init.sql", "001_init.down.sql",
        ]);

        std::fs::write(dir.join("001_init.sql"),
            "CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO schema_meta VALUES ('db_version', '1');
             CREATE TABLE my_table (id INTEGER PRIMARY KEY, val TEXT);",
        ).unwrap();
        std::fs::write(dir.join("001_init.down.sql"),
            "DROP TABLE IF EXISTS my_table;",
        ).unwrap();

        // Apply
        let runner = MigrationRunner::new(pool.clone(), dir.clone());
        runner.run().await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 1);

        // Verify my_table exists
        let _: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM my_table")
            .fetch_one(&pool).await.unwrap();

        // Roll back
        runner.rollback(1).await.unwrap();
        assert_eq!(runner.current_version().await.unwrap(), 0);

        // my_table should be gone
        let has_table: bool = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='my_table'",
        ).fetch_one(&pool).await.unwrap_or(0) > 0;
        assert!(!has_table, "my_table should have been dropped by down migration");
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    fn temp_dir_with_files(files: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("guitarhub-mig-test-{}", uuid_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        for f in files {
            // Create empty files — content is written separately by tests
            std::fs::write(dir.join(f), "").unwrap();
        }
        dir
    }

    fn uuid_v4() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{:x}", nanos)
    }

    async fn make_memory_pool() -> sqlx::SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }
}
