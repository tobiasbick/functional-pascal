# AP22: Limited local inference

Status: agreed direction (Q14). Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Only the three permitted local initializer forms infer types; ordinary calls
and ambiguous cases require annotations, and public signatures remain explicit
and stable.

## Decisions (Q14)

- Allow omitted type annotations on local bindings only for literals, typed
  construction, and explicit conversions. The type must be evident from the
  initializer itself.
- Ordinary function calls and other initializer forms require an explicit
  type annotation; do not introduce general local inference.
- Explicit local annotations remain valid. Public parameters, results, and
  fields remain fully annotated.
- Empty collections and underconstrained constructors are not guessed.
- AP27 may rely on this limited form after its implementation; it must not
  broaden inference implicitly. AP27 is retained as optional (Q21).

Draft local bindings inside a routine or a plain scoping block; the surrounding
code supplies the referenced types and values:

```pascal
const P := Point(X := 1, Y := 2);
const Limit := 100;
const Level := Percent(Input);
const Name: string := Text.Trim(Input);   // ordinary call: annotation required
```

## Open decisions

- Which literal forms count as literals for inference: scalar literals only,
  or also non-empty array and dictionary literals. Decide before AP22.1.
- Whether enum variant constructors (for example `Shape.Circle(2.0)`) count as
  typed construction. Decide before AP22.2.

## Dependencies

- AP02 (diagnostic codes).
- AP16 (binding keywords).
- AP10.1 for the typed-construction form (AP22.2).

AP27 depends on this package.

## Order

AP22.1 covers literals and explicit conversions. AP22.2 adds typed
construction once AP10.1 exists.

## Work packages

- [ ] [AP22.1: Inference from literals and conversions](01-literals-and-conversions.md)
- [ ] [AP22.2: Inference from typed construction](02-typed-construction.md)

## Acceptance

Only the three permitted local initializer forms infer types; ordinary calls
and ambiguous cases require annotations, and public signatures remain explicit
and stable.
