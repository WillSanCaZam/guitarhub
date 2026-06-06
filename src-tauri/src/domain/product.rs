// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

/// Current state of a catalog sync operation.
///
/// Matches the `sync_state.status` CHECK constraint in the SQL schema.
/// Used by the state machine in `CatalogSyncService` to track lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncState {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "downloading")]
    Downloading,
    #[serde(rename = "validating")]
    Validating,
    #[serde(rename = "sanitizing")]
    Sanitizing,
    #[serde(rename = "inserting")]
    Inserting,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "failed_network")]
    FailedNetwork,
    #[serde(rename = "failed_schema")]
    FailedSchema,
    #[serde(rename = "failed_db")]
    FailedDb,
}

impl SyncState {
    /// Returns `true` if this state represents an in-progress sync.
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            SyncState::Downloading
                | SyncState::Validating
                | SyncState::Sanitizing
                | SyncState::Inserting
        )
    }

    /// SQL-compatible string representation matching the schema CHECK constraint.
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncState::Idle => "idle",
            SyncState::Downloading => "downloading",
            SyncState::Validating => "validating",
            SyncState::Sanitizing => "sanitizing",
            SyncState::Inserting => "inserting",
            SyncState::Done => "done",
            SyncState::FailedNetwork => "failed_network",
            SyncState::FailedSchema => "failed_schema",
            SyncState::FailedDb => "failed_db",
        }
    }

    /// Parse a `SyncState` from its SQL representation.
    pub fn from_label(s: &str) -> Option<Self> {
        match s {
            "idle" => Some(SyncState::Idle),
            "downloading" => Some(SyncState::Downloading),
            "validating" => Some(SyncState::Validating),
            "sanitizing" => Some(SyncState::Sanitizing),
            "inserting" => Some(SyncState::Inserting),
            "done" => Some(SyncState::Done),
            "failed_network" => Some(SyncState::FailedNetwork),
            "failed_schema" => Some(SyncState::FailedSchema),
            "failed_db" => Some(SyncState::FailedDb),
            _ => None,
        }
    }
}

/// Filters for product search queries sent to `SearchService`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub source: Option<String>,
}

/// Sort order for product search results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    #[serde(rename = "relevance")]
    Relevance,
    #[serde(rename = "price_asc")]
    PriceAsc,
    #[serde(rename = "price_desc")]
    PriceDesc,
    #[serde(rename = "name_asc")]
    NameAsc,
    #[serde(rename = "name_desc")]
    NameDesc,
}

/// Paginated search result with total count and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub products: Vec<RawProduct>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
}

/// Top-level structure wrapping a catalog export from a source (e.g. Reverb).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogFile {
    pub schema_version: String,
    pub source_id: String,
    pub generated_at: String,
    pub run_id: String,
    pub products: Vec<RawProduct>,
}

/// A single product listing as ingested from a source catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawProduct {
    pub sku: String,
    pub name: String,
    pub brand: String,
    pub model: String,
    pub category: String,
    pub subcategory: String,
    pub price: f64,
    pub currency: String,
    pub condition: String,
    pub availability: String,
    pub url: String,
    pub image_url: String,
    #[serde(default)]
    pub specs_json: String,
    pub seller: String,
    pub location: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_catalog_file_with_products() {
        let json = r#"{
            "schema_version": "1.0",
            "source_id": "reverb",
            "generated_at": "2026-06-01T12:00:00Z",
            "run_id": "sync-001",
            "products": [
                {
                    "sku": "FENDER-STRAT-001",
                    "name": "Fender American Professional II Stratocaster",
                    "brand": "Fender",
                    "model": "American Professional II",
                    "category": "Electric Guitars",
                    "subcategory": "Solid Body",
                    "price": 1599.99,
                    "currency": "USD",
                    "condition": "new",
                    "availability": "in_stock",
                    "url": "https://reverb.com/item/fender-strat-001",
                    "image_url": "https://images.reverb.com/fender-strat.jpg",
                    "specs_json": "{\"body_wood\": \"alder\"}",
                    "seller": "Reverb Bazaar",
                    "location": "USA"
                }
            ]
        }"#;

        let catalog: CatalogFile = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(catalog.source_id, "reverb");
        assert_eq!(catalog.schema_version, "1.0");
        assert_eq!(catalog.run_id, "sync-001");
        assert_eq!(catalog.products.len(), 1);

        let product = &catalog.products[0];
        assert_eq!(product.sku, "FENDER-STRAT-001");
        assert_eq!(product.name, "Fender American Professional II Stratocaster");
        assert_eq!(product.price, 1599.99);
        assert_eq!(product.condition, "new");
        assert_eq!(product.url, "https://reverb.com/item/fender-strat-001");
        assert!(product.specs_json.contains("alder"));
    }

    #[test]
    fn deserialize_catalog_file_empty_products() {
        let json = r#"{
            "schema_version": "1.0",
            "source_id": "test-source",
            "generated_at": "2026-06-01T12:00:00Z",
            "run_id": "sync-002",
            "products": []
        }"#;

        let catalog: CatalogFile = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(catalog.source_id, "test-source");
        assert!(catalog.products.is_empty());
    }

    // ── SyncState ───────────────────────────────────────────────────────

    #[test]
    fn sync_state_is_running_returns_true_for_active_states() {
        assert!(SyncState::Downloading.is_running());
        assert!(SyncState::Validating.is_running());
        assert!(SyncState::Sanitizing.is_running());
        assert!(SyncState::Inserting.is_running());
    }

    #[test]
    fn sync_state_is_running_returns_false_for_terminal_states() {
        assert!(!SyncState::Idle.is_running());
        assert!(!SyncState::Done.is_running());
        assert!(!SyncState::FailedNetwork.is_running());
        assert!(!SyncState::FailedSchema.is_running());
        assert!(!SyncState::FailedDb.is_running());
    }

    #[test]
    fn sync_state_as_str_matches_schema_values() {
        assert_eq!(SyncState::Idle.as_str(), "idle");
        assert_eq!(SyncState::Downloading.as_str(), "downloading");
        assert_eq!(SyncState::Validating.as_str(), "validating");
        assert_eq!(SyncState::Sanitizing.as_str(), "sanitizing");
        assert_eq!(SyncState::Inserting.as_str(), "inserting");
        assert_eq!(SyncState::Done.as_str(), "done");
        assert_eq!(SyncState::FailedNetwork.as_str(), "failed_network");
        assert_eq!(SyncState::FailedSchema.as_str(), "failed_schema");
        assert_eq!(SyncState::FailedDb.as_str(), "failed_db");
    }

    #[test]
    fn sync_state_round_trips_through_from_str() {
        for state in &[
            SyncState::Idle,
            SyncState::Downloading,
            SyncState::Validating,
            SyncState::Sanitizing,
            SyncState::Inserting,
            SyncState::Done,
            SyncState::FailedNetwork,
            SyncState::FailedSchema,
            SyncState::FailedDb,
        ] {
            let s = state.as_str();
            let parsed = SyncState::from_label(s).unwrap_or_else(|| panic!("expected Some for {s}"));
            assert_eq!(&parsed, state);
        }
    }

    #[test]
    fn sync_state_serializes_to_schema_string() {
        assert_eq!(
            serde_json::to_value(SyncState::Downloading).unwrap(),
            serde_json::json!("downloading")
        );
        assert_eq!(
            serde_json::to_value(SyncState::FailedNetwork).unwrap(),
            serde_json::json!("failed_network")
        );
    }

    #[test]
    fn sync_state_deserializes_from_schema_string() {
        let state: SyncState = serde_json::from_str("\"downloading\"").unwrap();
        assert_eq!(state, SyncState::Downloading);

        let failed: SyncState = serde_json::from_str("\"failed_network\"").unwrap();
        assert_eq!(failed, SyncState::FailedNetwork);
    }

    // ── SearchFilters ───────────────────────────────────────────────────

    #[test]
    fn search_filters_defaults_to_all_none() {
        let filters = SearchFilters::default();
        assert!(filters.category.is_none());
        assert!(filters.price_min.is_none());
        assert!(filters.price_max.is_none());
        assert!(filters.source.is_none());
    }

    #[test]
    fn search_filters_round_trips_through_json() {
        let filters = SearchFilters {
            category: Some("Electric Guitars".into()),
            price_min: Some(100.0),
            price_max: Some(2000.0),
            source: Some("reverb".into()),
        };
        let json = serde_json::to_string(&filters).unwrap();
        let restored: SearchFilters = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.category.unwrap(), "Electric Guitars");
        assert!((restored.price_min.unwrap() - 100.0).abs() < f64::EPSILON);
        assert!((restored.price_max.unwrap() - 2000.0).abs() < f64::EPSILON);
        assert_eq!(restored.source.unwrap(), "reverb");
    }

    // ── SortOrder ───────────────────────────────────────────────────────

    #[test]
    fn sort_order_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_value(SortOrder::PriceAsc).unwrap(),
            serde_json::json!("price_asc")
        );
        assert_eq!(
            serde_json::to_value(SortOrder::NameDesc).unwrap(),
            serde_json::json!("name_desc")
        );
    }

    #[test]
    fn sort_order_deserializes_from_snake_case() {
        let order: SortOrder = serde_json::from_str("\"relevance\"").unwrap();
        assert_eq!(order, SortOrder::Relevance);
    }

    // ── SearchResult ────────────────────────────────────────────────────

    #[test]
    fn search_result_holds_products_and_metadata() {
        let result = SearchResult {
            products: vec![],
            total: 0,
            offset: 0,
            limit: 20,
        };
        assert!(result.products.is_empty());
        assert_eq!(result.total, 0);
        assert_eq!(result.limit, 20);
    }

    #[test]
    fn deserialize_raw_product_without_specs_json_defaults_to_empty() {
        let json = r#"{
            "sku": "TEST-001",
            "name": "Test Guitar",
            "brand": "TestBrand",
            "model": "TM-100",
            "category": "Electric Guitars",
            "subcategory": "Solid Body",
            "price": 999.99,
            "currency": "USD",
            "condition": "new",
            "availability": "in_stock",
            "url": "https://example.com/item",
            "image_url": "",
            "seller": "Test Seller",
            "location": "USA"
        }"#;

        let product: RawProduct = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(product.sku, "TEST-001");
        assert_eq!(product.specs_json, "");
    }
}
