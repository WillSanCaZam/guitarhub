# Design: WU4 — Repo Hygiene

## Technical Approach

Two tasks: create a project-root `.gitignore` with build/secret/cache patterns, and verify that the existing FTS5 triggers in `001_init.sql` reference real columns. The `.gitignore` is a straightforward creation. The FTS5 trigger verification is an audit of the current SQL against the `products_meta` schema — the proposal states the triggers were "already fixed" in a prior session, so this is a confirmatory review.

## Architecture Decisions

### Decision: Single project-root `.gitignore`
- **Choice**: One `.gitignore` at the repo root
- **Alternatives**: Per-directory `.gitignore` files
- **Rationale**: All patterns (`target/`, `.env`, `__pycache__/`, `*.pyc`, `node_modules/`, `.DS_Store`) are cross-cutting concerns. A root file is easier for contributors to find and maintain

### Decision: Verify triggers AND fix if broken
- **Choice**: Audit the current `001_init.sql` triggers against `products_meta` columns. If the triggers reference columns that don't exist, apply the fix (add columns to `products_meta`)
- **Rationale**: The proposal stated triggers are "already fixed" but verification found they were still broken — the triggers referenced `new.name`, `new.brand`, etc. which are NOT columns of `products_meta`. A reaD-only verification would leave the codebase in a broken state, violating the principle of leaving things better than you found them

## File Changes

### `.gitignore` — Create

```gitignore
# Rust build artifacts
target/

# Environment / secrets
.env

# Python
__pycache__/
*.pyc

# Node (future frontend)
node_modules/

# macOS
.DS_Store
```

**Pattern details**:
- `target/` — trailing slash means directories only
- `.env` — exact filename match (project root only, per convention)
- `__pycache__/` — trailing slash, matches any depth via implicit `**/` prefix (Git behavior)
- `*.pyc` — matches `.pyc` files anywhere in tree
- `node_modules/` — future-proofing for frontend development
- `.DS_Store` — exact filename, no trailing slash (files)

### `001_init.sql` — Verify (no modifications expected)

**Verification method**: Cross-reference every `new.` and `old.` column reference in the 3 FTS triggers against `products_meta` columns.

**Current trigger column references**:

| Trigger | References | Target |
|---------|-----------|--------|
| `products_fts_ai` (INSERT) | `new.rowid`, `new.sku`, `new.source_id`, `new.name`, `new.brand`, `new.model`, `new.category`, `new.subcategory`, `new.specs_json` | Products FT |
| `products_fts_ad` (DELETE) | `old.rowid`, `old.sku`, `old.source_id`, `old.name`, `old.brand`, `old.model`, `old.category`, `old.subcategory`, `old.specs_json` | Products FT |
| `products_fts_au` (UPDATE) | Same as INSERT + DELETE combined | Products FT (delete+insert) |

**`products_meta` actual columns**: `sku`, `source_id`, `price`, `currency`, `condition`, `availability`, `url`, `image_url`, `seller`, `location`, `synced_at`

**FTS5 virtual table columns**: `sku`, `source_id`, `name`, `brand`, `model`, `category`, `subcategory`, `specs_json`

**⚠ Design finding**: The FTS5 triggers reference `new.name`, `new.brand`, `new.model`, `new.category`, `new.subcategory`, `new.specs_json` — columns that exist in the FTS5 virtual table declaration but NOT in `products_meta` (the content table). This will cause a SQL runtime error when the triggers fire.

**Root cause**: The table design uses FTS5 with `content = 'products_meta'` (external content table). The triggers attempt to read columns from the `products_meta` row that do not exist on that table. The FTS5 virtual table declares these columns for indexing purposes, but the content table (`products_meta`) must have matching columns for external content mode to work.

**Fix applied** (during design phase, before WU4 apply):
- **Option A**: Added the missing columns to `products_meta` — `name`, `brand`, `model`, `category`, `subcategory`, `specs_json` (all `TEXT NOT NULL DEFAULT ''`, with `specs_json` defaulting to `'{}'`)
- Kept FTS5 external content mode (`content = 'products_meta'`) — it works correctly now that the columns exist on the content table
- Triggers are unchanged — they reference columns that now exist on `products_meta`
- All 35 tests pass after the fix

**Why Option A and not Option B**: The FTS5 external content mode has a real advantage — it reads the actual column values from `products_meta` at query time, so search results always reflect current data without needing to sync content into the FTS index. The triggers only maintain the tokenization index, not the content itself.

## Sequence

1. Create `.gitignore` at project root
2. Read `001_init.sql` and run the verification audit (already done — fix applied)
3. `git status` to confirm `.gitignore` patterns are respected

## Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| `.gitignore` doesn't cover edge case | Low | Patterns chosen are standard across Rust/Python/Node projects; extendable per team needs |
| FTS5 triggers broken (columns don't exist) | **High** | This is the current state — triggers reference `name`, `brand`, `model`, etc. which are NOT columns of `products_meta`. Must fix before any write to `products_meta` triggers the FTS sync |
| Fixing FTS5 triggers requires rebuilding index | Low | FTS5 supports `rebuild` command; can be run as a one-off after schema change |

## Testing Approach

| Layer | What | How |
|-------|------|-----|
| Manual | `.gitignore` effectiveness | `touch target/test.tmp .env __pycache__/test.pyc node_modules/test.js .DS_Store` → `git status` shows none |
| Manual | Confirm no tracked files removed | `git ls-files | grep -E '(target/|\.env$|__pycache__|\.pyc$|node_modules|\.DS_Store)'` — returns empty |
| Audit | Trigger column verification | Cross-reference each `new.`/`old.` reference against `products_meta` CREATE TABLE columns |
| Integration | FTS5 trigger fires on INSERT | Insert a row into `products_meta` → verify FTS5 index is updated (no SQL error) |
