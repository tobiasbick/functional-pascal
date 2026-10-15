# 04 – Recovery and false positives

How foreign spellings are recognized without follow-up errors and without ever firing on valid
FPAS. Part of the [foreign-habit diagnostics plan](README.md).

## Problem

The probe in [02-catalog.md](02-catalog.md) shows that the main weakness is not missing hints but
**cascades**: `procedure Q;` yields 10 errors, Ada `function F return T is` yields 12, `const N = 3`
yields 8. The first message is generic and the real cause is buried. A good habit diagnostic is
only half the work; the parser must also resume at the right place.

## Rules

1. **Recognize only after the valid path failed.** Detection runs in the error branch of the
   existing parse step (where `expect`/`parse_error` is called today). Valid FPAS never reaches it,
   so a habit diagnostic cannot change the meaning of a correct program.
2. **Recognize by a short, distinctive token pattern.** Each detector looks at the current token
   and at most a fixed small lookahead (typically 1–3 tokens). Examples:
   - `const` `Ident` `=` → Pascal untyped constant
   - `Ident` `=>` → Ada named argument (as `=` `>`, since FPAS has no `=>` token)
   - `function` `Ident` `return` → Ada function header
   - `/` `=` with no whitespace between → Ada inequality
3. **Consume the whole foreign construct, then continue as if the FPAS form had been written.**
   For example after `const N = 3;` the parser records a constant `N` with an unknown type so later
   uses of `N` do not report "undefined". Where reconstruction is not reasonable, skip to the next
   synchronization point (`;`, a named `end`, or a declaration keyword) and suppress errors until
   then.
4. **One habit, one diagnostic.** A detector that fired marks its span; diagnostics inside that span
   from the same parse step are dropped. Sema does not report on AST nodes marked as recovered from
   a foreign construct.
5. **Lexer forms are tokens.** Foreign characters such as `@`, `^`, `"`, `&&` produce a dedicated
   error token with the foreign text, so the parser can treat `"abc"` as a string literal for
   recovery and `&&` as `and`.

## Ambiguities to respect

| Case | Risk | Rule |
| --- | --- | --- |
| `begin … end;` after `then`/`do` | Valid FPAS: a nested block followed by the missing `end if;` | Only diagnose when the enclosing named ending is missing; mention both possibilities |
| `/=` | `A / =B` is never valid either; low risk | Require adjacency of `/` and `=` |
| `Result :=` | A local variable named `Result` is valid | Only fire when `Result` is not resolvable; sema decides, not the parser |
| `F := 1` in function `F` | Valid if `F` is shadowed by a local variable | Only fire when the target resolves to the enclosing routine itself |
| `Inc`, `Exit`, `Length`, `nil` | User code may declare these names | Only fire on unresolved names (existing unknown-name path) |
| `T.Create(...)` | A record may have a method named `Create` | Only fire when no such member exists |
| `--` | `X - -1` is valid | Only treat `--` as a comment at the start of a line or after a statement terminator; otherwise leave it to the expression parser |

## Negative tests

Every detector gets at least one negative test with valid FPAS that uses the same tokens in a
legitimate way, taken from the ambiguity table above. These tests assert **zero** diagnostics.
See [06-docs-and-tests.md](06-docs-and-tests.md#rust-tests).

## Language server

The language server uses the same parser and checker, so improved diagnostics appear in the editor
automatically. Recovery must keep the AST usable for hover and completion around the foreign
construct; the existing LSP recovery tests must keep passing.

## Decisions

- **Q-3 (decided).** Recovery reconstructs declarations and routine headers (rule 3, first
  option); statements such as `for I in 1 .. 3 loop` skip to the next synchronization point.
