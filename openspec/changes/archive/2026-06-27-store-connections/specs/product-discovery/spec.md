# Delta for product-discovery

## MODIFIED Requirements

### Requirement: get_featured_products MUST return N random active products

`get_featured_products(limit: u32, user_id: Option<String>) -> Vec<RawProduct>` — `SELECT ... FROM products_meta WHERE is_active=1 AND (user_id IS NULL OR user_id = ?) ORDER BY RANDOM() LIMIT ?`. The `user_id` parameter is OPTIONAL; when omitted, returns only public products. When provided, includes the user's connected products alongside public ones.
(Previously: only queried public `is_active=1` products, no `user_id` filter)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Returns N random public | 50 public active products | `limit=6` | Returns 6 distinct public products |
| Includes user products | 8 user-owned products | `limit=6, user_id="u1"` | May include user products + public products |
| Less than N | 2 public + 3 user products | `limit=6, user_id="u1"` | Returns 5 products |
| Empty catalog | 0 products | invoke | Returns `[]` |

### Requirement: get_price_drops MUST return products with largest absolute price drops

`get_price_drops(limit: u32, user_id: Option<String>) -> Vec<RawProduct>` — JOIN `products_meta` with `price_history` subquery, includes public AND user-connected products when `user_id` is provided.
(Previously: only queried public active products)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Includes user drops | User product dropped $500 | `limit=5, user_id="u1"` | User product in results among public drops |
| No user_id | User product dropped $500 | `limit=5, no user_id` | Only public drops returned |

### Requirement: get_new_arrivals MUST return most recently synced products

`get_new_arrivals(limit: u32, user_id: Option<String>) -> Vec<RawProduct>` — `WHERE is_active=1 AND (user_id IS NULL OR user_id = ?) ORDER BY synced_at DESC LIMIT ?`.
(Previously: only queried public active products)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| User products first | User's products synced recently | `limit=6, user_id="u1"` | User's newest products appear in results |
| Public only | No user_id given | `limit=6` | Only public products returned |

### Requirement: All discovery commands MUST handle concurrent calls safely

No change — the `user_id` parameter is an additional optional param that does not affect concurrency.

### Requirement: Discovery commands MUST return within 50ms

No change — the indexed `user_id` column SHALL ensure performance with the added filter.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Under limit with user | 10k products + 1k user products | Invoke any command with `user_id` | Completes in < 50ms |
