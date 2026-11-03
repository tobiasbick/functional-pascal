# Global values and CLR type initialization

Programs and units now support the planned top-level const/var declarations,
with mandatory types and initializers. The
[shared-state example](../../examples/shared-state/README.md) builds through the
SDK and shares a counter between unit routines, a delegate, and a ref call.

```pascal
unit Game.State;
public const Limit: int := 100;
public var Moves: long := 0L;
public procedure Advance();
begin Moves := Moves + 1L; end procedure;
end unit;
```

## Fields, scope, and access

Each binding emits an actual static field on its unit/program container with
the exact source name and type. Const fields are readonly (InitOnly), not Literal
fields; their initializers can execute ordinary expressions. Var fields are
writable. Visibility defaults to private; public globals belong in units.
Public field types must be publicly accessible, recursively through aliases,
arrays, and generic arguments. Global names cannot duplicate another global,
routine, type, or import in the same unit, or the program's implicit args.

Routines read their own globals without qualification; another source unit uses
its explicit unit alias. Private globals are accessible only inside their own
unit, including its records and lambdas. Local bindings/parameters and record
methods take precedence over outer globals. Lambdas read global fields directly;
they do not capture a copy of their initial values. Async routines also read the
same storage after suspension.

Source and imported public vars support assignment and managed ref/out/in
arguments under normal storage/type rules. Const fields cannot be assigned or
passed by ref/out; in can read them. Const reference bindings retain the ordinary
shallow rule: referenced objects and array elements may still be mutable.
Struct receiver mutation retains normal writable-storage/defensive-copy rules.

Nullable field annotations are emitted and imported, including nested generic
and array annotations. Global field reads do not acquire stable local null-flow
facts; copy a nullable field to a local or coalesce it before dereferencing.

## Initialization

A container with globals has one private static .cctor with SpecialName and
RTSpecialName flags, and no BeforeFieldInit flag. Initializers run once in their
written order within that source, including declarations interleaved with types
or routines. A static method call or field access triggers ordinary CLR type
initialization. Merely importing a unit, inspecting its metadata, or obtaining
its type does not initialize it.

There is no eager application-wide ordering or generated dependency scheduler.
Forward and cyclic reads use CLR default storage while initialization is still
in progress; reference storage can therefore be null before its initializer has
run. A failed initializer follows CLR TypeInitializationException behavior,
retains its original inner exception, and is not retried. Concurrent first
accesses use the CLR's type-initialization synchronization. Later shared field
mutation needs ordinary .NET synchronization.

Program globals initialize before the entry body and do not have access to args.
Initializers are synchronous and cannot await. They can construct ordinary
Task/delegate values without a language task owner or automatic resource cleanup.
No source unit initialization/finalization blocks are introduced.

## Constants, contracts, and diagnostics

A source global const is a runtime readonly field even when initialized from a
literal. It is not a CLR metadata compile-time constant: it cannot be an optional
parameter default, enum/subrange bound, or constant pattern. Contract conditions
likewise do not read global fields, including readonly fields. Pass the relevant
value as an initialized parameter/capture instead. Literals and genuine imported
CLR metadata constants remain available in these contexts.

FP3032 reports global-name collisions and public fields exposing private types.
Existing diagnostics cover missing annotations, type errors, illegal assignment,
visibility, nullable flow, managed arguments, contracts, and invalid await.

## Verification

Tests execute initialization order, shared reads/writes, lexical/record scope,
closures, async delegates, shallow const values, and managed arguments. Mixed
C#/FPAS projects verify readonly/public/private fields, nested nullable metadata,
separate FPAS consumers, multiple unit owners, and globals alongside records,
unions, enums, domains, closures, and async state machines. Runtime tests cover
parallel first access, initializer failures, cached failures, forward reads, and
cross-unit initialization cycles. Compiler tests inspect PDB initializer sequence
points, XML field documentation, and deterministic PE/PDB output.

[Named nested routines](nested-routines.md) are implemented.
[Typed/repeat loops](loops.md) are implemented.
Authored interfaces, arbitrary attributes,
fully general nested generic type paths, lifted numeric nullable arithmetic,
and LSP/query tooling remain outside this milestone.
