# AP13.5: Case arms

Package: [AP13: Explicit block boundaries](README.md)

Status: complete.

## Result

Each case arm starts with `when Labels [if Guard]:` and contains a nonempty
scoped statement list. The next `when`, final catch-all `else`, or `end case;`
ends that arm. Empty-action arms use `null;`; explicit compound blocks add scope.

Labels, pattern matching, guards and exhaustiveness retain their current rules.
Scalar labels and range endpoints require compile-time constants (AP16).
Closed-enum catch-all removal belongs to AP03; explicit payload binding syntax
belongs to AP20.

Catch-all locals are scoped in checking, lowering and capture discovery.
Recovery preserves the next arm and enclosing named endings.

## Regression coverage

Parser, sema, compiler, formatter and CLI tests cover scalar/variant arms,
ordered guards, scopes, returns, loop control, captures and comments.
See [pattern matching](../../../pascal/language/pattern-matching/README.md).
