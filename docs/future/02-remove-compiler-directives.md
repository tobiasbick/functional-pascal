# Remove: Compiler Directives

**Status: done** (implemented).

## What was removed

- The entire preprocessor layer.
- Directives: `{$DEFINE}`, `{$UNDEF}`, `{$IFDEF}`, `{$IFNDEF}`, `{$ELSE}`,
  `{$ENDIF}`, `{$I}`, `{$INCLUDE}`.
- The `fpas-lexer/src/preprocessor/` module.
- CLI flags `-D` / `--define` (conditional symbols).
- Language doc chapter `docs/pascal/12-compiler-directives.md`.

## Rationale

No example uses compiler directives. Conditional compilation is not needed
for a single-platform TUI language. Shared source is expressed with units and
`uses` in multi-file projects.

## Implementation notes

- **Lexer:** `{$...}` is scanned and rejected with a lexer error; no directive token.
- **Parser / project:** lex only — no post-lex preprocessor pass.
- **Grammar (EBNF):** `docs/specs/grammar.ebnf` — directive productions removed; note under brace comments.
- **Diagnostics:** `LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED` (F0010); former F0011–F0014 removed.
