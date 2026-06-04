# Future: Libraries

## Implemented (current scope)

Source-level library projects are supported today:

- `kind = "library"` in `.fpasprj` (units only, no `main`).
- Consumption from a `program` via `[dependencies].projects` and `[dependencies].workspace`.
- Transitive dependencies, cycle detection, `fpas check` on libraries and workspaces.

Spec: [`docs/pascal/10-projects.md`](../pascal/10-projects.md). Examples: [`examples/pascal/library-deps/`](../../examples/pascal/library-deps/), [`examples/pascal/monorepo/`](../../examples/pascal/monorepo/).

## Explicitly out of scope (for now)

Do **not** plan or implement these unless product direction changes:

- Precompiled library artifacts (`.fpaslib` or similar).
- Separate `fpas build` / install steps for third-party libraries.
- Package registries, lockfiles, or semver dependency resolution.

Dependencies stay **paths to `.fpasprj` files** (or workspace member names). The compiler always parses and links library **sources** with the consumer.

## Under consideration (later)

- **Project-level export API** — explicit public surface beyond per-unit `private` (for example hiding internal units from consumers without listing every symbol).
