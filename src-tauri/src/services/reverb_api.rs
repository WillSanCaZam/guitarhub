// SPDX-License-Identifier: GPL-3.0-or-later

use crate::AppError;
use reqwest::Client;
use serde::Deserialize;

const REVERB_BASE_URL: &str = "https://api.reverb.com";

/// Account response from `GET /api/my/account`.
#[derive(Debug, Deserialize)]
struct AccountResponse {
    shop: Option<ShopInfo>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ShopInfo {
    name: Option<String>,
}

/// A single listing from the Reverb API.
#[derive(Debug, Clone, Deserialize)]
pub struct ReverbListing {
    pub id: i64,
    pub title: String,
    pub price: ReverbPrice,
    pub condition: Option<ReverbCondition>,
    #[serde(default)]
    pub photos: Vec<ReverbPhoto>,
    pub _links: ReverbLinks,
    pub state: Option<String>,
    pub listing_strategy: Option<String>,
    pub inventory_type: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub finish: Option<String>,
    pub year: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverbPrice {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverbCondition {
    pub uuid: Option<String>,
    pub display_name: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverbPhoto {
    pub _links: ReverbPhotoLinks,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverbPhotoLinks {
    #[serde(default)]
    pub small: Option<ReverbHref>,
    #[serde(default)]
    pub full: Option<ReverbHref>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverbLinks {
    pub web: Option<ReverbHref>,
    pub next: Option<ReverbNext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverbHref {
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReverbNext {
    pub href: Option<String>,
}

/// Paginated response from `GET /api/my/listings`.
#[derive(Debug, Deserialize)]
pub struct ReverbListingsResponse {
    pub listings: Vec<ReverbListing>,
    pub _links: ReverbListingsLinks,
    pub total: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ReverbListingsLinks {
    pub next: Option<ReverbNext>,
}

/// Client for the Reverb API with configurable base URL (overridable for tests).
pub struct ReverbApiClient {
    http_client: Client,
    base_url: String,
}

impl ReverbApiClient {
    /// Create a new Reverb API client pointing at the production API.
    pub fn new(http_client: Client) -> Self {
        Self {
            http_client,
            base_url: REVERB_BASE_URL.to_string(),
        }
    }

    /// Create a new Reverb API client with a custom base URL (for testing).
    pub fn new_with_url(http_client: Client, base_url: String) -> Self {
        Self {
            http_client,
            base_url,
        }
    }

    /// Validate a Personal Access Token against the Reverb API.
    ///
    /// Calls `GET /api/my/account` with Bearer auth.
    /// Returns the shop name if available, otherwise the email username.
    ///
    /// # Errors
    ///
    /// - `AppError::TokenInvalid` on 401
    /// - `AppError::RateLimited` on 429
    /// - `AppError::Network` on other HTTP/connection errors
    pub async fn validate_token(&self, token: &str) -> Result<String, AppError> {
        let url = format!("{}/api/my/account", self.base_url);
        let response = self
            .http_client
            .get(&url)
            .header("Accept-Version", "3.0")
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        match response.status().as_u16() {
            200 => {
                let account: AccountResponse = response.json().await.map_err(|e| {
                    AppError::InvalidInput(format!("failed to parse account response: {e}"))
                })?;

                // Return shop name, then email local part, then fallback
                if let Some(shop) = account.shop {
                    if let Some(name) = shop.name {
                        if !name.is_empty() {
                            return Ok(name);
                        }
                    }
                }
                if let Some(email) = account.email {
                    if let Some(local) = email.split('@').next() {
                        if !local.is_empty() {
                            return Ok(local.to_string());
                        }
                    }
                }
                Ok("connected".to_string())
            }
            401 => Err(AppError::TokenInvalid),
            429 => Err(AppError::RateLimited),
            status => Err(AppError::Network(format!("HTTP {status}"))),
        }
    }

    /// Fetch a page of user listings from the Reverb API.
    ///
    /// Returns the listings response with pagination metadata.
    ///
    /// # Errors
    ///
    /// - `AppError::TokenInvalid` on 401
    /// - `AppError::RateLimited` on 429
    /// - `AppError::Network` on other HTTP/connection errors
    pub async fn fetch_listings(
        &self,
        token: &str,
        page: u32,
    ) -> Result<ReverbListingsResponse, AppError> {
        let url = format!(
            "{}/api/my/listings?page={}&per=50",
            self.base_url, page
        );

        let response = self
            .http_client
            .get(&url)
            .header("Accept-Version", "3.0")
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        match response.status().as_u16() {
            200 => Ok(response.json().await.map_err(|e| {
                AppError::InvalidInput(format!("failed to parse listings response: {e}"))
            })?),
            401 => Err(AppError::TokenInvalid),
            429 => Err(AppError::RateLimited),
            status => Err(AppError::Network(format!("HTTP {status}"))),
        }
    }
}

/// Normalize a Reverb condition slug to the 4-value vocabulary.
pub fn normalize_reverb_condition(condition: &ReverbCondition) -> &'static str {
    match condition.slug.as_deref() {
        Some("excellent" | "very_good" | "good" | "fair" | "poor") => "used",
        Some("mint" | "brand_new") => "new",
        Some("non_functioning") => "used",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use reqwest::Client;

    // ── validate_token ───────────────────────────────────────────────────

    #[tokio::test]
    async fn validate_token_returns_shop_name_on_200() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/my/account")
                .header("Authorization", "Bearer pat_valid_123")
                .header("Accept-Version", "3.0");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(r#"{"shop": {"name": "@guitarist"}}"#);
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.validate_token("pat_valid_123").await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
        assert_eq!(result.unwrap(), "@guitarist");
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn validate_token_returns_email_when_no_shop() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/my/account")
                .header("Authorization", "Bearer pat_email_test");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(r#"{"email": "user@example.com"}"#);
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.validate_token("pat_email_test").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "user");
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn validate_token_returns_connected_fallback() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/my/account");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(r#"{}"#);
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.validate_token("pat_fallback").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "connected");
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn validate_token_returns_401_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/my/account");
            then.status(401);
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.validate_token("bad_token").await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), AppError::TokenInvalid),
            "expected TokenInvalid"
        );
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn validate_token_returns_429_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/my/account");
            then.status(429);
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.validate_token("pat").await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), AppError::RateLimited),
            "expected RateLimited"
        );
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn validate_token_returns_500_as_network_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/my/account");
            then.status(500);
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.validate_token("pat").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(&err, AppError::Network(msg) if msg.contains("HTTP 500")),
            "expected Network(HTTP 500), got: {err}"
        );
        mock.assert_calls(1);
    }

    // ── fetch_listings ───────────────────────────────────────────────────

    #[tokio::test]
    async fn fetch_listings_returns_listings_on_200() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/my/listings")
                .query_param("page", "1")
                .query_param("per", "50")
                .header("Authorization", "Bearer pat_valid")
                .header("Accept-Version", "3.0");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(
                    r#"{
                        "listings": [
                            {
                                "id": 123,
                                "title": "Fender Stratocaster",
                                "price": {"amount": 1599.99, "currency": "USD"},
                                "condition": {"slug": "excellent", "display_name": "Excellent"},
                                "photos": [{"_links": {"small": {"href": "https://img.example.com/strat.jpg"}}}],
                                "_links": {"web": {"href": "https://reverb.com/item/123"}},
                                "state": "published",
                                "make": "Fender",
                                "model": "Stratocaster"
                            }
                        ],
                        "_links": {"next": {"href": "https://api.reverb.com/api/my/listings?page=2&per=50"}},
                        "total": 1
                    }"#,
                );
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.fetch_listings("pat_valid", 1).await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
        let response = result.unwrap();
        assert_eq!(response.listings.len(), 1);
        assert_eq!(response.listings[0].title, "Fender Stratocaster");
        assert!(
            response._links.next.is_some(),
            "expected next link for pagination"
        );
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn fetch_listings_handles_401_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/my/listings")
                .query_param("page", "1")
                .query_param("per", "50");
            then.status(401);
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.fetch_listings("bad", 1).await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), AppError::TokenInvalid),
            "expected TokenInvalid"
        );
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn fetch_listings_handles_429_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/my/listings");
            then.status(429);
        });

        let client = Client::new();
        let api = ReverbApiClient::new_with_url(client, server.base_url());
        let result = api.fetch_listings("pat", 1).await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), AppError::RateLimited),
            "expected RateLimited"
        );
        mock.assert_calls(1);
    }

    // ── normalize_reverb_condition ───────────────────────────────────────

    #[test]
    fn normalize_condition_excellent_returns_used() {
        let cond = ReverbCondition {
            uuid: None,
            display_name: None,
            slug: Some("excellent".to_string()),
        };
        assert_eq!(normalize_reverb_condition(&cond), "used");
    }

    #[test]
    fn normalize_condition_mint_returns_new() {
        let cond = ReverbCondition {
            uuid: None,
            display_name: None,
            slug: Some("mint".to_string()),
        };
        assert_eq!(normalize_reverb_condition(&cond), "new");
    }

    #[test]
    fn normalize_condition_brand_new_returns_new() {
        let cond = ReverbCondition {
            uuid: None,
            display_name: None,
            slug: Some("brand_new".to_string()),
        };
        assert_eq!(normalize_reverb_condition(&cond), "new");
    }

    #[test]
    fn normalize_condition_none_slug_returns_unknown() {
        let cond = ReverbCondition {
            uuid: None,
            display_name: None,
            slug: None,
        };
        assert_eq!(normalize_reverb_condition(&cond), "unknown");
    }

    #[test]
    fn normalize_condition_non_functioning_returns_used() {
        let cond = ReverbCondition {
            uuid: None,
            display_name: None,
            slug: Some("non_functioning".to_string()),
        };
        assert_eq!(normalize_reverb_condition(&cond), "used");
    }

    #[test]
    fn normalize_condition_unknown_slug_returns_unknown() {
        let cond = ReverbCondition {
            uuid: None,
            display_name: None,
            slug: Some("nonexistent".to_string()),
        };
        assert_eq!(normalize_reverb_condition(&cond), "unknown");
    }

    #[test]
    fn normalize_condition_very_good_returns_used() {
        let cond = ReverbCondition {
            uuid: None,
            display_name: None,
            slug: Some("very_good".to_string()),
        };
        assert_eq!(normalize_reverb_condition(&cond), "used");
    }
}
