# AP36: Consistent calls and construction

Status: complete (AP36.1–AP36.2). Effort: large.
Completion is tracked in the [central plan](../README.md).
AP09/AP10/AP20 remain complete; their records include AP36's delivered extensions.

## Result

Fixed-signature calls accept complete positional or named arguments, including
callable values and fixed-arity polymorphic operations. Labels come from the
visible static signature; assignment compatibility remains positional. Enum,
Option, and Result patterns accept either complete field form, independently
at each nesting level. Some/Ok/Error use the canonical named payload `Value`.
Records retain named construction and defaults; variadic calls retain their
positional sequence. Compiler, formatter, editor, grammar, and handbook agree
on these rules. [AP36.2](02-unified-call-rules.md) records implementation owners,
regressions, and verification.

## Recorded routine-call decisions

Preserve AP09 and extend its all-positional-or-all-named rule to callable values:

- A fixed-signature routine call is entirely positional or entirely named;
  mixing styles in one call is rejected. Named calls use `Name := Value`, with
  case-insensitive names, every parameter supplied exactly once, and no defaults.
- Apply the same rule to free/imported routines, instance/static methods, bound
  methods, closures, callable bindings/parameters/fields, fixed-signature native
  operations, and eligible `go` calls. Fixed-arity polymorphic operations follow
  the same rule; polymorphism alone does not justify positional-only arguments.
- Callable type syntax retains mandatory parameter names. Named calls resolve
  labels from the callee's visible static signature, never from the routine
  selected at runtime. An explicit callable annotation supplies its own labels;
  inferred bindings use their resolved static signature. Label lookup must be
  unambiguous, including case-insensitive names in written callable types.
- Assignment and callback compatibility remain positional: parameter count,
  types, `var` modes, result type, and existing generic/variadic restrictions
  apply. Differently named compatible signatures remain compatible. Assignment
  maps parameter positions; it does not rename or reorder the implementation.
- Named arguments may be reordered. Evaluate the receiver first, then arguments
  once in written left-to-right order, before mapping them to parameter slots.
  Preserve writable-reference selection and the `Name := var Designator` form.
- Variadic routine calls remain entirely positional. Their argument sequence has
  no fixed set of labels; a named prefix followed by positional values is also
  rejected. Existing task eligibility and mutation rules remain unchanged.

AP36 extends AP09's named-call support; it does not remove either call style or
make parameter names part of structural callable compatibility. AP09's current
callable-value prohibition is replaced by AP36.2. AP08's
parameter separators and AP12's deferred callable-expression syntax stay outside
this decision. AP34's inference contract must not change implicitly.

## Recorded construction and pattern decisions

- Preserve AP09.2: user enum construction is fully positional or fully named,
  with every field supplied exactly once and no mixing. Named fields match
  case-insensitively and may be reordered; supplied values evaluate once in
  written order before being assigned to their declared field positions.
- Preserve AP10: records use named typed construction only, including existing
  field defaults, visibility, and aliases. Supplied fields evaluate in written
  order, followed by omitted defaults in declaration order. No positional record
  construction or anonymous record literal is introduced.
- Enum, Option, and Result patterns accept fully positional or fully named
  field lists. Mixing is rejected independently at each nesting level. An outer
  named list may contain a nested positional pattern, and conversely. Named
  labels select declared field positions, may be reordered, and do not bind names.
- Every pattern field must occur exactly once. Use `const Name` to bind a field,
  `_` to ignore it, or an existing comparison/nested pattern to test it. No field
  omission, implicit rest pattern, or field default applies during matching.
  Unknown, repeated, missing, and mixed field labels receive concrete diagnostics.
- Some/Ok/Error construction and patterns support both complete forms, using the
  single payload label `Value` in named forms. `Some(3)`, `Ok(3)`, and `Error(E)`
  remain valid; add `Some(Value := 3)`, `Ok(Value := 3)`, and `Error(Value := E)`.
  `Value` does not restrict the error payload to a string or introduce new accessors.
- `None` and fieldless user enum variants retain their ordinary value spelling
  without parentheses. This decision does not alter empty-record construction
  or zero-argument routine calls, which retain their existing parentheses.
- Bindings remain immutable and scoped to case guards/arms. Binding names need
  not match field names; field-label resolution and pattern comparisons precede
  arm bindings as in AP20. Nested patterns, coverage, and covered-arm checks remain.
- The same positional-or-named pattern lists apply to AP35's nonbinding `is`.
  Every `const Name` in an is pattern remains rejected at any depth, and `_`
  still cannot stand for a whole matched value. Contract operand restrictions remain.

AP36.2 replaces AP20's positional-only field rule and the existing positional-only
Some/Ok/Error constructor restriction. It preserves AP09.2 and AP10 and does not
introduce record patterns, new coverage rules, scalar-conversion forms, implicit
payload access, or changes to AP34/AP35 semantics. AP39 retains coverage ownership.

## Selected examples

These fragments show implemented behavior:

```pascal
function Double(N: integer): integer;
begin
  return N * 2;
end function;

const F: function(Value: integer): integer := Double;
const A := Double(N := 3);
const B := F(Value := 3);
const C := F(3);
```

`F(N := 3)` is rejected: the visible signature names `Value`. The assignment
remains valid even though Double's implementation names its parameter `N`.

Construction and patterns use the same field labels on both sides:

```pascal
const Rectangle := Shape.Rectangle(10.0, 20.0);
const NamedRectangle := Shape.Rectangle(Height := 20.0, Width := 10.0);
const Item := Some(3);
const NamedItem := Some(Value := 3);

case NamedItem of
  when Some(Value := const N): Use(N);
  when None: null;
end case;

const Present: boolean := NamedItem is Some(Value := _);
```

`Shape.Rectangle(const W, _)` and
`Shape.Rectangle(Width := const W, Height := _)` are equivalent pattern forms.
`Shape.Rectangle(const W, Height := _)` is rejected as mixed, and
`Shape.Rectangle(Width := const W)` is rejected for a missing field.

## Open decisions

None within the delivered scope. Both work packages are complete, including
implementation, regressions, documentation, and independent review.

## Dependencies and work packages

AP09/AP10/AP20 are complete. AP36.1 precedes AP36.2. AP34 owns type inference;
neither package may change the other's rule implicitly. Callable-label handling
must cover inferred bindings, generic specialization, decision expressions,
aliases, and callable-valued aggregates without selecting runtime callee names.

- [x] [AP36.1: Argument conventions](01-argument-conventions.md)
- [x] [AP36.2: Unified call rules](02-unified-call-rules.md)

## Acceptance

Extracting a routine into a callable value follows the same positional-or-named
call rule, with labels from its visible static signature and existing structural
compatibility. Enum/Some/Ok/Error constructors and patterns accept either complete
form, while records retain named construction and defaults. Every pattern field
is explicit, mixing is rejected per field list, and nested lists are independent.
Argument evaluation and `var` behavior remain predictable and are preserved.
