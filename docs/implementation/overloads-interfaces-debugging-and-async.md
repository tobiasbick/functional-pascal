# Source overloads, CLR interfaces, debugging, and async

These four compiler steps implement the existing [routine](../pascal/language/functions.md),
[interop](../pascal/interop/README.md), [CLR mapping](../specs/clr-mapping.md), and
[concurrency](../pascal/language/concurrency.md) specifications.

## Source overloads

Free routines and record methods may share a name when their CLR parameter
signatures or generic arities differ. Calls use the same conversions, inference,
constraints, named arguments, optional arguments, and params selection as imported
methods. Public overload groups work across units and from C#.

Result types, parameter labels, reference nullability, static/instance status,
and ref/out/in distinctions alone do not distinguish CLR signatures. By-value
and managed-reference parameters do. A delegate target selects a compatible
overload; generic method groups still require explicit type arguments.
Duplicate signatures report FP3004; failed/ambiguous calls report FP3003.

## Records implementing CLR interfaces

```pascal
type Descending = record implements Collections.IComparer<int>
  function Compare(Left: int; Right: int): int;
  begin return Right - Left; end function;
end record;
```

Records list imported interfaces after implements. Public instance methods and
read-only field properties implement matching methods/getters, including inherited
and generic members. Signature checks include nullability safety, generic
constraints, and parameter modes. Labels may differ. Emission uses InterfaceImpl,
virtual final methods, and MethodImpl metadata, including required CLR modifiers
on in parameters. Record instances retain reference identity through interfaces.

Default interface implementations are usable, including most-specific inherited
defaults. A source method may replace a default. Missing implementations,
unresolved default diamonds, incompatible signatures, setters/indexers/events,
static abstract requirements, and unsupported required signatures report FP3029.
There are no source interface declarations or explicit-interface implementations.

## Portable symbols

The CLI writes an assembly-matching portable PDB by default. Use --debug none
to disable symbols and remove the conventional stale PDB. Publication backs up
and restores the assembly, runtime configuration, and PDB together on failure.
The compiler API opts in with CompilationRequest.EmitDebugInformation and returns
the symbols as CompiledProgram.PortablePdb.

PDBs carry source documents with SHA-256 checksums, statement sequence points,
hidden compiler plumbing, local names/lexical scopes, closure method locations,
async kickoff associations, and yield/resume offsets. PE CodeView/checksum entries
identify the matching symbols. Identical inputs produce identical PE/PDB bytes.

Fpas.NET.Sdk defaults to DebugType=portable and also accepts none. Symbols follow
normal output/reference copying to C# hosts. Missing or modified intermediate
PDBs invalidate the compilation fingerprint; Clean removes generated symbols.
[XML documentation generation](xml-documentation.md) is implemented; IDE-specific
debugger integration remains pending.

## Async functions and programs

```pascal
async function Read(Value: Tasks.Task<int>): Tasks.Task<int>;
ensures (Returned: int): Returned > 0;
begin return await Value; end function;
```

Async functions and expression/block lambdas return Task or Task<T>. Their bodies
return the inner value, or use return/fallthrough for non-generic Task. Async
programs receive a synchronous CLR entry bridge which waits for the program task.
Ordinary async functions begin on the caller and suspend at an incomplete await.

The compiler emits CLR IAsyncStateMachine classes and AsyncTaskMethodBuilder
calls directly. No generated C#, blocking await wrapper, Task.Run wrapper, or
custom scheduler is involved. Task, Task<T>, ValueTask, ValueTask<T>, and their
standard ConfigureAwait awaitables are supported. Context capture, task faults,
and cancellation use their ordinary .NET behavior.

Locals, parameters, generic types, and closure storage survive suspension.
Structured control flow lowers to resumable blocks, including await in catch
and finally. Await using consumes IAsyncDisposable and finishes reverse-order
cleanup before leaving a scope, including return, break, continue, failed later
acquisition, faults, and cancellation. Synchronous using remains available.
Catch-filter exceptions preserve the original exception and reject that filter.

Requires checks run inside the state machine before the body and fault the task
on failure. Ensures binds the inner result after successful cleanup; faults and
cancellation bypass postconditions. Contracts also work through delegates and
CLR interfaces. Async void/procedures, ref/out/in async parameters, custom
awaiters, awaits in filters, and live managed references across suspension are
rejected. Evaluate awaited arguments before taking a reference when required.
FP3030 identifies async-context, awaitable, and reference-lifetime errors.

## Verification

The compiler tests inspect signatures, invalid declarations, PDB checksums,
sequence points, scopes, deterministic outputs, and async stepping metadata.
CLI tests execute overloads, generics, closures, async programs, expressions,
loops, cleanup, and contracts, and check actual source stack traces.

Manual acceptance with netcoredbg 3.2.0-1092 on Linux verified source breakpoints,
local inspection in a synchronous function, parameter/hoisted-local inspection
after await, and successive next steps across an incomplete await onto its
continuation thread. State-machine fields use the usual CLR debugger naming
conventions. Compiler plumbing stays hidden from source stepping.

Mixed C#/FPAS builds test exported overloads, interface dispatch/defaults,
negative interface signatures, task suspension with controlled completion,
synchronization-context capture, ConfigureAwait(false), faults/cancellation,
acquisition/disposal failures, and SDK PDB copying/invalidation/Clean.
The same solution build and separate test execution run on Windows and Linux.

[Primitive numbers and scalar domains](scalar-types.md) are implemented.
[Global fields and initialization](global-values.md) are implemented.
Named nested routines, authored
interfaces, events, arbitrary attributes, and IDE tooling
remain outside these four steps.
