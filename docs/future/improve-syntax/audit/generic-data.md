# Generic data, construction and decisions

Scope: the completed generic-data, construction, pattern and decision-expression
slice in the [owning checklist](../stages/04-functional-core.md). Generic data,
canonical construction, recursive patterns and lazy decisions are implemented,
including consumer conversion, old-path removal, documentation and verification.
Later bindings, caller mutation, purity and member removal retain their own owners.
Bare task removal follows the [approved coordinated boundary](task-migration-boundary.md).

## Implemented equality foundation

`Equatable` supports equality-only generic code. `Comparable` implies equality
and ordering; `Numeric` implies both. Constraint checks inspect every component
of records, payload enums, Option, Result, arrays and dictionaries, including
recursive nominal references. Native resource records carry explicit resource
metadata that survives compiled-unit interfaces and transparent aliases.
Resource, task and callable components cannot acquire data equality through a
container or a generic constraint.

Runtime data equality traverses nested values with an explicit stack. Arrays
compare ordered elements; dictionaries with distinct keys compare key/value
mappings independently of insertion order. Dictionary indexing, updates,
membership and standard key
operations share structural key equality. Real comparisons use IEEE equality at
every depth: NaN differs from itself, and signed zero compares equal. Bytecode
representation equality retains its separate codec/debug-state behavior.

`Std.Arrays.Contains` and `IndexOf` require equatable elements and search values.
Empty constructors retain absent type components, while inference fills them from
all concrete arguments and collection entries before checking constraints.
This preserves existing Option/Result comparisons and prevents an early empty
argument from concealing a later callable or resource component.

Owners: `fpas-sema/src/types/{constraints,components,inference}.rs`,
`check/expr/{equality,literals,operators}.rs`, generic call inference, standard
resource registration, compiled-unit interfaces, `fpas-bytecode/src/value/equal.rs`,
VM collection operations and `fpas-std/src/{array,dict}.rs`. Collection literal
typing was moved out of the larger expression module. Interface format version
8 records generic data parameters, applications and arguments alongside resource
metadata and the equality constraint.

Current documentation: generic types/routines, operators, records, dictionaries,
array searches, keyword inventory, grammar and editor highlighting.

Dictionary key admissibility and duplicate-key normalization are implemented in
the later construction checkpoint below, rather than claimed by this original
equality preparation.

## Equality preparation verification

Positive, negative and edge regressions cover nested value equality, dictionary
order and dictionary-valued keys, resource aliases, equality-only constraints,
capability forwarding, NaN/signed zero, deep comparison stacks, empty constructor
context, both argument orders and rejection of nested callables/resources.
CLI tests exercise imported constraints and located JSON diagnostics. The FPAS
runner regression is `tests/runner/structural_collection_equality_test.fpas`.

Targeted sema, bytecode, compiler and CLI equality tests pass. FPAS formatting,
the editor grammar verifier and the full FPAS suite pass: 463 tests and one
intentional skip. `cargo fmt`, `cargo build` and the full workspace suite pass:
3,531 Rust tests across 188 successful test groups. Intrinsic editor declarations
were regenerated with no resulting source changes. Rust documentation paths and
the Git diff check pass.

Full verification initially exposed 28 compiler-test failures because the new
component check rejected uninstantiated Option/Result components. That regression
is corrected without weakening concrete component checks. The seven additional
Rust regressions and expanded FPAS runner test cover empty constructor context,
both argument orders, later literal elements, membership and rejection of nested
callables/resources. Existing compiler tests pass unchanged.

## Corrected record-default alias export

An existing exported record can declare a scalar field default:

```pascal
unit Demo.Model;
public type Settings = record
  public Count: integer := 3;
end record;
end unit;
```

A facade reexports the same nominal type:

```pascal
unit Demo.Facade;
uses Demo.Model as Model;
public type Settings = Model.Settings;
end unit;
```

Before the correction, the original interface contained `Count`'s default
`Integer(3)`; the facade's interface contained `None`. A consumer importing only
the facade and constructing `var Value: Facade.Settings := record end record;`
received `F2015`:
`Required field Count is missing from record literal for type demo.model.settings`.
The CLI reproduced one located JSON diagnostic. The direct-import control printed
`3`, and an explicit `Count := 42` through the alias printed `42`.

`interface/conversion/to_interface.rs` reconstructs record descriptors with empty
default metadata. The previous export path reapplied defaults only when the
exporting source declaration itself had `TypeBody::Record`; it skipped aliases.
The nominal record type does not contain the defaults stored in
`RecordDefaultsMap`, so conversion cannot preserve them automatically. Both
paths existed in the pre-redesign baseline and the committed callable
baseline; the authorized scalar default-expression correction did not introduce
this loss.

The authorized correction resolves defaults by the original nominal record
identity, rather than the exporting declaration's syntax. The focused
`interface/record_defaults.rs` module handles exported and imported metadata,
including records embedded in collection aliases. Supporting type interfaces
also install defaults for contextual construction through imported routines.
Existing scalar constant evaluation and default expression identities are reused.

Five semantic regressions cover local and transitive aliases, collection aliases,
supporting types, and original ownership/visibility. A compiler regression covers
default expressions and callable defaults through local aliases. Three CLI tests
cover executable projects, sidecar reuse, overrides, required fields, and located
private-field diagnostics. The FPAS default-expression runner also constructs a
record through an alias chain. Current record and type-alias pages describe the
preserved metadata. The full Rust workspace suite passes: 3,540 tests across 188
successful groups. Format/build, FPAS formatting and the full FPAS suite also
pass: 463 tests and one intentional skip.

## Generic data and named construction

Nominal record and enum descriptors retain declared parameters and concrete
arguments. Deferred applications preserve recursive references without expanding
their type descriptors forever. Header resolution isolates each declaration's
parameter scope; concrete applications require exact arity and satisfy declared
constraints. Concrete aliases retain identity, ownership and defaults.

`TypeName(Field := Value, ...)` resolves a record declaration, infers arguments
from fields and expected types, and checks required, unknown, duplicate and private
fields. Generic enum constructors infer positional payloads and expected arguments.
Payloadless variants are values; missing arguments request an annotation. Routine
inference does not treat the callee's unresolved parameters as concrete context.

The compiler shares one erased layout per generic nominal declaration across
independently compiled units. Projection metadata restores concrete field and
index types for reads, writes and calls. Supplied record fields evaluate in written
order, followed by omitted defaults in declaration order. Enum payloads preserve
positional evaluation order. Shared component checking terminates recursive
applications and retains task, callable, equality and finite-value capabilities
through nested instances of the same generic declaration.

Owners include parser type parameter and construction modules, semantic
`types/{data,substitution,components}.rs`, `check/expr/construction/`,
`check/name_resolution/types/applications.rs`, finite header validation, interface
conversion, compiler layout reservation, aggregate lowering and typed projections.

The construction checkpoint passes format/build, 3,577 Rust tests across 188
groups, FPAS formatting and 464 FPAS tests with one intentional skip. Coverage
includes generic constraints, nesting and recursion, mandatory cycles, alias
metadata, unresolved arguments, nominal identity, callable fields, nested updates,
defaults and evaluation order. CLI tests cover independently compiled units,
concrete reexports and sidecar reuse; the runner includes
`tests/runner/generic_data_construction_test.fpas`.

At that construction checkpoint the old forms still had source consumers.
Their resolved migration and strict removal are now complete, as recorded in the
canonical delivery below. Bare task annotations retain their separate approved
binding/task migration owner.

## Corrected payload-enum pattern identity

Before the authorized correction, this arm was accepted while matching an
`ExpectedChoice` value, even though the declared payload types differed:

```pascal
type ExpectedChoice = enum Present(Value: integer); Missing; end enum;
type OtherChoice = enum Present(Value: string); Missing; end enum;
// In a case on ExpectedChoice:
// when OtherChoice.Present(Value): ...
```

An unknown qualifier such as `MissingQualifier.Present(Value)` was also accepted.
Both CLI reproductions passed check and printed `42` when matching
`ExpectedChoice.Present(42)`. Subsequent correctly qualified arms covered every
variant, so the reproduction does not depend on an `else` exception.

`collect_variant_pattern_bindings` previously took only the final member name and
looked it up in the expected enum. It did not resolve the supplied qualifier. This
code is unchanged in both the pre-redesign baseline and the committed callable
baseline. The current pattern documentation already required the pattern variant
to belong to the scrutinee type; this was a missed implementation check.

The checker now resolves the full source name through the ordinary import rules,
requires an enum-variant symbol and checks nominal declaration identity. Concrete
alias arguments must agree with the scrutinee. Indexed paths are not variant names.
Generic declaration patterns use the expected concrete payload types, including
recursive applications and callable payloads. The compiler distinguishes erased
payload storage from the concrete binding type.

Enum-pattern checking was split from the larger label module into
`check/stmt/control_flow/if_case/enum_patterns.rs`. Five semantic regressions cover
foreign declarations, unknown/indexed qualifiers, concrete aliases, generic and
recursive payloads. CLI coverage checks located JSON errors, explicit import aliases,
reexports and sidecar reuse; compiler coverage executes callable payloads. The full
Sema suite and targeted compiler/CLI regressions pass. The owner correction's
full format/build/workspace checkpoint passes 3,584 Rust tests across 188 groups.

## Nested patterns and value decisions

The parser represents recursive patterns separately from expressions. Explicit
`const` bindings, payload `_`, qualified variants, static values and ranges share
one checker for statement and value cases. Full nominal-owner resolution remains
in place. Bindings have precise declaration identities, arm-local scopes and
immutable types; grouped labels share one binding set and one guard/body.

Coverage specializes pattern products through enum, Option, Result and Boolean
constructors. Guards do not establish coverage. Closed recursive cases require
all nested payload alternatives and explicit top-level variants, and reject
catch-alls and unreachable rows. Open scalar statements may fall through; value
cases require a fallback unless finite coverage is proven. Integer and string
range coverage detects rows covered by several earlier ranges.

`if` and `case` expressions hold exactly one value per branch. Expected types
reach constructors, wrappers and collections. Without context, compatible
branches complete inference holes together, independently of their order.
Unresolved values, incompatible branches and procedure calls are rejected.
Procedure values can be selected and stored. Callable task-bound state propagates
through selected values, enum payloads and pattern bindings.

Lowering evaluates conditions and scrutinees once, reads payloads only after
checking their variant, and runs only the selected branch. Merge locals preserve
typed values across expression continuations. Pattern-bound closures retain the
correct arm binding, including grouped alternatives and multiple same-typed
payloads. Formatter, source maps, capture traversal and closure discovery visit
the complete decision AST. Stopped debugger evaluation explicitly rejects these
compiled expressions.

New focused owners are parser `ast/{pattern,decisions}.rs`, `parser/patterns.rs`,
`parser/expr/decisions.rs`; sema `check/patterns/`,
`check/expr/{decisions,expected,try_values}.rs`; compiler
`lowering/case/{patterns,recursive}.rs`, `lowering/expr/{decisions,operators}.rs`;
and formatter `emit/expr/decisions.rs`. Existing scalar constant evaluation is
shared with coverage instead of duplicated. Expression dispatchers were split
before extending their responsibilities.

Targeted parser, sema, compiler, formatter and imported-project CLI regressions
pass. `tests/runner/nested_decisions_test.fpas` executes generic Option payloads,
ordered lazy selection and an escaping closure with two integer bindings.
The existing scalar-label gap identified during integration is corrected as
recorded below. Canonical pattern and construction removal is complete in the
final delivery.

## Existing scalar-label constant-check gap

The full checkpoint exposed a separate existing gap. Scalar labels in
`check/stmt/control_flow/if_case/labels.rs` check expression types but do not
require compile-time values. The same path is unchanged in the pre-redesign
baseline and the committed callable baseline. The existing grammar already says
value labels must be compile-time constants; the target also requires static
values and range endpoints.

A program with `when NextLabel():` passes CLI check and executes the function
during matching. The reproduction prints `matched` followed by `1`, confirming
one side effect. A separate `when Lower..Upper:` range with variable bounds also
passes check and prints `matched`. All four check/run operations exit zero.
These forms still use the unchanged scalar-label path, so acceptance does not
come from the new nested-pattern checker.

Two compiler regressions, `case_label_preserves_subject_across_try` and
`case_range_preserves_subject_and_lower_comparison_across_try`, explicitly expect
non-static `try ReadValue(...)` labels. Recursive pattern checking now rejects
them with F2014 instead of executing them. The other 213 compiler tests pass in
the full retry. The earlier wildcard transition regression and this original
constant-check gap have distinct causes.

The user authorized the correction and continuation. Static-value checking now
shares `check/patterns/values.rs` for every scalar label and range endpoint.
Three semantic and two CLI regressions reject variables, calls, `try` and dynamic
endpoints; positive coverage includes imported static constants and sidecar
reuse. The two compiler fixtures execute their dynamic comparisons in valid
explicit-binding guards, preserving success/error paths and evaluation order.
Two process-API tests now assert all three located incorrect array elements
reported by contextual typing. The parser/sema/compiler/formatter checkpoint
passes 1,191 tests across 23 groups. The full dictionary/decision checkpoint below
also passes. Final delivery verification is recorded below.

## Dictionary construction

Dictionary keys use the existing structural equality component checker. All
represented equatable value types are supported; resource, task and callable
components are rejected through containers and aliases. Generic keys require
`Equatable` or a stronger capability. Forward key-type checks wait until all
nominal headers are resolved. Literal inference and expected contexts apply the
same check without restricting stored values.

Runtime `Value::dict` normalizes equal keys by retaining their first position and
last supplied value. This matches ordinary indexed writes and dictionary merge.
VM literal lowering already evaluates every key/value once in source order, so
normalization preserves effects. Mapping-valued keys compare independently of
their internal insertion order; signed zero and NaN retain IEEE equality.

Focused owners are sema `check/expr/dictionaries.rs`, pending header checks in
`check/decl/types/collection.rs`, and bytecode `value/dictionary.rs`. Five semantic
tests cover supported/unsupported keys, generic capabilities, forward references
and value containers. Two bytecode tests cover mapping keys, replacement order,
signed zero and NaN. Two CLI tests cover independently compiled generic record
keys, aliases, located errors and sidecar reuse. The FPAS runner is
`tests/runner/dictionary_construction_test.fpas`. Format/build and the full Rust
workspace checkpoint pass: 3,629 tests across 188 groups. FPAS formatting and the
complete FPAS suite pass: 466 tests, one intentional skip, zero failures. Editor
grammar verification and the Git diff check also pass.

## Canonical source delivery

Resolved conversion covers standard-library units, applications, examples and
regression sources, including imported aliases and generic callable contexts.
Rust-embedded complete sources and assembled fragments, handbook snippets,
generated intrinsic declarations, authoring guidance and editor fixtures use the
same forms. Intentional error fixtures preserve their original semantic failure.
Conversion does not introduce implicit local inference or change binding mutation.

All generic `of` lists are parenthesized and routine/data headings use `of (...)`.
Records construct through visible type names with named fields. Every enum variant,
including Option and Result, is qualified; payloadless variants are values.
Short enum aliases and ambiguity-only resolver tables are removed. Unique short
names remain errors even with an expected type; ordinary same-named routines
continue to resolve independently.

`CaseLabel` shares the recursive `Pattern` AST. Old destructuring variants,
implicit scalar binding tables and separate enum/scalar lowering paths are removed.
Both statement and expression cases use checked bindings and coverage. Proven
coverage reaches lowering, so exhaustive imported enum aliases and finite scalar
cases can return without an implicit unit path. Declared enum backing values are
preserved through aliases and independent compilation.

Executable anonymous records are removed from parser/AST, semantic inference,
nominal compatibility, compiler aggregates, capture traversal, formatting and
debugger expression lowering. `InvalidRecord` retains only a recovery span;
it discards old field expressions and always produces a semantic error. The
single located diagnostic uses an assignment, argument, collection or return's
resolved nominal target, including original identity through reexports. Without
context it requests named construction rather than guessing from visible fields.
The formatter rejects this recovery node. Named field/default validation lives
in `check/expr/construction/fields.rs`; static construction still checks omitted
defaults.

Current generic type rendering is canonical in semantic diagnostics and portable
debug metadata. Nested debug names parse as source types. The debugger continues
to reject compiled decision/construction expressions through its existing metadata
boundary. Enum/Result/Option replacement expressions use qualified constructors;
protocol descriptor names and value summaries retain their existing display form.

The approved bare-task dependency remains in coordinated local inference and
procedure-task work. No source-level `unit` type or permanent exception was added.

### Final verification

- `cargo fmt`, `cargo build`, `cargo test --workspace`: 3,649 tests passed across
  188 groups, zero failures.
- FPAS formatting for `lib/`, `examples/`, `tests/`, `apps/` and changed editor
  fixtures: passed. Full FPAS suite: 466 passed, one intentional skip, zero failures.
- All 109 application/example program sources and 22 project manifests: passed
  with declared project context where required.
- All 56 complete handbook programs: passed, including their shown source units.
- TextMate grammar verifier, actual VS Code extension-host suite and 31 assembled
  debugger programs, generated API export, 771 current documentation/source links,
  EBNF production references and Git diff check: passed.

Positive, negative and boundary coverage spans parser, sema, compiled-unit
interfaces, compiler/verifier/VM, formatter, CLI/runner and editor integration.
Dedicated regressions cover generic arity/constraints, finite recursion, defaults
and visibility through aliases, nominal identity, unresolved inference, qualified
pattern owners, duplicate/shadowed/grouped bindings, guarded coverage, lazy branch
order, escaping closures, equality/key restrictions and old-form rejection.
The first generic-data checkbox is complete; the full functional-core stage is not.
