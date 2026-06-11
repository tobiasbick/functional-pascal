# Formatter implementation plan

Phased checklist for the **`fpas-fmt`** crate (AST pretty-printer). CLI wiring lives in [cli.md](cli.md) and is intentionally out of scope here.

**Normative style:** [style.md](style.md) — implement the emitter to match examples and tables there.

**Success criteria:** `format_compilation_unit(&unit)` returns stable canonical text; round-trip `parse(format(parse(source)))` succeeds for valid fixtures; formatter tests run in `cargo test -p fpas-fmt`.

---

## Phase 0 — Design lock-in

- [x] Draft [style.md](style.md) (indent, keywords, semicolons, literals, golden full-file examples).
- [x] Review and sign off style for v1 ([style.md — Formatted output](style.md#formatted-output-fpas-fmt)).
- [x] Blank lines: after `program` / `unit` and after `uses` — exactly one each; after `type` block before next section; blank line before record methods ([style.md — Blank lines](style.md#blank-lines)).
- [x] Blocks: always emit `begin` / `end` for `if` / `else`, `for`, `while`, `case` arms (not for `repeat` bodies) ([style.md — Blocks](style.md#blocks-begin--end)).
- [x] Record fields: semicolon after every field, including the last ([style.md — Semicolons](style.md#semicolons)).
- [x] Lossy output confirmed: no comments, normalized literals, no optional single-statement branch form in output.
- [x] Public API shape:
  - `format_compilation_unit(unit: &CompilationUnit) -> String`
  - `format_program(program: &Program) -> String`
  - `format_unit(unit: &Unit) -> String`
- [x] Round-trip policy: formatted output must parse without errors; semantic equivalence optional in tests later.

---

## Phase 1 — Crate scaffold

Target layout:

```text
crates/fpas-fmt/
 ├── Cargo.toml          — depends on fpas-parser (and fpas-lexer only if needed for Span helpers)
 ├── src/
 │   ├── lib.rs          — public API + module declarations
 │   ├── style.rs        — constants (indent width, keyword strings) mirroring style.md
 │   └── emit/
 │     ├── mod.rs        — `Emitter` struct (output buffer, indent level, blank line helpers)
 │     ├── program.rs    — Program, Unit, uses, qualified ids, header blank lines
 │     ├── decl.rs       — const, var, type, function, procedure
 │     ├── stmt.rs       — all Stmt variants
 │     ├── expr.rs       — Expr, Designator, operators
 │     └── types.rs      — TypeExpr, formals, generics
```

- [x] Add `fpas-fmt` to workspace `crates/*` (new directory under `crates/`).
- [x] `Cargo.toml`: `fpas-parser` path dependency; no CLI / project / compiler deps.
- [x] `lib.rs`: crate docs linking to `docs/future/formater/style.md` and `docs/pascal/`; public API stubs (`format_compilation_unit`, `format_program`, `format_unit`).
- [x] `style.rs`: `INDENT_WIDTH`, `INDENT`.
- [x] `emit/mod.rs`: `Emitter` with `writeln`, `indent`, `dedent`, `with_indent`, `blank_line`.
- [x] `emit/{program,decl,stmt,expr,types}.rs`: placeholder modules for Phases 2–5.
- [x] `cargo build -p fpas-fmt` and `cargo test -p fpas-fmt` succeed.

---

## Phase 2 — Types and expressions (leaf emitters)

- [x] `emit/types.rs`: `TypeExpr` (named, `array of`, `dict of K to V`, `result of`, `option of`, `function` / `procedure` types).
- [x] `emit/types.rs`: formal parameters (`name: type`, `mutable`); `format_type_params` for `<T>` / `<T: Constraint>`.
- [x] `emit/expr.rs`: literals (int decimal, real, string, bool per style.md).
- [x] `emit/expr.rs`: identifiers and qualified designators (`a.b`, `a[i]`, calls).
- [x] `emit/expr.rs`: unary / binary operators with precedence-aware parenthesis re-insertion.
- [x] `emit/expr.rs`: `record` / `array` / `dict` literals; `record … with … end` updates.
- [x] `emit/expr.rs`: `Ok` / `Error` / `Some` / `None`, `try`, `go`.
- [ ] `emit/expr.rs`: anonymous `function` / `procedure` expressions — **N/A** (not in expression AST; only `function` / `procedure` *types*).
- [x] Unit tests in `emit/types.rs` and `emit/expr.rs` (parse snippet → format → assert).

---

## Phase 3 — Statements

- [x] `emit/stmt.rs`: `Block` with semicolon rules from [style.md](style.md).
- [x] `emit/stmt.rs`: `var` / `mutable var` statements.
- [x] `emit/stmt.rs`: assignment, `return`, `panic`.
- [x] `emit/stmt.rs`: `if` / `else if` / `else` — **always** wrap branch bodies in `begin` / `end` (even for a single `return`).
- [x] `emit/stmt.rs`: `case` / `of` — label on own line; arm body in `begin` / `end`; `;` after each arm’s `end` (except last before `else`); guards and destructure labels.
- [x] `emit/stmt.rs`: `for` / `for .. in` — `begin` / `end` loop body.
- [x] `emit/stmt.rs`: `while` — `begin` / `end` body.
- [x] `emit/stmt.rs`: `repeat` / `until` — statement list **without** extra `begin` / `end` wrapper.
- [x] `emit/stmt.rs`: `break`, `continue`, call statements, `go`.
- [x] Tests: `format_block_stmts` on parsed program bodies (`if`, `case`, loops, `var`/assign/call).

---

## Phase 4 — Declarations

- [x] `emit/decl.rs`: `const`, `var`, `mutable var` with visibility (`private` prefix when needed).
- [x] `emit/decl.rs`: `type` aliases, `record` (fields with trailing `;` on every field, blank line before methods), `enum` (variants, associated data, backing values).
- [x] `emit/decl.rs`: `function` / `procedure` (signature, generics, `begin` / `end` body).
- [ ] `emit/decl.rs`: `forward` — **N/A** (parser rejects `forward` in current grammar).
- [x] `emit/decl.rs`: nested declarations inside `type` blocks.
- [x] Tests: [style.md — Examples — record types](style.md#examples--record-types) (1 field, 5 fields, defaults, methods).

---

## Phase 5 — Compilation units

- [x] `emit/program.rs`: `program Name;` + blank line + `uses` + blank line + optional `type` + blank line + `begin` … `end.`
- [x] `emit/program.rs`: `unit Qualified.Name;` + blank line + `uses` + blank line + declarations.
- [x] `emit/program.rs`: `format_compilation_unit` dispatches `Program` vs `Unit`.
- [x] End-to-end round-trip tests (`parse` → `format` → `parse`) on:
  - [x] `examples/hello.fpas`
  - [x] `examples/pascal/units-basic/src/math_utils.fpas`
  - [x] `examples/pascal/generics/generic_functions.fpas`
  - [x] `examples/pascal/pattern-matching/guards.fpas`
  - [x] `examples/pascal/record-methods/point.fpas`

---

## Phase 6 — Hardening

- [x] Golden-file test harness: `tests/golden/*.expected.fpas` + `tests/golden_output.rs` (`assert_eq!`).
- [x] Fuzz-light: `tests/common/corpus.rs` (parser-test snippets) + `tests/round_trip.rs` (`examples/pascal/**/*.fpas`).
- [x] Document known intentional diffs: [style.md — Intentional diffs from source](style.md#intentional-diffs-from-source).
- [x] `cargo fmt`, `cargo build --workspace`, `cargo test -p fpas-fmt`.

---

## Phase 7 — CLI

Implemented per [cli.md](cli.md).

- [x] `fpas fmt` subcommand in `fpas-cli`.
- [x] File discovery / project / workspace paths (parity with `fpas check`).
- [x] Write-in-place only when content changes.
- [x] `--check` exit code `2` for CI.

---

## Out of scope (v1)

- Trivia-preserving lexer / comment retention.
- `.fpasfmt.toml` configuration.
- Formatting inside string literals.
- Emitting optional single-statement form (language allows it; formatter does not).
- `fpas fmt` as a library stable ABI beyond the Rust API.
- IDE LSP format-on-save (future editor integration).

---

## Dependency graph

```text
fpas-cli (later) ──► fpas-fmt ──► fpas-parser ──► fpas-lexer
                      │
                      └── does not depend on fpas-compiler, fpas-project, or fpas-vm
```

Project loading for multi-file runs stays in `fpas-cli`; `fpas-fmt` only formats one `CompilationUnit` at a time.
