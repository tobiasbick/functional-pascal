# AP11.2: One keyword per declaration

Package: [AP11: Individual declarations](README.md)

Status: complete.

## Result

Every type, constant and variable declaration repeats its keyword. Exported
declarations repeat `public`; visibility is not inherited. Program and unit
scopes allow types, constants and variables. Local statement lists allow
`const` and `var`, while local type declarations are unsupported.

FP2015 identifies a declaration missing its keyword and shows the complete
prefix. Formatter, generated APIs and source consumers emit individual
declarations while preserving names, scope, exports and initialization order.

## Regression coverage

Parser, formatter, compiler, CLI and editor tests cover each keyword, recovery,
visibility, local scope, imports, initialization effects and round trips.
The [handbook declaration tests](../../../../crates/fpas-parser/tests/documentation/declarations.rs)
extract the formatter's complete type snippet and the optional-handler example
directly from Markdown. Both examples parse with individual keywords; removing
the second keyword reproduces FP2015. The formatter snippet includes its
built-in option alias.
See [constants](../../../pascal/language/basics/constants.md),
[variables](../../../pascal/language/basics/variables.md) and
[formatter style](../../../pascal/tools/fmt-style.md).
