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
        let discovered = self.discover()?;
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
        // E.g., if current is v1 and only v3 is pending, v2 is missing → error.
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

            // ── Transaction start ──
            let mut tx = self.pool.begin().await.map_err(|e| {
                MigrationError::SqlError {
                    filename: migration.filename.clone(),
                    source: e,
                }
            })?;

            // Execute each statement in the migration file.
            for statement in split_statements(&sql) {
                let trimmed = statement.trim();
                if trimmed.is_empty() {
                    continue;
                }
                sqlx::query(trimmed)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| MigrationError::SqlError {
                        filename: migration.filename.clone(),
                        source: e,
                    })?;
            }

            // Update the schema version in schema_meta.
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

            // ── Transaction commit ──
            tx.commit().await.map_err(|e| MigrationError::SqlError {
                filename: migration.filename.clone(),
                source: e,
            })?;

            tracing::info!("Applied migration {}", migration.filename);
        }

        Ok(())
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
fn tokenize_sql(sql: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();

    for ch in sql.chars() {
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
