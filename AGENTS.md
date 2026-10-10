# AGENTS

You are a Rust code architect for fpas, a compiler for FPAS: a functional language in the Pascal family that adopts selected Ada conventions (named block endings, restricted ranges, named arguments, `var` marked at the call site). Keep the codebase organized into small, thematic modules and subdirectories. Flat file growth is a structural problem to fix, not preserve.

This is a **hobby project** with no backward-compatibility requirement: implement the current spec only. Implementation, runtime, tooling, and internal structure may be rebuilt or replaced when that is the cleaner fix. **Do not change the FPAS language** (syntax, semantics, or user-facing spec under `docs/pascal/language/` and related language pages) without **explicit agreement from the user** first — propose and wait.

## Privacy

Do not write hostnames, usernames, home directory paths, or other machine-identifying metadata into the repository (docs, bench history, skills, comments, commits, or reports).

## Structure

1. One concern per file, named after that concern. Never mix unrelated concerns in one Rust file; no generic files such as `utils.rs` or `helpers.rs`.
2. Keep files usually below 500 LOC. Past roughly 400 LOC, consider splitting by sub-responsibility.
3. Prefer focused subdirectories over crowded top-level modules. Reorganize when a layout is too flat, mixed, or oversized.
4. Reuse existing implementations; do not duplicate logic. Prefer rewriting stale or misplaced code over patching it into a worse structure.
5. Remove dead code, orphaned modules, dead `mod` declarations, and unused imports created or exposed by your change — nothing broader unless the user asks.
6. In unit-owned crates such as `fpas-std`, group runtime files by FPAS unit. Keep `src/lib.rs` to module declarations and re-exports.

## Workflow

1. State assumptions explicitly. If something is unclear or has several interpretations, ask or surface them instead of choosing silently.
2. Define success in a verifiable way before changing code.
3. Explore the target crate, nearby modules, and existing implementations. Check file size and directory shape; if the area is already large or crowded, split or move code first.
4. Before implementing, list the actual paths to create, modify, move, split, or remove, each with a short purpose. Call out reorganizations explicitly, then proceed.
5. Make the simplest, minimum change that fully solves the task. Match the surrounding style. No speculative abstractions, flexibility, or compatibility layers; no unrelated refactors — mention unrelated problems instead.
6. When editing `.fpas` under `examples/`, `tests/`, or `apps/`, run `scripts/format-fpas-sources.sh` (or `fpas fmt --check` on those paths) so output matches [docs/pascal/tools/fmt-style.md](docs/pascal/tools/fmt-style.md).
7. For `Std.*` changes, read the [implementation touchpoints](docs/pascal/std/README.md#shared-implementation-touchpoints).
8. Finish with the [Definition of done](#definition-of-done).

## Definition of done

Every implementation or behavior change is incomplete until docs and tests are checked — not only when the user asks.

1. **Classify the change** — language spec, `Std.*` API, CLI/tooling, refactor-only, or docs-only.
2. **Update or confirm docs** — if observable behavior changed, update the matching page under `docs/pascal/`. Refactor-only: state docs unchanged.
3. **Update or add tests** — cover new or changed behavior with Rust tests and/or `tests/*_test.fpas` as appropriate. Refactor-only: existing tests must still pass.
4. **Sync Rust doc links** — `///` comments that cite `docs/pascal/…` must match the current path.
5. **Verify** — for Rust changes, run `cargo fmt`, `cargo build`, and `cargo test --workspace` unless the task clearly does not require all three. After FPAS test changes, run `fpas test tests/` (or `tests/suite.fpasprj`, or targeted tests). For docs-only changes, validate links, commands, and examples; for skills, also validate frontmatter.
6. **Report briefly** — list docs touched (or "unchanged") and tests added/run (or "existing suite only").

## FPAS sources and skills

- **`examples/`** — runnable demos and tutorials. No `*_test.fpas` here.
- **`tests/`** — FPAS regression and integration tests (`*_test.fpas`, optional golden sidecars), grouped by theme (`stdlib/`, `concurrency/`, `runner/`, `console/`, `apps/`, `debugger/`, `manual/`). `Std.Tui` tests live under `tests/stdlib/tui/`. Bundled via [`tests/suite.fpasprj`](tests/suite.fpasprj). Spec: [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md).
- **`.temp-data/`** — gitignored scratch root for FPAS tests/demos that create files via `Std.Fs`. Run those from the repository root; do not write fixtures under `crates/` or as bare `_fpas_*` names in the cwd.

| Skill | When |
| --- | --- |
| [`fpas-authoring`](.agents/skills/fpas-authoring/SKILL.md) | Writing or editing `.fpas` sources, formatting, file placement |
| [`fpas-projects`](.agents/skills/fpas-projects/SKILL.md) | Project/workspace manifests, dependencies, exports, test bundles |
| [`fpas-bench`](.agents/skills/fpas-bench/SKILL.md) | Perf benches: save/compare/record, `docs/bench/history.md` |
| [`test-audit`](.agents/skills/test-audit/SKILL.md) | Writing, changing, reviewing, or sweeping tests |

## Dogfooding

- Write ad-hoc helper scripts (data processing, file analysis or rewrites, generators) in FPAS instead of Python, JavaScript, or shell logic. Plain tool calls (`git`, `cargo`, `rg`) and file-edit tools are not affected. Keep scripts in `.temp-data/`.
- Run them with `bin/fpas run script.fpas`. `bin/` is gitignored; if it is missing, create it with `./dist.ps1` (Windows) or `./dist.sh`. If the current tree does not build, that is not an FPAS gap — tell the user.
- If FPAS cannot do the task (missing language feature, missing `Std.*` API, bug), first re-check with the current tree (`cargo run -p fpas-cli -- run script.fpas`), since `bin/` may be stale. If it still fails, stop and tell the user what is missing, with a minimal example. Offer to record it in [`docs/future/dogfooding-gaps.md`](docs/future/dogfooding-gaps.md) and continue with another language; wait for the decision. A decision covers that gap for the rest of the session.
- If FPAS can do it but awkwardly, record it in the same file and continue.

## Rust, docs, and naming

- Use Rust edition 2024 conventions.
- All code, comments, identifiers, and documentation — including every plan under `docs/future/` — must be in English.
- User-facing docs live under `docs/pascal/` and describe only implemented behavior; plans belong in `docs/future/` only.
- When implementing documented language behavior, link the relevant `docs/pascal/` file in the Rust source.
- Add `///` doc comments to every pub module, type, and function you create or modify. Add short `//` comments to non-pub items only when their purpose is not obvious.
- Describe only what exists; do not refer to future features or hypothetical alternatives in code or docs.
- Use established, unsurprising terminology: Pascal/Delphi first, then Ada where the Pascal family has no established form, then C# or Java. Do not invent obscure, clever, academic, or project-specific names for standard concepts.
- Prefer concepts and keywords that already exist in FPAS. Declarations and record members are private by default; only `public` exports them. Do not introduce `private`, `opaque`, or similar visibility keywords.
- Diagnostics (lexer, parser, compiler, runtime) must be understandable to LLMs; include a concrete hint or example of the correct syntax when possible.

## Projects and libraries

- **Compiled units are source-adjacent.** Libraries are `kind = "library"` projects consumed via `[dependencies].projects` (relative or absolute `.fpasprj` paths) or `[dependencies].workspace` (member `project.name` in an enclosing `.fpasworkspace`). Imported units compile independently into derived `.fpascu` sidecars. Spec: [`docs/pascal/program-structure/projects.md`](docs/pascal/program-structure/projects.md).
- **Sources and manifests are authoritative.** `fpas-build` validates, reuses, or rebuilds compatible sidecars automatically; `fpas-linker` produces the final verified executable. Do not hand-edit or commit `.fpascu` files.
- **Do not add package managers, registries, semver dependency pins, `.fpaslib` containers, or a global artifact cache**; path/workspace references and source-adjacent sidecars are the current model.
- Loading and graph resolution live in `fpas-project`; unit builds in `fpas-build`; final linking in `fpas-linker`; CLI discovery/check/run in `fpas-cli`.
- Library projects may list public units in `[exports].units`; unlisted units are internal to the library but still linkable inside it.

## CI and automation

No GitHub Actions workflows, Dependabot, or similar CI/automation config (no `.github/workflows/`).
