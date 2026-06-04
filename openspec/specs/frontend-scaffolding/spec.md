# Frontend Scaffolding Specification

> **Status**: New infrastructure  
> **Change**: fix-critical-fallas

## Purpose

Provide the minimal SvelteKit project structure required by Tauri 2 to serve the frontend. The 5 existing `.svelte` components (`ProductCard`, `PriceBadge`, `PriceChart`, `Settings`, `ProductDetail`) exist but no project scaffold supports them.

## Requirements

### Requirement: SvelteKit scaffold files MUST exist

The system MUST provide under `src/`: `package.json` (deps: `@sveltejs/kit`, `@sveltejs/adapter-static`, `svelte`, `vite`, `typescript`), `svelte.config.js` (with `adapter-static`, no prerender), `vite.config.ts` (SvelteKit plugin), `tsconfig.json` (strict mode), and `src/app.html` (minimal `<html>` entry point with `%sveltekit.head%` and `%sveltekit.body%`).

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| npm install succeeds | `package.json` with correct deps | `npm install` | exit 0, `node_modules/` created |
| Static build succeeds | All scaffold files present | `npm run build` | exit 0, `build/` directory with `index.html` |
| Dev server starts | All scaffold files present | `npm run dev` | Vite serves on localhost:5173, no compile errors |

### Requirement: Routes MUST exist and reference components

The system MUST provide `src/routes/+layout.svelte` (wrapper with `<slot/>`) and `src/routes/+page.svelte`. The page route MUST import existing components from `$lib/components/` (ProductCard, PriceBadge, PriceChart, Settings, ProductDetail) and render them.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Components resolve | `$lib/components/ProductCard.svelte` exists | Render `+page.svelte` | No module-not-found errors |
| Page renders | Dev server running | Navigate to `/` | Components render without JS errors |
