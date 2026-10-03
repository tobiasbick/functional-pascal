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
- [ ] Implement generic records/enums and construction, then nested patterns and
  exhaustive case expressions; implement if expressions with the same type rules.
- [ ] Implement bindings, inference, value copying, checked numeric behavior, and
  coordinated explicit caller mutation and default-purity checks; migrate all
  affected standard APIs.
- [ ] Replace member mechanisms only after their ordinary-function replacements
  work. Remove dead resolution, AST, runtime, formatter, and editor paths.
- [ ] Update grammar and current function/type/pattern/binding/error pages;
  remove obsolete method/property/event/fluent-call pages and repair their links.
- [ ] Verify generic nesting/recursion, private construction, defaults, inference
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

Status: arbitrary callable targets, capture rules and result consumption/discard
are complete, including the mutable-parameter capture correction, source consumer
migration, current documentation and positive/negative/edge coverage. Workspace
format/build/tests, FPAS formatting/suite, application/example checks, editor
grammar and documentation/diff checks pass. Stage 4 remains open.

Next: implement generic records/enums and construction, followed by shared
decision-expression typing. Bindings, caller mutation, purity and default checks
remain coordinated with stage 5. Member removal follows working ordinary-function
replacements.
