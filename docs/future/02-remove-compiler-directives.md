# Remove: Compiler Directives

> Priority: 2 — isolated preprocessor stage, no downstream features depend on it.

## What to remove

- The entire preprocessor layer.
- Directives: `{$DEFINE}`, `{$UNDEF}`, `{$IFDEF}`, `{$IFNDEF}`, `{$ELSE}`,
  `{$ENDIF}`, `{$I}`, `{$INCLUDE}`.
- The `fpas-lexer/src/preprocessor/` module.

## Current state

No example uses compiler directives. Conditional compilation is not needed
for a single-platform TUI language.

## Scope

- **Lexer:** remove `preprocessor/` module and directive token handling.
  `{$...}` becomes a lexer error ("unknown syntax").
- **Grammar (EBNF):** remove `directive` and `directive_content` productions.
- **Docs:** remove `12-compiler-directives.md`. Remove directive mention from
  `01-overview.md` and any cross-references.
