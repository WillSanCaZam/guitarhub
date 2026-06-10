# Delta for ci-pipeline

Relates to: `wu3-ci-cd-hardening` — CI frontend job.

## Purpose

Extend the CI `frontend` job to include build and type-check steps, preventing broken builds and type errors from reaching production.

## ADDED Requirements

### Requirement: CI frontend job MUST include build and type-check

The `frontend` job in `.github/workflows/ci.yml` MUST run `npm run build` and `npm run check` after `npm ci` and `npx svelte-kit sync` and before any signing step. The `check` script MUST invoke `svelte-check`. The `svelte-check` package MUST be added as a devDependency in `package.json`. The `check` script MUST be defined in `package.json` scripts.

These steps MUST appear in this order:
1. `npm ci`
2. `npx svelte-kit sync`
3. `npm run test`
4. `npm run build`
5. `npm run check`
6. Signing dry-run

If either `npm run build` or `npm run check` fails, the job MUST fail and subsequent steps MUST be skipped.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Clean code passes | All frontend code is valid | CI runs | `npm run build` OK, `npm run check` OK, CI passes |
| Type error fails CI | A TypeScript type is mismatched | CI runs | `npm run check` fails, job fails, error output includes file + line |
| Build break fails CI | A Svelte component has syntax error | CI runs | `npm run build` fails, job fails, error output includes file + location |

### Requirement: `svelte-check` MUST be a devDependency

`package.json` `devDependencies` MUST include `"svelte-check"` with a version compatible with Svelte 5. The `check` script in `scripts` MUST be `"svelte-check"`.

```json
{
  "devDependencies": {
    "svelte-check": "^4.0.0"
  },
  "scripts": {
    "check": "svelte-check"
  }
}
```

#### Scenario: svelte-check installed

- GIVEN `package.json` is inspected
- THEN `devDependencies` contains `svelte-check`
- AND the version range is `^4.0.0`

#### Scenario: check script defined

- GIVEN `npm run check` is invoked
- THEN it runs `svelte-check` against the project source
