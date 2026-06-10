# Delta for user-agent

> **Change**: mvp-fixes — Version alignment

## ADDED Requirements

### Requirement: User-agent string MUST match package.json version

The `reqwest::Client` user-agent in `lib.rs` and `image_cache.rs` MUST use the string `GuitarHub/0.2.0`, matching the `version` field in `package.json`. The version MUST be kept in sync: if `package.json` declares `0.3.0`, the Rust user-agents MUST also say `0.3.0`.

(Previously: `lib.rs:67` and `image_cache.rs:88` both used `GuitarHub/0.1`, lagging behind `package.json` version `0.2.0`.)

#### Scenario: User-agent in sync with package.json

- GIVEN `package.json` declares `"version": "0.2.0"`
- AND `lib.rs` constructs the HTTP client with `user_agent("GuitarHub/0.2.0")`
- AND `image_cache.rs` constructs its client with `user_agent("GuitarHub/0.2.0")`
- WHEN any outbound HTTP request is made
- THEN the `User-Agent` header is `GuitarHub/0.2.0`

#### Scenario: Version bump keeps user-agent in sync

- GIVEN a future change updates `package.json` to `0.3.0`
- WHEN the developer updates the user-agent strings
- THEN both `lib.rs` and `image_cache.rs` MUST be updated together