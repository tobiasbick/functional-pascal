# FPAS 2 language directions

Status: proposal collection; implemented topics retain a short status and reference.
Topics are open proposals unless their section records implementation or an
agreement. The [numeric design](#21-explicit-width-numeric-types-money-and-decimal)
is agreed at the stated scope, with its listed open points. Design agreement does
not change the current specification or authorize implementation.
For graphics and packed multimedia data, native library buffers without new
scalar numeric types are the current preference; see
[packed numeric data](#22-packed-numeric-data-and-binary-buffers). This is a
preferred alternative, not an agreed buffer API or an implementation decision.
FPAS 1 stabilization is complete. Selected FPAS 2 topics use the
[AP workflow and templates](improve-syntax/README.md); selecting a topic still
requires an explicit decision before implementation.

The [standard-library namespace policy](../specs/std-namespaces.md) is binding
for contributor naming decisions. The current handbook documents the implemented
canonical imports. Future allocations do not authorize implementation of those
capabilities.

Selected foundation work includes [worker-releasing host I/O](#14-host-io-that-releases-vm-workers)
and the byte-buffer contract under `Std.Bytes`.
Continue the remaining selected FPAS 2 work before
graphics and the
[shared terminal/graphical UI plan](shared-ui.md). While deciding relevant
runtime and application topics, consult its coordination note so those decisions
do not accidentally preclude the later UI. This does not authorize early UI work
or change the status of any FPAS 2 proposal.

## Boundary

Keep functional Pascal with selected Ada conventions, predictable authoring and
few special cases. FPAS 1 should not gain a new effects system, ownership type
system or broad standard-library redesign to address these proposals. Prefer a
single replacement contract over parallel old/new APIs if a proposal is selected.

Preserve the deliberately different roles of positional/named complete calls,
value/InPlace operations, Result propagation/local handling, and lexical scopes
versus dynamic task groups. Similar expressiveness alone is not grounds for removal.

Keep a broad bundled standard library, preferably implemented as FPAS-facing Rust
wrappers around established Rust libraries. Native FFI and dynamically loadable
FPAS libraries extend this model so third parties can add missing functionality
independently of FPAS releases. Bundled libraries should continue to expose
consistent FPAS types, error handling and resource management.

This document owns shared language and runtime directions. The
[graphics application case](graphics.md) records concrete rendering needs and
keeps graphics-specific API, platform and deployment questions separate.

## 1. Typed operational errors

Status: implemented. See [native error families](../pascal/language/types/errors.md),
[MapError](../pascal/language/types/result-operations.md) and the
[Std reference](../pascal/std/README.md).

## 2. Uniform first-class operations

[Ordinary generic routines](../pascal/language/functions/generic-routines.md)
can specialize from a receiving callable signature. Some polymorphic intrinsics,
such as Math.Abs, cannot be used directly that way. Native receiver operations
also require wrapper functions when used as callbacks.

Consider making declared signatures sufficient to obtain callable values,
independent of whether implementation uses source code or an intrinsic. Preserve
static parameter labels, receiver capture by value, var modes and task effects.
Arbitrary callable-expression chaining remains deferred and is outside this topic.

Open questions: which native operations can be bound, variadic operations,
generic specialization and the cost of consistent compiler/editor support.

## 3. Separate equality from ordering constraints

Status: implemented. See [dictionary keys](../pascal/language/types/dictionaries.md) and
[constraints](../pascal/language/types/generics.md#built-in-constraints).

## 4. Task completion and result retrieval

`Wait` consumes a result once across all aliases of a handle, while WaitAll,
WaitAny and TaskCase only observe completion. A copied handle therefore does
not provide an independently readable result; see
[waiting](../pascal/std/concurrency/task.md#waiting-for-one-task).

Consider whether completed results should be readable repeatedly, keeping the
existing ownership and cleanup boundaries. Compare the simpler caller model
against result retention, shared-resource payloads, failure propagation and
supervised logical tasks. Do not assume a new linear/ownership type system is
necessary. Until a replacement is agreed, retrieve once and pass the value on.

## 5. Scope-bound cleanup actions

Status: preferred syntax proposal; behavior and implementation remain open.

Tasks already have [lexical ownership and cleanup](../pascal/language/concurrency/README.md).
Other resources still have explicit close operations; for example, callers must
[close an HTTP body stream](../pascal/std/network/http.md#streaming-responses)
when abandoning a response early. Adding a return or failed `try` can otherwise
leave application cleanup missing from an exit path.

### Preferred syntax

The preferred form attaches an indented `finally` section to an explicit
`begin ... end` block and closes it with `end finally;`. It follows FPAS's named
closers and is inspired by Pascal cleanup blocks and Ada's scope-bound resource
management. This is a proposal, not implemented syntax or an approval to implement.
The existing `try` expression continues to propagate Result/Option failures.

```pascal
uses Std.Io.Fs as Fs;

function ReadFirstLine(Path: string): result of (option of string, Fs.FsError);
begin
  const File := try Fs.Open(Path, Fs.FileMode.Read);

  begin
    return Fs.ReadLine(File);

    finally
      discard Fs.Close(File);
    end finally;
  end;
end function;
```

The proposed `finally` section belongs to its surrounding block rather than
being an ordinary statement after `return`. It runs when that block is left,
including by `return` or a failed `try`. If opening the file fails before the
block is entered, there is no close action. The explicit `discard` in this example
deliberately ignores a possible close error; it is not a general error policy.

Nested blocks can close resources in the reverse order of acquisition:

```pascal
function CopyChunk(SourcePath: string; TargetPath: string): result of (integer, Fs.FsError);
begin
  const Source := try Fs.Open(SourcePath, Fs.FileMode.Read);

  begin
    const Target := try Fs.Open(TargetPath, Fs.FileMode.Create);

    begin
      const Bytes := try Fs.Read(Source, 4096);
      const Written := try Fs.Write(Target, Bytes);
      return Ok(Written);

      finally
        discard Fs.Close(Target);
      end finally;
    end;

    finally
      discard Fs.Close(Source);
    end finally;
  end;
end function;
```

If opening `Target` fails, the outer block still closes `Source`. This example
writes one chunk and returns the number of bytes actually written; partial writes
remain possible. The proposal concerns explicit resource cleanup and does not
make the existing task `scope` automatically close file handles.

### Open questions and evaluation

Define visibility and initialization of values used in `finally`, and whether
the form is allowed on every block or also directly on routines and task scopes.
Specify exit behavior for normal completion, `return`, failed `try`, `break`,
`continue`, cancellation and `panic`; nested blocks and loop iterations need
clear lifetimes. Define which control transfers are permitted inside cleanup,
cleanup failure handling and precedence over an existing failure, and ordering
relative to postconditions and task joins. Cooperative cleanup must not imply
a hard shutdown deadline. Go-style function-level `defer` remains an alternative,
not the selected form.

Evaluation should cover real resource consumers, nested cleanup, every supported
exit path, multiple cleanup failures and interactions with task cancellation.

## 6. Entry-value snapshots in postconditions

[Postconditions](../pascal/language/functions/contracts.md) read completed `var`
parameter values but cannot refer to their values on entry. A routine that
increments a caller's counter cannot directly state that the final counter is
its entry value plus one.

Consider an explicit entry-snapshot operation restricted to postconditions,
inspired by [Ada's Old attribute](https://www.adaic.org/resources/add_content/standards/22rm/html/RM-6-1-1.html).
Keep the current distinction between contract violations and expected Result
errors. Value-oriented functions can already compare their returned value with
read-only inputs; establish the benefit for routines that mutate caller storage.
This proposal does not reopen the rejected purity/effects system.

Open questions: syntax, eligible parameter paths and expressions, snapshot timing
relative to preconditions and local initialization, conditional expressions,
aggregate copying costs, aliased `var` arguments, and shared-resource handles.
Specify whether unsupported snapshots are rejected; copying a handle must not be
mistaken for a snapshot of its external resource state.

Evaluation should cover scalar and aggregate parameters, aliases, early returns,
failed `try` returns, unchanged panic behavior and diagnostics that distinguish
entry values from final values.

## 7. Compile-time format-string checking

[String formatting](../pascal/language/types/string/format-chars.md) currently
reports malformed templates, argument-count errors and argument-type mismatches
at runtime. Even a literal template with statically known arguments can therefore
fail only when that call executes.

Consider checking known format strings during compilation, inspired by
[Rust's format checking](https://doc.rust-lang.org/std/fmt/). Earlier diagnostics
could identify the incorrect placeholder or argument without introducing another
formatting syntax. Keep one template grammar and argument-type contract across
compile-time checks and dynamic runtime formatting.

Open questions: literal-only versus compile-time constant templates, coverage of
Format and FormatInPlace, diagnostic locations and replacement hints, and the
boundary between static rejection and runtime validation of dynamic templates.

Evaluation should cover valid existing specifiers, escaped percent signs,
malformed templates, missing and extra arguments, type mismatches, constant
templates and dynamic templates. Confirm that checking does not evaluate runtime
arguments or change their evaluation order.

## 8. Option-valued array and string access

[Dictionaries](../pascal/language/types/dictionary-operations.md) already provide
Get returning Option. [Array access](../pascal/language/types/arrays.md) and
[string indexing](../pascal/language/types/string/edit.md#scalar-indexing-textindex)
instead report invalid indices as runtime failures. Programs expecting a missing
element must check bounds before indexing.

Consider an Option-valued lookup for arrays and strings, inspired by
[Rust's get operation](https://doc.rust-lang.org/std/primitive.slice.html#method.get).
Evaluate a common operation shape with dictionary Get, alongside existing checked
indexing. An absent element would be explicit data that existing case and try
rules can handle. Keep FPAS value semantics and Unicode scalar positions when
adapting this idea.

Open questions: operation names and signatures, negative and out-of-range
indices, value-copy and shared-handle behavior, and whether optional slice access
belongs in the same bounded proposal or separate work.

Evaluation should cover empty collections, first and last positions, both index
boundaries, Unicode strings, generic elements, and unchanged direct-index
behavior. Compare equivalent dictionary, array and string lookup consumers.

## 9. Pattern binding with an early exit

[Case arms](../pascal/language/pattern-matching/syntax.md#pattern-bindings) bind
variant payloads; [is tests](../pascal/language/pattern-matching/is-test.md) do
not introduce names. [Try](../pascal/language/error-handling/try.md) extracts a
success payload only when failure can propagate through a compatible enclosing
return type. Local handling with a different return value can require nested
case statements around the remaining work.

Consider one pattern-binding form with an explicit early-exit branch, inspired
by [Rust's let-else](https://doc.rust-lang.org/book/ch06-03-if-let.html#staying-on-the-happy-path-with-letelse).
On a successful match, bindings would be available to subsequent statements;
otherwise, the alternative must leave that continuation. Reuse existing FPAS
patterns and binding rules when choosing syntax. Establish the benefit for local
error handling and custom variants beyond existing try propagation.

Open questions: syntax, eligible patterns, binding scope and mutability, permitted
exit forms, diagnostics for an alternative that can fall through, and treatment
of patterns that always match. Preserve exhaustive case checking and define
interactions with lexical task scopes and cleanup actions.

Evaluation should cover Option, Result and user variants, nested patterns,
single evaluation of the selector, binding collisions, rejected fallthrough,
and every permitted exit form. Include examples that distinguish local handling
from automatic failure propagation.

## 10. Distinct duration and clock types

[Std.Time](../pascal/std/host/time.md) represents durations, UTC timestamps and
monotonic clock readings as integer. [Task timeouts](../pascal/std/concurrency/task.md)
also accept integer millisecond counts. These signatures cannot distinguish a
duration from a timestamp or a reading from the wrong clock.

Consider distinct types for duration and each clock domain, taking inspiration
from [F# units of measure](https://learn.microsoft.com/en-us/dotnet/fsharp/language-reference/units-of-measure).
First evaluate existing FPAS [distinct types](../pascal/language/types/distinct-types.md)
and explicit library operations. A full units-of-measure type system, unit
algebra and implicit conversions are not part of this proposal.

The [graphics application case](graphics.md#time-resolution) also needs
sub-millisecond monotonic readings for frame pacing, interpolation and profiling.
Evaluate microsecond or nanosecond resolution within the same clock and duration
contract, including explicit conversion to APIs that still use milliseconds.
Clock resolution alone does not guarantee timer accuracy or a frame deadline.

Open questions: type and constructor names, supported arithmetic and comparisons,
explicit unit conversions, valid ranges and overflow, cross-unit API ownership,
and migration of timeouts, backoff values and clock consumers. Include clock
resolution and precision loss at unit boundaries. Distinguish unit
conversion from conversion between wall-clock and monotonic readings.

Evaluation should include rejected duration/timestamp and clock-domain mixups,
valid elapsed-time calculations, sub-millisecond readings, conversion boundaries,
and migrated time and task consumers. Preserve existing clock and scheduling
semantics.

## 11. C foreign-function interface

Native standard-library integrations currently use built-in Rust code and
intrinsics. Applications cannot describe an arbitrary additional native library
and call it through a general FFI. The existing
[C-FFI direction](c-rewrite.md#c-libraries-in-applications-through-ffi) already
describes this goal independently of a prospective implementation-language change.

Consider a general C-ABI FFI that lets application authors describe exported
functions and C signatures in reusable FPAS binding units. A new binding and a
compatible native shared library should work with an already-built FPAS
distribution, without adding an intrinsic, changing the FPAS implementation or
rebuilding the compiler/runtime. Higher-level FPAS units could wrap those bindings
with ordinary FPAS values and error handling.

The initial interoperability target is the platform's C ABI. Libraries written
in Rust, C++ or Go can use the same route when they expose a compatible C
interface; direct import of arbitrary Rust crates or C++ interfaces is outside
this proposal. Keep the bundled standard library's preferred Rust-wrapper model.
Evaluate libffi for signature-driven native calls; loading, symbol resolution,
value conversion and ownership still need FPAS contracts. Existing project and
workspace dependencies should identify binding units.

Open questions: binding syntax and declarations, C type widths and layouts,
calling conventions, string encoding, buffers, native handles and pointer
lifetimes, allocation/freeing responsibilities, callbacks, variadic calls and
records passed by value. Define native-library discovery and deployment,
blocking-call scheduling, cancellation and failure behavior, and the boundary
with the Rust workspace's current prohibition of unsafe code.

Coordinate buffers and native layouts with
[packed numeric data](#22-packed-numeric-data-and-binary-buffers) and
[small-record representation](#23-small-records-without-heap-allocation).
Bundled graphics can use Rust-backed intrinsics independently of this general FFI.

Evaluation should use one unchanged FPAS distribution with a subsequently added
binding unit and native library. Cover target OS/architecture compatibility,
supported conversions, resource lifetimes, and diagnostics for missing libraries,
symbols and unsupported signatures. General native-code memory safety cannot be
established by FPAS bytecode verification alone.

## 12. Dynamically loadable FPAS libraries

[Library projects](../pascal/program-structure/projects.md) already provide
public unit interfaces and separately compiled source-adjacent objects. Their
units are linked into a complete executable before execution. The
[debugger's compatible reload](../pascal/tools/debugger.md) can replace inactive
function bodies but does not admit new functions, layouts or globals as a general
application extension mechanism.

Consider loading independently built FPAS libraries into a running application,
checking their exported interfaces and calling their routines through typed
bindings. This would allow third-party functionality and plugins to be added
without rebuilding the host application or the FPAS distribution. Build on
existing units, exports, project dependencies, interface metadata, relocatable
bytecode and executable verification. Treat this as a separate runtime capability
from native C-library loading; neither proposal requires the other.

Open questions: loading and invocation syntax/API, exported-signature checks,
artifact and compiler/bytecode compatibility, runtime symbol resolution, shared
type identity, dependency discovery and conflicts, and per-library globals and
initialization. Define module lifetimes while functions, closures or tasks still
refer to their code or values. Decide the loadable artifact and distribution
contract explicitly: current `.fpascu` sidecars are derived source-backed build
artifacts, not an existing source-less library distribution format. Preserve the
path/workspace dependency model without introducing a package manager or registry.
Unloading and replacement of active libraries require separate explicit decisions.

Evaluation should load a library built after the host application and call its
exports without rebuilding the host. Cover repeated loads, shared dependencies,
cross-library value and callback exchange, initialization failures, incompatible
interfaces/artifacts and missing exports. Failed loads must leave the running
application consistent; existing task ownership and cleanup rules must still hold.

## 13. Managed host processes and interactive pipes

Status: core process management, interactive pipes and selectable pipe waits are
implemented. Scope-bound cleanup and native-buffer payloads remain separate proposals.

[Std.Os.Proc](../pascal/std/host/proc.md) provides per-launch working directories and
environment overrides, supplied stdin, bounded stdout/stderr capture, timeouts
and cancellation through its options APIs. Those calls supervise the owned
process tree, reap the child and finish pipe cleanup before returning. Unix
process-group and Windows Job Object ownership and their limits are documented
in the [options contract](../pascal/std/host/proc.md#calls-with-options).

[Interactive process handles](../pascal/std/host/proc.md#interactive-processes)
provide `Start`, `ReadStdout`, `ReadStderr`, `WriteStdin`, `CloseStdin`, `TryWait`,
`TryWaitStatus`, `Terminate` and `Close`. Pipe operations probe pending/data/EOF
or partial-write progress; status observation is repeatable and does not consume
the pipes or handle. `ProcessExitStatus` preserves native exit codes and Unix
signal information. `RunInteractive` uses an explicitly provided host terminal
with timeout, cancellation and process cleanup.

Handles are VM-owned and aliases share the same child and close state. Callers
close completed handles explicitly; VM teardown releases remaining children and
pipes. A lexical task scope is not a general process-handle close scope.
Process startup, Run variants, RunInteractive and close release pool workers and
all-stop debugger tasks through the [managed host-I/O contract](#14-host-io-that-releases-vm-workers).
Interactive handles offer typed read, write,
stdin-close and status cases for `Tasks.Select`, combined with its timer and
cancellation cases. Selecting or abandoning a case does not close the handle.

Remaining work belongs to [scope-bound cleanup](#5-scope-bound-cleanup-actions), and any selected readiness
integration with [host events](#25-native-event-loops-and-selectable-host-events).
Pipe payloads currently use bounded `array of integer` bytes. Coordinate a future
change with the shared [native-buffer design](#22-packed-numeric-data-and-binary-buffers).
Further stop/grace-period policies require their own scope decision.

Existing regression coverage includes configuration validation, interactive and
captured I/O, partial progress, EOF, repeated status observation, timeout,
cancellation, output limits and owned process cleanup. Selectable pipe regressions
also cover single-worker progress, binary duplex traffic, discarded cases, inline
callbacks and debugger execution, plus exact allocation balance after abandonment.

## 14. Host I/O that releases VM workers

Status: implemented for the managed standard-library host operations. Filesystem,
socket configuration/I/O/TLS close, process startup/I/O/teardown, Console and ACP
host operations suspend pool tasks and ordinary all-stop debugger tasks, including
inline callbacks. OS entropy and PRNG seeding use bounded native jobs; seeded
PRNG draws stay in memory. MCP executor initialization and publication run on
its owned native thread from the first call.

Native requests contain owned Rust data; FPAS arguments and results remain on
their task. Admission bounds include undelivered results. Abandonment cancels
queued work, joins started work and disposes of unpublished resources. Shutdown
signals before joining, including from host callbacks. Main-program and explicit
synchronous evaluation retain their documented contracts. OS/library calls and
arbitrary native providers already in progress must return before joining; effects
are not rolled back. Short native queries and CPU computations remain synchronous.

One-worker gates, nested callbacks, ordinary debugger runs, bounded admission,
startup failures and exact first/repeated allocation balances cover these paths.
No throughput or latency improvement is claimed. Execution and ownership contracts:
[filesystem](../pascal/std/host/fs.md#execution-and-cancellation),
[file handles](../pascal/std/host/fs-streams.md#execution-and-native-ownership),
[network](../pascal/std/network/net.md#task-execution-and-resource-ownership),
[processes](../pascal/std/host/proc.md#blocking-and-concurrency),
[Console](../pascal/std/console/README.md#task-execution-and-ownership),
[ACP](../pascal/std/ai/acp/host.md#execution-and-response-rights),
[entropy](../pascal/std/cryptography/crypto.md#execution-and-ownership),
[PRNG](../pascal/std/numeric/random.md#execution-and-ownership) and
[MCP](../pascal/std/ai/mcp/README.md#ownership-and-execution).

[Native event loops](#25-native-event-loops-and-selectable-host-events) separately
cover main-thread ownership and host wakeups.

## 15. Shared deadlines and linked cancellation

[Std.Tasks](../pascal/std/concurrency/task.md) provides cancellation tokens and
individual timeouts. [Std.Net.Server](../pascal/std/network/server.md#one-shutdown-budget)
already shares one remaining budget across shutdown phases. There is no general
operation context carrying a shared deadline and linked cancellation across
task, network and process operations.

Consider a composable library contract for parent/child cancellation and a common
monotonic deadline. A child operation should receive the remaining budget rather
than restarting a full timeout at each step. Reuse existing cancellation and task
ownership; connect the design to typed operational errors and distinct duration
and clock types if those proposals are selected.

Open questions: API shape, parent/child lifetime, deadline inheritance, explicit
cancellation versus expiry, disposal of registrations and result precedence when
completion races with cancellation. Define how supported host operations observe
the context and how unfinished work remains owned after a waiting timeout.

Evaluation should cover nested operations sharing one budget, earlier parent
deadlines, parent cancellation, independent child cancellation, completion races
and cleanup. Preserve the distinction between cooperative cancellation and
confirmed termination of an external process.

## 16. Reusable concurrency patterns

Tasks, bounded channels, groups, selection and supervised retries already supply
the foundations for pipelines, worker pools and message loops. Applications still
compose their own coordination for many recurring patterns.

Consider bundled library operations for bounded parallel Map, semaphores,
request/reply, broadcast subscriptions and latest-value state notifications.
Small actor/message-loop helpers can build on the same ownership model; a full
actor runtime or supervision tree needs a separate scope decision. Prefer FPAS
composition where existing primitives suffice and focused Rust-backed support
where runtime integration is needed.

Open questions: operation names, result ordering, concurrency and queue limits,
failure propagation, cancellation and joining unfinished work. Define semaphore
permit lifetimes, reply ownership, subscriber cleanup and slow-consumer policies.
Broadcast delivery and latest-value notification need distinct contracts for
retaining or dropping intermediate values. Additional supervision policies must
state how they extend the existing fixed-retry behavior.

Evaluation should use real pipeline, worker-pool and request/reply consumers.
Cover empty input, concurrency limits, failures, cancellation, permit release,
closed replies, slow subscribers and shutdown without abandoned tasks.

## 17. Fair selection

[Select](../pascal/std/concurrency/task.md#ordering-and-callbacks) currently commits
the first ready case in input order. This permits explicit priority but does not
guarantee fairness. Repeated selection with an always-ready early case can leave
later ready cases unserved.

Consider a selection contract that prevents starvation among eligible ready
cases. Decide whether fairness replaces ordered selection or is explicitly
requested before selecting an API. Retain atomic winner commitment, exactly one
callback, and the rule that losing cases do not transfer channel values.

Open questions: rotation versus randomized choice, the scope and lifetime of
fairness state across newly created single-use cases, explicit priority for
cancellation and timers, task-failure precedence, and deterministic debugger
behavior. State the scheduling and readiness assumptions under which a fairness
guarantee holds; fairness alone does not establish a wall-clock deadline.

Include [selectable host events](#25-native-event-loops-and-selectable-host-events)
when defining eligible sources. Event-loop integration must preserve whichever
priority or fairness contract is selected here.

Evaluation should cover multiple continuously ready sources, changing readiness,
recreated cases, competing selectors, cancellation/timer priority and callback
failure. Verify the selected starvation guarantee and preserved ownership using
controlled readiness rather than timing-dependent expectations.

## 18. Validation and typed decoders

[Std.Encoding.Json.Fields](../pascal/std/text/json-fields.md) already provides strict typed
field access and rejects unknown fields using typed `FieldError` variants with
the original parser cause for parse failures.
There is no general library of composable validation rules and typed decoders
with structured, accumulated errors.

Consider reusable validators for scalar values, collections and nested objects,
and decoders returning a declared FPAS type or validation issues with field paths,
codes and messages. Support form input as well as external data. Invalid input
should remain expected Result data; a TUI draft must retain invalid text so users
can correct it. Existing generics, callables and records are the starting point.

Open questions: rule and decoder composition, optional fields, defaults,
transformations, cross-field rules, error accumulation and validation limits.
Start with explicitly declared record types and decoding functions. Automatic
record-type derivation from schemas or type-driven decoding requires a separate
decision on generation or compiler support.

Evaluation should cover nested paths, multiple independent errors, missing and
unknown fields, conversions, defaults and successful typed construction. Include
a TUI form that retains its draft and associates errors with the correct fields.

## 19. Application stores and selectors

Coordinate decisions with the later [shared UI plan](shared-ui.md#sequence-and-relationship-to-other-plans).

[Std.Ui.Terminal](../pascal/std/tui/application.md#update-and-view) already owns an
application model processed through Update and View. Generics and closures can
express typed state operations, but there is no reusable application-store and
selector-subscription library.

Consider typed stores with explicit actions, state snapshots, derived selectors
and subscriptions notified when their selected value changes. Integrate with
Model-Update-View and serial host-task ownership; background work should continue
to deliver typed messages through the application inbox. Allow explicit equality
callbacks, including for types outside current generic comparison constraints.

Open questions: store/model ownership, action ordering, subscription lifetime and
unsubscription, batched changes, reentrant updates, callback failures and selector
comparison. Relate default comparison to the equality-constraint proposal without
making it a prerequisite for explicit comparison callbacks.

Evaluation should cover independent state slices, unchanged selections, previous
and current values, subscription changes during notification, batching, nested
actions and shutdown. Include background-message integration and two independent
application instances.

## 20. Selective TUI updates

Status: tree comparison, layout reuse and damaged-row painting are implemented.
Explicit dependencies, selectors and cached subview construction remain open.
Coordinate decisions with the later [shared UI plan](shared-ui.md#sequence-and-relationship-to-other-plans).

The [TUI host](../pascal/std/tui/application.md#headless-frame-and-routing-order)
calls View for each render request, compares the resulting immutable tree with
the last successful tree, reuses stable layout, and repaints damaged rows.
Unchanged trees can skip validation, layout and painting. Terminal presentation
then transfers only changed cells. View construction itself is not skipped.

Consider explicit dependencies and cached subviews that reuse unaffected view
and layout results after a model change. Connect invalidation to selectors where
useful, while preserving immutable element trees, controlled inputs and serial
host execution. A store subscription alone does not define layout invalidation.

Open questions: cache identity and lifetime, dependency declaration, effect rules
for cached View callbacks, comparison, and propagation to parents and siblings.
Define invalidation for resize, focus, pointer state, modal changes and palettes,
including cleanup when elements disappear. Automatic dependency tracking and
language-level reactivity require their own explicit decisions.

Evaluation should compare visible output and input routing with uncached
execution, including focus, resize, menus, editing and background updates. Verify
unaffected work is skipped and measure benefits on representative TUI workloads.

## 21. Explicit-width numeric types, money and decimal

Status: agreed numeric design, carried forward from the
[graphics application case](graphics.md#numeric-types) and extended with
arbitrary-precision `decimal` for exact decimal calculations and lossless JSON
number values. The open points below still need decisions. No implementation is
scheduled. These types are not FPAS 1 prerequisites, and graphics is only one
consumer of the design.

The [preferred native-buffer alternative](#22-packed-numeric-data-and-binary-buffers)
can meet graphics storage needs without adding these scalar types or renaming
`integer` and `real`. Graphics therefore does not require implementing this
numeric design first. The design remains recorded for independent numeric
language work; the buffer preference does not cancel it. `money` and `decimal`
are outside this comparison of storage alternatives and retain their existing status.

FPAS offers exactly one way to express each numeric concept: every built-in
numeric type has one name, without alternative built-in aliases.

### Types

| Kind | Types |
| --- | --- |
| Signed integer | `integer8`, `integer16`, `integer32`, `integer64` |
| Unsigned integer | `unsigned8`, `unsigned16`, `unsigned32`, `unsigned64` |
| Binary floating point | `real32`, `real64` |
| Exact monetary amount | `money` |
| Arbitrary-precision decimal | `decimal` |

- `integer` is renamed to `integer64` and `real` to `real64`. The old names are
  removed, without an alias. This rename is a separate, mechanical step before
  the new types are added. A diagnostic for `integer` or `real` names the replacement.
- Subranges (`type Percent = 0..100;`) remain the only way to restrict values.
  Width types describe representation; subranges describe permitted values.
  There is no separate range type.
- There are no 128-bit integers and no 8-bit floats. A 16-bit float may later
  become a storage format of packed arrays only.

### Arithmetic and literals

- Arithmetic stays within its type: `integer8 + integer8` is `integer8`.
  Overflow is a runtime error, as for `integer` today.
- Operands of different numeric types are a compile-time error whose hint shows
  the explicit conversion.
- Wraparound never happens implicitly.
- A numeric literal takes the expected type: `var Pixel: unsigned8 := 200;`.
  Without an expected type, an integer literal is `integer64` and a real literal
  is `real64`. A literal outside the expected type, such as `300` for `unsigned8`,
  is a compile-time error.

### Conversions

There are no implicit conversions in any direction, including assignments,
arguments, and return values.

| Conversion | Form | Behavior |
| --- | --- | --- |
| Integer to another integer type | `unsigned8(X)`, `integer16(X)` | Checked; an out-of-range value panics. A known constant out of range is a compile-time error. |
| Integer to floating point | `real32(X)`, `real64(X)` | Replaces `Math.IntToReal`. Large magnitudes may lose precision. |
| `real64` to `real32` | `real32(X)` | Rounds to the nearest representable value; overflow panics. |
| Floating point to integer | `Math.Round`, `Math.Trunc`, `Math.Floor`, `Math.Ceil` | The rounding mode stays visible. The result is `integer64`; narrow it explicitly afterwards. |
| Value into a subrange | `Percent(X)` | Unchanged. |

- `X in unsigned8` tests whether a value fits without converting, using the
  existing subrange membership syntax.
- Saturation is written explicitly: `unsigned8(Math.Clamp(X, 0, 255))`.

### Bit operations

- The Pascal operators `and`, `or`, `xor`, `not`, `shl`, and `shr` apply to the
  unsigned integer types only. Signed values are converted explicitly first.
- Shifting by the bit width of the type or more is a runtime error, not a silent zero.

### Money

- `money` is an amount, not a currency. A currency code is an ordinary record
  field next to it.
- It is stored as a 64-bit integer scaled by 10,000, giving four exact decimal
  places. This fixed-scale type remains separate from general-purpose `decimal`.
- Literals in a `money` context are parsed as exact decimals:
  `var Price: money := 19.99;`. `money(X)` converts integers exactly.
- A runtime `real64` becomes `money` only through an explicit rounding function
  in `Std.Math`.
- Addition and subtraction are exact.

### Decimal

- `decimal` is the single type name for arbitrary-precision finite decimal
  values, not a fixed-precision `decimal128` or an alias for `money`.
- Conceptually, a value is an arbitrarily large signed integer coefficient
  multiplied by a power of ten: `123.456` is `123456 * 10^-3`. Precision is not
  capped at a fixed number of digits. Storage layout, exponent bounds and
  resource limits remain implementation decisions; no public big-integer type
  is implied.
- Literals in a `decimal` context and decimal text parsing construct the value
  directly from decimal digits, never through `real32` or `real64`.
- Addition, subtraction and multiplication are exact within resource limits;
  for example, decimal `0.1 + 0.2` equals decimal `0.3`. Exceeding a limit must
  report a failure, not silently discard digits or round the result.
- Division may have a non-terminating expansion, such as `1 / 3`. Without an
  explicit rounding request, a result that is not a finite decimal must fail.
  Rounded division and precision reduction require an explicit target scale or
  precision and rounding mode, not an ambient mutable precision setting.
- Equality and ordering are numeric: `1.0` and `1.00` compare equal. Original
  spelling, trailing zeros and requested display scale are formatting or source
  metadata, not distinct numeric identities.
- Conversion from integer types and `money` to `decimal` is exact. Conversions
  to fixed-width integers check integrality and range. Conversions to `money`
  check range and require explicit rounding if four decimal places do not suffice.
  Binary-floating conversions are always explicit; their precise rounding and
  failure contracts remain open.
- The type represents finite decimal values, not exact repeating fractions,
  infinity or NaN. TOML special floats need a separate format-level representation.

JSON is a concrete consumer: the current `JsonValue.Number(real)` loses integer
precision beyond binary64's exact range and cannot retain arbitrary decimal
digits. A future JSON value model can use `decimal` without routing numbers
through binary floating point. Exact numeric round trips do not preserve lexical
spelling: `1e2` and `100.00` may normalize to the same value. Preserve original
text separately when that distinction matters. TOML integration must retain its
integer/float distinction and explicitly decide how decimal precision relates
to its binary64 float interoperability rules; it is not changed by this agreement.

### Runtime representation

Fixed-width operations keep computing in 64-bit integers or 64-bit floats. The
width of a typed operation is an instruction operand, and the result is checked
or narrowed when it is stored. This keeps the opcode set small and the runtime
value at 16 bytes. The width matters for storage only in packed arrays.

`decimal` needs variable-sized coefficient storage rather than the fixed-width
register-only representation. Evaluate shared heap-backed value storage without
enlarging common runtime values. Decimal layout and packed-storage eligibility
are not agreed by this numeric design.

### Open points and evaluation

- The rounding rule for `money` multiplication and division, for example four
  places with ties to even, as `Math.Round` uses.
- The name of the rounding function from `real64` to `money`.
- Decimal parsing, formatting, exact and rounded division interfaces, including
  the meaning and names of target scale, precision and rounding modes.
- Exactness, rounding, signed zero and range behavior for conversions between
  `decimal` and binary floating point, and applicable generic constraints.
- Decimal coefficient storage, exponent bounds, allocation/work limits and
  failure contracts, including oversized or adversarial numeric input.
- JSON integration and any TOML integration, including typed field access,
  source-spelling preservation and nonfinite TOML values. Parser migration is a
  separate implementation decision, not a prerequisite for the numeric design.

Before implementation, assess migration of source code, standard-library
signatures, compiler/runtime formats and editor tooling. Coordinate storage with
[packed numeric data](#22-packed-numeric-data-and-binary-buffers). Evaluation must
cover literal boundaries, mixed-type rejection, checked arithmetic and
conversions, shifts, `real32` rounding, exact money literals and the selected
money-rounding rules. Decimal evaluation must cover values beyond binary64 and
fixed-width integer precision, more than four fractional places, exact arithmetic,
numeric equality across scales, terminating and non-terminating division,
explicit rounding and conversion, resource-limit failures and exact text round
trips. Include portable-image persistence and editor support. Verify diagnostics
for the removed type names.

## 22. Packed numeric data and binary buffers

Status: open proposal. Native library buffers without new scalar numeric types
are the current preference. No language form, library API or storage layout is
agreed, and implementation is not authorized by this preference.

The first selected scope is a byte buffer under `Std.Bytes`.
The binding [namespace specification](../specs/std-namespaces.md) allocates its
home and puts specialized media buffers in their respective domains. This
naming decision leaves the buffer API, value/mutation semantics, limits,
storage layout and implementation authorization open.

The [graphics application case](graphics.md#bulk-numeric-data) needs contiguous
vertex, index and pixel data without one tagged runtime value per element or
repeated conversion at each native handoff. Binary files, codecs, audio, process pipes and native FFI share the need
for a canonical byte representation. The
[standard-library roadmap](std-roadmap.md#open-decisions) leaves that choice open.

### Preferred alternative: native buffers with existing scalar types

Use one shared native binary-buffer foundation, with specialized interfaces for
pixels, vertices, indices and audio samples. Illustrative names are `ByteBuffer`,
`PixelBuffer`, `VertexBuffer`, `IndexBuffer` and `AudioBuffer`; these are possible
opaque standard-library types, not agreed declarations or new built-in types.
Binary files, codecs, process pipes and native consumers should share the byte
representation instead of introducing a separate one for each subsystem.

FPAS continues to calculate with its existing `integer` and `real`. Storage
formats such as `RGBA8`, `Float32`, `Index16` or signed 16-bit PCM select a native
layout; they do not introduce scalar types such as `unsigned8`, `integer16` or
`real32` into FPAS expressions. Neither adding these types nor renaming the
existing scalar types is a prerequisite for this route.

Buffer writes check ranges and perform any required narrowing or rounding once
when data enters the selected representation. Reads return ordinary FPAS scalar
values. Native decoders and generators can fill compatible buffers directly.
Bulk operations such as fill, blit and rectangular writes should avoid making
FPAS execute a call for every byte or sample. A vertex-layout description must
identify attribute formats, offsets, stride and alignment without depending on
the heap representation of ordinary FPAS records.

Native consumers should accept compatible storage without unpacking tagged FPAS
arrays or converting every element again at each handoff. GPU upload can still
require a copy, staging storage or row padding; contiguous host storage alone
does not promise zero-copy GPU access or a stable C ABI. GPU resources and native
audio delivery remain owned by their respective host layers.

This route keeps numeric expression rules smaller, at the cost of explicit
buffer-format APIs and runtime boundary checks. It does not supply general
fixed-width scalar arithmetic, arbitrary C FFI layouts, or an allocation-free
representation for vector and matrix calculations. Coordinate those separate
needs with the numeric, FFI and small-record proposals. `money` and `decimal`
are outside this comparison.

### Alternative: language-level packed arrays and explicit-width types

Keep packed arrays using the
[explicit-width numeric design](#21-explicit-width-numeric-types-money-and-decimal)
as an alternative. They could expose general typed contiguous numeric and
eligible fixed-layout record data, with corresponding compiler, VM, tooling and
image-format work. Compare this generality with the preferred library-buffer
route before selecting an implementation. Graphics alone does not require it.

### Open questions and evaluation

Open questions: construction, indexing and slicing, value semantics and mutation,
borrowing across native calls, task sharing, bounds and allocation limits,
alignment, byte order and record padding, asynchronous resource lifetimes and
release. Define supported storage formats, range and rounding rules, bulk access,
and layout validation. For the packed-array alternative, also decide which
numeric types and fixed-layout records are eligible.
Contiguous FPAS storage does not automatically establish C-ABI compatibility.

Evaluation should include binary-file round trips, embedded zero bytes, malformed
input, typed upload and download, bounds, aliasing and native-buffer lifetimes,
including task transfer, cancellation and cleanup. Include image, vertex/index
and audio consumers, scalar conversion boundaries and bulk operations.
Measure memory use and conversion costs on representative data before claiming
an improvement over ordinary arrays. Preserve one byte representation across
the affected standard units.

## 23. Small records without heap allocation

Status: open proposal; no new record declaration form is agreed.

Small vectors, matrices, colors and rectangles in the
[graphics application case](graphics.md#small-value-aggregates) expose allocation
costs from heap-backed records and intermediate arithmetic results.

First evaluate an optimized representation of existing
[records](../pascal/language/types/records.md) that preserves their value semantics.
Establish whether compiler/runtime changes can remove allocation without adding
another user-visible record category. A separate value-record declaration or
layout guarantee requires its own justification and explicit agreement.

Open questions: eligible field types and sizes, nested and generic records,
register and call representation, collection storage, copy costs and interaction
with shared-resource fields. Coordinate native layouts with
[packed numeric data](#22-packed-numeric-data-and-binary-buffers) and FFI; an
allocation optimization alone does not promise a stable external layout.

Evaluation should compare existing observable behavior across assignment,
field updates, arguments, returns, closures, generics and task transfer. Measure
allocations and execution costs in representative vector, transform and animation
code before selecting a representation or claiming a performance improvement.

## 24. Record arithmetic operators

Status: open proposal; user-defined operators are not approved.

The [graphics application case](graphics.md#vector-arithmetic-notation) expresses
vector arithmetic with calls such as `Add(Scale(V, 2.0), W)`. It suggests operator
declarations for arithmetic and comparisons on records.

Evaluate this against the rule of one canonical form per concept. Shorter or
more familiar notation alone does not justify a second form. Record concrete
transform and matrix examples, establish whether ordinary unit-level routines
serve them cleanly, and decide whether operator declarations belong in FPAS at all.
This topic is separate from equality and ordering constraints.

If selected, consider only established arithmetic and comparison operators with
pure, statically resolved implementations. Open questions include declaration
syntax, eligible operand/result types, generic resolution, mixed-type rejection,
equality and ordering consistency, diagnostics and tooling. New symbolic
operators, implicit numeric conversions and a general effects system are outside
this proposal.

Evaluation should compare equivalent routine and operator expressions, cover
matrix multiplication and scalar/vector operands, and test rejected ambiguous or
effectful declarations under the selected contract. Implementation requires an
explicit design decision after this evaluation.

## 25. Native event loops and selectable host events

Status: open proposal; no host-event API or new Select case is agreed.
Coordinate decisions with the later [shared UI plan](shared-ui.md#sequence-and-relationship-to-other-plans).

Windowing systems can require event pumping on the main thread. The
[main task's thread](../pascal/language/concurrency/scheduling.md) provides a
starting point, but the [graphics application case](graphics.md#event-loop-integration)
cannot yet wait on native events, channel messages and a frame deadline through
one integrated host loop.

Consider host-event readiness and wakeups integrated with task suspension and
selection. Evaluate a selectable source or host-loop integration that can combine
native events, channels, task completion and deadlines without busy polling.
Keep main-thread ownership explicit for event pumping and native callbacks;
background tasks can deliver messages to the serial application host.

Open questions: event-loop ownership, registration and disposal, thread affinity,
reentrancy, wakeup/completion races, frame scheduling, cancellation and shutdown.
Coordinate with [host I/O](#14-host-io-that-releases-vm-workers),
[clock types and resolution](#10-distinct-duration-and-clock-types) and
[selection fairness](#17-fair-selection). Preserve atomic winner commitment and
the ownership rules for losing cases. Graphics backend and window APIs remain
in the application case.

Evaluation should cover a main-thread native event source alongside continuously
ready channels and frame deadlines, progress with one VM worker, registration
races, cancellation and teardown. Verify thread affinity and the selected
priority/fairness contract; measure idle polling and frame-pacing behavior on the
chosen platforms before claiming a performance or latency improvement.

## 26. File handles and random-access file I/O

Status: implemented within the agreed scope. See
[file streaming](../pascal/std/host/fs-streams.md) for handles, random access,
`Size`, `SetLength`, complete UTF-8 text output and cooperative cancellation.
EOF uses existing read results; no `AtEnd` operation is added.

Sharing/locking controls and typed record files remain deferred, requiring
separate contracts. Native buffers, lexical resource release and worker-releasing
I/O belong to [packed data](#22-packed-numeric-data-and-binary-buffers),
[scope-bound cleanup](#5-scope-bound-cleanup-actions) and
[host I/O](#14-host-io-that-releases-vm-workers).

## 27. Standard-library namespaces

Status: implemented. See the binding [namespace specification](../specs/std-namespaces.md)
and [Std reference](../pascal/std/README.md). Future priorities belong to the
[Std roadmap](std-roadmap.md).

## Other deferred scope

A purity/effects system was rejected and is not reopened here. Reconsideration
would require a new decision and a demonstrated benefit. FPAS currently remains
function-oriented with explicit binding mutation, rather than a guarantee that
every function is pure.

## Evaluation before implementation

For each selected topic, first record concrete failing or awkward examples,
the smallest replacement contract, migration scope, compiler/runtime/tooling
cost and tests that establish the improvement. Preserve recorded design agreements
and resolve remaining open decisions with the user before creating implementation
work. An agreed design still needs an explicit implementation decision. This
document is not a second package-completion index.
