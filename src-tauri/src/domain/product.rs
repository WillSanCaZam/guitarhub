use serde::{Deserialize, Serialize};

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
