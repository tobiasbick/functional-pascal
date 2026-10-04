# Checked integer arithmetic and IEEE real operations

This delivery implements the runtime arithmetic and static checks of
[stage 4](../stages/04-functional-core.md). The two findings below are corrected
with the user's authorization and verification passes. The additional standard-constant and imported-storage findings below are also
corrected and verified. Later coordinated deliveries are recorded in
[purity/defaults](purity-and-defaults.md) and [member removal](member-functions.md).

## Implementation and coverage

`fpas-ir/src/constants.rs` exposes checked scalar evaluation separately from
optional optimizer folding. Failed integer operations remain executable outside
static contexts. Addition, subtraction, multiplication, negation, division and
remainder use signed 64-bit checked operations.

`fpas-sema/src/interface/constants/` separates static value storage,
interface conversion, aggregate data, operator selection and error-aware evaluation. The checker
uses lexical declaration identities and reports F2020 for invalid static binding
components, record defaults, scalar labels and range endpoints. Lazy Boolean
evaluation skips the unused operand. Ordinary calls remain non-static.

VM typed, dynamic and immediate paths share checked integer operations.
Integer overflow retains F4012, with F4001/F4002 for zero division/remainder.
Real folding and execution follow binary64 IEEE division and comparisons:
signed infinities and NaNs are values, and NaN ordering returns false.
Invalid dynamic operand types still report their existing type diagnostic.

Boundary regressions cover both integer endpoints, overflow operations,
truncating division and signed remainder, lazy static evaluation, aggregate
components, generics, emitted failing operations, typed/dynamic/immediate VM
instructions, imported scalar/default metadata and reused unit/program artifacts.
CLI tests check located semantic/runtime JSON diagnostics and isolated runner
classification. Existing checked conversion and bit-shift implementations are
reused, with source-level coverage of finite conversion boundaries, NaN rejection
and zero-filled shifts at counts 0 and 63.

Current operators, constants, numeric-literal and diagnostic pages describe the
implemented rules. Old runtime-only tests now use writable initializers or
routine calls where evaluation is intentionally non-static.

## Verification and newly discovered baseline issue

The focused semantic/compiler/VM numeric matrix passes in debug and release
builds; the IR constant boundary tests also pass in release. Imported IEEE
constants and defaults pass cold/warm reuse tests. The full FPAS suite reports
470 passed, one intentional skip and zero failures. Source formatting passes.
All 109 example/application programs and 57 complete handbook programs pass in
their appropriate source/project contexts, including 22 project manifests.
Changed Markdown links and the Git diff check pass.

The first workspace run exposed old wraparound expectations and statically
invalid debugger failure fixtures. The compiler boundary fixture now uses a
representable endpoint; debugger fixtures read a writable zero binding so the
division remains a runtime operation. Focused compiler/debugger verification
passes after these migration corrections. The real editor fixtures use the same
runtime form. Final verification passes: `cargo fmt`, `cargo build` and
`cargo test --workspace --no-fail-fast` (3,807 tests across 192 groups), FPAS
formatting and the full bundle (470 passed, one intentional skip), and the real
VS Code host. The expanded semantic/compiler numeric matrix also passes in
release (13 tests). All 109 example/application programs, 22 project manifests
and 57 complete handbook programs pass in their source/project contexts.
Changed documentation links and the Git diff check pass. The compiled-program
format page now matches the actual envelope/bytecode versions, 15/16.

The artifact audit discovered that
`crates/fpas-build/build/compiler_identity.rs` excludes `fpas-ir` from its source
fingerprint. This omission exists in the pre-change checkout. Both `fpas-sema`
and `fpas-compiler` depend on that crate, whose folding behavior affects stored
constants and executable code. An IR-only source change does not invalidate
derived artifacts. The existing inventory test repeats the same incomplete list.
The current numeric delivery changes included semantic/compiler sources too,
so those changes invalidate artifacts, but the general omission remains.

The user authorized correcting both findings and continuing the plan. The
fingerprint now includes IR sources. Tests exercise the real hash calculation:
editing an IR file and adding a nested IR module each change the identity;
unchanged contents and a relocated checkout preserve it.

A subsequent manual counterexample finds a gap in the new static checker:
`const Value := [1 div 0] = [0];` passes `fpas check` (exit 0), then `fpas run`
reports F4001 (exit 2). Static classification recognizes the expression, but
scalar evaluation returns no value for the aggregate comparison. The validator
does not visit its embedded array element, so the required F2020 is missing.
The same ownership boundary needs tests for membership and comparisons nested
under lazy Boolean operators; a fix must preserve skipped operands.

The shared evaluator now visits eager aggregate components and uses runtime
structural equality and dictionary normalization to evaluate comparison guards.
It preserves skipped operands, written field order, omitted defaults, lexical
declaration identities and alias field paths. Tests cover both sides of equality,
membership, nested collections, records/updates, Option/Result, enums, NaNs and
dictionary order/duplicate keys.

Static aggregate metadata is persisted separately from scalar defaults in
`fpas-unit/src/interface/values.rs`. It accompanies immutable runtime globals;
compiler imports and object visibility retain that storage model. The envelope
format is version 9, and stale sidecars rebuild automatically. Metadata encoding,
canonical names, real bits and dependency-hash changes have regressions. Real CLI
tests cover cold/warm imports and diagnose newly reached errors after an imported
aggregate value changes.

## Standard constant resolution correction

Extending the aggregate audit to standard constants revealed another missing
source of static values. Before the authorized correction, this program passed
`check` (exit 0) and reported F4001 when run (exit 2), instead of static F2020:

```pascal
program StaticBuiltinEdge;
uses Std.Math as Math;
begin
  const Value := ([Math.Pi] = [Math.Pi]) and (1 div 0 = 0);
end program;
```

Semantic registration marked `Math.Pi` static but retained only its type and symbol
kind. Its value belonged to compiler lowering, so the checker could not establish
the aggregate guard. The scalar evaluator before this delivery also lacked
standard constant values. Aggregate evaluation exposed that existing ownership
gap in reached-operation checking.

The user authorized sharing existing intrinsic constant definitions between
checker and compiler. Their common owner is now
`crates/fpas-std/src/std_units/symbols/constants.rs`; the compiler-only module is
removed. Semantic lookup reaches the common values only after resolving a checked
static symbol, and lexical declaration identities still take precedence.

Six new regressions cover all registered standard constants and their declared
types, case-insensitive names, forbidden unqualified access, independent local
bindings, reached/skipped scalar and aggregate guards, runtime agreement, case
labels, and cold/reused exported values and record defaults. Math documentation
also uses the existing alias-only access and alias-reservation rules.

## Imported storage alias collision correction

The new CLI regression exposed a separate compiler error when an import alias
matches an exported global's name. It also reproduces with the preceding binary
and the same root-resolution and short-name installation code exists in HEAD:

```pascal
unit Demo.Data;
public const Values: array of (integer) := [1];
end unit;
```

```pascal
program Main;
uses Demo.Data as Values;
begin
  const Copy := Values.Values;
end program;
```

In a project containing both files, `check` and `run` exit 1 with F9001,
`field access on non-record`, at `Values.Values`. The import is legal under the
existing alias-only source contract. Compiler import installation also registers
the global under the unqualified name `values`. `designator_root` then treats the
alias as that global and consumes one segment instead of two.

The user's standing permission to fix pre-existing bugs covers this correction.
Import installation no longer creates unqualified imported-global entries. Source
aliases resolve through canonical unit prefixes, while local and unit-owned
storage retain their existing lookup rules. This implements the current import
contract without adding a language exception.

The standard-constant regression uses a distinct alias so its cold/warm value
coverage remains independent. Two separate CLI regressions cover equal alias and
member names, mixed case, snapshots, field/index writes, var calls and native array
mutation. They also cover unrelated imported globals named `Math` and `Console`,
which must not suppress standard constant access. Reused artifacts preserve the
same behavior; readonly paths still report F2005, and forbidden unqualified/full
unit access reports F2003 rather than an internal compiler failure.

This numeric checkpoint preceded coordinated purity/default integration.
The owning stage records final acceptance.

Verification for both additional corrections: `cargo fmt`, `cargo build`,
`cargo test --workspace` (3,815 passed across 192 groups), five release standard
constant tests, FPAS formatting and the full bundle (470 passed, one intentional
skip, zero failures). All 109 example/application programs pass in their proper
source/project contexts, with the same 14 loose entries requiring projects;
22 projects and all 57 complete handbook programs pass in their proper contexts.
Changed documentation links and the Git diff check pass. The original standard
guard now reports F2020, and the original alias collision runs successfully.
