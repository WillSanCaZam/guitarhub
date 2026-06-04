# Contributing to GuitarHub

> **⚠️ Note:** The `scraper/` directory (Python scraper pipeline) is not yet
> implemented. Sections below that reference `scraper/ports/`, `scraper/tests/`,
> or `scraper/run_all.py` describe the **planned architecture** and will work
> once the scraper is built. The Rust backend and Svelte frontend are active.

Thanks for your interest! GuitarHub is an offline-first guitar gear catalog
aggregator. This guide covers the full contributor workflow.

## Quick start

1. **Fork and clone** the repo.
2. **Open in a dev container** (recommended): VS Code will prompt "Reopen in
   Container" when you open the project — this installs all dependencies
   automatically. See [`.devcontainer/`](../.devcontainer/) for details.
3. **Or set up manually**:
   - Rust toolchain via `rustup`
   - Python 3.12 + pip
   - Node.js 20
   - Tauri system deps (see [Dockerfile](../.devcontainer/Dockerfile))
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
make test          # Rust + Python tests
make test-app      # Rust tests only
make test-scraper  # Python tests only
```

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

The scraper uses a **SourcePort** abstraction to decouple each marketplace from
the ingestion pipeline. To add a new source:

### Step 1: check the port interface

Look at `scraper/ports/source_port.py`. Your adapter must implement:

```python
class YourSourcePort(SourcePort):
    NAME: ClassVar[str] = "your_source"
    REQUIRED_ENV: ClassVar[tuple[str, ...]] = ()

    async def fetch_listings(self) -> AsyncIterator[dict]:
        ...
```

### Step 2: create the adapter file

```bash
touch scraper/adapters/sources/your_source.py
```

Implement `fetch_listings` — yield raw listing dicts. Handle pagination,
rate-limiting, and authentication.

### Step 3: register the adapter

Add your source name to the CI matrix in `.github/workflows/scrape.yml`:

```yaml
matrix:
  source: [reverb, mercadolibre, guitarras_co, your_source]
```

### Step 4: test the adapter

```bash
make test-scraper
ruff check scraper/adapters/sources/your_source.py
mypy scraper/ --strict
```

### Step 5: document required env vars

If your adapter needs API keys, add them to `REQUIRED_ENV` and document them in
[`.env.example`](../.env.example).

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

Open a [GitHub Discussion](https://github.com/user/guitarhub/discussions) or
check the `docs/` folder for architecture guides and RFCs.
