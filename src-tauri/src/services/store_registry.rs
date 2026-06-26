// SPDX-License-Identifier: GPL-3.0-or-later

use crate::domain::store::{StoreAuthType, StoreDef};

/// All supported store definitions, hardcoded at compile time for zero
/// runtime cost and type safety.
pub static STORES: &[StoreDef] = &[StoreDef {
    id: "reverb",
    name: "Reverb",
    auth_type: StoreAuthType::Pat,
    icon: "reverb",
    website: "https://reverb.com",
    token_url: "https://reverb.com/settings/api",
}];

/// Look up a store definition by its string ID.
///
/// Returns `None` if no store with that ID exists in the registry.
pub fn by_id(id: &str) -> Option<&'static StoreDef> {
    STORES.iter().find(|s| s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── stores() ─────────────────────────────────────────────────────────

    #[test]
    fn stores_returns_reverb_as_first_entry() {
        let all = STORES;
        assert_eq!(all.len(), 1, "expected exactly 1 store in registry");
        assert_eq!(all[0].id, "reverb");
        assert_eq!(all[0].name, "Reverb");
        assert!(matches!(all[0].auth_type, StoreAuthType::Pat));
        assert_eq!(all[0].icon, "reverb");
        assert_eq!(all[0].website, "https://reverb.com");
        assert_eq!(all[0].token_url, "https://reverb.com/settings/api");
    }

    #[test]
    fn stores_each_def_has_required_fields() {
        for store in STORES {
            assert!(!store.id.is_empty(), "store id must not be empty");
            assert!(!store.name.is_empty(), "store name must not be empty");
            assert!(!store.icon.is_empty(), "store icon must not be empty");
            assert!(!store.website.is_empty(), "store website must not be empty");
            assert!(!store.token_url.is_empty(), "store token_url must not be empty");
        }
    }

    // ── by_id() ──────────────────────────────────────────────────────────

    #[test]
    fn by_id_returns_reverb_for_reverb() {
        let def = by_id("reverb");
        assert!(def.is_some(), "expected Some for 'reverb'");
        assert_eq!(def.unwrap().name, "Reverb");
    }

    #[test]
    fn by_id_returns_none_for_unknown_store() {
        let def = by_id("ebay");
        assert!(def.is_none(), "expected None for 'ebay'");
    }

    #[test]
    fn by_id_returns_none_for_empty_string() {
        let def = by_id("");
        assert!(def.is_none(), "expected None for empty string");
    }
}
