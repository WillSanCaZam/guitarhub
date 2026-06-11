# AGENTS.md — GuitarHub

## 1. Project Overview

GuitarHub is a native cross-platform desktop app (the "Mihon of guitars") that aggregates guitar, amp, pedal, and accessory listings from multiple online stores into a single unified catalog. Offline-first, FOSS (GPL-3.0), zero server costs, zero tracking. When you find a product, you are redirected to the original store to buy. GuitarHub never holds inventory, never processes payments, and never touches money.

Stack: Tauri 2.x (Rust backend) + Svelte 5 (TypeScript frontend) + SQLite/FTS5 + Python scraper (Ports & Adapters). CI/CD via GitHub Actions, distribution via GitHub Releases.

## 2. Gentle-AI Context (READ FIRST)

This project uses OpenCode + gentle-ai (Gentleman Stack).
Active profile: `gentle-orchestrator`.
Engram enabled: important decisions are persisted in `.engram/`.

Before any task, the agent MUST:

1. Read this AGENTS.md file completely.
2. Load the relevant skill(s) from the Skills Index (Section 3) BEFORE writing code or docs.
3. Check Engram for prior decisions on the topic:
   ```
   engram search "<topic>"
   engram tui
   ```
4. Run `/sdd-init` if this is the first session on the project or if the stack changed since the last session.

### Delegation Triggers (inherited from gentle-ai)

These are HARD STOPS. If any fires, delegate or explain why delegation is unsafe:

- Reading 4+ files to understand a flow → delegate exploration
- Touching 2+ non-trivial files → use a writer or get review before closing
- Commit/push/PR after code changes → run fresh review (GGA)
- Long session with accumulated complexity → pause and re-plan with SDD
- Merge/git accident recovery → audit before continuing

### Engram Persistence

Persist ALL architecture decisions, non-obvious bug fixes, and convention changes:
```
engram add --project guitarhub "<decision description>"
```

### Doc Sync Obligation

The agent MUST NEVER finish a task without verifying that all relevant `.md` files are synced with the actual code state (see Section 11).

## 3. Skills Index (Load Before Writing Code)

When working on this project, load the relevant skill(s) BEFORE writing any code or docs. Read the SKILL.md file at the indicated path and follow ALL its patterns.

| Skill | Trigger | Path |
|-------|---------|------|
| gentle-ai-issue-creation | Create issue, report bug, request feature | `skills/issue-creation/SKILL.md` (gentle-ai) |
| gentle-ai-branch-pr | Create PR, prepare changes for review | `skills/branch-pr/SKILL.md` (gentle-ai) |
| gentle-ai-chained-pr | Change too large for one PR, stacked PRs | `skills/chained-pr/SKILL.md` (gentle-ai) |
| cognitive-doc-design | Write docs that reduce cognitive load | `skills/cognitive-doc-design/SKILL.md` (gentle-ai) |
| comment-writer | PR comments, issue replies, async updates | `skills/comment-writer/SKILL.md` (gentle-ai) |
| work-unit-commits | Split work into deliverable commits | `skills/work-unit-commits/SKILL.md` (gentle-ai) |
| guitarhub-rust-backend | Touch `src-tauri/` (commands, services, repository) | `docs/skills/rust-backend/SKILL.md` |
| guitarhub-svelte-frontend | Touch `src/` (components, stores, Svelte 5 routes) | `docs/skills/svelte-frontend/SKILL.md` |
| guitarhub-scraper-adapter | Add or modify adapters in `scraper/` | `docs/skills/scraper-adapter/SKILL.md` |
| guitarhub-security-review | Touch `capabilities/`, deps, CI secrets, `.env` | `docs/skills/security-review/SKILL.md` |
| guitarhub-design-system | Touch `src/` with visual impact or add components | `docs/skills/design-system/SKILL.md` |
| guitarhub-adr | Propose or document an architecture decision | `docs/skills/adr/SKILL.md` |
| guitarhub-doc-sync | Any task that modifies logic or structure | `docs/skills/doc-sync/SKILL.md` [AUTO] |

**Note on guitarhub-doc-sync**: This skill activates ALWAYS when a task modifies logic, structure, or convention. It is not optional. The agent loads it automatically when finishing any code task.

## 4. Dev Environment Setup

### Dev Container (recommended)

Open the project in VS Code → "Reopen in Container". Installs: Rust toolchain, Python 3.12, Node.js 22, Tauri system deps, gentle-ai, Engram, pre-commit.

### Manual Setup

```bash
rustup toolchain install stable  # see rust-toolchain.toml
python3.12 -m pip install --upgrade pip
npm install -g pnpm   # optional, npm works
make setup            # installs all dependencies
cp .env.example .env  # configure variables
pre-commit install    # install hooks
```

### First Use with gentle-ai

```bash
# In OpenCode, when opening the project for the first time:
/sdd-init
gentle-ai skill-registry refresh
```

## 5. Build, Test & Audit Commands

| Command | Description |
|---------|-------------|
| `make dev` | Start Tauri + Svelte with hot reload |
| `make build` | Production build for all platforms |
| `make test` | Rust + Python + frontend + E2E (full suite) |
| `make test-app` | Rust tests only (`cargo test`) |
| `make test-scraper` | Python tests only (`pytest`) |
| `make test-frontend` | Frontend tests only (`vitest`) |
| `make test-e2e` | E2E tests (requires `tauri-driver` + debug binary) |
| `make lint` | clippy + ruff + mypy + svelte-check |
| `make lint-rust` | clippy `-D warnings` only |
| `make lint-py` | ruff check + mypy `--strict` only |
| `make audit` | cargo audit + pip-audit |
| `make clean` | Remove build artifacts and cache files |
| `make help` | List all available targets |

### Pre-commit (runs automatically on each git commit)

```bash
pre-commit run --all-files   # run manually on all files
```

## 6. Code Style Rules

### Rust (`src-tauri/`)

- `cargo fmt` before any commit.
- `clippy -D warnings`: zero warnings tolerated.
- `commands/`: IPC glue only. Delegate to `services/`. No business logic.
- `services/`: all business logic. No direct DB access.
- `repository/sqlite/`: sole point of access to SQLite. No logic.
- `Result<T, E>` explicit everywhere. No `.unwrap()` in production code.
- Migrations: create new with next number, NEVER modify existing.
- Doc comments `///` on every public function and struct.

### Svelte 5 / TypeScript (`src/`)

- TypeScript strict: no `any`, no `@ts-ignore`, no `as unknown as X`.
- Svelte 5 runes exclusively: `$state`, `$derived`, `$effect`, `$props`.
- Single quotes, no semicolons.
- Components: PascalCase. Files: kebab-case.
- CSS: only via design system custom properties (`--color-*`, `--spacing-*`). No hardcoded color or spacing values.
- `svelte-check` must pass before commit.

### Python (`scraper/`)

- `ruff check .` with no errors.
- `mypy . --strict` with no errors.
- Hexagonal architecture strict:
  - `domain/`: pure entities. Zero imports from I/O, adapters, frameworks.
  - `use_cases/`: orchestration of domain + ports. No direct I/O.
  - `ports/`: abstract interfaces (ABC). No implementations.
  - `adapters/`: concrete implementations. May import external libs.
- Files: snake_case. Classes: PascalCase.
- Docstring on every public function and class.

## 7. Architecture Constraints

Rules the agent MUST NEVER violate, under no context, under no justification:

- No network dependencies in frontend: all external calls via Tauri commands.
- No business logic in `commands/`: delegate to `services/`.
- No SQL outside `repository/sqlite/`.
- Never modify existing migrations: create new with next number.
- No tracking, analytics, or telemetry of any kind.
- No dependencies requiring a server at runtime.
- No hardcoding secrets, tokens, or API keys in any repo file.
- Scraper: never import `adapters/` from `domain/` or `use_cases/`.
- Tauri permissions in `capabilities/`: minimum privilege. No `shell:all`, `fs:all`, or `http:all` without an ADR justifying it.
- Any architecture decision affecting more than one layer requires an ADR in `docs/adr/` before implementation.

## 8. Contributor Roles

Table of roles: humans + AI sub-agents. Sub-agents are invoked in OpenCode with the corresponding skill or as delegated sessions.

| Role | Scope | Sub-agent / Skill |
|------|-------|-------------------|
| Maintainer | Merge, releases, project vision | main agent (no restriction) |
| Project Manager | Issues, milestones, roadmap, triage | skill: gentle-ai-issue-creation |
| Architect | ADRs, RFCs, architecture review | skill: guitarhub-adr |
| Security Engineer | Auditing, CVEs, permissions, supply chain | skill: guitarhub-security-review |
| UI/UX Designer | Design system, tokens, a11y WCAG 2.2 | skill: guitarhub-design-system |
| QA Engineer | Test strategy, coverage, E2E | skill: work-unit-commits + tests |
| DevOps / Release Eng. | CI/CD, SemVer releases, SBOM, platforms | Makefile + `.github/workflows/` |
| Docs Steward | Continuous sync of all docs | skill: guitarhub-doc-sync [AUTO] |
| OSPO / License Auditor | Licenses, GPL-3.0 compliance, SBOM | skill: guitarhub-security-review |

### Rules for All Sub-agents

- NEVER close a task without verifying that all relevant `.md` files are synced (see Section 11: Autonomous Doc Maintenance).
- NEVER approve a PR that introduces a dependency with a known CVE or license incompatible with GPL-3.0 without a tracking issue.
- NEVER disable a CI check as a workaround.
- Persist all non-obvious decisions in Engram before closing the session.

### Role-Specific Rules

**agent:pm**
- Atomic issues: one problem = one issue. Epics as parent issue with checklist.
- Required labels: type (bug/feat/chore), milestone, size/XS|S|M|L|XL.
- NEVER close an issue without an explanatory comment.

**agent:architect**
- ADR format: `# ADR-NNN / Status / Context / Decision / Consequences / Alternatives Considered`
- NEVER approve a new dependency without reviewing its SPDX license and last commit date.
- Every RFC must include "Rejected Alternatives" with at least 2 options.

**agent:security**
- Tools: cargo audit, pip-audit, npm audit, syft, trivy, semgrep (rulesets: p/rust, p/python, p/typescript).
- Security findings: report as GitHub Security Advisory (private) before any public fix.
- Tauri permissions in `capabilities/`: review in every PR that touches them.
- SBOM: generate on every release with `syft --output spdx-json`.

**agent:designer**
- Design tokens in `docs/design/tokens.json`:
  - `--color-primary`, `--color-surface`, `--color-on-surface`
  - `--spacing-xs(4px)`, `--spacing-sm(8px)`, `--spacing-md(16px)`
  - `--spacing-lg(24px)`, `--spacing-xl(40px)`
  - `--radius-sm(4px)`, `--radius-md(8px)`, `--radius-lg(16px)`
  - `--font-sans`, `--font-mono`
  - `--transition-fast(100ms)`, `--transition-base(200ms)`
- Minimum contrast 4.5:1 (WCAG 2.2 AA) on all themes.
- Icons: SVG inline or sprite. NEVER font-icons.
- NEVER create a new component without an entry in `docs/design/components.md`.
- Bundle size target: installed app < 15MB.

**agent:ospo**
- Allowed licenses: MIT, Apache-2.0, BSD-2/3-Clause, MPL-2.0, LGPL-2.1+, ISC.
- Licenses incompatible with GPL-3.0: AGPL without exception, SSPL, BSL, Commons Clause, proprietary.
- Tools: cargo-license, pip-licenses, license-checker.
- Exceptions: document in `docs/LICENSE-EXCEPTIONS.md` with justification.

## 9. CODEOWNERS Mapping

Suggested content for `.github/CODEOWNERS` (ready to commit). The initial mapping points everything to `@WillSanCaZam` with comments indicating which handles to replace as the team grows:

```
# .github/CODEOWNERS
# Replace @WillSanCaZam with real handles as the team grows.

*                             @WillSanCaZam  # Maintainer: approves everything
/docs/adr/                    @WillSanCaZam  # Architect
/docs/rfc/                    @WillSanCaZam  # Architect
/src-tauri/src/               @WillSanCaZam  # Architect + Backend
/src-tauri/capabilities/      @WillSanCaZam  # Security Engineer
/.github/workflows/           @WillSanCaZam  # DevOps + Security
/SECURITY.md                  @WillSanCaZam  # Security Engineer
/src/                         @WillSanCaZam  # UI/UX Designer + Frontend
/docs/design/                 @WillSanCaZam  # UI/UX Designer
/scraper/                     @WillSanCaZam  # Architect + QA + Security
/docs/                        @WillSanCaZam  # Docs Steward
/README.md                    @WillSanCaZam  # Docs Steward
/AGENTS.md                    @WillSanCaZam  # Maintainer
```

## 10. PR Workflow

### Commit Format (Conventional Commits)

```
<type>(<scope>): <imperative description in lowercase>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `security`, `design`, `perf`

Valid scopes: `rust`, `svelte`, `scraper`, `ci`, `docs`, `db`, `e2e`, `deps`, `design`, `adr`

Examples:
```
feat(scraper): add mercadolibre source adapter
fix(rust): handle sqlite migration error on first run
security(deps): bump rusqlite past CVE-2024-XXXX
design(svelte): update product card to spacing tokens
docs(adr): add ADR-005 for delta sync strategy
chore(ci): add audit workflow for supply chain checks
```

### PR Title

```
[scope] imperative description
[scraper] Add Reverb source adapter
[rust] Fix FTS5 ranking for exact matches
[security] Bump rusqlite past CVE-2024-XXXX
```

### Mandatory Checklist Before Opening PR

- [ ] `make test` passes
- [ ] `make lint` passes
- [ ] `make audit` passes
- [ ] `pre-commit run --all-files` passes
- [ ] Tests added/updated for new behavior
- [ ] Skill `guitarhub-doc-sync` executed: all relevant `.md` files synced with current code state
- [ ] If touching `capabilities/`: reviewed with skill `guitarhub-security-review`
- [ ] If touching `src/`: reviewed with skill `guitarhub-design-system` (tokens, a11y)
- [ ] If adding a dependency: license verified (ospo) + CVEs (security)
- [ ] If changing architecture: ADR created with skill `guitarhub-adr`
- [ ] Important decisions persisted in Engram

### After Opening

- No force-push: add commits on top of the PR.
- The maintainer does squash-merge to master.
- Critical CVE PRs have absolute priority over any other PR.

## 11. Autonomous Doc Maintenance (MANDATORY)

This section defines the agent's autonomous and continuous responsibility to keep documentation in sync. The agent working on GuitarHub is responsible for keeping these files synchronized WITHOUT anyone asking.

### Fundamental Principle

The agent does NOT wait for instructions to update documentation. Every time a task modifies logic, structure, convention, or behavior, the skill `guitarhub-doc-sync` activates automatically when finishing and the agent updates the affected `.md` files BEFORE committing.

### Update Trigger Map

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

### Sync Protocol at Task Completion

When finishing ANY task that modified code or structure:

1. Identify which `.md` files correspond to the change per the table above.
2. Read the current state of each affected `.md`.
3. Update ONLY the sections that need changes — do not rewrite entire files.
4. If the change is decisive (new architecture, convention, non-obvious bug):
   ```
   engram add --project guitarhub "<decision description>"
   ```
5. Include the updated `.md` files in the same commit as the code change (not a separate commit, unless `guitarhub-doc-sync` indicates otherwise for extensive doc changes).
6. Verify `AGENTS.md` is current: if a new skill, role, or convention was added, update the corresponding sections here.

### AGENTS.md Is a Living Document and Self-Updates

AGENTS.md is not static. It is the ONLY file in the repo that the agent has permission and OBLIGATION to modify without anyone explicitly asking.

#### Self-Update Triggers for AGENTS.md

The agent updates AGENTS.md automatically when it detects any of these events, without waiting for external instructions:

| Detected Event | Section(s) to Update in AGENTS.md |
|----------------|-----------------------------------|
| New skill created in `docs/skills/` | Section 3 (Skills Index): add row |
| Existing skill modified or deleted | Section 3: update or remove row |
| New contributor role defined | Section 8 (Roles): add subsection |
| New tool in the stack (lang, framework, lib) | Project Context + Section 5 |
| New Make command added to Makefile | Section 5 (Build & Test Commands) |
| New env variable in `.env.example` | Section 4 (Dev Setup) or Section 13 |
| New CI workflow in `.github/workflows/` | Section 12 (CI/CD Notes) |
| New top-level folder in the repo | Project Context (structure) |
| New commit or PR convention | Section 10 (PR Workflow) |
| New `.md` type requiring sync | Section 11 (trigger map) |
| New architecture constraint identified | Section 7 (Architecture Constraints) |
| Change in Tauri perms or security policy | Section 8 (agent:security rules) |
| New target platform in release | Section 12 (CI/CD Notes) |
| New ADR that reverses a prior decision | Section 7 or Section 8 per context |

#### Self-Update Protocol for AGENTS.md

When any trigger from the table above is detected, follow this flow:

**STEP 1 — DETECT** (at session start, before any task):
```bash
git log --oneline -20                    # recent commits
git diff HEAD~5 --name-only              # recently changed files
ls docs/skills/                          # current project skills
grep "^[a-z].*:" Makefile                # current Makefile targets
ls .github/workflows/                    # current CI workflows
```
Compare against the current AGENTS.md content. If discrepancy → STEP 2.

**STEP 2 — UPDATE**:
- Read the AGENTS.md section corresponding to the detected change.
- Edit ONLY the affected subsections. Do not rewrite entire sections unless the change requires it.
- If adding a row to the Skills table: aligned columns, trigger in present imperative, same format as existing rows.
- If adding a role: subsection with Responsibilities, Rules, skill reference — same format as existing roles.

**STEP 3 — COMMIT**:
If the update happens within an active task, include it in the same commit as the originating change:
```
docs(agents): sync AGENTS.md — add skill guitarhub-nueva-feature
```

If the desync was detected at session start (no active task), make a dedicated commit before anything else:
```
docs(agents): sync AGENTS.md to current project state
```

Never close a session with AGENTS.md out of date.

**STEP 4 — PERSIST IN ENGRAM**:
```
engram add --project guitarhub "AGENTS.md updated: <description>"
```

### Integrity Check at Session Start

The agent runs this internal check before any task:

- [ ] Section 3 (Skills): every file in `docs/skills/` has its row in the table
- [ ] Section 5 (Commands): every Makefile target is documented
- [ ] Section 8 (Roles): every active role has its complete subsection
- [ ] Section 11 (Doc triggers): the map covers all `.md` files that exist in the repo
- [ ] Section 12 (CI/CD): every workflow in `.github/workflows/` is mentioned
- [ ] Project Context: folder structure reflects actual state (`ls` at root and compare against the structure block in the file)

If any check fails → fix AGENTS.md BEFORE continuing with the session.

### Sync Verification Tools

To verify docs are synced before a PR:
```bash
git diff --name-only HEAD~1 | grep -v '\.md$'   # changed code files
git diff --name-only HEAD~1 | grep '\.md$'       # changed docs
# Verify that for every code file there is a corresponding updated .md
```

To search for prior decisions before modifying a convention:
```bash
engram search "topic or file name"
engram tui   # visual browser of project memory
```

## 12. CI/CD Notes

- Workflows in `.github/workflows/` run on every PR and push to master.
- `ci.yml`: Python lint/test, frontend lint/test/build, Rust lint/test/audit.
- `scrape.yml`: matrix by source, each marketplace in parallel.
- `e2e.yml`: E2E tests with tauri-driver.
- `release.yml`: multi-platform build (linux-x86_64, win-x86_64, macos-aarch64, macos-x86_64), SBOM with syft, publish to GitHub Releases.

### Versioning (SemVer)

- `PATCH`: bugfixes without schema changes
- `MINOR`: new features, backward-compatible migrations
- `MAJOR`: breaking schema or public API changes

### CHANGELOG.md

Update on every release following [Keep a Changelog](https://keepachangelog.com/).

The agent NEVER releases without all CI checks green. The agent NEVER disables a CI check as a workaround.

## 13. Key Files Reference

| File/Directory | Responsible | Purpose |
|----------------|-------------|---------|
| `Makefile` | devops | Entry point for all tasks |
| `.env.example` | security+docs | Required environment variables |
| `rust-toolchain.toml` | devops | Exact Rust version |
| `src-tauri/src/commands/` | architect | Tauri IPC handlers (glue only) |
| `src-tauri/src/services/` | architect | Business logic layer |
| `src-tauri/src/repository/sqlite/` | architect | Data access layer |
| `src-tauri/capabilities/` | security | Tauri permissions (minimum privilege) |
| `scraper/domain.py` | architect | Domain models (pure entities) |
| `scraper/ports.py` | architect | Adapter interfaces (Protocol) |
| `scraper/adapters/` | qa+security | Marketplace implementations |
| `.github/workflows/` | devops | CI/CD pipelines |
| `.github/CODEOWNERS` | maintainer | Path-to-responsible mapping |
| `docs/ARCHITECTURE.md` | architect | System architecture overview |
| `docs/CONTRIBUTING.md` | docs | Contribution guide |
| `docs/RELEASE.md` | devops | Release process |
| `SECURITY.md` | security | Vulnerability disclosure policy |
| `CHANGELOG.md` | devops+docs | Release history |
| `.engram/` | all | Persistent agent memory |

## 14. Quick Agent Checklist

Every agent MUST complete this checklist before considering a task done in GuitarHub:

- [ ] `make test && make lint && make audit` pass without errors
- [ ] Skill `guitarhub-doc-sync` executed: affected `.md` files are updated
- [ ] `AGENTS.md` is current: if you changed a convention, role, skill, or tool
- [ ] No secrets, tokens, or hardcoded colors introduced
- [ ] No dependencies with known CVE or license incompatible with GPL-3.0
- [ ] If you changed architecture: ADR exists in `docs/adr/`
- [ ] Important decisions persisted in Engram before closing the session
- [ ] `pre-commit run --all-files` passes on all modified files
