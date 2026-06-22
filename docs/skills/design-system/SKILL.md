---
name: guitarhub-design-system
trigger: Touch src/ with visual impact or add components
scope: design
---

# Design System Skill — GuitarHub ("Amp Glow")

## Token Reference

All tokens in `src/lib/styles/design-tokens.css` and `docs/design/tokens.json`.

### Backgrounds (The Void)

| Token | Value | Usage |
|-------|-------|-------|
| `--void-deep` | #07070C | Deepest background |
| `--void-mid` | #0F0F18 | Main surface |
| `--void-raised` | #161622 | Cards, elevated elements |
| `--void-hover` | #1E1E2E | Hover states |
| `--void-active` | #252538 | Active states |

### Glow (Warm Valve Light)

| Token | Value | Usage |
|-------|-------|-------|
| `--glow-primary` | #FF7A3D | Primary actions, links |
| `--glow-warm` | #FFB84D | Secondary glow |
| `--glow-hot` | #FF4D4D | Danger, urgency |
| `--glow-cool` | #4DE1FF | Info, accent |
| `--glow-gold` | #FFD700 | Premium, featured |

### Text

| Token | Value | Usage |
|-------|-------|-------|
| `--text-bright` | #F5F0EB | Primary text |
| `--text-warm` | #C4B8A8 | Secondary text |
| `--text-dim` | #7A6E5E | Tertiary text |
| `--text-muted` | #4A4339 | Disabled, placeholder |

### Rules

- Minimum contrast 4.5:1 (WCAG 2.2 AA) on all themes
- Icons: SVG inline or sprite. NEVER font-icons
- NEVER create a new component without entry in `docs/design/components.md`
- Bundle size target: installed app < 15MB
- Light mode: "Studio Day" variant (warm whites, adjusted glow)

### Components Registry

See `docs/design/components.md` for the full component list with token usage.
