# Condition Normalization Specification

## Purpose

`normalize_condition()` maps arbitrary condition strings from all scrapers into a 4-value vocabulary (`new`, `used`, `refurbished`, `unknown`). Runs in Rust's `sanitize()` pipeline before database write, ensuring CHECK constraint compliance regardless of source.

## Requirements

### Requirement: normalize_condition SHALL map to 4-value vocabulary

`normalize_condition()` MUST map arbitrary strings to one of four values.

| Input | Result |
|-------|--------|
| `"new"`, `"brand_new"`, `"mint"`, `"Open Box"`, `"Blemished"` | `"new"` |
| `"used"`, `"excellent"`, `"great"`, `"good"`, `"fair"`, `"poor"`, `"Used > Excellent"` | `"used"` |
| `"refurbished"`, `"restock"`, `"Restock"` | `"refurbished"` |
| `""`, `"unknown"`, anything else | `"unknown"` |

#### Scenario: Reverb brand_new maps to new

- GIVEN a Reverb condition slug `"brand_new"`
- WHEN `normalize_condition()` is called
- THEN it returns `"new"`

#### Scenario: GC raw for used variants maps to used

- GIVEN a GC raw value like `"Used > Excellent"`
- WHEN `normalize_condition()` is called
- THEN it returns `"used"`

#### Scenario: Unrecognized input yields unknown

- GIVEN any unrecognized string or empty string
- WHEN `normalize_condition()` is called
- THEN it returns `"unknown"`

### Requirement: sanitize SHALL call normalize_condition and preserve original

`RawProduct::sanitize()` MUST invoke `normalize_condition()` on the condition field. The original value MUST be preserved in `specs_json.condition_original`.

#### Scenario: Original preserved in specs_json

- GIVEN a `RawProduct` with `condition = "brand_new"`
- WHEN `sanitize()` completes
- THEN `product.condition` is `"new"` AND `specs_json.condition_original` is `"brand_new"`

#### Scenario: Already canonical value passes through

- GIVEN a `RawProduct` with `condition = "new"`
- WHEN `sanitize()` completes
- THEN `condition` stays `"new"` and `condition_original` is `"new"`
