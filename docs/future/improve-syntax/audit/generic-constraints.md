# Generic forwarding and record-default corrections

Scope: corrections required before stage 4's generic-data and decision-expression
implementation. The [owning checklist](../stages/04-functional-core.md) remains
open. This is not delivery evidence for generic records, enums or decisions.

## Corrected generic forwarding

An unconstrained generic caller could bypass a callee's `Numeric` constraint when
both used the same parameter name. With different names, a valid constrained
caller was rejected instead. Both causes were present before the language-plan
rewrite and the callable delivery.

Call-site inference now maps callee parameters to the caller's generic types,
including nested collections and callable signatures. Constraint checks validate
the caller's declared capability rather than skipping generic arguments.
`Numeric` guarantees `Comparable` and `Printable`; `Comparable` guarantees
`Printable`. An unconstrained parameter provides no constrained guarantee.
Independent caller parameters remain distinct, including when their constraints
are equal.

Constraint rules were extracted to
`crates/fpas-sema/src/types/constraints.rs`; inference and validation reuse them.
Current documentation is updated in
`docs/pascal/language/functions/generic-routines.md` and
`docs/pascal/language/types/generics.md`.

## Verification

| Evidence | Result |
|---|---|
| `fpas-sema` generic forwarding tests | Eight passed; positive/negative checks for matching and renamed parameters, unconstrained/weaker constraints, capability implications, nested arrays/callbacks, procedures and distinct caller parameters |
| Full `fpas-sema` tests | 451 unit tests and 18 integration tests passed |
| `fpas-compiler` execution tests | Two passed; chained integer/real forwarding and array/callback/procedure forwarding |
| `fpas-cli` project tests | Two passed; imported constraints execute and invalid forwarding produces one located JSON `F2013` diagnostic |
| `cargo fmt`, `cargo build`, `cargo test --workspace` | Passed; 3,498 tests across 188 successful test groups |
| Original standalone reproductions | Constraint bypass now fails with `F2013`; valid renamed forwarding succeeds |

## Corrected record-default expressions

Continuing the construction audit exposed two existing failures for the static
default expression `1 + 2`. The literal `3` works. These failures are independent
of the forwarding correction; their implementation paths are unchanged.

```pascal
program RecordDefaults;

type Settings = record
  Count: integer := 1 + 2;
end record;

begin
  var Value: Settings := record end record;
end program;
```

Before the correction, both `fpas check` and `fpas run` reported internal error `F9001`: expression type is
missing after semantic analysis. Sema checks the source expression, then clones
it into `RecordDefaultsMap`; lowering receives cloned defaults while expression
types use original AST addresses as keys. The cloned binary expression has no
matching type entry. Owners are `check/decl/types/records.rs`, `check/context.rs`
in `fpas-sema` and `lowering/aggregates/records.rs` in `fpas-compiler`.

For a unit-exported record, declare `public type Settings` and `public Count`
with the same expression. Before the correction, an importing program failed with internal `F9001`:
`cannot persist semantic interface type: exported record field defaults must be
scalar constant expressions`. In `fpas-sema/src/interface/export.rs`, the
constant exporter handles literals, parentheses and negation, but does not
evaluate binary static expressions. A public record default of `3` checks and
runs successfully through the same project path.

The user authorized both corrections. Field defaults now use shared AST nodes,
so default expansion retains expression types, intrinsic dispatch and closure
identities. Closure discovery includes record defaults. Field types provide
context for nested record and collection expressions.

Scalar interface export evaluates literal/operator expressions and references to
local or directly imported constants. It reuses the constant evaluator extracted
from compiler optimization into `crates/fpas-ir/src/constants.rs`. Boolean
short-circuiting and the current arithmetic runtime behavior are preserved.
Explicit fields skip their defaults; executed failing defaults still fail at
runtime.

Ten new Rust regressions cover local construction contexts, nested records,
collections, intrinsics, callable defaults, overrides, runtime errors, exported
operators/constants, imported aliases and located negative diagnostics. The FPAS
runner regression is `tests/runner/record_default_expression_test.fpas`.
Targeted tests and `cargo build` pass. FPAS formatting passes, and the full FPAS
suite passes 462 tests with one intentional skip. `cargo fmt`, `cargo build`
and the full workspace suite pass: 3,508 Rust tests across 188 successful test
groups. The string-ordering opcode regression uses runtime parameters so
constant folding cannot remove the operations it verifies.

The original generic-data/construction/pattern/decision checkbox remains open.
Constructor order and default purity remain part of the coordinated stage-4 and
stage-5 work; these bug corrections do not mark that work complete.
