# Template Selection: App vs Library

## Summary

Add a project type selection step to the nullslate CLI so users can scaffold either a full-stack TanStack Start application or a publishable library package with Vite lib mode + tsup.

## UX Flow

After project name, prompt "What are you building?" with App or Library options. Each type shows its own feature set:

- **App**: auth, docs, db (existing behavior, unchanged)
- **Library**: language (TS/JS), react, css/tailwind, testing

Non-interactive: `ns init my-lib -y --lib` or `ns init my-lib -y --type lib --react --css --testing`

## Template Repos

| Type | Repo | Default URL |
|------|------|-------------|
| App | `nullslate/app-template` | `https://github.com/nullslate/app-template.git` |
| Lib | `nullslate/lib-template` | `https://github.com/nullslate/lib-template.git` |

## Lib Template Structure

```
lib-template/
├── package.json          # {{project_name}}, vite + tsup
├── tsconfig.json         # (TS only)
├── tsup.config.ts
├── vite.config.ts        # Vite lib mode for dev
├── src/
│   ├── index.ts          # Main export
│   └── utils.ts          # Example utility
├── .npmrc                # @thesandybridge registry scope
├── .gitignore
└── CLAUDE.md
```

### Feature: React

Adds `src/components/` with example component, react + react-dom as peer deps, JSX in tsconfig.

### Feature: CSS/Tailwind

Adds `src/styles/` with base CSS, `@tailwindcss/vite` + tailwindcss deps.

### Feature: Testing

Adds `vitest.config.ts`, `src/__tests__/` with example test, vitest devDep.

### Language: JavaScript

No TypeScript deps or tsconfig. `.js` files instead of `.ts`. tsup still handles bundling.

## CLI Code Changes

1. New `ProjectType` enum (App, Lib) in `ui.rs`
2. New `prompt_project_type()` — select between App/Library
3. New `LibFeature` enum — React, Css, Testing
4. New `prompt_lib_features()` and `prompt_language()` prompts
5. Branch in `cmd_init` — use appropriate template URL, features, processing per type
6. Extend `features.rs` with lib-specific skip patterns and dependency removals
7. CLI flags: `--type <app|lib>` or `--lib` shorthand, `--react`, `--css`, `--testing`

## Approach

Hardcoded template logic per type (not template-driven config). Each project type has its own URL, features, skip patterns, and dependency lists in the CLI. Matches the existing pattern for the app template.
