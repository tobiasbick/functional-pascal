# 03 – Architecture

Where foreign-habit detection lives, how existing hints are consolidated and how hints are
shaped. Part of the [foreign-habit diagnostics plan](README.md).

## Current state

Habit hints exist but are scattered across stages and files:

| Location | Covers |
| --- | --- |
| `crates/fpas-lexer/src/lexer/foreign/comments.rs` (moved in P0) | `{…}` and `(*…*)` comments (FP1013) |
| `crates/fpas-parser/src/parser/block_closers.rs` | `end;` instead of a named ending, unit with trailing `begin` |
| `crates/fpas-parser/src/parser/decl/data/recovery.rs` | missing `var`/`const`/`type` keyword (FP2015) |
| `crates/fpas-parser/src/parser/decl/type_arguments.rs`, `decl/data/type_defs.rs` | `<T>` instead of `of` |
| `crates/fpas-parser/src/parser/stmt/case/mod.rs` | case arm without `when` |
| `crates/fpas-sema/src/check/expr/boolean.rs` | integer `and`/`or`/`xor` → `Std.Bits` |
| `crates/fpas-sema/src/check/name_resolution/parser_migration.rs` | `StrToInt` and retired parser names → `Std.Parse` |
| `crates/fpas-sema/src/std_registry/native/mod.rs` (`native_migration_hint`) | `IsSome`, `Contains`, `CharAt`, free type-helper calls such as `Length(S)` |

Several of these are not foreign habits but FPAS-internal migrations (retired names). They stay
where they are; only their hint text is reviewed when a catalog entry touches them.

## Target layout

One module per stage owns **recognition of foreign spellings**. Each file covers one habit
group, named after it (AGENTS.md: one concern per file, no generic names).

### Lexer

```text
crates/fpas-lexer/src/lexer/foreign/
  mod.rs              // dispatch from the lexer's unexpected-character path
  comments.rs         // moved from invalid_comments.rs in P0; adds Ada `--`
  operators.rs        // `@`, `^`, `&&`, `||`, `!`, `==`, `!=`
  strings.rs          // double-quoted strings
```

`invalid_comments.rs` moved into `foreign/comments.rs` in P0; no duplicate stays behind.

### Parser

```text
crates/fpas-parser/src/parser/foreign/
  mod.rs              // shared helpers: recognize a foreign form, emit, skip it
  bindings.rs         // `const N = 3`, var sections, grouped names, bounded arrays (P2)
  classes.rs          // Delphi `class` (P2)
  generics.rs         // `TList<T>` and other angle-bracket applications (P2)
  units.rs            // interface/implementation, Delphi RTL units (P2)
  arguments.rs        // `WriteLn(X:0:2)` width specifiers (P4)
  routines.rs         // `procedure Q;`, Ada `function F return T is`, `Result :=`
  (Ada `X => 1`, `/=` and `^T` are recognized by the lexer in P4)
  control_flow.rs     // `begin … end` bodies, `else` without `;`, Ada/Pascal `for`, `try/except`
```

Existing parser hints whose purpose is a foreign habit (`<T>` → `of`, case arms without
`when`, FP2015) move into the matching file when their package touches them. Hints that are
generic FPAS rules (named block endings for FPAS-native code) stay in place.

### Semantic analysis

```text
crates/fpas-sema/src/check/foreign/
  mod.rs              // lookup for unresolved names and assignments
  builtins.rs         // `Inc`, `Dec`, `Exit`, `nil`, `char`, `IntToStr` member naming, …
  constructors.rs     // `T.Create(…)`
  result_assignment.rs// assignment to the function name
```

`builtins.rs` is a table: lowercase Delphi/Ada name → FPAS form, so new entries are one line.
The sema entry point is the existing unknown-name path in `check/name_resolution/mod.rs`,
which today already consults `parser_migration_hint` and `native_migration_hint`.

## Diagnostic codes

Proposal for assumption A-1 in the [README](README.md#assumptions):

- **Reuse** the existing code when it already describes the error correctly: FP2015 for a
  missing declaration keyword, FP3003 for an unknown routine, FP3027 for a missing `var`.
  The habit only improves `message` and `help`.
- **Allocate** one new parser code `PARSE_FOREIGN_SYNTAX` and one lexer code
  `LEX_FOREIGN_TOKEN` for constructs that FPAS has no equivalent token or form for and whose
  current code is misleading (FP1001 "Unexpected character", FP2005 "Expected expression").
  Allocation follows the workflow in `crates/fpas-diagnostics/src/codes.rs`.
- No per-habit codes. The catalog ID is a planning artifact, not a stable user-facing code.

**Q-2 (decided):** one foreign-syntax code per stage (`LEX_FOREIGN_TOKEN`, `PARSE_FOREIGN_SYNTAX`)
instead of reusing FP2001. One code lets agents and tests filter "you wrote another language".

## Hint format

Every habit diagnostic follows one shape:

```text
error[FP2xxx]: Ada named argument `X => 1` is not FPAS syntax
  7 |   const A := F(X => 1);
    |                ^^^^^^
  help: FPAS writes named arguments with `:=`: `F(X := 1)`.
```

- **Message:** names the recognized habit and its origin when unambiguous.
- **Span:** covers the whole foreign construct, not just the first unexpected token.
- **Help:** "FPAS writes …" plus a concrete example built from the user's own names where the
  parser has them, otherwise a fixed example.

To prepare A3 without building it, each module constructs hints through one small helper that
takes the span, the foreign text and the FPAS replacement text separately. A3 can later attach
the replacement as structured data at that single point. No new field is added to
`Diagnostic` in this plan.

## Rust documentation

New modules carry `//!` headers linking the handbook section defined in
[06-docs-and-tests.md](06-docs-and-tests.md#handbook), as required by `AGENTS.md`.
