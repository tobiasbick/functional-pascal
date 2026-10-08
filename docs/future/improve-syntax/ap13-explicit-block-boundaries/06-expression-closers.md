# AP13.6: Expression closers

Package: [AP13: Explicit block boundaries](README.md)

Status: complete.

## Result

Anonymous routines close with `end function` or `end procedure`; record updates
close with `end with`. Expression closers have no terminating semicolon of
their own. Any following semicolon belongs to the enclosing statement or
binding. An anonymous routine argument ends before `,` or `)` without an extra
semicolon; its body statements still require their terminators.

Diagnostics and recovery distinguish expression endings from declaration and
statement endings. Formatting preserves nesting, comments and field layout.
Record literals were later removed by AP10.3.
Decision expressions and task scopes remain planned in AP21 and AP26.

## Regression coverage

Parser, formatter, compiler, CLI and editor tests cover arguments, bindings,
returns, nested closures, updates, rejected extra semicolons and evaluation.
See [closures](../../../pascal/language/functions/closures.md) and
[record updates](../../../pascal/language/types/record-update.md).
