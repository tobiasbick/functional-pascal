# Closure environments

First-class functions are ordinary CLR delegates. Captured environments retain
their actual types and use CLR object lifetime; no FPAS ownership scheduler or
reflection invocation is involved. The language rules are in
[first-class functions](../pascal/language/functions/first-class.md) and
[nested routines](../pascal/language/functions/nested-routines.md).

## Values and cells

A lambda emits a private nested closure class with a constructor, capture fields,
and an instance invocation method. Immutable bindings store their current value.
Mutable captured locals share System.Runtime.CompilerServices.StrongBox<T> cells
with the declaring routine and sibling/nested closures. Struct mutation addresses
the cell's actual value, rather than a boxed replacement.

Capture fields use source-derived names so ordinary CLR debuggers can expand
them. Duplicate names within one environment receive stable numeric suffixes
such as Value_1. Field definitions and references use the same bound capture
name. The storage remains the actual captured value or shared cell; no detached
debug-only copy is created. Stock NetCoreDbg exposes these through the closure
receiver, with mutable values under the cell's Value field, rather than
automatically flattening all captures into source locals.

Separate routine calls own separate cells. Cells declared inside a loop are
allocated per iteration; read-only loop counters are copied into each environment.
Captured references share their object. Globals remain static field reads and
are not copied into captures. Capturing a disposable object does not extend its
usable lifetime after its using scope disposes it.

Delegate construction has no identity cache. A virtual method group preserves
virtual dispatch and evaluates its receiver once. A struct method group captures
a boxed snapshot; a lambda capturing a mutable struct variable uses shared storage.
Delegate Invoke calls propagate normal CLR exceptions and use the callee's own
loop, return, catch, and finally contexts.

## Lexical routine groups

A block containing named nested routines has one private generated closure type.
Its instance methods keep source routine names, overloads, generic parameters,
nullable annotations, and managed/optional/params parameter metadata. Local
declarations have no runtime statement, public API, or public XML entry.

The block shares one capture layout. Every captured binding in that layout must
be initialized before a call or delegate conversion constructs its environment;
FP3033 diagnoses premature construction. Recursive and sibling calls reuse the
current receiver. Uninitialized captures never silently become default values.

Captured nullable facts are conservative because callbacks can change shared
cells. Binding traversals include calls, properties/indexers, event operands,
guards, cleanup, and loop/ref/out writes that invalidate those facts.

## Generic environments

Enclosing type and method parameters become parameters of the closure type.
Its fields, constructor, and invocation method use translated !n slots; a creating
generic method supplies its !!n arguments when constructing that type. Constraints
and nullable annotations are re-emitted with the same translation.

A named nested method's own type parameters remain method !!n slots, independent
of the closure's parameters. Combined environments preserve both scopes without
object erasure. A nested closure's metadata name counts only newly introduced
arity beyond its containing type. GenericParam rows follow TypeOrMethodDef owner
order rather than incidental emission order.

An async closure or lexical routine carries these parameters and values into its
state machine. [Portable symbols](debug-information.md) identify source methods,
capture cells, locals, and async yield/resume locations.

## See also

- [Architecture overview](README.md)
- [Generic CLR mapping](../specs/clr-mapping.md#generic-routines-and-closures)
- [Async and cleanup](async-and-cleanup.md)
- [Capture and generic tests](../../tests/coverage.md#language-and-metadata)
