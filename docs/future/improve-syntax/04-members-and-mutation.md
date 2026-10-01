# Members and mutation

See the [steering document](README.md). These agreed directions still require
approval of unresolved semantics and verification of the current behavior.

## AP14: Remove computed properties

Properties are used only in examples and one formatter golden file at planning
time; no library or app API depends on them.

### Decided rules

- `property`, `read`, and `write` are removed without replacement grammar.
- Getters become ordinary instance functions called with parentheses
  (`Cam.Zoom()`); setters become instance procedures.
- Real record fields remain direct data access.

### Tasks

- [ ] Migrate the examples and the formatter golden file; remove the
  property page from `docs/pascal/` and its grammar rule.
- [ ] Diagnose `property` declarations with the method replacement.

Acceptance: computation and state changes are visible as calls, and no
property grammar remains.

## AP15: Remove event declarations

Events are used only in one example and one formatter golden file at planning
time.

### Decided rules

- `event`, the event-only `nil`, and `Assigned` are removed without replacement
  grammar.
- A handler is an ordinary field of type `Option of HandlerType`, set and
  cleared with record methods or record updates and invoked through a `case`
  or an `is` test.
- Multiple subscribers are not designed now; they need a concrete use case.

```pascal
type Button = record
  OnClick: Option of ClickHandler := None;
end record;

if B.OnClick is Some(const Handler) then
  Handler(B);
end if;
```

### Tasks

- [ ] Migrate the example and the formatter golden file; remove the event page
  from `docs/pascal/`, the grammar rule, and the `nil` literal.
- [ ] Diagnose `event` declarations and `Assigned` with the replacement.

Acceptance: existing event use cases are expressed with ordinary fields and
`Option`, and no event grammar remains.

## AP16: Immutable and mutable bindings

Current behavior: `var` declares an immutable binding, `mutable var` a mutable
one, and `const` a compile-time constant
([variables](../../pascal/language/basics/variables.md)). Readers trained on
Pascal and most other languages expect `var` to be reassignable.

### Decided rules

- `const` prevents reassignment of the binding and allows a computed initial
  value; `var` permits reassignment. Both initialize on scope entry.
- The `mutable` keyword is removed from the language, for bindings and
  parameters (see AP17).
- A `for` loop variable is an immutable binding for each iteration, written
  without a keyword. Assigning to it is an error.
- Binding immutability is separate from deep immutability of shared handles and
  referenced data.

```pascal
const Caption: string := Text.Trim(Input);
var Attempts: integer := 0;
Attempts := Attempts + 1;

for I: integer := 1 to 10 do
  Log(I);
end for;
```

### Tasks

- [ ] Preserve compile-time constants; reject computed `const` values where a
  compile-time constant is required (for example subrange bounds or case labels),
  with a diagnostic that names the non-constant part.
- [ ] Keep closure capture behavior: a captured `const` is copied, a captured
  `var` shares one mutable cell (today's `mutable var` rule).
- [ ] Diagnose `mutable var` with the replacement `var`, and assignment to a
  `const` with the hint to declare it `var`.
- [ ] Prefer `const` in teaching examples; use `var` only for reassignment.
  Retain explicit types until AP22's limited local inference is implemented;
  ordinary calls continue to require annotations afterward.
- [ ] Migrate existing immutable `var` to `const`, and `mutable var` to `var`,
  in one step with AP17.
- [ ] Test reassignment, dynamic initialization, constant expressions, loop
  variables, captures, and shared values; preserve each migrated binding's
  mutability.

Acceptance: initialization, constant contexts, and captures have explicit rules;
the migration is more than a keyword replacement and preserves mutability.

## AP17: Visible caller mutation

### Decided rules

- Parameters are read-only by default. `var` in the signature allows the
  routine to change the caller's variable, and every call marks the argument
  with `var`: `Increase(var Counter)`. Named form: `Increase(Value := var Counter)`.
- A `var` argument must be a `var` binding or a field or element of one;
  constants, `const` bindings, and temporaries are invalid.
- Two `var` arguments of one call must not share a root variable. This also
  rejects `Swap(var A[I], var A[J])`; use a routine such as `SwapAt(var A, I, J)`.
- `mutable` parameters are removed. Where a routine reassigned a `mutable`
  parameter locally, the migration introduces a local `var` copy; it never
  turns local reassignment into caller mutation.
- Intrinsics that change a caller variable follow the same rule, for example
  `Push(var Items, 3)`.
- A closure cannot capture a `var` parameter, and a `var` argument cannot be
  passed to a `go` call; the reference must not outlive the call.
- A `var` parameter may be forwarded as a `var` argument to another routine,
  written `var Value`.

```pascal
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;

Increase(var Counter);
Increase(Value := var Counter);
Increase(var P.X);
Swap(var A[I], var A[J]);   // error: both arguments refer to 'A'
```

### Tasks

- [ ] Verify existing `mutable` parameter behavior and how array intrinsics such
  as `Push` and `Pop` mutate the caller today.
- [ ] Diagnose a missing `var` at the call site with the corrected call, and a
  `var` argument for a read-only parameter.
- [ ] Specify evaluation order of `var` arguments (root and indices evaluated
  once, left to right) and the visible effect when the routine fails.
- [ ] Test local copies, invalid arguments, aliasing, field and element
  arguments, named `var` arguments, forwarding, rejected captures, and rejected
  `go` calls.

Acceptance: caller mutation is visible at declaration and invocation, aliasing
between `var` arguments is rejected, and local reassignment cannot become a
caller-visible change during migration.
