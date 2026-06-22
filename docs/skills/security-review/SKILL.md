---
name: guitarhub-security-review
trigger: Touch capabilities/, deps, CI secrets, .env
scope: security
---

# Security Review Skill — GuitarHub

## Security Checklist

### Tauri Permissions (`capabilities/`)

- Minimum privilege: NO `shell:all`, `fs:all`, or `http:all`
- Every permission change requires justification
- Review in every PR that touches `capabilities/`

### Dependencies

Before adding ANY dependency:
1. Check SPDX license (must be GPL-3.0 compatible)
2. Run `cargo audit` / `pip-audit` / `npm audit`
3. Check last commit date (>2 years = flag)
4. Document in `docs/LICENSE-EXCEPTIONS.md` if non-standard

### Secrets

- NEVER hardcode secrets, tokens, or API keys in any repo file
- Use `.env` for local dev (gitignored)
- CI secrets via GitHub Actions secrets only

### CSP Headers

- Strict Content Security Policy in `tauri.conf.json`
- No `unsafe-inline` or `unsafe-eval` in production

### Audit Tools

```bash
make audit          # cargo-audit + pip-audit
npm audit           # npm dependencies
cargo audit         # Rust dependencies
pip-audit -r scraper/requirements.txt  # Python dependencies
```

### SBOM

Generate on every release: `syft --output spdx-json`
