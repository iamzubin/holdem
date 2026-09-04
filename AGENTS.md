# AGENTS.md

Three projects in this repo:
1. **app/** - Tauri desktop app (React + TypeScript + Vite + Tailwind 3.x, Windows-only)
2. **holdem_website/** - Next.js marketing site (Next.js 15 + React 19 + Tailwind 4.x)
3. **thumb-rs** - Git submodule, reference only; `app/` uses its own port in `src-tauri/src/thumbnail.rs`

## Commands

```bash
cd app && npm install
npm run build         # TS check + Vite build
npm run tauri build   # production build
# Do not run npm run dev — the user handles that.

cd holdem_website && npm install
npm run dev           # dev server (:3000)
npm run build / npm run start / npm run lint
npx prettier --write .  # formatting (single quotes, no semicolons, width 80)

cd app/src-tauri
cargo build
cargo test <name>              # single test
cargo clippy -- -D warnings    # run before committing
```

## Conventions

- Files: kebab-case. Components: PascalCase. Functions/variables: camelCase.
- Imports: `@/*` alias for `./src/*`; merge classes with `cn()` (`clsx` + `tailwind-merge`).
- Prefer `interface` for shapes, `type` for unions. Radix UI primitives for accessible components.
- Rust: explicit signatures, `thiserror` + `Result<T, Error>`, `AsRef<Path>` for path args.
- Tauri: add `#[tauri::command]` in `src-tauri/src/lib.rs`, call via `invoke()` from the frontend.

## Structure

```
app/src/{components,hooks,lib,pages,types.ts}
app/src-tauri/src/            # Rust backend (lib.rs, thumbnail.rs)
holdem_website/{app,components,hooks}
```

## Notes

- Desktop app is Windows-only (Tauri 2.x).
- Tailwind versions differ: 3.x in `app/`, 4.x (CSS-based config) in `holdem_website/`.
