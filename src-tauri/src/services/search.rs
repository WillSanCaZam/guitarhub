// SPDX-License-Identifier: GPL-3.0-or-later

use crate::domain::product::{RawProduct, SearchFilters, SearchResult, SortOrder};
use crate::AppError;
use sqlx::SqlitePool;

/// Sanitize user input for FTS5 MATCH queries.
///
/// Strips FTS5 operators (`AND`, `OR`, `NOT`, `NEAR`, `"`, `*`, `(`, `)`, `-`)
/// and wraps each remaining word (≥3 chars) in double quotes to prevent FTS5
/// injection and syntax errors.
///
/// Returns an empty string when no valid terms remain after sanitization.
pub fn sanitize_fts_input(query: &str) -> String {
    const KEYWORD_OPS: &[&str] = &["AND", "OR", "NOT", "NEAR"];

    query
        .split_whitespace()
        .filter_map(|token| {
            // Skip FTS5 keyword operators (case-insensitive, whole token)
            let upper = token.to_uppercase();
            if KEYWORD_OPS.contains(&upper.as_str()) {
                return None;
            }

            // Strip FTS5 special characters
            let clean: String = token
                .chars()
                .filter(|c| !matches!(c, '"' | '*' | '(' | ')' | '-'))
                .collect();

            // Minimum 3 characters (trigram tokenizer requirement)
            if clean.len() < 3 {
                return None;
            }

            Some(format!("\"{}\"", clean))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// FTS5-powered product search service with input sanitization,
/// dynamic WHERE clause generation, pagination, and sorting.
pub struct FtsSearchService {
    pool: SqlitePool,
}

impl FtsSearchService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Build the ORDER BY clause for the given sort order.
    fn order_by_clause(sort: &SortOrder) -> &'static str {
        match sort {
            SortOrder::Relevance => "ORDER BY fts.rank",
            SortOrder::PriceAsc => "ORDER BY m.price ASC",
            SortOrder::PriceDesc => "ORDER BY m.price DESC",
            SortOrder::NameAsc => "ORDER BY m.name ASC",
            SortOrder::NameDesc => "ORDER BY m.name DESC",
        }
    }

    /// Execute a full-text search across the FTS5 index with optional filters,
    /// pagination, and sorting.
    pub async fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
        sort: SortOrder,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResult, AppError> {
        // Validate input
        if query.len() < 3 {
            return Err(AppError::InvalidInput(
                "query too short".to_string(),
            ));
        }

        let sanitized = sanitize_fts_input(query);
        if sanitized.is_empty() {
            return Err(AppError::InvalidInput(
                "query too short".to_string(),
            ));
        }

        // Clamp page and page_size
        let limit = page_size.max(1).min(100) as i64;
        let offset = (page.saturating_sub(1) * page_size as u32) as i64;

        // ── Build SQL with sequential ? params ──────────────────────────

        // Count query — uses subquery for FTS match, then JOIN for filters
        let mut count_sql = String::from(
            "SELECT COUNT(*)
             FROM (
               SELECT rowid FROM products_fts WHERE products_fts MATCH ?
             ) fts
             JOIN products_meta m ON m.rowid = fts.rowid",
        );

        // Data query — subquery extracts rank for relevance ordering
        let mut data_sql = String::from(
            "SELECT m.sku, m.source_id, m.name, m.brand, m.model,
                    m.category, m.subcategory, m.price, m.currency,
                    m.condition, m.availability, m.url, m.image_url,
                    m.seller, m.location, m.synced_at
             FROM (
               SELECT rowid, rank FROM products_fts WHERE products_fts MATCH ?
             ) fts
             JOIN products_meta m ON m.rowid = fts.rowid",
        );

        // Build WHERE clauses — using simple ? (sequential, no numbers)
        if filters.category.is_some() {
            count_sql.push_str(" AND m.category = ?");
            data_sql.push_str(" AND m.category = ?");
        }
        if filters.price_min.is_some() {
            count_sql.push_str(" AND m.price >= ?");
            data_sql.push_str(" AND m.price >= ?");
        }
        if filters.price_max.is_some() {
            count_sql.push_str(" AND m.price <= ?");
            data_sql.push_str(" AND m.price <= ?");
        }
        if filters.source.is_some() {
            count_sql.push_str(" AND m.source_id = ?");
            data_sql.push_str(" AND m.source_id = ?");
        }

        // Add ORDER BY and LIMIT/OFFSET to data query
        let order_by = Self::order_by_clause(&sort);
        data_sql.push_str(&format!(" {} LIMIT ? OFFSET ?", order_by));

        // ── Execute count query ─────────────────────────────────────────
        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql).bind(&sanitized);
        if let Some(ref category) = filters.category {
            count_query = count_query.bind(category);
        }
        if let Some(price_min) = filters.price_min {
            count_query = count_query.bind(price_min);
        }
        if let Some(price_max) = filters.price_max {
            count_query = count_query.bind(price_max);
        }
        if let Some(ref source) = filters.source {
            count_query = count_query.bind(source);
        }

        let (total,): (i64,) = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // ── Execute data query ──────────────────────────────────────────
        let mut data_query =
            sqlx::query_as::<_, RawProductRow>(&data_sql).bind(&sanitized);
        if let Some(ref category) = filters.category {
            data_query = data_query.bind(category);
        }
        if let Some(price_min) = filters.price_min {
            data_query = data_query.bind(price_min);
        }
        if let Some(price_max) = filters.price_max {
            data_query = data_query.bind(price_max);
        }
        if let Some(ref source) = filters.source {
            data_query = data_query.bind(source);
        }
        data_query = data_query.bind(limit).bind(offset);

        let rows: Vec<RawProductRow> = data_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let products: Vec<RawProduct> = rows
            .into_iter()
            .map(|r| RawProduct {
                sku: r.sku,
                name: r.name,
                brand: r.brand,
                model: r.model,
                category: r.category,
                subcategory: r.subcategory,
                price: r.price,
                currency: r.currency,
                condition: r.condition,
                availability: r.availability,
                url: r.url,
                image_url: r.image_url,
                specs_json: String::new(),
                seller: r.seller,
                location: r.location,
            })
            .collect();

        Ok(SearchResult {
            products,
            total: total as u64,
            offset: offset as u32,
            limit: limit as u32,
        })
    }
}

/// Internal row type for sqlx query_as deserialization from products_meta.
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct RawProductRow {
    sku: String,
    source_id: String,
    name: String,
    brand: String,
    model: String,
    category: String,
    subcategory: String,
    price: f64,
    currency: String,
    condition: String,
    availability: String,
    url: String,
    image_url: String,
    seller: String,
    location: String,
    synced_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    // ── Helpers ─────────────────────────────────────────────────────────

    /// Create an in-memory pool with products_meta and products_fts tables.
    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory pool");

        // Create schema_meta (needed for FTS5 content table ref)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create products_meta (the content table for FTS5)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS products_meta (
                sku          TEXT PRIMARY KEY,
                source_id    TEXT NOT NULL,
                name         TEXT NOT NULL DEFAULT '',
                brand        TEXT NOT NULL DEFAULT '',
                model        TEXT NOT NULL DEFAULT '',
                category     TEXT NOT NULL DEFAULT '',
                subcategory  TEXT NOT NULL DEFAULT '',
                specs_json   TEXT NOT NULL DEFAULT '{}',
                price        REAL,
                currency     TEXT,
                condition    TEXT CHECK(condition IN ('new','used','refurbished','unknown')),
                availability TEXT CHECK(availability IN ('in_stock','out_of_stock','unknown')),
                url          TEXT NOT NULL CHECK(url LIKE 'https://%'),
                image_url    TEXT CHECK(image_url = '' OR image_url LIKE 'https://%'),
                seller       TEXT,
                location     TEXT,
                synced_at    INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create FTS5 virtual table
        sqlx::query(
            "CREATE VIRTUAL TABLE IF NOT EXISTS products_fts USING fts5(
                sku, source_id, name, brand, model, category, subcategory, specs_json,
                tokenize = 'trigram',
                content = 'products_meta',
                content_rowid = 'rowid'
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create FTS sync triggers
        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS products_fts_ai AFTER INSERT ON products_meta BEGIN
                INSERT INTO products_fts(rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
                VALUES (new.rowid, new.sku, new.source_id, new.name, new.brand, new.model, new.category, new.subcategory, new.specs_json);
            END",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS products_fts_ad AFTER DELETE ON products_meta BEGIN
                INSERT INTO products_fts(products_fts, rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
                VALUES ('delete', old.rowid, old.sku, old.source_id, old.name, old.brand, old.model, old.category, old.subcategory, old.specs_json);
            END",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS products_fts_au AFTER UPDATE ON products_meta BEGIN
                INSERT INTO products_fts(products_fts, rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
                VALUES ('delete', old.rowid, old.sku, old.source_id, old.name, old.brand, old.model, old.category, old.subcategory, old.specs_json);
                INSERT INTO products_fts(rowid, sku, source_id, name, brand, model, category, subcategory, specs_json)
                VALUES (new.rowid, new.sku, new.source_id, new.name, new.brand, new.model, new.category, new.subcategory, new.specs_json);
            END",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    /// Insert a product into products_meta for test seed data.
    async fn insert_product(
        pool: &SqlitePool,
        sku: &str,
        name: &str,
        brand: &str,
        category: &str,
        price: f64,
        source_id: &str,
    ) {
        let synced_at = 1000i64;
        sqlx::query(
            r#"INSERT INTO products_meta
               (sku, source_id, name, brand, model, category, subcategory,
                price, currency, condition, availability, url, image_url,
                seller, location, synced_at)
               VALUES (?1, ?2, ?3, ?4, '', ?5, '', ?6, 'USD', 'new', 'in_stock',
                       'https://example.com/' || ?1, '', 'Test Seller', 'USA', ?7)"#,
        )
        .bind(sku)
        .bind(source_id)
        .bind(name)
        .bind(brand)
        .bind(category)
        .bind(price)
        .bind(synced_at)
        .execute(pool)
        .await
        .unwrap();
    }

    // ── Test: sanitize_fts_input ────────────────────────────────────────

    #[test]
    fn sanitize_removes_double_quotes_and_wraps_terms() {
        let result = sanitize_fts_input(r#"guitar "electric""#);
        assert_eq!(result, r#""guitar" "electric""#);
    }

    #[test]
    fn sanitize_removes_fts5_operators_and_keeps_valid_terms() {
        let result = sanitize_fts_input("guitar NOT bass");
        assert_eq!(result, r#""guitar" "bass""#);
    }

    #[test]
    fn sanitize_handles_unicode_trigram_input() {
        let result = sanitize_fts_input("吉他");
        assert_eq!(result, r#""吉他""#);
    }

    #[test]
    fn sanitize_removes_all_fts5_special_chars() {
        let result = sanitize_fts_input(r#"Fender "Stratocaster" (USA) -electric"#);
        assert_eq!(result, r#""Fender" "Stratocaster" "USA" "electric""#);
    }

    #[test]
    fn sanitize_filters_short_terms_below_3_chars() {
        let result = sanitize_fts_input("a an the cat");
        assert_eq!(result, r#""the" "cat""#);
    }

    #[test]
    fn sanitize_returns_empty_for_all_operators() {
        let result = sanitize_fts_input("AND OR NOT NEAR");
        assert_eq!(result, "");
    }

    #[test]
    fn sanitize_returns_empty_for_empty_input() {
        let result = sanitize_fts_input("");
        assert_eq!(result, "");
    }

    // ── Test: search with category filter ───────────────────────────────

    #[tokio::test]
    async fn search_filters_by_category() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-001", "Fender Strat", "Fender", "Electric Guitars", 999.99, "reverb").await;
        insert_product(&pool, "SKU-002", "Precision Bass", "Fender", "Bass Guitars", 799.99, "reverb").await;

        let filters = SearchFilters {
            category: Some("Electric Guitars".into()),
            price_min: None,
            price_max: None,
            source: None,
        };
        let result = svc.search("guitar", &filters, SortOrder::Relevance, 1, 20).await.unwrap();
        assert_eq!(result.total, 1, "expected 1 product in Electric Guitars");
        assert_eq!(result.products.len(), 1);
        assert_eq!(result.products[0].sku, "SKU-001");
    }

    // ── Test: search with price range filter ────────────────────────────

    #[tokio::test]
    async fn search_filters_by_price_range() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-100", "Cheap Guitar", "Yamaha", "Electric Guitars", 100.0, "reverb").await;
        insert_product(&pool, "SKU-200", "Mid Guitar", "Fender", "Electric Guitars", 200.0, "reverb").await;
        insert_product(&pool, "SKU-500", "Expensive Guitar", "Gibson", "Electric Guitars", 500.0, "reverb").await;

        let filters = SearchFilters {
            category: None,
            price_min: Some(150.0),
            price_max: Some(400.0),
            source: None,
        };
        let result = svc.search("guitar", &filters, SortOrder::Relevance, 1, 20).await.unwrap();
        assert_eq!(result.total, 1, "expected 1 product in price range 150-400");
        assert_eq!(result.products.len(), 1);
        assert_eq!(result.products[0].sku, "SKU-200");
    }

    // ── Test: search with source filter ─────────────────────────────────

    #[tokio::test]
    async fn search_filters_by_source() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-R1", "Reverb Guitar", "Fender", "Electric Guitars", 999.99, "reverb").await;
        insert_product(&pool, "SKU-E1", "eBay Guitar", "Gibson", "Electric Guitars", 899.99, "ebay").await;

        let filters = SearchFilters {
            category: None,
            price_min: None,
            price_max: None,
            source: Some("reverb".into()),
        };
        let result = svc.search("guitar", &filters, SortOrder::Relevance, 1, 20).await.unwrap();
        assert_eq!(result.total, 1, "expected 1 product from reverb");
        assert_eq!(result.products.len(), 1);
        assert_eq!(result.products[0].sku, "SKU-R1");
    }

    // ── Test: search with combined filters ──────────────────────────────

    #[tokio::test]
    async fn search_combines_all_filters() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-C1", "Fender Jazz Bass", "Fender", "Bass Guitars", 1200.0, "reverb").await;
        insert_product(&pool, "SKU-C2", "Fender Strat", "Fender", "Electric Guitars", 1500.0, "reverb").await;

        let filters = SearchFilters {
            category: Some("Electric Guitars".into()),
            price_min: Some(1000.0),
            price_max: Some(2000.0),
            source: Some("reverb".into()),
        };
        let result = svc.search("fender", &filters, SortOrder::Relevance, 1, 20).await.unwrap();
        assert_eq!(result.total, 1, "expected 1 product matching all filters");
        assert_eq!(result.products[0].sku, "SKU-C2");
    }

    // ── Test: pagination ────────────────────────────────────────────────

    #[tokio::test]
    async fn search_paginates_first_page() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        for i in 1..=10 {
            let sku = format!("SKU-P{:03}", i);
            insert_product(&pool, &sku, &format!("Product {}", i), "Brand", "Electric Guitars", i as f64 * 100.0, "reverb").await;
        }

        let filters = SearchFilters::default();
        let result = svc.search("product", &filters, SortOrder::NameAsc, 1, 3).await.unwrap();
        assert_eq!(result.total, 10, "total should be 10");
        assert_eq!(result.products.len(), 3, "first page should have 3 products");
    }

    #[tokio::test]
    async fn search_paginates_second_page() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        for i in 1..=10 {
            let sku = format!("SKU-P{:03}", i);
            insert_product(&pool, &sku, &format!("Product {}", i), "Brand", "Electric Guitars", i as f64 * 100.0, "reverb").await;
        }

        let filters = SearchFilters::default();
        let result = svc.search("product", &filters, SortOrder::NameAsc, 2, 3).await.unwrap();
        assert_eq!(result.total, 10, "total should still be 10");
        assert_eq!(result.products.len(), 3, "second page should have 3 products");
        // Verify offset
        assert_eq!(result.offset, 3);
    }

    #[tokio::test]
    async fn search_paginates_beyond_results() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        for i in 1..=5 {
            let sku = format!("SKU-P{:03}", i);
            insert_product(&pool, &sku, &format!("Product {}", i), "Brand", "Electric Guitars", i as f64 * 100.0, "reverb").await;
        }

        let filters = SearchFilters::default();
        let result = svc.search("product", &filters, SortOrder::NameAsc, 10, 10).await.unwrap();
        assert_eq!(result.total, 5, "total should still be 5");
        assert_eq!(result.products.len(), 0, "page beyond results should have 0 products");
    }

    // ── Test: all SortOrder variants ────────────────────────────────────

    #[tokio::test]
    async fn search_sorts_by_price_ascending() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-S3", "C Guitar", "Brand", "Guitars", 300.0, "reverb").await;
        insert_product(&pool, "SKU-S1", "A Guitar", "Brand", "Guitars", 100.0, "reverb").await;
        insert_product(&pool, "SKU-S2", "B Guitar", "Brand", "Guitars", 200.0, "reverb").await;

        let filters = SearchFilters::default();
        let result = svc.search("guitar", &filters, SortOrder::PriceAsc, 1, 20).await.unwrap();
        assert_eq!(result.products.len(), 3);
        assert_eq!(result.products[0].sku, "SKU-S1");
        assert_eq!(result.products[1].sku, "SKU-S2");
        assert_eq!(result.products[2].sku, "SKU-S3");
    }

    #[tokio::test]
    async fn search_sorts_by_price_descending() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-D1", "A Guitar", "Brand", "Guitars", 100.0, "reverb").await;
        insert_product(&pool, "SKU-D3", "C Guitar", "Brand", "Guitars", 300.0, "reverb").await;
        insert_product(&pool, "SKU-D2", "B Guitar", "Brand", "Guitars", 200.0, "reverb").await;

        let filters = SearchFilters::default();
        let result = svc.search("guitar", &filters, SortOrder::PriceDesc, 1, 20).await.unwrap();
        assert_eq!(result.products.len(), 3);
        assert_eq!(result.products[0].sku, "SKU-D3");
        assert_eq!(result.products[1].sku, "SKU-D2");
        assert_eq!(result.products[2].sku, "SKU-D1");
    }

    #[tokio::test]
    async fn search_sorts_by_name_ascending() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-N2", "B Guitar", "Brand", "Guitars", 200.0, "reverb").await;
        insert_product(&pool, "SKU-N1", "A Guitar", "Brand", "Guitars", 100.0, "reverb").await;
        insert_product(&pool, "SKU-N3", "C Guitar", "Brand", "Guitars", 300.0, "reverb").await;

        let filters = SearchFilters::default();
        let result = svc.search("guitar", &filters, SortOrder::NameAsc, 1, 20).await.unwrap();
        assert_eq!(result.products.len(), 3);
        assert_eq!(result.products[0].sku, "SKU-N1");
        assert_eq!(result.products[1].sku, "SKU-N2");
        assert_eq!(result.products[2].sku, "SKU-N3");
    }

    #[tokio::test]
    async fn search_sorts_by_name_descending() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-M1", "A Guitar", "Brand", "Guitars", 100.0, "reverb").await;
        insert_product(&pool, "SKU-M2", "B Guitar", "Brand", "Guitars", 200.0, "reverb").await;
        insert_product(&pool, "SKU-M3", "C Guitar", "Brand", "Guitars", 300.0, "reverb").await;

        let filters = SearchFilters::default();
        let result = svc.search("guitar", &filters, SortOrder::NameDesc, 1, 20).await.unwrap();
        assert_eq!(result.products.len(), 3);
        assert_eq!(result.products[0].sku, "SKU-M3");
        assert_eq!(result.products[1].sku, "SKU-M2");
        assert_eq!(result.products[2].sku, "SKU-M1");
    }

    // ── Test: empty results ─────────────────────────────────────────────

    #[tokio::test]
    async fn search_returns_empty_for_no_match() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-E1", "Fender Strat", "Fender", "Electric Guitars", 999.99, "reverb").await;

        let filters = SearchFilters::default();
        let result = svc.search("xyznonexistent", &filters, SortOrder::Relevance, 1, 20).await.unwrap();
        assert_eq!(result.total, 0, "total should be 0 for no match");
        assert!(result.products.is_empty());
    }

    #[tokio::test]
    async fn search_returns_empty_when_no_products_exist() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        let filters = SearchFilters::default();
        let result = svc.search("guitar", &filters, SortOrder::Relevance, 1, 20).await.unwrap();
        assert_eq!(result.total, 0);
        assert!(result.products.is_empty());
    }

    // ── Test: input validation ──────────────────────────────────────────

    #[tokio::test]
    async fn search_rejects_empty_query() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        let filters = SearchFilters::default();
        let err = svc.search("", &filters, SortOrder::Relevance, 1, 20).await.unwrap_err();
        assert!(
            err.to_string().contains("query too short"),
            "Expected 'query too short', got: {err}"
        );
    }

    #[tokio::test]
    async fn search_rejects_short_query() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        let filters = SearchFilters::default();
        let err = svc.search("ab", &filters, SortOrder::Relevance, 1, 20).await.unwrap_err();
        assert!(
            err.to_string().contains("query too short"),
            "Expected 'query too short', got: {err}"
        );
    }

    #[tokio::test]
    async fn search_rejects_query_that_sanitizes_to_empty() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        let filters = SearchFilters::default();
        let err = svc.search("AND OR NOT", &filters, SortOrder::Relevance, 1, 20).await.unwrap_err();
        assert!(
            err.to_string().contains("query too short"),
            "Expected 'query too short' after all operators stripped, got: {err}"
        );
    }

    // ── Test: pagination boundary (limit clamping) ──────────────────────

    #[tokio::test]
    async fn search_clamps_page_size_minimum() {
        let pool = setup_db().await;
        let svc = FtsSearchService::new(pool.clone());

        insert_product(&pool, "SKU-B1", "Test Guitar", "Brand", "Guitars", 100.0, "reverb").await;

        let filters = SearchFilters::default();
        let result = svc.search("guitar", &filters, SortOrder::Relevance, 1, 0).await.unwrap();
        // page_size 0 clamped to 1
        assert_eq!(result.limit, 1);
    }
}
