# Contributing to Radicle Desktop

Thanks for taking an interest. This document covers how to get the app
running, what the code is laid out like, and what your change has to pass
before it can be merged.

Radicle Desktop is developed on Radicle itself, so patches arrive as Radicle
patches rather than as pull requests. If you have never sent one, the
[Submitting a patch](#submitting-a-patch) section walks through it.

## Getting in touch

Questions are welcome before you write any code, especially for anything
larger than a bug fix. The maintainers are on the
[`#desktop` stream on Zulip][zul]. Bugs and feature requests belong in the
repository's issues, which you can read in the app itself or with
`rad issue list`.

## Prerequisites

- [Node.js][nod] 24 or higher, and npm 11.16 or higher
- A [Rust][rus] toolchain (the pinned version is in `rust-toolchain.toml`)
- [Tauri's system dependencies][tau] for your platform

On Windows, npm's script shell must be a POSIX compatible shell such as
[Git Bash][gfw]:

```
npm config set script-shell "C:\\Program Files\\git\\bin\\bash.exe"
```

## Getting the code

```
rad clone rad:z4D5UCArafTzTQpDZNQRuqswh3ury
cd radicle-desktop
npm install
```

Without a local Radicle node, clone over HTTP instead:

```
git clone https://seed.radicle.dev/z4D5UCArafTzTQpDZNQRuqswh3ury.git radicle-desktop
```

## Running the app

```
npm run dev
```

This builds the Rust backend and starts the app with hot reload for the
frontend. The first build takes a while; later ones are incremental.

Frontend-only work does not need the Tauri runtime. `npm run start:http`
serves the same backend over HTTP, which is what the end-to-end tests drive,
and `npm start` then runs Vite against it in a normal browser.

## How the code is laid out

```
src/                      Svelte 5 frontend
crates/radicle-types/     Domain types, ports (traits), adapters, ts-rs bindings
crates/radicle-tauri/     Tauri driver — thin IPC wrappers around port methods
crates/test-http-api/     Axum HTTP driver — the same ports, used by the e2e tests
tests/                    Vitest unit tests and Playwright e2e tests
```

The backend is a hexagonal (ports and adapters) design. Each port is a trait
in `crates/radicle-types/src/traits/` carrying a full default implementation,
and both drivers are thin shells over those traits. **Business logic belongs
in the trait, not in the command handler.**

TypeScript types for anything crossing the boundary are generated from the
Rust structs by [ts-rs][tsr] — never hand-write an interface that duplicates
one. After changing a type that derives `TS`:

```
npm run generate-types
```

Commit the regenerated files under `crates/radicle-types/bindings/` along
with your change.

Adding or changing a command touches these layers, in this order:

1. `crates/radicle-types/src/` — the types, then `npm run generate-types`
2. `crates/radicle-types/src/traits/` — the port method and its default
   implementation
3. `crates/radicle-tauri/src/commands/` — the `#[tauri::command]` wrapper,
   registered in `crates/radicle-tauri/src/lib.rs`
4. `crates/test-http-api/src/` — the mirrored route, so the e2e tests keep
   working
5. `src/` — the caller, via `invoke<T>("command_name", args)`

## Checks and tests

These are what CI runs on every change, so run them before you send a patch.

```
npm run check      # tsc, svelte-check, eslint, prettier, cargo fmt, clippy, cargo test
npm run test:unit
npm run test:e2e -- --project webkit
```

`npm run check` is the JavaScript and the Rust side together; `npm run
check-js` and `npm run check-rs` run one at a time. Most formatting and lint
complaints fix themselves with:

```
npm run format
```

The end-to-end tests need the Radicle binaries and a built HTTP backend.
Run this once after cloning, and again whenever
`tests/support/heartwood-release` changes:

```
./scripts/install-binaries
npm run build:http
```

A single spec, and a single project, run faster:

```
npm run test:e2e -- tests/e2e/issues.spec.ts --project webkit
```

If you are only editing `.spec.ts` files and the fixtures already exist from
an earlier full run, `SKIP_SETUP=true` skips recreating them. Any change to
app code, the backend, or the fixtures themselves needs a full run.

## Code conventions

Formatting is handled by prettier, eslint, and `cargo fmt`, so the
conventions worth stating are the ones a tool cannot check.

**Svelte** — Svelte 5 runes (`$state`, `$derived`, `$props`, `$effect`).
Do not mix the legacy syntax (`export let`, `$:`) with runes in the same
component. Styles are scoped and built from the design tokens
(`var(--color-text-primary)`, `var(--txt-body-m-regular)`,
`var(--border-radius-sm)`) rather than literal values.

**TypeScript** — prefer `undefined` over `null`. Import backend types from
`@bindings/*`. Use ES private fields (`#field`) rather than TypeScript's
`private`.

**Rust** — all serialized types live in `crates/radicle-types/src/` and carry
`#[derive(Serialize, TS)]`, `#[serde(rename_all = "camelCase")]`,
`#[ts(export)]` and `#[ts(export_to = "<dir>/")]`. Errors are
`radicle_types::error::Error`.

**Comments** explain why something is the way it is, in whole sentences.
Code that speaks for itself does not need any.

## Commit messages

Write the subject in the imperative mood, capitalized, with no trailing
period, and keep it within 50 characters — "Add feature", not "Added
feature." Put the reasoning in the body, wrapped at 72 characters, and say
what the change is for rather than restating what it does. If the change
closes an issue, reference it by its ID.

Each commit should stand on its own and leave the checks passing.

## Submitting a patch

Work on a branch off `main`:

```
git fetch rad main
git checkout -b my-change rad/main
```

When the checks pass, open a patch:

```
git push rad HEAD:refs/patches
```

Radicle opens an editor for the patch title and description. Add
`-o patch.draft` to the push to open it as a draft instead, for work you
want visible but not yet reviewed.

To revise a patch after feedback, push the new commits to the patch's own
ref:

```
git push rad HEAD:refs/heads/patches/<patch-id>
```

Each revision takes its own description, and it is worth updating the
patch's title and description too when the shape of the change has moved.
`rad patch show <patch-id>` prints the current state of either.

Patches are reviewed in the app and with `rad patch`. A maintainer merges
once the patch is approved.

[gfw]: https://gitforwindows.org
[nod]: https://nodejs.org
[rus]: https://www.rust-lang.org/
[tau]: https://v2.tauri.app/start/prerequisites/#system-dependencies
[tsr]: https://github.com/Aleph-Alpha/ts-rs
[zul]: https://radicle.zulipchat.com/#narrow/stream/444463-desktop
