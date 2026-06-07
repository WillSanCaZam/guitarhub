# Delta for wu2-security-hardening

## MODIFIED Requirements

### Requirement: SSRF protection on image URL fetch

`src-tauri/src/commands/image_command.rs` MUST parse `image_url` with `url::Url::parse()` at the IPC boundary. MUST reject non-`https` schemes, IP literal hosts, and domains not in the configured allowlist. The allowed domains MUST be read from `get_setting("allowed_image_domains")` at validation time. If the setting is empty, unparseable, or missing, the system MUST fall back to `["reverb.com", "mlstatic.com"]`.
(Previously: domains were checked against a hardcoded `static ALLOWED_DOMAINS`; now read from settings with fallback)

#### Updated Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Domain in settings allowlist | Setting = "reverb.com,mlstatic.com,newstore.com" | `url = "https://newstore.com/pedal.jpg"` | Passes, download proceeds |
| Domain in fallback only | Setting empty | `url = "https://reverb.com/pedal.jpg"` | Passes (fallback allows) |
| Domain rejected | Setting = "reverb.com" | `url = "https://evil.com/payload"` | REJECTED |
| IP literal | Setting = "reverb.com" | `url = "https://10.0.0.1/config"` | REJECTED |
| HTTP scheme | Any setting | `url = "http://internal.dev"` | REJECTED |
| Malformed setting | Setting = "not,a,domain" | Parse, validate | Fallback to static, proceed per fallback rules |

## ADDED Requirements

### Requirement: Settings UI MUST expose domain allowlist

`Settings.svelte` MUST provide a text input field for the `allowed_image_domains` setting, accepting a comma-separated list of domain names. On save, the value MUST be persisted via the IPC `save_setting` command. The field SHOULD display the current value with a placeholder hint.

#### Scenario: User updates allowlist
- GIVEN the Settings view is open
- WHEN the user enters "reverb.com,mlstatic.com,newstore.com" in the domain field
- AND clicks Save
- THEN `get_setting("allowed_image_domains")` returns the saved value
- AND images from `newstore.com` are now allowed

#### Scenario: Empty allowlist accepted
- GIVEN the Settings view is open
- WHEN the user clears the domain field
- AND clicks Save
- THEN the system falls back to `["reverb.com", "mlstatic.com"]` on next validation
