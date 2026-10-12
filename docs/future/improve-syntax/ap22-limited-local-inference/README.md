# AP22: Limited local inference

Status: complete (Q14). Effort: large.
Completion is tracked in the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Only the three permitted local initializer forms infer types; ordinary calls
and ambiguous cases require annotations, and public signatures remain explicit
and stable.

## Decisions (Q14)

- Allow omitted type annotations on local bindings only for literals, typed
  construction, and explicit conversions. The type must be evident from the
  initializer itself.
- Literal forms include scalar literals and non-empty array and dictionary
  literals with uniquely determined element, key, and value types. Empty or
  ambiguous collections require an annotation.
- Enum variant constructors count as typed construction, including payloads
  inferred by AP24. Undetermined type arguments require an annotation.
- Ordinary function calls and other initializer forms require an explicit
  type annotation; do not introduce general local inference.
- Explicit local annotations remain valid. Public parameters, results, and
  fields remain fully annotated.
- Empty collections and underconstrained constructors are not guessed.
- Deferred optional AP27 may rely on this implemented limited form if reopened;
  it must not broaden inference implicitly. Value and spelling remain open (Q21).

Local bindings inside a routine or a plain scoping block; the surrounding
code supplies the referenced types and values:

```pascal
const P := Point(X := 1, Y := 2);
const Limit := 100;
const Id := UserId(Input);
const Name: string := Normalize(Input);   // ordinary call: annotation required
```

## Open decisions

None for the scope above.

## Dependencies

- AP02 (diagnostic codes).
- AP16 (binding keywords).
- AP19 (explicit distinct conversions).
- AP10.1 for the typed-construction form (AP22.2).
- AP24 for generic constructor inference (AP22.2).

AP27 retains this dependency if its deferred proposal is reopened.

## Order

AP22.1 delivers literals and explicit conversions. AP22.2 adds record and enum
construction using the implemented AP10.1 and AP24 constructor rules.

## Work packages

- [x] [AP22.1: Inference from literals and conversions](01-literals-and-conversions.md)
- [x] [AP22.2: Inference from typed construction](02-typed-construction.md)

## Acceptance

Only the three permitted local initializer forms infer types; ordinary calls
and ambiguous cases require annotations, and public signatures remain explicit
and stable.
