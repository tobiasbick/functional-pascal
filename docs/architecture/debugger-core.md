# Own CoreCLR debugger

`Fpas.Debugging` contains a protocol-neutral session core and a native CoreCLR
connection. DAP and batch use the same supervised operations and passive values.
The [debugging guide](../pascal/tools/debugging.md) describes the public command;
the [batch reference](../pascal/tools/batch-debugging.md) owns detached reports.

## Scope

The debugger launches already built .NET 10 executable DLLs on Windows x64 and
Linux x64, including Linux processes inside WSL2. It uses CoreCLR's public native
debugging interfaces through the packaged DbgShim. Source, symbols and runtime
dependencies are supplied by the caller; debugging does not build or restore.

The public frontends are batch and stdio DAP. Editor-specific extensions and
configuration are outside this debugger. Public attach, dump inspection, Native
AOT targets, native/mixed-mode debugging, mutation, managed function evaluation,
tracepoints, hot reload, instruction quotas and deterministic scheduling are
unsupported. The [test reference](../../tests/README.md#own-coreclr-connection-tests)
describes native Windows and WSL verification for Debug and Release tool builds.

## Sessions and process ownership

`Hosting.DebugSessionHost` starts one private worker through the same CLI
executable. `Fpas.Processes.OwnedProcess` establishes a Windows kill-on-close Job
Object or a Linux process session before the worker can launch a target.
The host never discovers ownership by scanning PIDs. The native dependency and
callback contract are described in the [native reference](../../src/Fpas.Debugging/Native/README.md).

The [single-file distribution](../build/distribution.md) uses this same executable
for workers and Linux target brokers. Its shim is materialized from the bundle
into the verified asset cache; no external worker or debugger download is needed.

The private channel uses a current-user named pipe, length-prefixed UTF-8 JSON,
a 1 MiB message ceiling and at most 64 outstanding requests, including a reserved
shutdown slot. It is separate from target input/output and is not a public
debugging protocol. Internal CLI entry points are not user commands.

The worker owns typed session state: created, starting, running, stopped, exited
or failed. Each worker has a fresh session ID; each stop has a fresh stop ID.
Monotonic revisions order replies and asynchronous state changes. Resume
invalidates the old stop. Inspection requires the current stopped identity.
Control operations serialize with initialization and cleanup while native
callbacks and the event reader remain active.

Step preparation completes before the running notification. An unavailable or
ambiguous caller rejects that operation without publishing a resume, changing
the stop revision or invalidating the held frames. Successful preparation
publishes running immediately before execution, so a subsequent immediate stop
or exit remains ordered after it.

On Linux, a private launch broker in the same owned process session is the
target's OS parent. It creates the suspended target and collects `waitpid` exit
status independently of DBI. The worker attaches through dbgshim. This preserves
actual nonzero exits and signals even when a managed observer cannot recover
exit status after native reaping. Windows observes its retained process handle.

## Bounds and shutdown

Worker startup and target preparation/initialization have 30-second deadlines;
ordinary operations have a 10-second deadline. Native/worker failure, an operation
deadline or cancellation ends the affected session. Unsupported inspection or
step preparation returns an operation error while preserving a valid stop.
Shutdown first sends a stop request with an independent
five-second grace, allowing native unregistration, target reaping and worker
exit. A hung worker then receives owned process-tree termination. Concurrent
failure cannot cancel that stop request's completion. Repeated stop/disposal
joins the same cleanup operation.

Target stdin defaults to closed. A caller may supply at most 1 MiB of explicit
bytes, written independently of control requests and output before EOF. Shutdown
reaps the worker before joining a blocked input writer. The host continuously
drains stdout and stderr, retaining at most 1 MiB with monotonic chunk sequences. Expired output cursors
report truncation. Decoding retains complete surrogate pairs across reads;
invalid UTF-8 becomes replacement characters with an encoding-loss flag. Byte
accounting measures original stream bytes, including incomplete UTF-8 at EOF.
Prefix capture never resumes retaining text after an omitted code point; a
partial code point at the quota waits for continuation or EOF. Total received
bytes and quota notification are independent of retained text. Programs requiring exact text
capture must select UTF-8 output. Output and state survive worker failure.

Ordinary shutdown joins readers and releases owned/native resources before
returning. An external OS kill of the supervisor, OOM or an uninterruptible
kernel operation cannot promise a final result.

DAP shutdown acknowledges completed owned cleanup, including forced termination
of an unresponsive worker. Internal worker-stop grace expiry is handled by that
cleanup before the protocol request completes.

## Sources and stopped context

The symbol catalog reads assembly metadata and portable PDBs without loading
target code into the debugger. PE CodeView GUID/stamp must match the PDB, and
primary source files must retain their recorded SHA-256 or SHA-1 checksums before
native process creation. Paths use native OS comparison rules. Windows accepts
case variants; Linux distinguishes them. Source locations use one-based UTF-16
coordinates and retain hidden-point and availability information.

Metadata interpretation also requires the MVID read from the actual loaded
module through `ICorDebugModule.GetMetaDataInterface` and
`IMetaDataImport.GetScopeProps` to match the catalog entry. After DLL/PDB
replacement, matching open metadata remains usable; new metadata with a different
MVID reports `module_mismatch`. Failed stack captures release
their newly retained frames rather than leaking handles into the next request.

At a stable stop, thread inspection enumerates CLR threads, with the stop thread
first. The catalog rejects more than 1024 threads. Managed stacks traverse CLR
chains and retain at most 4096 frames across the stop; unmanaged chains have
explicit omission counts. Pages contain at most 128 frames. A complete walk
reports a total; a clipped walk reports an unknown total and truncation.
Frames retain method/module/source facts, mapping precision and generated-code
status. Repeated pages preserve identity; resume invalidates every frame.

Frame binding lookup uses the selected invocation's arguments and lexical PDB
scopes. The innermost active binding wins when source names shadow one another.
The passive binding contracts below cover current values and availability.
Captures use the portable source-storage mapping described below.

## Passive scalar bindings

Frame binding pages read arguments and active lexical locals, with inner source
bindings taking precedence. Async pages read the mapped hoisted fields. Reads
return current storage, availability and independent errors; one missing value
does not discard the rest of a frame. Optimized storage is marked `optimized_out`
with no scalar. Resuming invalidates the stopped context rather than caching its
values for a subsequent stop.

| Value | Representation |
| --- | --- |
| Boolean and signed/unsigned integers, including native integers | Exact invariant scalar strings; integers retain their full width. |
| Binary floating point | Round-trip invariant text, including NaN, Infinity, -Infinity and negative zero. |
| Decimal | Stored fields decoded without methods; sign, scale and full precision remain intact. |
| CLR enum | Runtime type name and exact stored integral value; no target ToString call. |
| Char and string | UTF-16 content; lone surrogates have replacement display text plus hexadecimal raw code units. |
| Null | Explicit null kind and a null scalar. |

Strings retain at most 4096 Unicode scalars and mark omitted content. A binding
page accepts a nonnegative start and 1..128 entries, default 32. The catalog is
bounded at 4096 visible bindings. A conservative 512 KiB escaped-message budget
can shorten a page; total count and truncation remain explicit, and the next
page starts after the returned entries. Paging does not remove remaining names.
Type descriptions are bounded at 16 levels, 128 nodes and 64 generic arguments.
The [passive expression reference](debugger-expressions.md) describes the bounded
language subset, primitive operations and read limits.

## Passive stored values

Source frame pages include captures before arguments and active lexical locals;
the latter bindings take precedence. Portable storage markers distinguish direct
captures from compiler StrongBox cells, including hoisted async locals and
async closure receivers. Source names survive generated-field suffixes.
Cells expose their current stored Value. An unmarked user StrongBox remains an
ordinary object. Raw frame pages retain closure receivers and actual cells.
Recognized FPAS record/union instance frames expose their receiver as `self`,
including the stored receiver of an executing async method. Raw frames retain
CLR names. Invalid optional receiver metadata falls back to `this` with a
separate source error on the value, preserving raw inspection and the stop.
An ambiguous capture name reports an independent error rather than choosing a
storage location. See the [symbol contract](debug-information.md#passive-source-storage).

Raw expansion reads stored instance fields, including inherited fields, boxed
values and constructed generic objects. Runtime type descriptions retain the
loaded module receipt, MVID, type token, element kind and actual type arguments.
Equal names or assembly identities in separate load contexts do not merge types.
No getter, display attribute, proxy constructor or target ToString executes.
Unmanaged pointers, typed references and function pointers are unavailable and
are never dereferenced.

Arrays retain their actual rank, lengths and lower bounds independently of the
declared signature. Pages use row-major order and index labels with the runtime
bounds, including negative lower bounds. Zero-length arrays have no children.

Value references describe paths from a retained frame through stored fields or
array elements. They reread the current stop's storage and retain no native
value objects. Repeated paths have stable references; aliases share an object
identity. Cycles are marked without another expandable reference. Resume clears
all paths and object identities, and old references remain invalid at later
stops. Managed reference indirection is bounded at 16 levels.

Expansion is bounded at eight path levels, 4096 references, 4096 object identities,
4096 stored fields and 64 base types per object. Depth clipping is explicit;
reference exhaustion makes additional values unavailable while existing paths
remain readable. Child pages use the same entry and message budgets as bindings.

Static pages use the selected frame's declaring type or an explicitly named
loaded type. A constructed object's static view retains its actual generic
arguments. Active generic frames supply their declaring type arguments before
method arguments. Reads pass the selected CLR frame for thread-relative storage.
The implementation uses ICorDebugType.GetStaticFieldValue and never requests
type initialization. Missing storage has an independent unavailable result.
Literal fields are decoded from metadata, including full-width integers and
UTF-16 strings, with the same text limits as runtime strings.

An open generic definition needs a constructed value or active frame. Duplicate
loaded type names report ambiguity. Explicit type discovery examines at most
65536 metadata types; type names have at most 64 nesting levels and 8192 UTF-16
code units. Static fields use the ordinary binding/page/reference bounds.

Source value expansion uses the versioned Fpas.Core record, union-case and
domain contracts. Records expose source field order/names from stored backing
fields. Union cases expose their metadata tag and payload without calling Tag
or a proxy, including cases with no payload. Domains expose stored primitive
values; subranges decode their unsigned offset against the declared lower bound
with the primitive's exact width. Decimal scale, strings, characters and the full
signed/unsigned 64-bit ranges remain intact.

The raw view retains actual field names and encoded domain storage. Descriptors
distinguish raw and source child counts. Empty constructed objects still receive
a reference so their statics can be inspected; empty arrays have no children.
Source properties without a recognized stored mapping report `requires_execution`.
Display/proxy attributes never authorize target execution. Invalid optional
presentation metadata produces a separate source error while leaving raw storage
available. Rejected source inspection preserves the current stop/revision.

Metadata marker blobs are bounded at 64 KiB, record/property/field catalogs at
4096 entries, and source names at 8192 UTF-16 code units. Unknown contract versions,
invalid constructor identities, duplicate markers, missing backing fields and
invalid encoded ranges are explicit errors.

## Breakpoints and source control

A launch without an initial observation breakpoint stops internally at the
managed entry point before program statements execute. The core installs that
entry breakpoint once for the start module. Loading another copy of the same
assembly does not install another entry stop. Configuring a source breakpoint
at the held entry location produces a breakpoint stop before execution, without
losing the requested first statement.

Source configuration replaces one document's requests. An empty set removes
them. A session supports 64 distinct requests; duplicate line/column requests
collapse in request order. Requests have stable identities while retained.
Different requests resolving to one IL location share its native breakpoint and
report all hit identities. Results retain requested and actual locations, hit
counts and bound/pending/rejected state. Non-executable and ambiguous requests
have explicit errors without preventing independent valid requests.

When JIT compilation rejects a pending IL point, a private native method-entry
breakpoint waits for the compiled code before rebinding that source point. It
does not count as a user hit. Binding may pass debug NOPs, operand loads and
stores to unnamed argument temporaries in the same source region. A store to a
slot named anywhere in the method's PDB, a call, control flow or another operation
forms a barrier. The scan is bounded at 128 instructions. A missing mapping or
exceeded bound rejects the affected request while preserving the session.

The source stop retains the known breakpoint location with an approximate
marker when the runtime IL position differs; the frame keeps its actual runtime
offset. The same binding policy applies to already compiled methods. The
debugger does not disable optimization or execute target methods to inspect or
manufacture a binding.

Module-load callbacks bind pending requests before module user code runs.
Collectible module unload releases native points and returns requests to pending.
Source ambiguity across loaded copies is rejected. Foreign invalid or unsupported
PDBs produce module-specific issues, preserving valid unrelated breakpoints.
The primary target's preflight remains strict. Breakpoint and module-issue
changes are ordered core notifications and do not create user stops.

Updates during execution use a balanced private CLR pause. User breakpoint or
pause stops that race an update remain real stops. Into/over/out operations use
CLR steppers with IL ranges and portable source points. Hidden regions, missing
symbols and step-through/non-user methods are skipped; source closures remain
eligible even when their containers have compiler-generated metadata. Returning
to a caller inside its final statement can expose an approximate source position
instead of running through the remaining recursive calls.

Continue, pause, a breakpoint or an exception cancels any prior stepper. Automatic
hidden transitions are bounded at 4096 per source step. Optimized stepping uses
the available CLR/PDB mapping and can reach the next representable source
statement. The target's debug/optimization metadata remains effective.

## Async state machines

Portable PDB kickoff, yield/resume and hoisted-scope records map live MoveNext
frames to their source methods. A physical CLR frame retains its actual thread,
runtime name and source method. Weak CLR handles give live async instances stable
logical IDs across thread changes, with at most 128 retained live identities.
The running-session status may describe a pending async step; it does not expose
a fabricated stack for a suspended continuation. Frame inspection still requires
a real stopped frame and the current stop identity.

If optimized code no longer exposes a live frame's receiver, the frame retains
its real method and source but has no invented logical instance ID. Identity
availability and affected hoisted bindings are marked `optimized_out`. A source
step requiring that receiver is unavailable without invalidating the held stop.

Await steps use private yield/resume breakpoints and a temporary strong handle
to the selected state machine. Hits from other instances of the same method are
ignored. The next source stop after resumption exposes current hoisted int32
bindings and the same logical instance on its actual CLR thread. Technical
dispatch blocks carrying a repeated source point do not become extra user steps.
Already completed tasks use the same FPAS resume mapping without requiring a
yield. Pause, stop and failures release the private points and strong handle.

Into steps also arm at most 128 candidate MoveNext entries, including newly
loaded modules, on the initiating CLR thread. This enters source bodies through
the generated step-through kickoff. A body hit, user stop or await yield removes
these candidates. The same control handles nested generic calls, repeated real
suspensions and generic/non-generic Task and ValueTask awaiters, including
ConfigureAwait forms. FPAS async routines retain their supported Task return
types; accepting a ValueTask awaiter does not add a new language return form.

Async step-out reads the selected routine's stored builder/task continuation.
It follows the unique waiting FPAS state machine, retains its actual loaded
module receipt and installs at most 128 private resumption points filtered by
that object's strong handle. Identical assembly MVIDs in different load contexts
do not select another instance. The caller becomes an inspectable frame only
when it actually resumes on a CLR thread. A physical source caller may be used
when it is present on the real stack; no suspended caller frame is invented.

Continuation discovery recognizes the pinned .NET 10 task boxes, action targets,
await continuations and continuation lists through stored fields. It traverses
at most 16 levels, 128 objects and 64 list entries. Several waiting callers report
`ambiguous_caller`; missing symbols or an unknown continuation shape report
`unavailable`. Inspection never calls task, delegate or display methods. Pause,
user breakpoints, selected exceptions and shutdown cancel the owned async step
and release its private breakpoints and strong handle.

## Passive exception facts

Unhandled managed exceptions stop by default. An optional raised policy stops
first-chance managed exceptions, including framework code; the core does not
advertise a separate user-unhandled classification. Inspection reads type
metadata and the stored System.Exception message field. It invokes neither the
Message getter nor ToString, display code or managed evaluation.

The report separates historical throw frames from the current physical stack.
Historical native addresses map to available portable source positions only
when their loaded module/code identities match. Missing mappings remain explicit.
The capture contains at most 64 throw frames and 32 current frames; either kind
of omission marks truncation. Text retains at most 4096 Unicode scalars, with
raw UTF-16 data when replacement text would lose an unpaired surrogate. Stop
identities govern exception-object lifetime; continuing invalidates inspection.

## Verification

`CoreSessionTests` exercises real compiled FPAS targets through the private
production host. It covers current/stale stops, resume/pause races, cancellation before
and during runtime initialization, normal/nonzero/signal exit, callback failure,
worker crash/hang, bounded UTF-8 output and unrelated-process survival. Process
exit is asserted before fixture cleanup. See the [test instructions](../../tests/README.md#own-coreclr-connection-tests).

`SymbolContextTests` covers recursive invocations with different arguments and
shadowed locals, paged/stale frames, matching and mismatched files, source
checksums, spaces/Unicode, native path-case behavior and library/program sources
with equal filenames. A real DLL/PDB replacement after module load reproduces
the identity mismatch. Required platform runs belong in the acceptance plan.

`SourceControlTests` verifies entry side effects, duplicate/moved/rejected
breakpoints, replacement/removal, running updates, pending and collectible module
lifetimes, foreign-symbol isolation and repeated main-assembly loads. Source
stepping checks same-line columns, loop backedges, recursive frame depths and
values, step-through methods and exception interruption. The compiler's portable
symbol and loop suites retain their own metadata/execution checks.

`ExceptionInspectionTests` checks raised/default-unhandled policies, stored Unicode
messages, source/throw/current stacks and deep-stack omission markers. A real CLR
exception whose Message/ToString overrides write sentinel files proves passive
inspection. `AsyncControlTests` checks actual suspension, concurrent instances,
thread changes, logical IDs, hoisted int32 values, all selected await families,
completed awaits, nested generic entry, repeated suspension and fault/cancellation
cleanup. Logical step-out checks unique, missing and multiple callers, identical
assemblies in separate load contexts, optimized source stops and preservation of
the event revision after rejected preparation.

`ScalarInspectionTests` checks exact scalar data, enum/decimal storage, Unicode
and surrogate preservation, binding pages and byte clipping. Fully optimized
first-JIT targets disable tiering explicitly and keep optimization enabled. They
verify fresh/missing locals, source stops before file side effects and per-request
rejection for removed mappings or an exceeded binding scan.

`CompoundInspectionTests` verifies generic and boxed storage, aliases, cycles,
negative array bounds, stale paths, depth/reference limits and hostile display
code. `RectangularInteropTests` independently checks canonical CLR signatures,
rejected noncanonical shapes and rectangular calls in both C# interop directions.
`StaticInspectionTests` checks shadowed globals, constructed and active generic
types, thread-relative statics, literal/runtime fields, unavailable uninitialized
types, getter/initializer sentinels and stale static-object paths.
`CaptureInspectionTests` checks declaring frames, ordinary and async closures,
hoisted cells, raw storage, user StrongBox objects and source-name shadowing
before, during and after an inner declaration. `CaptureStorageMetadataTests`
independently checks the emitted storage format and deterministic output.
`SourceProjectionTests` checks generic records, empty/payload union cases, nested
domains, decoded/default subranges, exact extreme values and raw views. A foreign
CLR object with an invalid presentation marker proves that the source error
does not hide valid raw storage or change the stopped context.
