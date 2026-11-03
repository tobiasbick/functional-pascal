# Named nested routines

Named functions/procedures now use ordinary routine declarations inside a
routine body or lexical block. The
[local-functions example](../../examples/local-functions/README.md) demonstrates
a forward recursive call and an escaping counter delegate.

```pascal
function MakeCounter(Start: int): function(): int;
begin
  var Count: int := Start;
  function Next(): int;
  begin Count := Count + 1; return Count; end function;
  return Next;
end function;
```

## Scope and calls

Signatures are visible throughout their declaring block, including before the
written declaration. Self-recursion and mutually recursive siblings work.
Declarations do not execute statements or introduce debugger stops, and may
appear after a return. Bodies are checked even when no caller uses them.

Nested routines remain private to their lexical block. Parameters/local values
and nested groups follow lexical shadowing; declarations cannot duplicate a
binding in the same scope or shadow imports, the program entry's args, or a
record's self. Nested parameters also cannot replace the record receiver.
Public/private/static local modifiers are not included.

Calls use the existing overload, generic inference/explicit arguments,
constraints, named/optional/params arguments, and ref/out/in rules. Nested generic
methods keep their own CLR method parameters separate from enclosing generic
type/method parameters. Contracts execute in the callee through ordinary calls
or delegates, including C# callers. Async functions return Task/Task<T>, retain
their environment across suspension, and preserve normal async contract rules.

## Captures and initialization

Immutable outer bindings are captured by value; mutable local vars share
StrongBox<T> cells with callers, sibling routines, and anonymous functions.
Captured objects are not copied. Global reads still use actual static fields.
Delegates can outlive the enclosing call. Capturing ref/out/in parameters remains
invalid, while a nested routine can declare and receive its own managed parameters.

Captures must be initialized before a call or delegate conversion creates its
environment. This compiler shares one capture layout between routines declared
in the same block. Initialize that block's complete captured set before calling
or converting any member, even if one member reads only part of the set.
FP3033 diagnoses premature creation. Runtime declaration order is not used to
silently substitute default values for an uninitialized capture.

Bodies use conservative nullable information for outer bindings; checks performed
only by a caller do not establish a non-null invariant inside a nested routine.
Validate/coalesce nullable captures inside the routine, including with requires
where allowed. Calls invalidate facts about shared mutable captured variables.

## CLR representation and tooling

Each declaring block with nested routines has a private generated closure type.
Its instance methods retain source routine names and ordinary CLR parameter,
generic, nullable, and async metadata. Enclosing generic parameters become type
parameters on the closure; a nested routine's own parameters remain method
parameters. A capture constructor receives values/cells at calls or delegate
conversion sites. Recursive/sibling calls reuse the current receiver.
This adds no public generated API or new Fpas.Core runtime feature.

Portable PDBs retain each method's source points and locals; generated environment
construction remains compiler plumbing. Nested Markdown documentation is retained
and validated, without public XML entries. PE/PDB generation remains deterministic.

## Verification and limits

Tests execute forward/self/mutual calls, escaping and transitive captures,
per-iteration cells, shadowing, managed/optional/params/named calls, overloads,
generic constraints, contract-only captures, and generic async methods. Mixed
C#/FPAS builds verify returned delegates, byref delegate invocation, callee
contracts, type/method generic slots inside generic records, actual suspension,
and calls from another FPAS assembly. Metadata tests inspect generic ownership,
PDB locations, private visibility, documentation, and deterministic output.

[Typed/repeat loops](loops.md) are implemented.
Authored interfaces/events, expression trees, arbitrary attributes,
fully general nested generic type paths, lifted numeric nullable arithmetic,
and LSP/query tooling remain outside this milestone.
