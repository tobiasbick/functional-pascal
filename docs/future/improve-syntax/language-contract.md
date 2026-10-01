# Stage 1: Language contract

This is the decided target model for the [redesign](README.md), not the current
language specification. Later stages own implementation and detailed acceptance
checks. Their rules refine this contract; they must not introduce competing forms.

## Values, resources, and bindings

- `const` binds a value once; `var` allows reassignment. Both permit computed
  initialization, evaluated once when their declaration is reached. Unit/program
  initializers run once in dependency/declaration order; cycles in initialization
  dependencies are rejected. There is no `mutable` keyword.
- Local bindings may omit a type when their initializer has one unambiguous type.
  Top-level bindings, routine parameters/results, and record fields have explicit
  types. Later uses or assignments never determine an earlier binding's type.
- A compile-time constant is a property of an expression, not another declaration
  syntax. Literal/operator expressions and references to other static constants
  qualify. Ordinary routine calls do not become compile-time evaluations, even if
  pure. Static contexts reject a computed value with the non-static part identified.
- Scalars, records, variants, arrays, and dictionaries have value semantics.
  Assigning or passing them must not create observable aliases to mutable value
  storage. Internal sharing and copy-on-write are implementation choices.
- Copying a resource handle copies its identity, not the resource. Resource
  identity propagates through containing records and collections. A const record
  cannot have its fields replaced, but a channel stored in it can still be used.
  I/O, channel operations, and resource lifetime remain explicit library actions.
- Ordinary closures copy immutable captures and share mutable capture cells.
  Copying a stateful closure shares that environment; callable values are not
  structurally comparable data. Pure closures have no mutable captures. A closure
  with mutable captures is task-bound, transitively through other captured values.
- Parameters are read-only unless declared `var`. Read-only value parameters are
  snapshots; resource parameters still designate the same resource. Capturing a
  `var` parameter or carrying it into a spawned task is forbidden.
- A loop variable is a fresh immutable binding on each iteration, including for
  closure capture. Its declaration belongs to the loop header; it is not a second
  general-purpose binding syntax.

## Functions and construction

- `function` returns a value; `procedure` performs an action without a value.
  Both support ordinary callable values and nested/anonymous declarations. No
  implicit return of a final expression, special result variable, or currying.
- Any function-valued expression can be called with parentheses. The target is
  evaluated once, followed by positional arguments once, from left to right.
  Parameter names do not participate in callable-type compatibility; parameter
  order, types, mutation modes, result type, and purity do.
- Unit routines replace user-defined instance/static methods and automatic
  receivers. A callable record field is just a stored value: invoking it never
  inserts an implicit receiver. There is no `self` convention with special lookup.
- Routine and variant-constructor calls are positional and have no defaults.
  Record construction is `TypeName(Field := Value, ...)` only; options records
  provide defaults. No contextual `record ... end` literals remain.
- A record's supplied fields evaluate in written order; omitted defaults then
  evaluate in declaration order, once per construction. Defaults must be pure
  and cannot refer to other fields or to mutable enclosing bindings.
- Non-public fields remain non-public. Outside the declaring unit, structural
  construction requires every field to be public, even omitted defaulted fields.
  Otherwise use an ordinary public factory. Field reads and updates obey visibility.
- Record update is `Value with Field := Expression; end with`. It returns a value,
  does not mutate the original, and checks duplicate/unknown/inaccessible fields.
  Evaluate the base once, then replacements once in written order.
- Declarations share a case-insensitive namespace within their scope. A
  type/routine collision is rejected, not resolved by preferring construction over invocation.
  Construction/conversion and invocation have disjoint resolved target kinds.

## Types and generics

- Always parenthesize `of` lists, including one item: `array of (integer)`,
  `Option of (User)`, `Result of (User, Error)`, `dict of (string, User)`,
  `channel of (Message)`, and `task of (integer)`. Bare `task` is not an inference
  shortcut; omit the entire local annotation when inference is wanted.
- Generic declarations also use `of (...)`: `type Box of (T) = record ...` and
  `function Identity of (T)(Value: T): T;`. Separate type parameters with commas;
  retain existing explicit constraints such as `T: Comparable`. No angle-bracket
  form and no special dictionary `to` form remain.
- Concrete type applications require their type arguments. Constructor calls use
  the declared type/variant name and infer arguments from supplied values and an
  expected type. An unresolved argument requires a type annotation. There is no
  alternate explicit type-argument syntax on constructor or routine calls.
- Generic routine calls infer type arguments from actual arguments. Taking a
  generic routine as a value requires a concrete expected callable type. Do not
  add first-class polymorphic values or infer routine parameters from a body.
- Type declarations in one unit/program are order-independent; constants and
  variables are not. Transparent alias cycles and record cycles with no finite
  representation are rejected. Recursive data may pass through a collection or
  an enum with a finite base case; it must have a finite representable value.
- Equality is structural for value data whose components support equality,
  including arrays (ordered elements) and dictionaries (same key/value mapping,
  independent of insertion order). It does not silently become handle identity
  equality for resources, tasks, or callables, including nested occurrences.
  Ordering stays with types that define ordering; records/collections receive no
  automatic lexicographic ordering. Generic constraints must distinguish equality
  requirements from ordering; audit and extend the existing small constraint set.

## Names and grammar

- One import declaration binds one explicit qualifier: `uses Std.Str as Text;`.
  Imported symbols are available only through it. Plain and wildcard imports,
  implicit short names, and a second full-name access path are not supported.
  Qualifier collisions with declarations are errors. Built-in types/keywords are
  language names; standard-library operations still use explicit imports.
- Variant constructors/patterns are qualified by their type, for example
  `Option.Some(Value)` and `Option.None`. Expected types resolve generic arguments,
  not unqualified constructor names. Unit qualification applies to imported types.
- Repeat `type`, `const`, or `var` for each declaration. Keep default non-public
  visibility and explicit `public`; no `private` keyword or inherited declaration
  group. Local type declarations are not added by this redesign.
- Every statement and declaration ends with `;`. Programs end with `end program;`,
  units with `end unit;`. Named constructs use named closers. Plain lexical blocks
  retain `begin ... end;`; repeat loops end at `until Condition;`.
- Expressions have no terminating semicolon. A named closer inside an expression
  belongs to that expression; surrounding statements supply their own terminator.
  Statement branches hold statement lists; expression branches hold one value.
- Keep semicolons between individually typed formal parameters and commas between
  actual arguments. These delimit declarations and expressions respectively;
  grouped formal declarations and comma-separated formals are invalid.
- `and`/`or` short-circuit left to right; `xor` evaluates both operands. All logical
  operators are boolean-only. Comparisons bind above `not`, which binds above
  binary logical operators. Mixing different binary logical operators requires
  parentheses. Comparison chains are rejected. Bit operations are library calls.
- All other operand/subexpression evaluation is left to right. Short-circuiting
  and selection of a single decision branch are explicit evaluation rules, not
  optimizer-dependent behavior.

## Decisions and errors

- `if`/`case` work as statements and as expressions in value positions, sharing
  conditions, pattern rules, and named closers. Expression branches each supply
  one compatible value. An expression `if` requires `else`.
- Enum cases list every top-level variant explicitly, without an enum catch-all.
  Nested patterns use `const Name` to bind, a literal/static constant to compare,
  and `_` to ignore payload data. A guard alone never establishes coverage.
  There is no binding `is` expression or alternative single-variant syntax.
- `Option` models absence, `Result` expected errors, and panic a non-regular
  failure. `try` forwards a compatible Result error. It does not convert panics
  or infer a new enclosing return type.
- Value-producing expressions cannot stand alone as statements. Use `discard`
  to intentionally ignore an ordinary value, including Option/Result. It is an
  error for a procedure call or any value containing a task handle.
- Bounds errors, integer overflow, integer division by zero, and invalid checked
  conversions panic at runtime; statically evaluated invalid operations are
  compile-time errors. Integer arithmetic is signed 64-bit and checked in every
  build mode. Real arithmetic uses IEEE 754 binary64, including infinities
  and NaNs; real division by zero follows that model. Optimizations must preserve
  these distinctions. Shift functions specify their own checked count domain.

## Purity and task responsibility

- Purity is explicit (`pure function`), never inferred from naming or a const
  binding. Ordinary functions may perform I/O and return Result. Pure functions
  may use local mutable data but cannot observe or change external mutable state,
  use resources/tasks, or call ordinary functions/procedures.
- Callable types can require `pure function`; an ordinary function can accept
  such a callback too. A pure callable can be used where an ordinary callable is
  expected, but not vice versa. This is one checked capability, not an effect system.
- Every spawn belongs to an explicit lexical `scope ... end scope;`. A routine
  with a spawn needs its own lexical scope; a caller's dynamic scope is not an
  implicit service. No program-root, background, or detached spawning exception.
- Both `go` forms register a child with the scope. A statement starts a procedure
  task without returning a handle; an expression starts a function task and
  returns a typed handle. A function result must be observed with Wait and then
  used or explicitly discarded. Use a procedure when no result is needed.
- Scope ownership, not handle reference count, determines lifetime. Task handles
  cannot escape their owning scope, including through containers, closures,
  resource storage, channels, returns, or outer mutable bindings. Synchronous
  non-escaping use by a helper is allowed and must be checked transitively.
- A normal scope exit waits. An exit crossing the scope boundary through return,
  break/continue, propagated error, or panic cancels outstanding children and
  waits. A child panic initiates the same cancellation. A returned Result.Error
  is an ordinary value and does not itself trigger scope failure.
- Cancellation is cooperative; scope exit promises ownership and joining, not
  a bounded shutdown time. No child may keep running after the owner exits.

## Necessary distinctions

| Distinction | Reason and limit |
|-------------|------------------|
| Record fields versus positional arguments | Data construction names stored fields; callable compatibility is positional. Neither has a second spelling. |
| Value data versus resource/callable identity | Copying data must isolate its mutable storage; copying a resource or stateful closure preserves identity. Containment propagates this difference. |
| Expressions versus statements | Values must be produced or consumed; actions need no dummy value. Both reuse the same branch and scope rules. |
| Static constants versus computed const bindings | Some type/label contexts need a compile-time value. Immutability alone does not provide it. |
| Closed variants versus open scalar domains | Variant extension must expose omitted cases; an unbounded scalar domain needs an explicit fallback where completeness is required. |
| Procedure tasks versus function tasks | An action has no result to observe; a function does. Ownership and cancellation are identical. |

Do not add per-library exceptions to these rules. Libraries express policies with
ordinary functions, records, and variants. Existing syntax not otherwise changed
by this plan stays in scope for normal maintenance, not an unbounded redesign.

## Work and acceptance

- [x] Audit value copying, collection mutation, closure captures, resource kinds,
  constant evaluation, numeric behavior, and task APIs against the target rules.
- [x] Map each change to owning crates, handbook pages, grammar productions,
  existing tests, and source consumers. Identify reusable behavior explicitly.
- [x] Write a bounded implementation sequence with actual paths and coordinated
  migration boundaries, particularly for stages 4 and 5.
- [x] Prepare representative positive, negative, and interaction test cases from
  [verification](verification.md); do not introduce a parallel acceptance index.

Acceptance: every target rule has an identified implementation owner and a
behavior-level verification route; migration dependencies are explicit. The
decisions above are settled. Backend representations and algorithms are chosen
during the audit without weakening observable contracts.

Evidence: [source audit and grammar/consumer mapping](audit/source-map.md),
[existing test inventory and target cases](audit/test-inventory.md), and
[bounded implementation sequence](audit/implementation-sequence.md).

Status: stage-1 design/audit deliverable complete. Existing tests were inspected;
target cases are specified, not implemented or executed. Current language behavior
and normative language documentation were unchanged by that audit. Stage 2 now
has its shared model and build/project source-error transport; stages 3-6 remain
unimplemented. Next: finish remaining error conversion and stage 2 CLI/runner
stream integration before syntax migration.
