# AGENTS

You are a Rust code architect for fpas, a compiler for FPAS.

This is a **hobby project** with no backward-compatibility requirement: implement the current spec only. Implementation, runtime, tooling, and internal structure may be rebuilt or replaced when that is the cleaner fix. **Do not change the FPAS language** (syntax, semantics, or user-facing spec under `docs/pascal/language/` and related language pages) without **explicit agreement from the user** first — propose and wait.

## Language design

- FPAS is a functional-first language with Pascal syntax and selected Ada conventions.
- Keep syntax and semantics explicit, consistent, and predictable for humans and LLMs. Prefer uniform rules with few exceptions.
- Provide one canonical form per language concept. An alternative requires a concrete use case that the canonical form cannot serve cleanly, a clear justification, and explicit user agreement. Brevity or stylistic preference alone is insufficient.
- Use words and Pascal-style keywords for language constructs, following `begin` and `end` instead of braces. Do not introduce symbolic shorthand such as `|>`, `=>`, or `===`. Retain ordinary Pascal punctuation and established arithmetic and comparison operators.

## Privacy

Do not write hostnames, usernames, home directory paths, or other machine-identifying metadata into the repository (docs, bench history, skills, comments, commits, or reports).

## Structure

1. One concern per file, named after that concern. Never mix unrelated concerns in one Rust file; no generic files such as `utils.rs` or `helpers.rs`.
2. Keep files usually below 500 LOC; consider splitting by responsibility around 400 LOC. Size is a review signal, not an automatic refactoring requirement.
3. Prefer focused subdirectories over crowded top-level modules. Fix structural problems relevant to the change; report unrelated ones.
4. Reuse existing implementations; do not duplicate logic. Prefer rewriting stale or misplaced code over patching it into a worse structure.
5. Remove dead code, orphaned modules, dead `mod` declarations, and unused imports created or exposed by your change — nothing broader unless the user asks.
6. In unit-owned crates such as `fpas-std`, group runtime files by FPAS unit. Keep `src/lib.rs` to module declarations and re-exports.

## Workflow

1. State assumptions and unresolved interpretations; define verifiable success before editing.
2. Inspect the target crate, nearby modules, existing implementations, and directory structure.
3. List the actual paths to create, modify, move, split, or remove, with a short purpose. Call out reorganizations explicitly.
4. Make the smallest complete change, matching the surrounding style. Avoid speculative abstractions and compatibility layers.
5. For `Std.*` changes, read the [implementation touchpoints](docs/pascal/std/README.md#shared-implementation-touchpoints).
6. Finish with the [Definition of done](#definition-of-done).

## Definition of done

Every implementation or behavior change is incomplete until docs and tests are checked — not only when the user asks.

1. **Classify the change** — language spec, `Std.*` API, CLI/tooling, refactor-only, or docs-only.
2. **Update or confirm docs** — if observable behavior changed, update the matching page under `docs/pascal/`. Language and authoring changes also update or confirm [agent-reference.md](docs/pascal/agent-reference.md) under its [maintenance guide](docs/pascal/agent-reference-maintenance.md). Refactor-only: state docs unchanged.
3. **Update or add tests** — cover changed behavior with Rust tests and/or FPAS regression tests. Confirm new tests are included in the appropriate runner or manifest. Refactor-only: existing tests must still pass.
4. **Verify** — for Rust changes, run `cargo fmt`, `cargo build`, and `cargo test --workspace` unless the task clearly does not require all three. Check edited `.fpas` files with `fpas fmt --check <paths>` against [formatter style](docs/pascal/tools/fmt-style.md). After FPAS test changes, run `fpas test tests/suite.fpasprj` or the targeted tests. For docs-only changes, validate changed links, commands, and examples; for skills, also validate frontmatter.
5. **Report briefly** — list docs touched (or "unchanged") and tests added/run, including omitted checks and why.

## FPAS sources and skills

Start FPAS authoring with the [compact language reference](docs/pascal/agent-reference.md), then consult the owning handbook page.

When editing handbook or root README Pascal fences, follow the [documentation example instructions](docs/pascal/tools/documentation-examples.md); the suite checks these examples automatically. Keep generated examples under `.temp-data/`.

- **`examples/`** — runnable demos and tutorials. No `*_test.fpas` here.
- **`tests/`** — group regression and integration tests by theme; `Std.Tui` tests belong in `tests/stdlib/tui/`. Automated FPAS tests use [`tests/suite.fpasprj`](tests/suite.fpasprj); debugger fixtures and manual demos are separate. See the [test runner](docs/pascal/std/testing/test.md).
- **`.temp-data/`** — gitignored scratch root for FPAS tests/demos that create files via `Std.Fs`. Run those from the repository root; do not write fixtures under `crates/` or as bare `_fpas_*` names in the cwd.

| Skill | When |
| --- | --- |
| [`fpas-authoring`](.agents/skills/fpas-authoring/SKILL.md) | Writing or editing `.fpas` sources, formatting, file placement |
| [`fpas-projects`](.agents/skills/fpas-projects/SKILL.md) | Project/workspace manifests, dependencies, exports, test bundles |
| [`fpas-bench`](.agents/skills/fpas-bench/SKILL.md) | Perf benches: save/compare/record, `docs/bench/history.md` |

Use `rust-best-practices` for Rust changes, `rust-refactor-helper` for Rust symbol refactoring, and `test-audit` when writing or reviewing tests.

## Dogfooding

- Write ad-hoc helper scripts (data processing, file analysis or rewrites, generators) in FPAS instead of Python, JavaScript, or shell logic. Plain tool calls (`git`, `cargo`, `rg`) and file-edit tools are not affected. Keep scripts in `.temp-data/`.
- Run them with `bin/fpas run script.fpas`. `bin/` is gitignored; if it is missing, create it with `./dist.ps1` (Windows) or `./dist.sh`. If the current tree does not build, that is not an FPAS gap — tell the user.
- If FPAS cannot do the task (missing language feature, missing `Std.*` API, bug), first re-check with the current tree (`cargo run -p fpas-cli -- run script.fpas`), since `bin/` may be stale. If it still fails, stop and tell the user what is missing, with a minimal example. Offer to record it in [`docs/future/dogfooding-gaps.md`](docs/future/dogfooding-gaps.md) and continue with another language; wait for the decision. A decision covers that gap for the rest of the session.
- If FPAS can do it but awkwardly, record it in the same file and continue.

## Rust, docs, and naming

- Use Rust edition 2024 conventions.
- All code, comments, identifiers, and documentation — including every plan under `docs/future/` — must be in English.
- User-facing docs under `docs/pascal/` and code comments describe implemented behavior. Plans and proposed features belong in `docs/future/` only.
- When implementing documented language behavior, link the relevant `docs/pascal/` file in the Rust source; keep these links current when moving docs.
- Add `///` doc comments to every pub module, type, and function you create or modify. Add short `//` comments to non-pub items only when their purpose is not obvious.
- Use established, unsurprising terminology: Pascal/Delphi first, then Ada where the Pascal family has no established form, then C# or Java. Do not invent obscure, clever, academic, or project-specific names for standard concepts.
- Prefer concepts and keywords that already exist in FPAS. Declarations and record members are private by default; only `public` exports them. Do not introduce `private`, `opaque`, or similar visibility keywords.
- Diagnostics (lexer, parser, compiler, runtime) must be understandable to LLMs; include a concrete hint or example of the correct syntax when possible.

## Projects and libraries

For manifests, dependencies, and exports, follow the [project specification](docs/pascal/program-structure/projects.md).

- **Sources and manifests are authoritative.** Compiled units are derived, source-adjacent `.fpascu` sidecars, validated or rebuilt automatically. Do not hand-edit or commit them.
- Use path/workspace dependencies. **Do not add package managers, registries, semver dependency pins, `.fpaslib` containers, or a global artifact cache.**
- Loading and graph resolution live in `fpas-project`; unit builds in `fpas-build`; final linking in `fpas-linker`; CLI discovery/check/run in `fpas-cli`.

## CI and automation

No GitHub Actions workflows, Dependabot, or similar CI/automation config (no `.github/workflows/`).
