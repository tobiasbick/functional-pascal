# AP09: Named arguments

Status: complete. Effort: medium. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Named calls have explicit role mapping and predictable evaluation; invalid
mappings receive concrete diagnostics.

## Decisions

- Named arguments use `Name := Value`, for example
  `CopyFile(Source := InputPath, Destination := BackupPath, Overwrite := false)`.
- A call is fully positional or fully named. Names are the public parameter
  names; there are no separate internal/external labels, so renaming a
  parameter changes the API.
- Parameters have no default values; every parameter is passed. Optional
  settings use an options record with field defaults (AP10).
- Named arguments apply to declared routines, methods, enum variant
  constructors, and fixed-signature native operations. Native receivers are
  unnamed; variadic `Format` is positional. Function values (closures, variables,
  or parameters of a function type) are called positionally only, because
  function-type compatibility ignores parameter names.
- Arguments are evaluated in written left-to-right order, even when named
  arguments reorder parameters.

```pascal
CopyFile(Source := InputPath, Destination := BackupPath, Overwrite := false);

const F: function(Value: integer): integer := Double;
const Answer: integer := F(3);         // valid: result consumed
const Bad: integer := F(Value := 3);   // error: named arguments need a declared routine
```

Function results are consumed here to match
[AP04](../ap04-discarded-function-values/README.md); the invalid call isolates
the named-argument restriction.

## Dependencies

- AP02 (diagnostic codes).

AP10 and AP17 depend on this package.

## Work packages

- [x] [AP09.1: Named arguments for routines and methods](01-named-routine-arguments.md)
- [x] [AP09.2: Named arguments for enum variant constructors](02-named-variant-arguments.md)

## Follow-up work recorded elsewhere

AP09 is complete. These gaps are tracked in the plans that own them:

- Named construction of generic enum variants:
  [AP24.3](../ap24-generic-data-structures/03-generic-enums.md).
- Editor rename of record fields used in `record Field := Value; end` literals:
  [AP10.3](../ap10-typed-record-construction/03-remove-record-literals.md)
  removes these literals. AP10.1 supports labels in typed construction.
- Named arguments in debugger evaluation:
  [compiler and language-limit follow-ups](../../compiler-panic-followups.md).

## Acceptance

Named calls have explicit role mapping and predictable evaluation; invalid
mappings receive concrete diagnostics.
