# AP17.3: Caller-mutating intrinsics

Package: [AP17: Visible caller mutation](README.md)

## Scope

Make intrinsics that change a caller variable use AP17's shared `var` safety
checks and explicit markers on written `var` arguments. Native array `Push`
and `Pop` use the agreed implicit-receiver exception: ordinary dot calls
without a receiver `var` marker or additional parentheses. Remove their
special simple-variable rule. These remain native type operations with one
public call form.

## Prerequisites

- AP17.1 (`var` arguments).
- AP06.1 (catalog and rules, including the recorded outcome of the
  user-requested discussion revisiting the implicit-receiver exception).

Coordinate with AP06.3's removal of the old type-helper units. Record the
concrete implementation order after the requested follow-up discussion and
before implementation; use the agreed native dot form throughout migration.

## Implementation

- Verify how `Push`, `Pop`, and other caller-mutating intrinsics mutate today.
- Record the writable-receiver mode in the registry and catalog-derived
  editor signatures; reuse the AP17.1 checks instead of the special rule.
  Include an implicit writable receiver in aliasing, lifetime, `go`,
  single-evaluation, and failure checks despite its unmarked syntax.
- Apply the agreed receiver exception: `Items.Push(Value)` and `Items.Pop()`
  require a writable receiver and have no receiver marker or extra
  parentheses. Other explicitly written `var` arguments keep their markers.

## Affected areas

- `crates/fpas-sema/src/std_registry/builtins/array/mutation.rs`,
  `crates/fpas-compiler/src/lowering/calls/arrays.rs`, generated
  native operation signatures replacing `lib/api/Std/Arrays.fpas`, and other
  mutating intrinsics found by the audit.

## Migration

Use ordinary dot syntax for every affected native receiver call in all
repository consumers. Add explicit `var` markers only to written arguments
that require them. Coordinate remaining ordinary-call and import removal
with AP06.3.

## Documentation

- Array operation/mutation documentation replacing the former
  `docs/pascal/std/collections/array/mutating.md` unit API, other affected
  pages, and `fluent-calls.md` or its AP06 successor.

## Verification

- Unmarked native calls on writable direct, field, element, and forwarded
  receivers; rejection of `const` and temporary receivers; explicit markers
  on other written `var` arguments. Verify receiver evaluation once, aliasing
  and `go` restrictions, and retained writes on failure. Preserve other
  array values that share storage and the existing `Push`/`Pop` result and
  chaining behavior; FPAS suite.
