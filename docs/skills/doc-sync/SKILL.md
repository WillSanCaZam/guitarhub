---
name: guitarhub-doc-sync
trigger: Any task that modifies logic or structure
scope: docs
---

# Doc Sync Skill — GuitarHub (AUTO-ACTIVATED)

This skill activates AUTOMATICALLY when finishing any code task. It is NOT optional.

## Update Trigger Map

| Change in Code | `.md` Files to Update |
|----------------|----------------------|
| New Tauri command in `commands/` | `README.md` (features), `CHANGELOG.md` |
| New DB migration | `docs/adr/` if schema changes, `CHANGELOG` |
| New source adapter in `scraper/` | `README.md`, `docs/CONTRIBUTING.md`, `.env.example`, `CHANGELOG.md` |
| New dependency (Cargo/pip/npm) | `THIRD_PARTY_LICENSES` (in release), `docs/LICENSE-EXCEPTIONS.md` if applicable |
| Change in `capabilities/` (Tauri perms) | `SECURITY.md`, ADR if major change |
| New Svelte component | `docs/design/components.md` |
| Change in design tokens | `docs/design/tokens.json` + `components.md` |
| New environment variable | `.env.example` (with description) |
| New CI workflow | `docs/CONTRIBUTING.md` (CI/CD section) |
| New SDD phase or agent skill | `AGENTS.md` (Skills Index, Section 3) |
| Architecture change (any layer) | `docs/adr/` (new ADR), `README` if impactful |
| New release | `CHANGELOG.md`, `README` badges, SBOM |
| New contributor role | `AGENTS.md` (Section 8), `CODEOWNERS` |

## Sync Protocol

When finishing ANY task that modified code or structure:

1. Identify which `.md` files correspond to the change per the table above
2. Read the current state of each affected `.md`
3. Update ONLY the sections that need changes
4. If the change is decisive: persist in Engram
5. Include updated `.md` files in the same commit as the code change
6. Verify `AGENTS.md` is current

## Verification

```bash
git diff --name-only HEAD~1 | grep -v '\.md$'   # changed code files
git diff --name-only HEAD~1 | grep '\.md$'       # changed docs
# Verify that for every code file there is a corresponding updated .md
```
