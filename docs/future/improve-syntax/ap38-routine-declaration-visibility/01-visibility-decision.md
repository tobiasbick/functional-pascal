# AP38.1: Visibility decision

Package: [AP38](README.md). Status: complete; agreed contract recorded.
Implementation and verification are delivered by [AP38.2](02-routine-signatures.md).

## Recorded decision

The user selected scope-wide named routine signatures with source-order value
initialization. Free routines, record methods, and nested named routines follow
the same rule. Later declarations and mutually recursive sibling routines are
allowed without `forward` or additional invocation syntax.

Name resolution selects the nearest lexical declaration. An inner named
routine shadows an outer name throughout the inner scope; duplicates in one
scope remain errors. Initialization state never causes resolution to fall
back to an outer declaration.

`var` and `const` values keep their initialization order, including callable
bindings. Routine bodies may reference later bindings, but invocation must be
safe at the actual use point. Track direct and transitive dependencies,
including callbacks, contracts, nested captures, and invoked record defaults.
The compiler rejects an unsafe or unprovable use. Creating a closure must not
copy a value whose initialization has not completed. Existing capture modes
and `var`-parameter lifetime restrictions remain in force.

Record defaults retain their declaring lexical environment and execute when
the record is constructed; check availability at that construction. Imported
units initialize before their consumers, and visibility boundaries remain
explicit. Independently compiled signatures must support the same checks.

No automatic initialization reordering, lazy values, user-written effect
annotations, or additional purity restrictions are introduced. A diagnostic
must identify the unavailable dependency and suggest a concrete repair.

## Decision examples

These fragments describe implemented behavior and are covered by AP38.2's
semantic and runtime regressions.

### Forward helper and safe initialization

Accepted: the helper's signature is available, and it uses only its argument.

```pascal
const Initial: integer := Twice(3);

function Twice(Value: integer): integer;
begin
  return Value * 2;
end function;
```

### Mutual recursion

Accepted: both names are available in the same scope; no nesting is required.

```pascal
function IsEven(Value: integer): boolean;
begin
  if Value = 0 then
    return true;
  else
    return IsOdd(Value - 1);
  end if;
end function;

function IsOdd(Value: integer): boolean;
begin
  if Value = 0 then
    return false;
  else
    return IsEven(Value - 1);
  end if;
end function;
```

### Nested shadowing

Accepted: `Outer()` returns `2`. Its earlier nested caller resolves to the
inner `Read()`, never the outer routine, even though the inner declaration is
later in the source.

```pascal
function Read(): integer;
begin
  return 1;
end function;

function Outer(): integer;
  function Use(): integer;
  begin
    return Read();
  end function;

  function Read(): integer;
  begin
    return 2;
  end function;
begin
  return Use();
end function;
```

### Unsafe transitive initialization

Rejected at `First`'s initializer: `Helper()` calls `ReadLater()`, which reads
`Later` before its initialization. Calling `Helper()` after `Later` has been
initialized is safe. The routine bodies themselves may reference that binding.

```pascal
const First: integer := Helper();

function Helper(): integer;
begin
  return ReadLater();
end function;

function ReadLater(): integer;
begin
  return Later;
end function;

const Later: integer := 7;
```

### Captures, defaults, and unknown callables

- A named nested helper may refer to a later enclosing local; call it only
  after the local is initialized. Apply the same availability check when the
  helper is used as a first-class value and its environment is captured.
- A local immutable capture is copied when the closure is created. Reject
  creation before its value is initialized. Mutable captures retain their
  shared-cell behavior; invocation must not access an uninitialized binding.
- A record default may call a later helper. Accept construction after all
  dependencies are initialized; reject premature construction, including
  omitted defaults reached through another routine.
- When a callback's possible dependencies cannot be shown to be initialized,
  reject its use during initialization. Passing a function value must not hide
  an unsafe dependency or bypass existing task/capture restrictions.

## Prerequisites and affected areas

AP11/AP24 are complete. [AP38.2](02-routine-signatures.md) implements this decision
in declaration collection, routine/capture checking, unit interfaces, lowering,
and editor resolution. AP38 replaces their routine/value source-position rules
without changing their completed status or type/member signature contract.

## Documentation and verification

[Declaration order](../../../pascal/language/types/declaration-order.md),
[nested routines](../../../pascal/language/functions/nested.md), related
handbooks, compact reference, grammar guidance, and authoring skill describe
the implemented contract. AP38.2 records regressions for these examples,
captures, defaults, callbacks, independent units, and editor identity.
No contract decision remains open.
