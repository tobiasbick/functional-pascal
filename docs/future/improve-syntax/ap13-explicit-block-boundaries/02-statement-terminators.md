# AP13.2: Statement and declaration terminators

Package: [AP13: Explicit block boundaries](README.md)

Status: complete.

## Result

Every statement and declaration requires `;`, including the final statement
before a closer, `elsif`, `when`, `else` or `until`. A program's final main-block
`end.` is the exception. Expressions have no terminator of their own.

Control-flow bodies are statement lists with named closers. Parser recovery
preserves following statements and owning boundaries, while formatting emits
every terminator and preserves attached comments.

## Regression coverage

Parser, formatter and execution tests cover missing final terminators,
branches, loops, repeats, comments, nested closers and idempotence.
See [grammar](../../../specs/grammar.ebnf) and
[formatter style](../../../pascal/tools/fmt-style.md).
