# Language gaps and deferred extensions

Record confirmed omissions from the [baseline](../baseline-comparison.md) with
their owning rule and reproducer. The proposals below are deferred extensions.
Maintenance limits belong in [dogfooding gaps](dogfooding-gaps.md); current
toolchain procedures belong in the [development workflow](../build/development.md).

## Deferred proposal: selected syntax from Rust FPAS

This plan proposes six changes for the CLR compiler. Implementation requires the
user's explicit syntax approval. Local `const`, `var`, and resource declarations,
`const` pattern bindings, and `ensures` result bindings all require explicit types.

### Array type spelling

Use `array of T` for a CLR vector and `array[,] of T` for a rectangular rank-two
array. For higher ranks, use one comma per additional dimension, up to rank 32.
Use `array of array of T` for a jagged array. `array of string?` allows null
elements. `(array of string)?` allows a null array. The forms compose, for
example `(array of string?)?` and `array of (array of string)?`.

Apply the spelling wherever a type is accepted, including aliases, generic
arguments, casts, and routine signatures. Reserve `array` as a keyword and use
`@array` for an identifier with that name. Use the existing array type nodes and
CLR signatures. Continue to allocate with `new int[Count]` or
`new int[Rows, Columns]`. Jagged allocation can use an element type alias.
Literals and indexing keep their existing bracket syntax. Replace postfix array
type annotations in FPAS sources and fixtures in the same change.

Verify rectangular and jagged shapes, nested generic type arguments, both levels
of nullability, and matching array metadata in a C# consumer. Format the new
type forms and diagnose retired spellings in type positions.
[Arrays](../pascal/language/types/arrays.md) · [Syntax](../specs/syntax.md)

### Record member visibility

An unmarked record field, instance method, or static method is private to the
declaring record. `public` opts a member into external access, subject to the
record's own accessibility. Apply the rule in programs and units. Require
explicit `public` on members used to implement interface signatures.

Keep the CLR mapping for public and private members. A private field makes the
all-fields constructor private under the existing constructor rule. During
migration, add `public` to the fields and methods needed by callers, interface
implementations, and existing exported signatures. Verify access inside the
record, rejection from other code in the same unit, and the emitted visibility
of constructors, properties, and methods in a separate consumer.
[Records](../pascal/language/types/records.md) · [CLR mapping](../specs/clr-mapping.md)

### Local routine placement

Place named helpers belonging to a routine after its `where`, `requires`, and
`ensures` clauses and before its outer `begin`. Apply the same placement to
block-bodied anonymous routines. Named helpers in inner blocks, branches, and
loop bodies stay in their declaring scopes. The program entry body keeps its
block form for local helpers.

Bind helpers before `begin` in the routine body's outer lexical scope. They can
capture its directly declared locals. Calls and delegate creation still require
initialized captures. Register all helper signatures before binding bodies so
forward calls and mutual recursion work. Reuse the existing nested-routine binder
and closure representation. Preserve the source location of each declaration for
formatting, editor ranges, and debugging.

Move existing routine-scope declarations to the new location. Verify mutual
recursion, an escaping helper with a mutable capture, rejection of calls before
capture initialization, and scope isolation for an inner block helper.
[Nested routines](../pascal/language/functions/nested-routines.md)

### Mixed logical operators

Allow a chain of one of `and`, `or`, or `xor` at each logical expression level.
Reject `A and B or C` at `or`. Accept `(A and B) or C` and `A and (B or C)`.
Parentheses around an operand alone, as in `A and (B) or C`, leave both operators
at the same expression level. Apply the rule in the parser so Boolean,
integral, enum, and constrained operands follow the same syntax. Keep the
evaluation rules for each operator and the lower precedence of `??`.

Parenthesize existing mixed expressions according to their current meaning.
Verify diagnostics, both accepted groupings, and short-circuit behavior with
side-effecting operands. [Operators](../pascal/language/basics/operators.md)

### Real division of integral operands

For built-in integral operands, convert both to `double` before `/`. This also
applies to native integers and to subranges after projection to their base type.
It admits mixed signed and unsigned integral operands. `1 / 2` yields `0.5`,
`1 / 0` yields positive infinity, and `0 / 0` yields NaN under the current
floating-point rules. Use `div` for the checked integral quotient. Existing
`float`, `double`, and `decimal` promotion applies when an operand already has
one of those types. Generic division through a CLR constraint still uses the
operator supplied by that constraint.

Lift the integral `/` rule to `double?` if either operand is nullable. Evaluate
both operands once from left to right and divide only when both are present.
Use the same conversion and division rules in constant evaluation and CIL
emission. Verify the result type, large mixed integral operands, zero divisors,
lifted absence and evaluation order, and the existing `div` and generic behavior.
[Operators](../pascal/language/basics/operators.md) · [Nullable types](../pascal/language/types/nullable-types.md)

### Multiline strings

Allow ordinary apostrophe-delimited strings to span source lines. Decode each
LF, CRLF, or bare CR inside a literal to a single LF. Keep the spaces and tabs
inside the quotes, including indentation and whitespace before the closing
quote. Doubled apostrophes still escape a quote. Line breaks immediately after
the opening quote or before the closing quote are part of the string.

Track line breaks inside the token for diagnostics and source positions. The
formatter can normalize source line endings to LF while preserving all other
literal content, even when it changes the indentation around the literal.
Verify the same string value for all supported source line endings, apostrophe
escaping, unterminated literal diagnostics, and a format/compile round trip.
[Lexical rules](../pascal/language/basics/lexical-rules.md) · [Formatter](../pascal/tools/fmt-style.md)

### Implementation order

1. Implement the mixed-operator rule and real division as separate changes.
   Migrate mixed expressions with the parser change to preserve their grouping.
2. Implement array type spelling and local routine placement separately. Update
   formatter and editor source ranges with each parser change. Migrate examples,
   fixtures, and FPAS sources alongside their compiler changes.
3. Change the record visibility default with the explicit `public` migration
   and C# metadata checks. Implement multiline strings with lexer and formatter
   checks in a separate change.
4. For each change, update the owning grammar and handbook rules and the agent
   reference in the same change. Run the affected compiler and execution tests,
   documentation examples, agent reference checks, and formatter checks through
   the [development workflow](../build/development.md). Remove each proposal
   entry once its language contract and verification are in place.

## Deferred proposal: compiler-checked pure functions

Add optional compiler-checked purity. Ordinary functions keep their current
[effect rules](../pascal/language/functions/scope-and-effects.md).
This is a deferred extension, not a missing baseline feature. `pure` names the
proposed property; declaration syntax is not yet specified or implemented.

- Results depend only on inputs and immutable receiver/captured data. Reject I/O,
  clock/random reads, mutable global or captured state, and mutations of
  caller-owned data, including writes through ref/out parameters.
- Permit local variables and loops when their effects remain local to the call.
- Check every invoked operation, including methods, property getters, operators,
  constructors, and callbacks. Calls must preserve the purity guarantee. Foreign
  .NET calls need a defined, reviewed trust boundary; a name or attribute alone
  does not prove purity. Delegate and virtual dispatch must preserve the same
  guarantee before those calls can be admitted.
- Deeply immutable inputs provide stable data, but immutable data alone does not
  make a function pure. Contracts remain executable checks under their existing
  rules and do not declare a function pure.

The owning specification must define permitted inputs and operations, exception
and allocation behavior, and how purity survives public CLR calls and imports.
Any new syntax requires explicit user approval before implementation.

## Deferred proposal: deep immutability

Keep the existing [binding rules](../pascal/language/basics/variables.md): `const`
prevents rebinding, and `var` permits it. Introduce a separate, compiler-checked
property for deeply immutable data. `immutable` names this proposed property;
its declaration/type syntax is not yet specified or implemented. This is a
deferred extension, not a missing baseline feature.

- Immutability belongs to the type/value, not just a local binding. All contained
  fields, collection elements, and nested data must also be immutable. Methods
  and property getters must preserve that state and expose no writable aliases
  into it.
- Preserve the guarantee through assignments, aliases, conversions, parameters,
  return values, and generic containers. Copying a value into another binding
  must not grant write access. A `var` may hold an immutable value and later be
  rebound to a different immutable value; the previous value remains unchanged.
- Use actual .NET immutable collection types such as
  `System.Collections.Immutable.ImmutableArray<T>`, `ImmutableList<T>`, and
  `ImmutableDictionary<TKey, TValue>`. Check their element/key/value types too:
  an immutable collection can still contain mutable objects. Read-only fields,
  `in` parameters, `IReadOnlyList<T>`, and read-only wrappers do not establish
  deep immutability.
- Admit imported .NET types only when their immutability is proved or covered by
  an explicit, reviewed trust boundary. Reject unknown cases. Declaring a
  binding immutable must not silently claim that an arbitrary mutable .NET
  object has been frozen. Any conversion from mutable data needs a defined
  immutable representation and must account for existing aliases.

The owning specification must define type checking, generic requirements,
construction, and conversions. Public CLR metadata, import validation, and calls
into foreign code must preserve the guarantee under the
[interop rules](../pascal/interop/README.md) and
[CLR mapping](../specs/clr-mapping.md). Any new syntax requires explicit user
approval before implementation.

## Scope boundary

Source event declarations, expression-tree conversion, user-authored class
hierarchies, custom awaiters, async iterators, unsafe/byref-like features,
operator declarations, automatic extension-method discovery, and interface
variance/default bodies remain outside the agreed scope. Additional platform/framework
coverage remains a separate toolchain proposal.
