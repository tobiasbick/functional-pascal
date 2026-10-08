# AP02.6: Parameter declaration diagnostics

Package: [AP02: Structured diagnostics](README.md)

Status: complete.

## Result

FP2014 diagnoses comma-separated and grouped parameter declarations, including
routine, method, anonymous-routine, and callable-type headings. The hint shows
the canonical individually typed, semicolon-separated form:

```pascal
function Add(A: integer; B: integer): integer;
```

Parameter parsing lives in `crates/fpas-parser/src/parser/decl/parameters.rs`.
Recovery preserves the enclosing declaration and following source.

## Regression coverage

Parser tests cover both rejected forms and the canonical form.
See [parameters](../../../pascal/language/functions/parameters.md).
