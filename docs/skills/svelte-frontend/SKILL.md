---
name: guitarhub-svelte-frontend
trigger: Touch src/ (components, stores, Svelte 5 routes)
scope: svelte
---

# Svelte 5 Frontend Skill — GuitarHub

## Code Style Rules

### Svelte 5 Runes (MANDATORY)

Use runes exclusively. NO legacy `writable()` stores.

```svelte
<script>
  let { prop1, prop2 } = $props()
  let localState = $state(initialValue)
  let derived = $derived(expression)
  $effect(() => { /* side effects */ })
</script>
```

### TypeScript

- Strict mode: no `any`, no `@ts-ignore`, no `as unknown as X`
- Single quotes, no semicolons

### Component Conventions

- Components: PascalCase. Files: kebab-case
- CSS: only via design system custom properties (`--color-*`, `--spacing-*`)
- NO hardcoded color or spacing values

### Design Tokens

All visual styling uses tokens from `src/lib/styles/design-tokens.css`:

```css
/* Backgrounds */
--void-deep, --void-mid, --void-raised, --void-hover, --void-active

/* Glow */
--glow-primary, --glow-warm, --glow-hot, --glow-cool, --glow-gold

/* Text */
--text-bright, --text-warm, --text-dim, --text-muted

/* Spacing */
--space-1 through --space-24 (4px base)
--space-xs(4), --space-sm(8), --space-md(16), --space-lg(24), --space-xl(40)
```

### Testing

- Component tests: `@testing-library/svelte` + `vitest`
- Store tests: test reactive state changes
- Run: `make test-frontend` or `npm run test`

### Verification

Before commit:
1. `npx svelte-check --tsconfig ./tsconfig.json` — 0 errors
2. `npm run test` — all passing
