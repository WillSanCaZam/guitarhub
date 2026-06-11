# Proposal: npm-audit-fix

## Intent

Fix the 6 npm audit vulnerabilities (1 HIGH RCE via serialize-javascript, 2 MODERATE, 3 LOW via cookie) that leave GuitarHub's dev/e2e dependency tree exposed. Zero code changes.

## Scope

### In Scope
- Add `overrides` block to `package.json` forcing `serialize-javascript@7.0.5` and `cookie@0.7.2`
- Run `npm install` to resolve updated lockfile
- Run `npm audit` to confirm zero vulnerabilities
- Run full test suite to verify nothing breaks

### Out of Scope
- Upgrading mocha or @sveltejs/kit to major versions
- Removing WDIO or E2E tests
- Any source code changes in Rust, Python, or Svelte

## Capabilities

> This section is the CONTRACT between proposal and specs phases.
> The sdd-spec agent reads this to know exactly which spec files to create or update.

### New Capabilities

None — this is a pure dependency resolution fix. No new spec-level behavior.

### Modified Capabilities

None — existing specs (including `dependency-security`) are unaffected. This change operates in the npm dependency tree only; Rust/Python dependency security is unchanged.

## Approach

Single-file change: add `overrides` to `package.json` for `serialize-javascript@7.0.5` and `cookie@0.7.2`. Both are safe minor bumps within the same major line. npm overrides force resolution regardless of the semver ranges declared by transitive dependents (mocha, @sveltejs/kit).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `package.json` | Modified | Add `overrides` section with two entries |
| `package-lock.json` | Modified | Auto-updated by `npm install` |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| serialize-javascript 6→7 breaks mocha | Low | Run `npm test` to confirm |
| cookie 0.6→0.7 breaks @sveltejs/kit SSR | Low | 0.6→0.7 is minor; run test suite |
| npm install version conflict | Low | Overrides designed for this use case |

## Rollback Plan

Revert the single commit that adds the `overrides` block, then run `npm install` to restore the original lockfile.

## Dependencies

None.

## Success Criteria

- [ ] `npm audit` exits 0 with 0 vulnerabilities
- [ ] `npm run test:coverage` passes (full test suite)
- [ ] `cargo build` and `cargo test` pass (unaffected, but verify)
