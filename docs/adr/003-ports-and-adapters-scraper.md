# ADR-003: Ports & Adapters in Python Scraper

## Status

Accepted

## Context

The scraper needs to fetch product listings from multiple online marketplaces (Reverb, and potentially others in the future). Each marketplace has different APIs, data formats, and authentication methods. The scraper must:
- Be easy to extend with new marketplaces
- Keep marketplace-specific code isolated from domain logic
- Be testable without hitting real APIs

## Decision

Implement the Ports & Adapters (Hexagonal) architecture pattern:

```
scraper/
├── domain.py      # CatalogProduct, CatalogFile (Pydantic models) — PURE
├── ports.py       # ScraperPort Protocol, error types — INTERFACES
├── adapters/      # Concrete implementations
│   └── reverb.py  # ReverbAdapter implements ScraperPort
├── cli.py         # CLI entry point
└── tests/         # unit/ + contract/
```

Key rules:
- `domain.py` contains pure Pydantic models — zero imports from adapters or I/O
- `ports.py` defines `ScraperPort` Protocol — the contract all adapters must implement
- Each adapter in `adapters/` implements `ScraperPort` for one marketplace
- Adding a new marketplace = new adapter file only, no pipeline changes

## Consequences

**Easier:**
- Adding new marketplace: create one file implementing `ScraperPort`
- Domain models are testable without any I/O
- Contract tests verify all adapters conform to the protocol
- Clean separation: marketplace-specific logic never leaks into domain

**More difficult:**
- Flat file structure (domain.py, ports.py at package root) differs from typical hexagonal (subdirectories)
- Protocol-based abstractions are lighter than ABC but less explicit
- Only 3 Python dependencies — adding more requires justification

## Alternatives Considered

1. **Monolithic scraper with if/else per marketplace** — Rejected: Hard to extend, marketplace logic entangled with orchestration
2. **Abstract base classes (ABC) instead of Protocol** — Rejected: Protocol is lighter, runtime_checkable, no inheritance required
3. **Subdirectory structure (domain/, use_cases/, ports/, adapters/)** — Considered but rejected: Flat structure is simpler for a scraper with 3 files and 1 adapter
