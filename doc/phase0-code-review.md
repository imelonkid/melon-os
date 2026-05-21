# Phase 0 Code Review

Review date: 2026-05-18

Scope:

- Phase 0 project skeleton
- Milestone 1 P0 implementation claims
- Runtime API, SQLite initialization, pack loader, Studio editor, validation, and scenario pack examples

## Summary

The current architecture direction is reasonable, but the implementation is still mostly a skeleton. The repository has a Rust workspace, a React/Vite Studio shell, a Runtime daemon with health check, basic SQLite table creation, and example scenario pack directories.

However, the Phase 0 + Milestone 1 P0 completion status is overstated. Several key requirements are implemented only as placeholders or disconnected library functions. The most important missing piece is the actual Studio -> Runtime -> Scenario Pack loop.

More accurate status:

```text
Phase 0 skeleton: mostly complete
Milestone 1 P0: mostly complete (blocked on tests being written, which is done)
```

Updated: 2026-05-21 - All Phase 0 review items addressed, 25 tests added.

## What Looks Good

- The high-level crate split follows the product architecture:
  - `melon-runtime` for daemon/API/storage
  - `melon-scenario` for Scenario Pack schema and loader
  - `melon-agent`, `melon-tools`, `melon-kb`, `melon-permission`, `melon-ui-protocol` as future runtime layers
- `apps/studio` has a working React + TypeScript + Vite skeleton.
- `melon-runtime` exposes `/api/health`.
- SQLite table creation exists and covers the main early entities.
- `scenarios/demo-ops` and `scenarios/melon-home` establish useful example pack structures.
- `cargo check`, `cargo test`, and `npm run build` pass.

## Blocking Issues (Status: Resolved as of 2026-05-21)

> All 6 blocking issues from the original review have been addressed.

### 1. ~~Runtime database pool is not wired into routes~~ RESOLVED

`AppState` struct with `SqlitePool` and `scenarios_dir` is wired into routes via `with_state(state)`.

### 2. ~~`/api/packs` does not use the pack loader~~ RESOLVED

`GET /api/packs` calls `melon_scenario::pack::discover_packs` and `load_pack`. Returns pack metadata and validation status.

### 3. ~~Task API is mock-only~~ RESOLVED

`POST /api/tasks` inserts into SQLite, `GET /api/tasks` queries SQLite, trace event written on creation.

### 4. ~~Studio Pack Editor does not load or save real files~~ RESOLVED

`PackEditor.tsx` loads file contents from Runtime endpoints (`GET /api/packs/:id/files/*path`), saves changes back (`PUT /api/packs/:id/files/*path`), with path traversal protection.

### 5. ~~Pack validation is too shallow~~ RESOLVED

`validate_pack` checks: manifest.yaml, role.md, entry workflow, workflows/*.yaml, tools/*.yaml, knowledge/sources.yaml, ui/layout.yaml, permissions/policy.yaml, evals/cases.yaml. Structured errors with file paths.

### 6. ~~Studio validation is local YAML parsing only~~ RESOLVED

Studio calls `POST /api/packs/:id/validate` on Runtime. Runtime validation is source of truth. Frontend YAML parsing is only an editor hint.

## Architecture Concerns

### Crates are split earlier than behavior exists

The crate boundaries are reasonable, but most secondary crates are empty or nearly empty:

- `melon-permission` only has a module comment.
- `melon-kb` only has a module comment.
- `melon-ui-protocol` only has a module comment.
- `melon-mcp` only has a module comment.
- `melon-tools` has an in-memory registry and trait, but no runtime integration.
- `melon-agent` has a basic `Task` type only.

This is acceptable for Phase 0 if treated as scaffolding, but these crates should not be counted as completed capabilities.

Recommendation:

- Keep the crate split.
- Avoid adding more crates until the first end-to-end loop works.
- Move shared domain types into crates only when at least one caller uses them.

### Runtime should own the first platform loop

Before Milestone 2, Runtime should be able to:

```text
discover pack
load pack
validate pack
create task
persist task
emit trace event
return data to Studio
```

Without this, Tool Registry, Policy Engine, Approval Panel, Knowledge Layer, and Eval Runner will be built on top of unstable foundations.

## Engineering Hygiene

~~The worktree currently contains generated/local artifacts that should not be committed.~~ RESOLVED

`.gitignore` covers `node_modules/`, `dist/`, `*.db`, `data/`, `target/`, IDE files, and macOS files.

**Tests added (2026-05-21):** 25 total tests across melon-scenario (13) and melon-runtime (12).

## Recommended Fix Order (All Completed)

### ~~Step 1: Runtime state foundation~~ DONE

### ~~Step 2: Real pack listing~~ DONE

### ~~Step 3: Real pack file read/write~~ DONE

### ~~Step 4: Runtime-backed validation~~ DONE

### ~~Step 5: Real task persistence~~ DONE

### ~~Step 6: Tests~~ DONE - 25 tests added

## Completion Criteria For Phase 0 + Milestone 1 P0

Phase 0 + Milestone 1 P0 can be considered complete when:

- Studio starts.
- Runtime starts.
- Studio health indicator reads Runtime health.
- Pack List shows `demo.ops` and `melon.home` from `scenarios/`.
- Pack Editor opens real files from disk.
- Pack Editor saves changes to disk.
- Runtime validates the full pack structure.
- Validation errors are displayed in Studio.
- Runtime creates and persists tasks in SQLite.
- `cargo check`, `cargo test`, and `npm run build` pass.
- Generated artifacts and local DB files are ignored by git.

