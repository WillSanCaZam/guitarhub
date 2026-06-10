# Delta for local-image-cache

> **Change**: mvp-fixes — Panic fix C4

## MODIFIED Requirements

### Requirement: Deduplicate concurrent downloads

The system MUST coalesce simultaneous requests for the same `image_url` into a single HTTP call. When the in-flight watch channel is dropped before all subscribers receive the result, the subscriber MUST receive a graceful `ImageCacheError::DownloadFailed` instead of panicking.

(Previously: Subscriber path used `expect()` on the watch channel result, which panics if the sender is dropped before the value is available.)

#### Scenario: Concurrent requests

- GIVEN 20 products share the same `image_url`
- WHEN the UI requests all at once
- THEN only ONE HTTP request is made
- AND all 20 resolve to the same blob

#### Scenario: Watch channel dropped under memory pressure

- GIVEN a coalesced request is in-flight
- WHEN the `InFlightTx` sender is dropped before the subscriber reads the result (e.g., the fetcher task is cancelled or the service is shut down)
- THEN the subscriber receives `Err(ImageCacheError::DownloadFailed("in_flight watch channel closed unexpectedly"))`
- AND the application does NOT panic

#### Scenario: Watch channel delivers normally

- GIVEN a coalesced request is in-flight
- WHEN the fetcher completes and sends the result through the watch channel
- THEN the subscriber receives the result via `wait_for` and `borrow`
- AND no `expect` or `unwrap` on channel operations can panic