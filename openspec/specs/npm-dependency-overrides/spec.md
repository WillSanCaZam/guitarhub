# npm Dependency Overrides Specification

## Purpose

Force-resolve transitive npm dependencies to patched versions via `package.json` `overrides` to eliminate actionable audit vulnerabilities that cannot be fixed by upgrading direct dependencies.

## Requirements

### Requirement: Override vulnerable transitive dependencies

`package.json` MUST include an `overrides` section with `"serialize-javascript": "7.0.5"` and `"cookie": "0.7.2"`. These SHALL force-resolve the transitive dependencies to patched versions regardless of the semver ranges declared by direct or indirect dependents.

#### Scenario: Override entries present

- GIVEN `package.json`
- WHEN inspected for `overrides`
- THEN it MUST contain `"serialize-javascript": "7.0.5"`
- AND it MUST contain `"cookie": "0.7.2"`

### Requirement: Lockfile reflects overridden resolutions

`npm install` MUST be run after adding overrides so that `package-lock.json` resolves `serialize-javascript@7.0.5` and `cookie@0.7.2`.

#### Scenario: Resolved versions in lockfile

- GIVEN the updated `package-lock.json`
- WHEN checked for `serialize-javascript`
- THEN its resolved version SHALL be `7.0.5`
- WHEN checked for `cookie`
- THEN its resolved version SHALL be `0.7.2`

### Requirement: Zero audit vulnerabilities

`npm audit` MUST report zero vulnerabilities after the override-based resolution.

#### Scenario: Clean audit output

- GIVEN the updated lockfile
- WHEN `npm audit` runs
- THEN its exit code SHALL be 0
- AND its output SHALL contain "0 vulnerabilities"

### Requirement: Full test suite integrity

All existing tests MUST continue to pass after the dependency resolution. The WDIO + mocha E2E test chain SHALL still function.

#### Scenario: Vitest suite passes

- GIVEN the overridden dependencies
- WHEN `npm test` runs
- THEN all tests SHALL pass

#### Scenario: Svelte check passes

- GIVEN the overridden dependencies
- WHEN `npm run check` runs
- THEN it SHALL exit 0

#### Scenario: E2E runner functions

- GIVEN the overridden dependencies
- WHEN WDIO + mocha E2E tests run
- THEN all E2E tests SHALL pass

### Requirement: Override compatibility with new dependencies

The overrides MUST NOT prevent installation of new dependencies that also transitively depend on `serialize-javascript` or `cookie`.

#### Scenario: New dependency resolves alongside override

- GIVEN `package.json` with the overrides
- WHEN a new dependency that depends on `serialize-javascript` is added
- THEN `npm install` SHALL succeed
- AND the override SHALL resolve to `7.0.5` regardless of the new dependency's semver range
