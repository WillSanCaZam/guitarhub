---
name: guitarhub-adr
trigger: Propose or document an architecture decision
scope: architecture
---

# ADR Skill — GuitarHub

## When to Create an ADR

An ADR is REQUIRED when:
- Architecture decision affects more than one layer
- New dependency is added
- Security model changes
- Testing strategy changes
- Deployment/distribution changes

## ADR Format

Every ADR in `docs/adr/` follows this format:

```markdown
# ADR-NNN: {Title}

## Status

{Accepted | Superseded by ADR-XXX | Deprecated}

## Context

{What is the issue that we're seeing that is motivating this decision or change?}

## Decision

{What is the change that we're proposing and/or doing?}

## Consequences

{What becomes easier or more difficult to do because of this change?}

## Alternatives Considered

{What other options were evaluated? Include at least 2 rejected alternatives with reasons.}
```

## Naming Convention

`docs/adr/NNN-lowercase-hyphenated-title.md`

- NNN: zero-padded sequential number (001, 002, ...)
- Title: lowercase, hyphens for spaces

## Existing ADRs

- ADR-001: Offline-first SQLite + FTS5
- ADR-002: Clean Architecture in Rust Backend
- ADR-003: Ports & Adapters in Python Scraper
- ADR-004: CSP Security Headers in Tauri

## Rules

- NEVER approve a new dependency without reviewing its SPDX license and last commit date
- Every RFC must include "Rejected Alternatives" with at least 2 options
- Persist decision in Engram after creating ADR
