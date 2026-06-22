---
name: guitarhub-scraper-adapter
trigger: Add or modify adapters in scraper/
scope: scraper
---

# Scraper Adapter Skill — GuitarHub

## Architecture Rules (Hexagonal / Ports & Adapters)

### Structure

```
scraper/
├── domain.py      # CatalogProduct, CatalogFile (Pydantic models) — PURE
├── ports.py       # ScraperPort Protocol, error types — INTERFACES
├── adapters/      # Concrete implementations
│   └── reverb.py  # ReverbAdapter implements ScraperPort
├── cli.py         # CLI entry point
└── tests/         # unit/ + contract/
```

### Adding a New Adapter

1. Create `scraper/adapters/{source_name}.py`
2. Implement `ScraperPort` protocol:
   ```python
   class NewAdapter:
       async def scrape(self, query: str, max_pages: int = 5) -> CatalogFile:
           ...
   ```
3. Map source API fields to `CatalogProduct` fields
4. Add CLI flag in `cli.py`: `--adapter {source_name}`
5. Create tests in `tests/unit/test_{source_name}.py`
6. Create contract test in `tests/contract/test_protocol.py`

### Code Style

- `ruff check .` — no errors
- `mypy . --strict` — no errors
- Files: snake_case. Classes: PascalCase
- Docstring on every public function and class
- Zero imports from `adapters/` in `domain.py` or `ports.py`

### Dependencies

Only 3 allowed: `curl_cffi`, `requests`, `pydantic`. Adding a new dep requires ADR.
