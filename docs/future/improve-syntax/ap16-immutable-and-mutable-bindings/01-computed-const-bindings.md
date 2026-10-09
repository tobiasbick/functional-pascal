# AP16.1: Computed const bindings

Package: [AP16: Immutable and mutable bindings](README.md)

Status: complete.

## Result

A `const` binding accepts a static or computed initializer at program, unit or
local scope. Local initialization runs once whenever execution reaches the
declaration, in statement order. Loop-body declarations initialize on each
reached iteration; untaken branches do not initialize them. Program and unit
initialization order is preserved. Immutability does not memoize or hoist calls.

Compile-time classification is separate from mutability. Scalar case labels and
both range endpoints require compile-time constants. Calls are computed,
including user routines, methods, intrinsics and native operations. References
to computed constants remain computed, including across unit boundaries;
optimization and apparent purity do not change this classification. Dynamic
conditions use guards. Closures copy captured `const` values.

Runtime initialization and immutable global imports support computed constants;
unit interfaces retain static/computed classification. Subrange bounds are
planned in AP18.1; arrays have dynamic sizes.

Known scalar fields of static records are retained through copies, updates,
nested projections and derived constants. Unit interfaces preserve these fields
for constant checks in imported and transitive consumers while aggregate values
remain immutable runtime globals. Record defaults retain their declaration-scope
values.

## Regression coverage

Tests cover reachability, call counts/order, loops, skipped branches, early
returns, failures, captures, transitive/imported dependencies and constant-label
rejection, record-field pattern coverage and duplicate labels.
See [constants](../../../pascal/language/basics/constants.md).
