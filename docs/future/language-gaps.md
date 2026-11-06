# Language gaps and deferred extensions

Record confirmed omissions from the [baseline](../baseline-comparison.md) with
their owning rule and reproducer. The proposals below are deferred extensions.
Maintenance limits belong in [dogfooding gaps](dogfooding-gaps.md); missing guides
belong in [documentation gaps](documentation-gaps.md).

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
coverage remains a separate toolchain proposal. Additional language-server services and IDE project-system integration
belong in [editor gaps](editor-gap.md), scheduled last.
