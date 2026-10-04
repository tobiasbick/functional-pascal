# Stage 4: Functional core

Prerequisites: [contract](../language-contract.md), diagnostics, and syntax/name
foundations. Coordinate bindings/value semantics and defaults with mutation/purity in
[stage 5](05-effects-and-tasks.md). Follow [shared verification](../verification.md).

## Bindings and value behavior

Implement computed `const`, mutable `var`, local initializer inference, static
constant classification, and value semantics together. Migrate immutable `var`
to `const` and `mutable var` to `var`. Existing locally reassigned parameters
become read-only parameters with explicit local copies, never caller references.

Inference uses only the initializer and an optional explicit expected type.
Ordinary calls, decision expressions, and typed constructors are all eligible.
No body-based inference changes routine signatures. Empty collections, payloadless
generic constructors, and incompatible branch types must request annotations.
An explicit annotation constrains the initializer; it does not force a guess.

Copies isolate nested mutable value storage. Resource handles and stateful
closures contained within those values retain their declared identity semantics.
Test array/dictionary updates, parameter snapshots, record updates, closures, and
copy-on-write sharing together, not just scalar reassignment. Implement the checked
integer rules from stage 1 across constant evaluation, compiler lowering, and VM.

## Callable values

Calls accept any correctly typed callable expression, including returned functions,
selected values, indexed functions, and record fields. No receiver is inserted.
Anonymous routines retain `function`/`procedure` and explicit signatures; named
nested routines use the same capture semantics. A procedure call produces no value
and cannot initialize a binding. A procedure value may be stored in a binding.

Target example (not a compiled fixture):

```pascal
function MakeAdder(Base: integer): function(Value: integer): integer;
begin
  return function(Value: integer): integer
  begin
    return Base + Value;
  end function;
end function;

procedure Demonstrate();
begin
  const Answer := MakeAdder(3)(5);
end procedure;
```

All function results in statement position require consumption or `discard`.
Apply this to ordinary calls and the final call in a postfix chain. `discard`
evaluates its operand exactly once and cannot hide a task handle in a container.

## Generic data and construction

Support generic records and payload enums, recursive data, explicit field types,
and existing small generic constraints. All `of` lists have parentheses, including
built-in arrays, dictionaries, channels, tasks, Option, and Result.

```pascal
type Lookup of (T) = enum
  Found(Value: T);
  Missing;
  Failed(Message: string);
end enum;

type Pair of (K, V) = record
  Key: K;
  Value: V;
end record;

function Identity of (T)(Value: T): T;
begin
  return Value;
end function;
```

An explicitly typed local may use `Lookup.Found(42)`; its variant name identifies
the declaring type and the expected type constrains generic arguments. A
payloadless variant such as `Lookup.Missing` is a value, not a zero-argument
routine. Without enough type context it is rejected. Patterns use the same
qualified variant names. Do not infer variants by searching visible declarations.

Record construction uses `Pair(Key := 1, Value := 'one')`; the fields determine
its type arguments. An explicit binding annotation may supply missing context.
All required fields must be supplied; defaults and visibility obey stage 1.
Named arguments on routines or variant constructors remain errors. Diagnose
obsolete record literals with the resolved target type, not a guessed name.

Structural equality follows the component rule in stage 1; dictionary order does
not affect equality. Add an `Equatable` constraint for equality-only generic code;
`Comparable` implies equality plus ordering. Reuse `Numeric` and `Printable`
where their audited definitions fit; there are no user-defined typeclasses or
operator implementations. Dictionary keys must satisfy the compiler's supported
key representation and equality contract; report unsupported keys explicitly.

## Patterns and value-producing decisions

Use nested patterns with explicit `const` bindings and `_` payload wildcards.
Literal or static-constant patterns compare; a plain identifier never introduces
a binding. Bindings are local to an arm and available to its guard and body.
Reject duplicate bindings and shadowing within the same arm scope. Ordinary
lexical shadowing of an outer local is allowed; imported qualifiers cannot be
shadowed. Guards evaluate only after their pattern matches.

Closed enums require explicit coverage of every variant; no top-level `_` or
`else`. Multiple patterns for one variant may cover nested alternatives, but
guards do not establish coverage. Payload `_` is permitted to cover the remaining
payload values of an explicitly named variant. Reject unreachable/duplicate arms.
When grouping labels in one arm, each label must introduce the same binding names
and types. A no-action arm contains `null;`.

An open scalar statement case may omit `else` and do nothing on no match. An open
scalar case expression requires `else` unless finite coverage is proven. Both
forms evaluate the scrutinee once and choose the first matching guarded arm.
Branch typing uses the expected type when supplied; otherwise all branches must
agree on one type, allowing only the ordinary implicit conversions. There is no
automatic union type or branch-order-dependent choice.

```pascal
function Describe(Response: Lookup of (string)): string;
begin
  return case Response of
    when Lookup.Found(const Text): 'Found: ' + Text;
    when Lookup.Missing: 'Missing';
    when Lookup.Failed(const Message): 'Failed: ' + Message;
  end case;
end function;
```

In statement position, `if`/`case` parse as statements; in a required value
position they parse as expressions. Each expression branch holds exactly one
expression. There is no implicit last-statement value and no binding `is` form.

## Remove redundant member mechanisms

Move actual methods and static record routines to their declaring units as
ordinary routines. Make receiver data an explicit positional parameter, or a
`var` parameter if it truly mutates the caller. Resolve collisions by explicit
routine renaming and update callers using resolved symbols.

Remove automatic receivers, computed properties, events, their event-only `nil`,
and `Assigned`. Replace getters/setters with ordinary calls and event slots with
`Option of (HandlerType)` fields. Match that Option explicitly and call the stored
handler normally. Preserve non-public fields and public factories; do not add
visibility merely to make migration easy. Multiple subscribers need an ordinary
collection API only when an actual application requires it.

## Work and acceptance

- [x] Audit reuse in parser/AST, sema, compiler/IR, VM, generic routines, closures,
  record construction/update, equality, and formatter before extending modules.
- [x] Implement and test arbitrary callable targets and capture rules first.
- [x] Implement generic records/enums and construction, then nested patterns and
  exhaustive case expressions; implement if expressions with the same type rules.
- [x] Implement bindings, inference, value copying, checked numeric behavior, and
  coordinated explicit caller mutation and default-purity checks; migrate all
  affected standard APIs.
- [x] Replace member mechanisms only after their ordinary-function replacements
  work. Remove dead resolution, AST, runtime, formatter, and editor paths.
- [x] Update grammar and current function/type/pattern/binding/error pages;
  remove obsolete method/property/event/fluent-call pages and repair their links.
- [x] Verify generic nesting/recursion, private construction, defaults, inference
  ambiguity, callable fields, mutation modes, structural equality, coverage,
  evaluation order, integer boundaries, and migrated application behavior.

Acceptance: real programs compose values and functions without hidden receiver
rules; local inference is uniform; generic data and nested decisions work together.
No obsolete member syntax remains, and migrated bindings preserve intended
mutability. Stage 4 completes only with coordinated mutation and purity support.

Owners: `fpas-parser`, `fpas-sema`, `fpas-ir`, `fpas-compiler`, `fpas-bytecode`,
`fpas-vm`, `fpas-fmt`, std registries/source units, and language-service adapters.

Evidence: [functional-core reuse audit](../audit/functional-core-reuse.md) maps
pre-implementation owners and gaps. The
[callable delivery](../audit/functional-core-callables.md) records implemented
scope, module splits, capture correction, consumer migration and coverage.
The [generic forwarding correction](../audit/generic-constraints.md) records
constraint regressions, record-default corrections and their verification.
The [generic-data delivery](../audit/generic-data.md) records equality rules,
coverage and the authorized record-default alias correction.

Status: arbitrary callable targets and the generic-data/decision slice are complete.
Generic records/enums retain concrete arguments, constraints, recursion, defaults
and nominal identities across compiled units. Named record construction and
qualified variants share contextual inference with lazy `if`/`case` expressions.
Both case forms use recursive patterns, explicit bindings and common coverage.
Structural equality, dictionary key checks and duplicate normalization are verified.

Canonical conversion covers standard units, applications, examples, FPAS tests,
Rust test sources, handbook/API examples and editor fixtures. Obsolete builtin
lists, angle headings, short variants, implicit pattern bindings and executable
anonymous record paths are removed. Obsolete record diagnostics report the resolved
target through aliases; formatters reject those expressions. Current grammar,
handbook, authoring guidance and generated intrinsic declarations are synchronized.

Verification: `cargo fmt`, `cargo build` and the full Rust workspace pass
(3,649 tests across 188 groups). FPAS formatting and the full suite pass
(466 passed, one intentional skip, zero failures). All 109 example/application
programs, 22 project manifests and 56 complete handbook programs pass in their
appropriate source/project contexts. Editor grammar, the real VS Code extension
suite, documentation links and the Git diff check pass. The delivery audit records
positive, negative and boundary coverage and the authorized baseline corrections.

The [task migration boundary](../audit/task-migration-boundary.md) assigns removal
of bare `task` to coordinated local inference and procedure-task work. There is no
public unit-result type and no permanent bare-task exception in the target contract.
This approved dependency does not leave the generic-data slice open.

In progress: the [bindings/effects audit](../audit/bindings-effects.md) records
the implemented storage-root reservation, exclusivity and reborrow foundation,
checked selected field/index paths, IR/bytecode and invocation integration, VM
cell access checks, mode-preserving artifact/linking support, coverage and
remaining source/signature work. This
foundation does not close the binding/mutation/purity acceptance item.

The audit's baseline string-index assignment error is corrected with the user's
authorization: semantic checking reports read-only string indices as F2005 before
compiler lowering, with direct, nested and imported-path regressions. This
correction does not close a stage-4 implementation item.

The next source-mode audit found a separate baseline error: immutable imported
storage was rejected only at runtime. The user-authorized correction shares
qualified storage-root resolution between typing and assignment permissions.
Scalar, nested field/element, interface round-trip and real CLI regressions verify
compile-time F2005, while mutable roots and independent snapshots remain valid.
The bindings/effects audit records the reproduction and correction. The affected
standard-unit, example, application and FPAS-test consumers now use explicit local
copies in place of locally mutable formal parameters. Compiler/CLI closure fixtures
and debugger storage-mutation programs use the same form. Legacy syntax-validation
fixtures remain until the corresponding parser/signature/member paths are replaced.
The following source-mode integration builds on that consumer migration.

Source `var` modes now pass through parser/AST, semantic signatures, callable
values, formatter/editor displays, compiler references and compiled units.
Selected caller storage preserves writable roots, frozen paths, written argument
order, snapshot reads, exclusivity and forwarding. Source regressions also cover
forbidden escapes and debugger mutation/inspection. `Std.Arrays.Push/Pop` require
explicit var arguments, support stored fields/elements and use ordinary var-call
activation and cleanup. Their consumers, handbook and generated declarations are
migrated. The bindings/effects audit records delivery and verification; bindings,
inference, checked numbers and purity still keep the acceptance item open.

The [binding initializer delivery](../audit/binding-initializers.md) implements
computed const, writable var, initializer-only local inference, lexical static
classification and consumer migration. Nested value-copy, resource/closure
identity, imported storage and inferred editor details have dedicated regressions.
Its full verification passes, including 3,778 Rust tests, the complete FPAS bundle,
consumers, handbook and real editor host. It does not close the coordinated
acceptance item. Checked numbers and stage-5 purity/default checks are next, followed by
member removal with working ordinary-function replacements. The full stage
remains open.

The [checked-number delivery](../audit/checked-numbers.md) implements checked
integer operations, static F2020 diagnostics and IEEE real division/comparison.
Focused tests and full verification pass. Its artifact audit found a
pre-existing compiler-fingerprint omission for `fpas-ir`. A manual static-aggregate
comparison also exposed a missing F2020 in the new checker. Both corrections are
authorized and implemented, including static aggregate values across compiled-unit
reuse and regressions for evaluated/skipped operands. Verification passes with
3,807 Rust tests, the full FPAS bundle, source/project consumers, handbook examples,
release numeric checks and the real editor host. A further aggregate guard using
`Std.Math.Pi` exposed missing standard constant values in semantic evaluation.
The authorized correction shares their values through standard-library metadata
and verifies static guards, runtime agreement and compiled-unit defaults/reuse.
Its CLI regression exposed a separate imported-global/alias name collision in
compiler storage resolution. The standing permission to fix pre-existing bugs
covers its correction: imported globals now use canonical unit identities only.
CLI regressions verify alias/member collisions, field/index paths, snapshots,
var calls, standard aliases and reused artifacts. Coordinated purity/default
checking follows; stage acceptance remains open. The audit records the
reproductions and corrections.

The preparatory purity audit found a pre-existing collision between independently
declared generic parameters with the same name. The
[generic parameter identity correction](../audit/generic-parameter-identity.md)
retains declaration identities through inference, substitution and compiled-unit
interfaces. Its positive closure regression also exposed a baseline lexical
sibling-call failure; anonymous runtime names retain their owner paths and
referenced sibling environments. Contextual generic callable instantiation,
concrete container typing and erased runtime result restoration have regressions.
Full verification passes with 3,835 Rust tests, 470 FPAS tests and one intentional
skip, source/project consumers, handbook programs, formatting and documentation
links. These corrections do not close the coordinated acceptance item.
The user approved option 1 in the
[default-purity boundary](../audit/default-purity-boundary.md): default evaluation
must be pure, but the field result type need not satisfy pure-function result
capabilities. The coordinated implementation is complete.
The [purity/default integration audit](../audit/purity-and-defaults.md) records
the shared capability checker, declaration-scoped default initializers, compiled
interfaces, selected generic-default obligations and consumer migration. Full
verification passes with 3,869 Rust tests, 470 FPAS tests and one intentional skip,
source/project consumers, handbook programs, formatting and the real editor host.
This closes the coordinated binding/mutation/default-purity item. The
[member-removal delivery](../audit/member-functions.md) replaces methods,
properties, events and fluent calls with ordinary functions and stored callables,
removes their AST/resolution/runtime/editor paths, and updates current grammar
and documentation. Final verification passes: 3,787 Rust tests, 470 FPAS tests
and one intentional skip, all 22 projects, 109 program entries and 54 complete
handbook programs in their proper contexts, plus the real VS Code host.
Stage 4 is complete; the approved bare-task migration remains with stage 5.
