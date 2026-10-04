# Generic parameter identity correction

Stage 4's purity audit found a pre-existing generic typing error. Two independent
parameters were compared and substituted by case-insensitive name alone. The
goal's standing permission to fix pre-existing bugs covers this correction.

## Reproduction

The checker accepted this program before the correction. Running it failed with
F4008 because `Inner` returned the captured string while its caller expected an
integer:

```pascal
program GenericShadow;
uses Std.Console as Console;
function Outer of (T)(Value: T): integer;
  function Inner of (T)(Other: T): T;
  begin
    return Value;
  end function;
begin
  return Inner(1);
end function;
begin
  Console.WriteLn(Outer('text') + 1);
end program;
```

The faulty name comparison also exists in the preceding committed source.
The corrected checker rejects the return with F2006 and explains that the two
parameters belong to different declarations. Legal lexical shadowing remains
supported; no spelling restriction or new source syntax was introduced.

## Implementation

- `fpas-parser` retains the source span of each declared generic parameter.
- `fpas-unit/src/interface/generics.rs` owns its serialized declaration identity.
  Source unit names and byte offsets distinguish declarations. Program identities
  also retain the source index; independently compiled units normalize that index
  to zero. Identities contain no filesystem paths or machine metadata.
- `fpas-sema/src/types/generics.rs` owns resolved declarations and argument maps.
  Compatibility, inference and substitution use identities rather than names.
  Inference is limited to the called or constructed declaration's own parameters.
- `fpas-sema/src/check/decl/types/generics.rs` shares identity construction across
  ordinary headers, routine bodies, methods and pending recursive type headers.
- Interface conversion retains identities, including generic parameters inside
  returned callables and nominal data applications. Compiled-unit format 10
  replaces format 9; stale sidecars rebuild through the existing project workflow.
  Source offsets can conservatively change an interface digest when declarations
  move, so dependent units may rebuild even when their visible shapes agree.

## Lexical closure resolution

The positive runtime regression exposed a second pre-existing error: an
anonymous closure calling a sibling nested routine failed with F9001. The binary
built before this correction reproduces that failure. Anonymous runtime names
did not retain their lexical owner, so ordinary named-call resolution could not
find the sibling.

`fpas-compiler/src/lowering/closures/lexical_scope.rs` now retains the owner's
runtime path in anonymous names. Lexical routine lookup handles calls
from anonymous bodies, including nested anonymous bodies and independent factory
activations. Referenced sibling routines are captured as callable values with
their own environments. Owner slots retain exact declaration identities for
debugger capture provenance. Closure parameters cannot hide a sibling's captured
variables; mutable cells and task-bound capabilities remain shared. Root entry
names remain display metadata and do not introduce a callable namespace.
The same regression also verifies that the nearest lexical routine shadows a
same-named root routine. `lowering/context/callables.rs` owns that lookup and
runtime result fallback, split from the larger value-storage module.

## Generic call results

Further runtime checking reproduced another baseline F9001 with
`Identity(41) + 1`. Lowering selected the erased runtime signature before the
semantic instantiation, so the result incorrectly remained dynamic. Source
expressions now retain their checked type in `lowering/context/mod.rs`, and
`lowering/calls.rs` restores it after a named call through existing typed local
storage. No verifier rule is weakened. Regressions cover arithmetic, integer
division, negation, text concatenation, booleans, numeric promotion and generic
factories returning callable values.

## Contextual callable instantiation

Whole-workspace verification exposed a required part of the identity migration:
the TUI runtime supplies generic routines as callbacks. Different declarations'
parameters can match only after the selected routine is instantiated, rather
than by comparing their spelling.

`check/expr/callable_instantiation.rs` instantiates generic routine values from
expected callable signatures. Inference defers unresolved generic callbacks
until other arguments determine the caller's parameters. The second argument
check then specializes the callback and validates its constraints and complete
signature. Direct annotations, procedure values, collections, forwarding,
repeated-parameter mismatches and compiled-unit reuse have regressions.

An additional baseline check accepted `[Identity]` without a concrete callable
context, permitting later calls with unrelated types. The preceding committed
inference code also failed to classify generic callable components as unresolved.
Inference now recognizes those components recursively through containers and
requires every routine type argument to be determined from its arguments.
Explicit nested callable annotations remain accepted. Semantic and real CLI
check/build/run regressions cover the rejection without reaching lowering.
Runtime coverage also verifies nested arrays, dictionary values, record fields,
Option and Result payloads. Lowered named callable values retain their checked
concrete signature through the existing typed-storage conversion, including when
they become wrapper payloads.

## Coverage and status

Focused regressions cover incompatible returns, initializers, assignments and
callback results; frozen enclosing parameters; legal case-insensitive shadowing;
constrained forwarding; nested runtime calls and closures; imported generic
records and closures; warm reuse; CLI check/build/run rejections; and serialized
identity round-trips, canonical unit names and dependency hashes.

Verification passes: `cargo fmt`, `cargo build`, `cargo test --workspace`
(3,835 tests across 192 groups), source formatting and the complete FPAS suite
(470 passed, one intentional skip, zero failures). All 109 example/application
programs and 22 projects pass in their source/project contexts. All 57 complete
handbook programs pass in their source/project contexts; changed documentation
links and the Git diff check pass. The corrections add 20 focused tests beyond
the preceding standard-constant/alias checkpoint. A concurrent project check
timed out on a shared sidecar lock; its uncontended repeat passes.

This baseline correction does not close the coordinated binding/purity/default
acceptance item. The [default-purity boundary](default-purity-boundary.md) records
the approved evaluation-purity rule for that implementation.
