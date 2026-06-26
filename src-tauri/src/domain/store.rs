// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;

/// Supported authentication methods for external store connections.
#[derive(Debug, Clone, Serialize)]
pub enum StoreAuthType {
    #[serde(rename = "pat")]
    Pat,
}

/// Definition of a supported external store (e.g. Reverb, Guitar Center).
///
/// All fields are `&'static str` for zero-cost const storage in the registry.
#[derive(Debug, Clone, Serialize)]
pub struct StoreDef {
    pub id: &'static str,
    pub name: &'static str,
    pub auth_type: StoreAuthType,
    pub icon: &'static str,
    pub website: &'static str,
    pub token_url: &'static str,
}

/// A user-connected store account.
///
/// This is the domain representation returned to the frontend. The encrypted
/// token is NEVER included in this struct — it stays in the database.
#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub id: i64,
    pub store_id: String,
    pub label: String,
    pub username: Option<String>,
    pub connected_at: i64,
    pub synced_at: Option<i64>,
    pub is_active: bool,
}

/// An AES-256-GCM encrypted token stored as raw bytes.
///
/// Debug output is intentionally redacted to prevent accidental token leakage
/// in logs or error messages.
#[derive(Clone)]
pub struct EncryptedToken(pub Vec<u8>);

impl std::fmt::Debug for EncryptedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptedToken(REDACTED)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // ── StoreAuthType ────────────────────────────────────────────────────

    #[test]
    fn store_auth_type_pat_serializes_to_pat() {
        let value = serde_json::to_value(StoreAuthType::Pat).unwrap();
        assert_eq!(value, serde_json::json!("pat"));
    }

    // ── StoreDef ─────────────────────────────────────────────────────────

    #[test]
    fn store_def_clone_and_debug() {
        let def = StoreDef {
            id: "reverb",
            name: "Reverb",
            auth_type: StoreAuthType::Pat,
            icon: "reverb",
            website: "https://reverb.com",
            token_url: "https://reverb.com/settings/api",
        };
        let cloned = def.clone();
        assert_eq!(cloned.id, "reverb");
        assert_eq!(cloned.name, "Reverb");
        let _debug = format!("{:?}", cloned);
    }

    #[test]
    fn store_def_serializes_to_json() {
        let def = StoreDef {
            id: "reverb",
            name: "Reverb",
            auth_type: StoreAuthType::Pat,
            icon: "reverb",
            website: "https://reverb.com",
            token_url: "https://reverb.com/settings/api",
        };
        let json = serde_json::to_value(&def).unwrap();
        assert_eq!(json["id"], "reverb");
        assert_eq!(json["name"], "Reverb");
        assert_eq!(json["auth_type"], "pat");
        assert_eq!(json["icon"], "reverb");
        assert_eq!(json["website"], "https://reverb.com");
        assert_eq!(json["token_url"], "https://reverb.com/settings/api");
    }

    // ── Connection ───────────────────────────────────────────────────────

    #[test]
    fn connection_serializes_without_token() {
        let conn = Connection {
            id: 1,
            store_id: "reverb".to_string(),
            label: "My Reverb".to_string(),
            username: Some("@guitarist".to_string()),
            connected_at: 1_700_000_000,
            synced_at: Some(1_700_000_100),
            is_active: true,
        };
        let json = serde_json::to_value(&conn).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["store_id"], "reverb");
        assert_eq!(json["label"], "My Reverb");
        assert_eq!(json["username"], "@guitarist");
        assert_eq!(json["connected_at"], 1_700_000_000);
        assert_eq!(json["synced_at"], 1_700_000_100);
        assert_eq!(json["is_active"], true);
        // Make absolutely sure there is NO token_encrypted field
        assert!(
            !json.as_object().unwrap().contains_key("token_encrypted"),
            "Connection must NOT serialize token_encrypted"
        );
    }

    #[test]
    fn connection_clone_and_debug() {
        let conn = Connection {
            id: 42,
            store_id: "reverb".to_string(),
            label: "Test".to_string(),
            username: None,
            connected_at: 1_700_000_000,
            synced_at: None,
            is_active: false,
        };
        let cloned = conn.clone();
        assert_eq!(cloned.id, 42);
        let _debug = format!("{:?}", cloned);
    }

    // ── EncryptedToken ───────────────────────────────────────────────────

    #[test]
    fn encrypted_token_debug_redacts_content() {
        let token = EncryptedToken(vec![1, 2, 3, 4, 5]);
        let debug = format!("{:?}", token);
        assert_eq!(debug, "EncryptedToken(REDACTED)");
        assert!(
            !debug.contains(&[1, 2, 3, 4, 5].iter().map(|b| format!("{b}")).collect::<String>()),
            "Debug output must not contain raw bytes"
        );
    }

    #[test]
    fn encrypted_token_clone_produces_same_bytes() {
        let original = EncryptedToken(vec![10, 20, 30]);
        let cloned = original.clone();
        assert_eq!(cloned.0, vec![10, 20, 30]);
    }
}
