# AP22: Limited local inference

Status: complete (Q14). Effort: large.
AP34 explicitly supersedes Q14's initializer-form restriction; see
[the current inference rules](../ap34-consistent-type-inference/02-inference-rules.md).
This package records AP22's delivered scope and remains complete.
Completion is tracked in the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Deliver inference for literal, conversion and constructor initializers while
keeping signatures explicitly typed. These forms remain supported under AP34's
uniform rule for every fully determined statement-local initializer.

## Decisions (Q14)

Q14's initializer-form whitelist is explicitly replaced by AP34. The retained
boundaries and supported forms are:

- Local literals, typed construction and explicit conversions infer their checked
  types. AP34 also admits calls, operators and other fully determined expressions.
- Literal forms include scalar literals and non-empty array and dictionary
  literals with uniquely determined element, key, and value types. Empty or
  ambiguous collections require an annotation.
- Enum variant constructors count as typed construction, including payloads
  inferred by AP24. Undetermined type arguments require an annotation.
- Explicit local annotations remain valid. Public parameters, results, and
  fields remain fully annotated.
- Empty collections and underconstrained constructors are not guessed.
- Deferred optional AP27 may rely on implemented inference if reopened;
  it must not broaden inference implicitly. Value and spelling remain open (Q21).

Local bindings inside a routine or a plain scoping block; the surrounding
code supplies the referenced types and values:

```pascal
const P := Point(X := 1, Y := 2);
const Limit := 100;
const Id := UserId(Input);
const Name := Normalize(Input);          // known result: current AP34 rule
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

Literals, conversions and constructors retain complete checked types, fixed
binding types and unchanged evaluation. Unknown types require context, and
signatures remain explicit. AP34 owns the broader expression/context rule.
