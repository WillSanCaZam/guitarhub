# Contributing to GuitarHub

Thanks for your interest! GuitarHub is an offline-first guitar gear catalog
aggregator. This guide covers the full contributor workflow.

## Quick start

1. **Fork and clone** the repo.
2. **Open in a dev container** (recommended): VS Code will prompt "Reopen in
   Container" when you open the project — this installs all dependencies
   automatically. See [`devcontainer.json`](../devcontainer.json) for details.
3. **Or set up manually**:
   - Rust toolchain via `rustup`
   - Python 3.12 + pip
   - Node.js 20
   - Tauri system deps (see [Dockerfile](../Dockerfile))
   - Run `make setup` to install everything
4. **Copy the environment file**: `cp .env.example .env` and adjust as needed.
   See [`.env.example`](../.env.example) for variable descriptions.

## Development

### Running the app

```bash
make dev    # Start Tauri with hot reload
```

### Running tests

```bash
make test          # Rust + Python + frontend + E2E (full suite)
make test-app      # Rust tests only
make test-scraper  # Python tests only
make test-frontend # Frontend tests only (vitest)
make test-e2e      # E2E tests (requires tauri-driver + debug binary)
```

#### E2E prerequisites

E2E tests require additional setup beyond the standard dev environment:

1. **Install `tauri-driver`**:
   ```bash
   cargo install tauri-driver
   ```
2. **Build the debug binary** (without bundling):
   ```bash
   cargo tauri build --debug --no-bundle
   ```
3. **Run E2E tests**:
   ```bash
   make test-e2e
   ```

If `tauri-driver` or the debug binary is not found, `make test-e2e` skips
automatically and prints instructions.

### Linting

```bash
make lint          # Rust clippy + Python ruff/mypy
make lint-rust     # Rust clippy only
make lint-py       # Python ruff + mypy only
```

All linting runs automatically via **pre-commit** hooks. Install them once:

```bash
pre-commit install
```

Then every `git commit` checks formatting, linting, and type correctness.

### Targets reference

Run `make help` for the full list of available targets.

## Code style

### Rust

- Format with `rustfmt` (run `cargo fmt`).
- Lint with `clippy` at `-D warnings` level.
- Follow the patterns in `src-tauri/`: `commands/`, `services/`,
  `repository/sqlite/`.

### Python (`scraper/`)

- Lint with **ruff** — run `ruff check .`
- Type-check with **mypy --strict** — run `mypy . --strict`
- Use Ports & Adapters (Hexagonal Architecture): `domain/`, `use_cases/`,
  `ports/`, `adapters/`.
- Name files in `snake_case`.
- Every public function and class has a docstring.

## Adding a new source adapter

The scraper uses a **Ports & Adapters** (Hexagonal) architecture. Each
marketplace is a self-contained adapter in `scraper/adapters/`. Source
adapters connect to REST/JSON APIs (e.g., the Reverb adapter uses the
public Reverb JSON API) — no HTML scraping or BeautifulSoup required. To
add a new source:

### Step 1: understand the port interface

Look at `scraper/ports.py`. Your adapter must implement the `ScraperPort`
protocol:

```python
class YourAdapter:
    def scrape(self, url: str = "") -> CatalogFile:
        """Fetch and parse products from the source."""
        ...
```

### Step 2: create the adapter file

```bash
touch scraper/adapters/your_source.py
```

The adapter lives directly in `scraper/adapters/` (not a subdirectory).
Implement the `scrape()` method handling pagination, rate-limiting,
authentication, and field mapping to `CatalogProduct`.

### Step 3: register the adapter in CLI

Add your adapter name to the choices list in `scraper/cli.py`:

```python
choices=["reverb", "guitarcenter", "your_source"]
```

And add a corresponding import/instantiation block in the adapter
selection section.

### Step 4: test the adapter

```bash
make test-scraper
ruff check scraper/adapters/your_source.py
mypy scraper/ --strict
```

Create unit tests in `scraper/tests/unit/test_your_source.py` and
add a protocol conformance test in
`scraper/tests/contract/test_protocol.py`.

### Step 5: document required env vars

If your adapter needs credentials, document them in
[`.env.example`](../.env.example) with extraction instructions. The
adapter should resolve credentials from constructor args with env var
fallback, raising `ValueError` if both are missing.

### Step 6: add a Makefile target

Add a `scrape-your_source` target to the `Makefile` for easy local runs.

## Running the scraper locally

```bash
# Single source (outputs to artifacts/)
python scraper/run_all.py --source reverb --output-dir artifacts/

# Publish index (merges artifacts into a single catalog)
python scraper/run_all.py --publish-index --input-dir artifacts/
```

Environment variables the scraper reads:

| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_RUN_ID` | No | Falls back to `"local"` |
| `GITHUB_TOKEN` | No | Needed for GitHub issue creation |
| `GITHUB_REPOSITORY` | No | Needed for health check issues |
| Source-specific vars | Varies | See `REQUIRED_ENV` in each adapter |

## PR process

### Before opening a PR

- [ ] `make test` passes (Rust + Python)
- [ ] `make lint` passes (clippy, ruff, mypy)
- [ ] pre-commit hooks pass on all files (`pre-commit run --all-files`)
- [ ] You've added tests for new behavior
- [ ] You've updated docs for user-facing changes
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)

### PR checklist

- Provide a clear description of what the PR changes and why.
- Link to any related issues.
- If the PR adds or changes a source adapter, include a sample output.
- Add the `size/X` label (estimated by the maintainers if unsure).

### After opening

- A maintainer will review within 2–3 business days.
- Address review feedback with additional commits (no force-push).
- Once approved, the maintainer will squash-merge to `main`.

## Need help?

Open a [GitHub Discussion](https://github.com/WillSanCaZam/guitarhub/discussions) or
check the `docs/` folder for architecture guides and RFCs.
