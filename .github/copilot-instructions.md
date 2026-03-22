## Purpose

This file gives concise, actionable guidance for AI coding agents working on the AutoDaily codebase. Focus on fast, safe edits: the project is a Vue 3 + Vite front-end (src/) and a Rust + Tauri backend (src-tauri/). The repo is actively being refactored toward a multi-process, IPC-based architecture.

---

## Quick dev commands (what humans run)

- Install frontend deps: `pnpm install` (uses pnpm; package.json present)
- Start frontend dev server: `pnpm dev` (Vite, serves on port 9999 per `vite.config.js`)
- Run Tauri dev workflow / build helpers: `pnpm tauri` (see `package.json` script `tauri` and `gen:types`)
- Build production: `pnpm build` then `pnpm tauri build` (tauri config expects `dist` at `src-tauri` build step; see `src-tauri/tauri.conf.json`)
- Generate TypeScript bindings: `pnpm run gen:types` (runs `cargo test export_bindings` in `src-tauri`)

Note: Vite dev server uses port 9999 and `strictPort: true` (see `vite.config.js`). Tauri's `devUrl` points to `http://localhost:9999` in `src-tauri/tauri.conf.json` — keep these ports in sync for local dev.

---

## High-level architecture (what to know immediately)

- Frontend (src/): Vue 3 + Vite. Main entry: `src/main.js`. Router in `src/router`. UI layouts in `src/layouts/` and `src/components/`.
- Backend (src-tauri/): Rust/Tauri. Library crate `auto_daily_lib` and an opt-in child binary `child` (enable with feature `child-bin`; see `src-tauri/Cargo.toml` and `src/main_child.rs`). Rust code is modularized under `src-tauri/src/` — notable modules: `api.rs`, `infrastructure/`, `domain/`, `dev_test`.
- IPC & multi-process: The project is moving to a multi-process architecture for device isolation and performance. Look for `interprocess`, `rayon`, `tokio`, and `crossbeam-channel` usages in `src-tauri/Cargo.toml` and `src-tauri/src`.
- Resources & models: Bundled model files are referenced in `src-tauri/tauri.conf.json` under `bundle.resources` (e.g. `./models/**/*`).

---

## Project-specific conventions and patterns

- Package manager: pnpm is preferred. Use `pnpm` commands in docs and scripts.
- Ports and dev host: The Vite server is pinned to port 9999. HMR is configured to use env var `TAURI_DEV_HOST` when doing remote dev; don't change port unless updating `tauri.conf.json` too.
- Type bindings: Rust -> TS bindings use `ts-rs` and are produced via `cargo test export_bindings` (exposed by the `gen:types` npm script).
- State/store: Frontend uses Tauri's `@tauri-apps/plugin-store` directly in `src/store/store.js` (see `Store.load('autodaily.config.json')`) and shared keys like `appThemeKey` are exported there — reference these constants when editing UI code.
- Feature toggles & commented code: Some integrations (Element Plus, Pinia) are present but commented in `src/main.js`. Enable only after verifying styles and dependencies.

---

## Where to look for common tasks (examples)

- Find frontend entry points: `src/main.js`, `src/App.vue`, `src/layouts/MainLayout.vue`.
- Find IPC / API surface: `src-tauri/src/api.rs` and `src-tauri/src/infrastructure/`.
- Build hooks and scripts: `package.json` (scripts: `dev`, `build`, `tauri`, `gen:types`) and `src-tauri/tauri.conf.json` (`beforeDevCommand`, `beforeBuildCommand`, `devUrl`).
- Examples and tests: `src-tauri/examples/` (e.g. `cpu_core_allocator_example.rs`) and `cargo test` for Rust side.

---

## Safe editing rules for AI agents (do this before changing behavior)

1. Keep frontend server port and Tauri `devUrl` in sync. If you change `vite.config.js` port, update `src-tauri/tauri.conf.json`.
2. When changing Rust public types consumed by the front-end, regenerate TS bindings (`pnpm run gen:types`).
3. Avoid changing low-level process/affinity code without running `cargo test` and the CPU allocator examples — these are sensitive to platform differences.
4. Don't remove `ignored: ["**/src-tauri/**"]` in Vite's watch config; `src-tauri` is built separately and should not be hot-watched by Vite.

---

## Debugging & tests (practical tips)

- Quick Rust compile check: `cd src-tauri && cargo check` or use workspace `cargo check` from repo root.
- Run Rust examples: `cd src-tauri && cargo run --example cpu_core_allocator_example` (see README examples).
- Frontend hot reload: `pnpm dev` then open `http://localhost:9999`.
- Use `pnpm run gen:types` after changing Rust types to avoid runtime mismatches.

---

## Files to reference while coding

- `README.md` — high-level roadmap, quick commands
- `vite.config.js` — Vite dev server settings (port 9999, ignored `src-tauri`)
- `package.json` — npm scripts (`dev`, `build`, `tauri`, `gen:types`)
- `src-tauri/` — Rust/Tauri backend, see `Cargo.toml`, `src/*.rs`, `examples/`
- `src/store/store.js` — Tauri plugin-store usage and shared keys

---

If any section above is unclear or you want more detail (examples of IPC handlers, how to regenerate bindings end-to-end, or build caveats on Windows), tell me which part to expand and I'll iterate.
