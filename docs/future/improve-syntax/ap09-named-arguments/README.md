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
- A call is fully positional or fully named. Names come from the visible static
  signature; there are no separate internal/external labels. Renaming a public
  signature parameter changes named calls through that signature.
- Parameters have no default values; every parameter is passed. Optional
  settings use an options record with field defaults (AP10).
- Named arguments apply to declared routines, methods, callable values, enum
  variant constructors, and fixed-signature native operations, including
  fixed-arity polymorphic operations (AP36). Native receivers are unnamed;
  variadic routines, including `Format`, remain positional. Callable assignment
  compatibility ignores names; calls use the receiving static signature.
- Arguments are evaluated in written left-to-right order, even when named
  arguments reorder parameters.

```pascal
CopyFile(Source := InputPath, Destination := BackupPath, Overwrite := false);

const F: function(Value: integer): integer := Double;
const Answer: integer := F(3);         // valid: result consumed
const NamedAnswer: integer := F(Value := 3); // valid: F's static label
```

Function results are consumed here to match
[AP04](../ap04-discarded-function-values/README.md).

## Independent follow-up

[AP36](../ap36-call-and-construction-conventions/README.md#recorded-routine-call-decisions)
retains AP09's complete positional-or-named call rule and implements named support
for callable values using their visible static signature. Differently named
compatible signatures remain positionally compatible; variadic calls stay
positional. AP36 also implements named fixed-arity polymorphic operations,
Some/Ok/Error constructors, and complete named field patterns. AP09.2's two
enum construction forms remain supported. The rules above include these
delivered extensions; AP09 remains complete.

## Dependencies

- AP02 (diagnostic codes).

AP10 and AP17 depend on this package.

## Work packages

- [x] [AP09.1: Named arguments for routines and methods](01-named-routine-arguments.md)
- [x] [AP09.2: Named arguments for enum variant constructors](02-named-variant-arguments.md)

## Related implemented behavior

Named construction of generic enum variants is implemented by
[AP24.3](../ap24-generic-data-structures/03-generic-enums.md).

## Acceptance

Named calls have explicit role mapping and predictable evaluation; invalid
mappings receive concrete diagnostics.
