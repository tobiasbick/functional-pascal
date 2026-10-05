# AP09: Named arguments

Status: agreed direction. Effort: medium. Completion is tracked in the
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
- Named arguments apply to declared routines, methods, and enum variant
  constructors. Function values (closures, variables or parameters of a
  function type) are called positionally only, because function-type
  compatibility ignores parameter names.
- Arguments are evaluated in written left-to-right order, even when named
  arguments reorder parameters.

```pascal
CopyFile(Source := InputPath, Destination := BackupPath, Overwrite := false);
Start(Host := 'localhost', Options := ServerOptions(Port := 9000));

var F: function(Value: integer): integer := Double;
F(3);            // valid
F(Value := 3);   // error: named arguments need a declared routine
```

## Dependencies

- AP02 (diagnostic codes).

AP10 and AP17 depend on this package.

## Order

AP09.1 covers routines and methods; AP09.2 extends the same rules to enum
variant constructors.

## Work packages

- [ ] [AP09.1: Named arguments for routines and methods](01-named-routine-arguments.md)
- [ ] [AP09.2: Named arguments for enum variant constructors](02-named-variant-arguments.md)

## Acceptance

Named calls have explicit role mapping and predictable evaluation; invalid
mappings receive concrete diagnostics.
