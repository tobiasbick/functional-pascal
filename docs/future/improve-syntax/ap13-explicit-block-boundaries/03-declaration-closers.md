# AP13.3: Named closers for declarations

Package: [AP13: Explicit block boundaries](README.md)

Status: complete.

## Result

Functions, procedures, records, enums and units close with `end function;`,
`end procedure;`, `end record;`, `end enum;`, and `end unit;` respectively.
Methods and nested routines follow the same named routine rules. Units have no
main block; programs retain `begin ... end.`.

The parser checks matching closers and recovers at enclosing boundaries.
Formatter, templates, generated APIs, snippets and consumers use these forms.

## Regression coverage

Parser, formatter, CLI and execution tests cover each declaration kind,
mismatches, missing names, nested declarations, comments and round trips.
See [grammar](../../../specs/grammar.ebnf).
