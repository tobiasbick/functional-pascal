# FPAS 2 language directions

Status: proposals retained for later evaluation; no implementation is scheduled.
These are design questions, not approved language changes or the current spec.
FPAS 1 stabilization is complete. Selected FPAS 2 topics use the
[AP workflow and templates](improve-syntax/README.md); selecting a topic still
requires an explicit decision before implementation.

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

## 1. Typed operational errors

Channel closure, timeout and cancellation are currently different strings inside
`Result of (T, string)`; see [task APIs](../pascal/std/concurrency/task.md).
Programs that distinguish them must compare diagnostic text.

Consider typed error variants such as Closed, Timeout and Cancelled, with
human-readable context kept separately. Callers could use exhaustive case
matching without depending on wording. Evaluate the scope across task, network
and host APIs together; do not add a second parallel family of wrappers.

Open questions: shared versus unit-specific error types, context payloads,
translation across API boundaries and compatibility with generic Result/try.

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

A concrete record with equality-capable fields may be a dictionary key, but a
generic key currently needs Comparable, which excludes records. See
[dictionaries](../pascal/language/types/dictionaries.md) and
[constraints](../pascal/language/types/generics.md#built-in-constraints).

Consider a distinct equality capability so generic code can accept every key
type accepted by equivalent concrete code. Keep ordering as the requirement for
sorting. Determine whether collection equality belongs in the same design;
do not automatically add interfaces, arbitrary constraints or user-defined operators.

Open questions: the smallest constraint vocabulary, structural propagation,
recursive types, and effects on existing dictionary and comparison checking.

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

Tasks already have [lexical ownership and cleanup](../pascal/language/concurrency/README.md).
Other resources still have explicit close operations; for example, callers must
[close an HTTP body stream](../pascal/std/network/http.md#streaming-responses)
when abandoning a response early. Adding a return or failed `try` can otherwise
leave application cleanup missing from an exit path.

Consider one explicit cleanup-registration construct inspired by
[Go's defer](https://go.dev/doc/effective_go#defer), placing cleanup beside resource
acquisition. Go attaches deferred calls to the enclosing function; decide whether
FPAS should attach them to a lexical block or a routine before selecting syntax.
Reuse the existing task ownership and failure rules when the constructs interact.

Open questions: ownership and registration lifetime, argument evaluation and
capture timing, reverse registration order, captured `var` storage, repeated
registration in loops, and behavior on return, break, continue, failed `try`,
cancellation and panic. Define cleanup-error handling, precedence over an
existing failure, and ordering relative to postconditions and task joins.
Cooperative cleanup must not imply a hard shutdown deadline.

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

Open questions: type and constructor names, supported arithmetic and comparisons,
explicit unit conversions, valid ranges and overflow, cross-unit API ownership,
and migration of timeouts, backoff values and clock consumers. Distinguish unit
conversion from conversion between wall-clock and monotonic readings.

Evaluation should include rejected duration/timestamp and clock-domain mixups,
valid elapsed-time calculations, conversion boundaries, and migrated time and
task consumers. Preserve existing clock and scheduling semantics.

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

[Std.Proc](../pascal/std/host/proc.md) starts external programs and waits for
completion. It does not expose live process handles, interactive stdin/stdout/
stderr pipes, per-process environment or working-directory settings, timeouts or
process-tree termination. Running a process call in go still occupies its worker.

Consider managed process handles with separate start, status observation and wait
operations, incremental pipe I/O, bounded capture and explicit stop policies.
Include working-directory and environment settings per launch. This would support
long-running command-line tools, stdio protocol servers and independently
terminable FPAS worker processes. Prefer a bundled Rust-backed implementation
integrated with existing task ownership, cancellation and error handling.

Open questions: process ownership, pipe and binary-buffer types, EOF and stdin
closure, exit-status representation, repeated waits, output limits, and cleanup
when an owning task exits. Define cancellation, grace periods, termination
requests, confirmed exit and child-process reaping separately. Process-tree
handling and platform differences need an explicit contract.

Evaluation should cover interactive request/reply, simultaneous stdout/stderr
draining, full pipes, output limits, launch failure, nonzero exit, cancellation,
timeout and process-tree cleanup. Include an FPAS worker process without requiring
general in-process task termination.

## 14. Host I/O that releases VM workers

[Task waits and channels](../pascal/std/concurrency/task.md#waiting-and-execution)
already suspend a task and release its pool thread. Process execution and other
host I/O can still occupy workers; even
[cancellable network I/O](../pascal/std/network/net.md) polls while retaining its
VM worker. Enough pending host calls can therefore occupy the available pool.

Consider integrating pending network, pipe and process operations with task
suspension so waiting does not retain a VM worker. Assess filesystem and native
FFI calls separately where the host operation cannot suspend directly. Compare
event-driven backends and bounded blocking execution using concrete workloads;
do not assume that cancellation polling or nonblocking sockets alone release a
VM worker.

Open questions: operation ownership, wake registration, completion races,
backpressure, pending-operation limits, shutdown and debugger integration.
Distinguish stopping the caller's wait from ending underlying host work, including
DNS, certificate verification and arbitrary native calls. Define late completion
and resource cleanup before choosing a backend.

Evaluation should demonstrate progress with one VM worker and overlapping pending
operations, including nested callbacks. Cover registration/wakeup races,
cancellation, timeout, teardown and bounded resource use. Measure throughput and
latency on the affected workloads before claiming a performance improvement.

## 15. Shared deadlines and linked cancellation

[Std.Tasks](../pascal/std/concurrency/task.md) provides cancellation tokens and
individual timeouts. [Std.Server](../pascal/std/network/server.md#one-shutdown-budget)
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

Evaluation should cover multiple continuously ready sources, changing readiness,
recreated cases, competing selectors, cancellation/timer priority and callback
failure. Verify the selected starvation guarantee and preserved ownership using
controlled readiness rather than timing-dependent expectations.

## 18. Validation and typed decoders

[Std.Json.Fields](../pascal/std/text/json-fields.md) already provides strict typed
field access and rejects unknown fields, returning individual string errors.
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

[Std.Tui](../pascal/std/tui/application.md#update-and-view) already owns an
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

The [TUI renderer](../pascal/std/tui/application.md#interactive-terminal) already
transfers only changed terminal cells. After each processed message, however,
the host calls View and performs validation, layout and painting again. Terminal
damage tracking does not itself skip view construction or layout work.

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

## Other deferred scope

Explicit-width numeric types and Decimal need a separate coherent numeric
design, migration and buffer/API assessment. They are not FPAS 1 prerequisites.
A purity/effects system was rejected and is not reopened here. Reconsideration
would require a new decision and a demonstrated benefit. FPAS currently remains
function-oriented with explicit binding mutation, rather than a guarantee that
every function is pure.

## Evaluation before implementation

For each selected topic, first record concrete failing or awkward examples,
the smallest replacement contract, migration scope, compiler/runtime/tooling
cost and tests that establish the improvement. Resolve the open decisions with
the user before creating implementation work. This document is not a second
package-completion index.
