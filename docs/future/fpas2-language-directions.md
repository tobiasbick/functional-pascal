# FPAS 2 language directions

Status: prioritized directions and scope decisions. This revision selects
priorities and closes the alternatives listed below; it authorizes no
implementation. Agents resolve technical/interface choices autonomously;
changes to FPAS syntax need the user's explicit approval.
Implemented topics retain a short status and handbook reference. The
[numeric design](#21-explicit-width-numeric-types-money-and-decimal) remains agreed
at its recorded scope, with its listed open points.

FPAS 1 stabilization is complete. Selected FPAS 2 topics use the
[AP workflow and templates](improve-syntax/README.md). Design agreement and
priority do not change the current specification or authorize implementation.

The [standard-library namespace policy](../specs/std-namespaces.md) is binding
for contributor naming decisions. The current handbook documents the implemented
canonical imports. Future allocations do not authorize implementation of those
capabilities.

The managed [host-I/O foundation](#14-host-io-that-releases-vm-workers) and
[native byte buffers](../pascal/std/bytes.md), including UTF-8 integration, are
implemented. Native library buffers are the selected storage direction;
language-level packed arrays will not be implemented in FPAS 2. Specialized
media interfaces and their implementation remain separate decisions.

Continue selected foundations before graphics and the
[shared terminal/graphical UI plan](shared-ui.md). Topics assigned to that phase,
P3 topics and rejected alternatives are not prerequisites for starting graphics.
Consult the UI plan's coordination note when deciding relevant foundations.

## Priorities and status

Priorities order remaining work; section numbers identify topics. Within a
priority, use actual prerequisites and bounded deliverables to choose the next
topic. Priority is independent of agreement, implementation and verification.

| Priority | Meaning |
| --- | --- |
| P1 | First remaining improvements. |
| P2 | Follow after the relevant P1 foundations. |
| P3 | Retained for a concrete application need; no scheduled implementation. |
| Consumer-driven | Schedule the integration with its concrete consuming feature. |
| Graphics/UI phase | Decide with the graphics or shared UI application work. |
| Measured need | Select only after profiling demonstrates the problem. |

| Remaining topic | Priority | Actual prerequisites or owner |
| --- | --- | --- |
| [28. Semantic source queries](#28-semantic-source-queries) | P1 | Follow the existing source-query plan's phases. |
| [7. Format-string checking](#7-compile-time-format-string-checking) | P1 | Existing formatter grammar and argument rules. |
| [8. Option-valued access](#8-option-valued-array-and-string-access) | P1 | Existing Option, value and Unicode indexing rules. |
| [10. Duration and clock types](#10-distinct-duration-and-clock-types) | P1 | Existing distinct types and time/task interfaces. |
| [18. Validation and decoders](#18-validation-and-typed-decoders) | P1 | Existing records, callables and typed field access. |
| [29. Structural test comparisons](#29-structural-test-comparisons) | P1 | Existing equality rules and test diagnostics. |
| [4. Task results](#4-task-completion-and-result-retrieval) | P2 | Existing task ownership and cleanup rules. |
| [6. Entry-value snapshots](#6-entry-value-snapshots-in-postconditions) | P2 | Existing contracts, value semantics and alias rules. |
| [15. Deadlines and cancellation](#15-shared-deadlines-and-linked-cancellation) | P2 | Settle 10's clock/duration contract before this interface. |
| [16. Concurrency patterns](#16-reusable-concurrency-patterns) | P2 | Existing tasks/channels; use 15 where a shared budget is exposed. |
| [17. Fair selection](#17-fair-selection) | P2 | Existing Select winner and ownership rules. |
| [21. Money and decimal](#21-explicit-width-numeric-types-money-and-decimal) | P2 | Resolve each type's open arithmetic, conversion and failure rules. |
| [30. Generative behavior tests](#30-generative-behavior-tests) | P2 | Existing test runner and seeded PRNG; reuse 29's diagnostics. |
| [31. Bounded record and replay](#31-bounded-record-and-replay) | P2 | A demonstrated gap in current debug tools; D2 owns the detailed scope. |
| [9. Early-exit pattern binding](#9-pattern-binding-with-an-early-exit) | P3 | A demonstrated benefit over case and try; syntax agreement. |
| [11. C FFI](#11-c-foreign-function-interface) | P3 | Native ABI, deployment and resource contracts; reuse byte buffers. |
| [12. Loadable FPAS libraries](#12-dynamically-loadable-fpas-libraries) | P3 | Typed exports, artifact compatibility and code/value lifetimes. |
| [21. Numeric widths](#21-explicit-width-numeric-types-money-and-decimal) | P3 | Retain the agreed design; renaming is a prerequisite of the first selected new type. |
| [21. JSON/TOML integration](#21-explicit-width-numeric-types-money-and-decimal) | Consumer-driven | Exact JSON numbers need decimal; decide TOML integration separately. |
| [19. Stores and selectors](#19-application-stores-and-selectors) | Graphics/UI phase | Shared UI owns application-state needs; demonstrate a gap beyond Update/View. |
| [22. Specialized media buffers](#22-packed-numeric-data-and-binary-buffers) | Graphics/UI phase | Existing byte foundation plus concrete media consumers. |
| [23. Existing-record optimization](#23-small-records-without-heap-allocation) | Measured need | Allocation profile and behavior-preserving representation design. |
| [25. Host events](#25-native-event-loops-and-selectable-host-events) | Graphics/UI phase | Main-thread/event ownership; coordinate 10 and 17. |

Implemented topics have no remaining priority at their delivered scope. They are
1, 2, 3, 5, 13, 14, 20, 26 and 27, plus the byte-buffer foundation in 22.
Their handbook references own current behavior. Broader extensions retain the
status recorded in their sections.

Sections use `Decision pending` for unresolved designs, `Agreed direction` for
accepted directions with open details, and `Implemented` for delivered scope.
`Transferred` names another owning plan. `Deferred` parks work pending a new
decision. `Rejected; closed for FPAS 2` means it will not be implemented under
this plan; reopening requires a new explicit decision. The AP index alone tracks
implementation packages and their completion.

The closed alternatives are user-defined record operators in 24, language-level
packed arrays in 22, and a separate record declaration category in 23. The
existing rejection of a general purity/effects system remains in force.

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
Reassess replacing individual Rust dependencies with FPAS implementations after
the FPAS2 plan is complete; use established Rust libraries for this work first.

This document owns shared language and runtime directions. The
[graphics application case](graphics.md) records concrete rendering needs and
keeps graphics-specific API, platform and deployment questions separate.

## 1. Typed operational errors

Status: implemented. See [native error families](../pascal/language/types/errors.md),
[MapError](../pascal/language/types/result-operations.md) and the
[Std reference](../pascal/std/README.md).

## 2. Uniform first-class operations

Status: implemented. Source routines, Rust intrinsics, factories and payload
constructors share the name/reference and parenthesized-call rule. Receiving
signatures specialize generic and variadic operations; native receivers are
captured once by value. InPlace operations retain writable-storage requirements.
See [function values](../pascal/language/functions/first-class.md#native-operation-values)
and [payload constructors](../pascal/language/functions/function-types.md#payload-constructors-as-functions).
Type names remain types; arbitrary callable-expression chaining remains deferred.

## 3. Separate equality from ordering constraints

Status: implemented. See [dictionary keys](../pascal/language/types/dictionaries.md) and
[constraints](../pascal/language/types/generics.md#built-in-constraints).

## 4. Task completion and result retrieval

Status: Decision pending. Priority: P2.

`Wait` consumes a result once across all aliases of a handle, while WaitAll,
WaitAny and TaskCase only observe completion. A copied handle therefore does
not provide an independently readable result; see
[waiting](../pascal/std/concurrency/task.md#waiting-for-one-task).

Consider whether completed results should be readable repeatedly, keeping the
existing ownership and cleanup boundaries. Compare the simpler caller model
against result retention, shared-resource payloads, failure propagation and
supervised logical tasks. Do not assume a new linear/ownership type system is
necessary. Until a replacement is agreed, retrieve once and pass the value on.

The goal is to reduce dependence on result-consumption state hidden behind
aliases. Evaluation must cover repeated reads through copied handles, failure
results, retained shared resources, cancellation, supervision and cleanup.
Preserve existing task ownership unless an explicit replacement is agreed.

## 5. Scope-bound cleanup actions

Status: implemented. See [resource locals](../pascal/language/basics/local-variables.md#resource-locals)
for `using const`, guarded initialization, lexical cleanup and failure rules.
[Record release procedures](../pascal/language/types/records.md#resource-release)
support custom resources; native handles use their existing close operations.

## 6. Entry-value snapshots in postconditions

Status: Decision pending. Priority: P2.

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

Status: Decision pending. Priority: P1.

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

Status: Decision pending. Priority: P1.

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

Status: Decision pending. Priority: P3. Require a concrete consumer before
selecting additional syntax.

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

Status: Decision pending. Priority: P1. Settle this contract before the shared
deadline interface in 15; additional numeric widths are not a prerequisite.

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

Status: Decision pending. Priority: P3. Retain the independent extension goal;
general FFI is not a prerequisite for bundled graphics or agent tooling.

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

Status: Decision pending. Priority: P3. This capability is independent of C FFI.

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
implemented. Pipe payloads use [native byte buffers](../pascal/std/bytes.md).
Process handles support [scoped cleanup](../pascal/language/basics/local-variables.md#resource-locals).

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
can close completed handles explicitly or use a resource local; VM teardown
releases remaining children and pipes. A lexical task scope alone is not a
general process-handle close scope.
Process startup, Run variants, RunInteractive and close release pool workers and
all-stop debugger tasks through the [managed host-I/O contract](#14-host-io-that-releases-vm-workers).
Interactive handles offer typed read, write,
stdin-close and status cases for `Tasks.Select`, combined with its timer and
cancellation cases. Selecting or abandoning a case does not close the handle.

Remaining readiness integration belongs to
[host events](#25-native-event-loops-and-selectable-host-events).
Pipe payloads use bounded `Std.Bytes.ByteBuffer` values from the shared
[native-buffer foundation](#22-packed-numeric-data-and-binary-buffers).
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

Status: Decision pending. Priority: P2. Depends on the clock/duration contract
in 10 and existing task ownership; it does not require a new ownership system.

[Std.Tasks](../pascal/std/concurrency/task.md) provides cancellation tokens and
individual timeouts. [Std.Net.Server](../pascal/std/network/server.md#one-shutdown-budget)
already shares one remaining budget across shutdown phases. There is no general
operation context carrying a shared deadline and linked cancellation across
task, network and process operations.

Consider a composable library contract for parent/child cancellation and a common
monotonic deadline. A child operation should receive the remaining budget rather
than restarting a full timeout at each step. Reuse existing cancellation and task
ownership and typed operational errors. Use the duration and monotonic-clock
contract selected in 10, with explicit conversion at remaining integer APIs.

Open questions: API shape, parent/child lifetime, deadline inheritance, explicit
cancellation versus expiry, disposal of registrations and result precedence when
completion races with cancellation. Define how supported host operations observe
the context and how unfinished work remains owned after a waiting timeout.

Evaluation should cover nested operations sharing one budget, earlier parent
deadlines, parent cancellation, independent child cancellation, completion races
and cleanup. Preserve the distinction between cooperative cancellation and
confirmed termination of an external process.

## 16. Reusable concurrency patterns

Status: Agreed direction for a bounded first scope. Priority: P2. Interface
details remain open; no implementation is authorized.

Tasks, bounded channels, groups, selection and supervised retries already supply
the foundations for pipelines, worker pools and message loops. Applications still
compose their own coordination for many recurring patterns.

Start with bounded parallel Map and request/reply, using real application
consumers. Each operation must define failure propagation, cancellation and
joining or releasing unfinished work. Reuse existing task/channel ownership and
15's context where the interface exposes a shared deadline. Prefer FPAS
composition where existing operations suffice and focused Rust-backed support
where runtime integration is needed.

Semaphores, broadcast subscriptions, latest-value notifications and actor helpers
are deferred pending concrete consumers. A full actor runtime or supervision
tree remains outside this first scope and needs a separate decision.

Open questions for the first scope: operation names, result ordering, concurrency
and queue limits, failure propagation, reply ownership, cancellation and joining
unfinished work. Avoid unbounded queues and abandoned reply waiters.

Evaluation should use real parallel-map and request/reply consumers. Cover empty
input, concurrency limits, ordered results, failures, cancellation, closed or
abandoned replies and shutdown without abandoned tasks. Later patterns need
their own lifetime, buffering and slow-consumer contracts.

## 17. Fair selection

Status: Decision pending. Priority: P2. Build on existing Select semantics and
coordinate future host-event eligibility with 25.

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

Status: Agreed direction. Priority: P1. Start with explicit record types and
decoding functions; composition and error contracts remain open.

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

Construct validated domain values once at the input boundary. Reuse distinct
types, subranges, variants and records with private fields where they prevent
mixups or bypassing validation. Distinct type identity alone does not validate
the wrapped value. Decoder composition should replace repeated field plumbing
while keeping missing-field, default and unknown-field rules explicit.

Exact numeric decoding needs a lossless input representation. Decoding cannot
restore digits already lost by a binary-floating JSON parser. Coordinate such
consumers with 21's separate decimal and format-integration decisions; ordinary
typed decoding does not require the complete numeric-width family.

Open questions: rule and decoder composition, optional fields, defaults,
transformations, cross-field rules, error accumulation and validation limits.
Start with explicitly declared record types and decoding functions. Automatic
record-type derivation from schemas or type-driven decoding requires a separate
decision on generation or compiler support.

Evaluation should cover nested paths, multiple independent errors, missing and
unknown fields, conversions, defaults and successful typed construction. Include
a TUI form that retains its draft and associates errors with the correct fields.

## 19. Application stores and selectors

Status: Transferred to the shared UI design for application-state requirements.
Priority: Graphics/UI phase. The reusable store interface remains undecided.

The [shared UI plan](shared-ui.md#sequence-and-relationship-to-other-plans) owns
the application need and host integration. This section retains the shared
library constraints; a standalone store library is not a prerequisite for that UI.

[Std.Ui.Terminal](../pascal/std/tui/application.md#update-and-view) already owns an
application model processed through Update and View. Generics and closures can
express typed state operations, but there is no reusable application-store and
selector-subscription library.

First identify a real state-management gap beyond the existing Update/View
model and cached subviews. If that gap requires reusable stores, consider
explicit actions, typed state snapshots, derived selectors
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

Status: implemented. [Cached subviews](../pascal/std/tui/subviews.md) use typed,
explicitly projected dependencies and caller-supplied equality. Each application
owns its entries; unused entries and closed hosts release retained dependencies
and trees. Tree comparison, layout reuse and damaged-row painting remain part of
the [host render contract](../pascal/std/tui/application.md#headless-frame-and-routing-order).
General stores/subscriptions belong to [application stores](#19-application-stores-and-selectors).
Automatic dependency tracking and language-level reactivity remain unselected.
Coordinate future host changes with the [shared UI plan](shared-ui.md).

## 21. Explicit-width numeric types, money and decimal

Status: Agreed direction for the numeric design, carried forward from the
[graphics application case](graphics.md#numeric-types) and extended with
arbitrary-precision `decimal` for exact decimal calculations and lossless JSON
number values. The open points below still need decisions. No implementation is
scheduled. These types are not FPAS 1 prerequisites, and graphics is only one
consumer of the design.

The [selected native-buffer direction](#22-packed-numeric-data-and-binary-buffers)
can meet graphics storage needs without adding these scalar types or renaming
`integer` and `real`. Graphics therefore does not require implementing this
numeric design first. The design remains recorded for independent numeric
language work; selecting native buffers does not cancel it.

### Delivery priorities and prerequisites

Plan these parts separately while preserving the agreed arithmetic and conversion
rules below. Selecting one part does not authorize the others.

| Part | Priority | Prerequisites and scope |
| --- | --- | --- |
| `money` | P2 | Decide multiplication/division rounding, conversions and failures. |
| `decimal` | P2 | Decide arithmetic, rounding, resource limits, representation and failures. |
| Additional numeric widths | P3 | Require a concrete numeric consumer independent of graphics storage. |
| JSON integration | Consumer-driven | Requires decimal and a separate lossless JSON value/parser contract. |
| TOML integration | Consumer-driven | Decide preservation of TOML integer/float distinctions independently. |

The agreed mechanical rename from `integer`/`real` to `integer64`/`real64` remains
a prerequisite before adding new numeric types. Schedule that shared prerequisite
with the first selected type, including money or decimal if they come first.
Prioritizing money/decimal does not require delivering all additional widths.

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
- There are no 128-bit integers and no 8-bit floats. A 16-bit float as a
  language-level packed-array storage format is outside FPAS 2 under 22's closure.

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
value at 16 bytes. This representation does not add compact array storage;
section 22 selects native buffers for packed data in FPAS 2.

`decimal` needs variable-sized coefficient storage rather than the fixed-width
register-only representation. Evaluate shared heap-backed value storage without
enlarging common runtime values. Decimal layout is not agreed by this numeric
design; any native-buffer integration needs a separate format contract.

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

Status: native byte buffers, strict UTF-8 integration and canonical binary APIs
are implemented and verified. The current [byte-buffer](../pascal/std/bytes.md)
and [UTF-8](../pascal/std/network/utf8.md) references own their API, value,
capacity, lifecycle and artifact/debugger contracts. Specialized media buffers
remain open for the Graphics/UI phase. Language-level packed arrays are
Rejected; closed for FPAS 2. Record/replay is a separate topic in 31.

The foundation is `Std.Bytes.ByteBuffer`, with string/byte transitions in
`Std.Text.Utf8` and bulk byte/text append. Existing binary APIs share this
representation. There is no separate string or UTF-8 buffer type, and no new
scalar numeric type is required. The binding
[namespace specification](../specs/std-namespaces.md) owns current unit placement
and future media allocations; reserving a name does not authorize implementation.

The [graphics application case](graphics.md#bulk-numeric-data) needs contiguous
vertex, index and pixel data without one tagged runtime value per element or
repeated conversion at each native handoff. Binary files, codecs, audio, process pipes and native FFI share the need
for a canonical byte representation. The
[standard-library roadmap](std-roadmap.md#selected-order) points to the implemented
byte foundation; specialized formats remain separate proposals.

### Selected direction: native buffers with existing scalar types

Use one shared native binary-buffer foundation, with specialized interfaces for
pixels, vertices, indices and audio samples. Illustrative names are `ByteBuffer`,
`PixelBuffer`, `VertexBuffer`, `IndexBuffer` and `AudioBuffer`; these are possible
opaque standard-library types, not new built-in types. Only `ByteBuffer` is
implemented; the specialized media declarations remain unagreed.
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
needs with the numeric, FFI and existing-record optimization proposals. `money`
and `decimal` retain their separate numeric design and delivery priorities.

### Language-level packed arrays will not be implemented in FPAS 2

Status: Rejected; closed for FPAS 2. Use the native-buffer foundation for packed
data instead of adding a second language-level storage mechanism. Do not plan
packed-array declarations, compiler/VM representation or artifact changes under
this topic. Reopening requires a new explicit decision. This closure does not
cancel the agreed scalar numeric design in 21 or preclude native buffer formats.

### Open questions and evaluation

The implemented byte-buffer and UTF-8 contracts have no open design decision.
Follow the current handbook rather than reopening their accepted value,
mutation, limit, ownership and artifact rules through this broader proposal.

For specialized media buffers, open questions include construction, indexing
and slicing, value semantics and mutation, borrowing across native calls, task
sharing, bounds and allocation limits,
alignment, byte order and record padding, asynchronous resource lifetimes and
release. Define supported storage formats, range and rounding rules, bulk access
and layout validation.
Contiguous FPAS storage does not automatically establish C-ABI compatibility.

Evaluation should include binary-file round trips, embedded zero bytes, malformed
input, typed upload and download, bounds, aliasing and native-buffer lifetimes,
including task transfer, cancellation and cleanup. Include image, vertex/index
and audio consumers, scalar conversion boundaries and bulk operations.
Measure memory use and conversion costs on representative data before claiming
an improvement over ordinary arrays. Preserve one byte representation across
the affected standard units.

## 23. Small records without heap allocation

Status: Decision pending for optimization of existing records. Priority: Measured
need. A separate record declaration category is Rejected; closed for FPAS 2.

Small vectors, matrices, colors and rectangles in the
[graphics application case](graphics.md#small-value-aggregates) expose allocation
costs from heap-backed records and intermediate arithmetic results.

First demonstrate the allocation cost with a profile, then evaluate an optimized
representation of existing [records](../pascal/language/types/records.md) that
preserves their value semantics.
Use compiler/runtime representation changes that preserve the existing record
declaration and caller behavior. A new value-record category or declaration form
will not be implemented in FPAS 2. Reopening requires a new explicit decision.
Native layout guarantees remain separate from allocation optimization.

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

Status: Rejected; closed for FPAS 2. User-defined record operators will not be
implemented under this plan. Reopening requires a new explicit decision.

Use named routines for vector, matrix and other record calculations, including
the [graphics application's](graphics.md#vector-arithmetic-notation)
`Add(Scale(V, 2.0), W)` form. Additional operator declaration and resolution rules
are outside FPAS 2. Existing built-in equality and ordering constraints are unchanged.

## 25. Native event loops and selectable host events

Status: Decision pending. Priority: Graphics/UI phase. No host-event interface
or new Select case is agreed.
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

Byte payloads use the [native byte-buffer foundation](#22-packed-numeric-data-and-binary-buffers),
and blocking calls follow the [host-I/O contract](#14-host-io-that-releases-vm-workers).
Sharing/locking controls and typed record files remain deferred, requiring
separate contracts. [Lexical resource release](#5-scope-bound-cleanup-actions)
is available for file handles.

## 27. Standard-library namespaces

Status: implemented. See the binding [namespace specification](../specs/std-namespaces.md)
and [Std reference](../pascal/std/README.md). Future priorities belong to the
[Std roadmap](std-roadmap.md).

## 28. Semantic source queries

Status: Agreed direction; detailed contracts remain proposed in
[source queries for coding agents](agent-source-queries.md). Priority: P1.
[AP56](improve-syntax/ap56-source-queries/README.md) plans the common foundation,
find, inspect, direct call queries and checked refactoring on `ap56`. Technical
choices are delegated to the implementing agent; only changes to FPAS syntax
require user approval. Implementation has not started.
AP56 extends the common `fpas-language-service` analysis core. CLI and LSP/editor
connections are required parts of each feature, with shared semantic validation
and explicit handling of saved sources versus unsaved editor snapshots.

Existing positional queries expose definitions, references and checked types;
rename already provides checked edits. The linked plan owns extensions for name
discovery, focused inspection, direct callers/callees and checked parameter changes.
Follow its phases, starting with common query contracts, then find and inspect.
Read queries do not depend on runtime replay, C FFI or new language syntax.

Reuse compiler resolution and exact source snapshots. Bound the returned context
and distinguish complete answers, ambiguity, omitted data and unresolved dynamic
calls. A missing proven caller is not proof that a routine cannot execute.
Checked refactorings follow read-only discovery. AP56 specifies a shared plan/apply
flow for rename, extract-variable, organize-imports and append-only parameter
insertion, with explicit unsupported cases and complete validation before writes.

Acceptance belongs to the linked plan. Verify declaration lookup, relevant type
context, known uses and checked changes on fixed source fixtures, including
incomplete results and refusal cases. Use compiler, behavior and adapter checks;
LLM task comparisons and model-based acceptance gates are outside AP56. No
productivity or error-rate improvement is claimed.

## 29. Structural test comparisons

Status: Agreed direction. Priority: P1. The comparison interface and supported
types remain Decision pending.

Current [test assertions](../pascal/std/testing/test.md) compare scalar values.
Consider structural comparisons for records, variants, arrays and dictionaries,
using existing equality rules. A failure should identify the differing field,
variant or element and show expected and actual values. Reuse existing test
diagnostics and structured reporting instead of requiring application-specific
debug printing for each aggregate.

Open questions: eligible generic types, callable/native-resource fields,
dictionary ordering, path notation, limits for nested or large values and
whether one failure reports the first difference or a bounded set. Comparing a
handle must not imply comparing the external resource's contents. Do not
introduce new equality semantics through assertions.

Evaluation should cover nested records, variant/payload differences, missing
dictionary keys, array lengths, Unicode strings and equal aggregate values.
Require deterministic paths and explicit truncation in bounded text/JSON output.
Unsupported comparisons must report their limitation rather than pass silently.

## 30. Generative behavior tests

Status: Agreed direction. Priority: P2. Generator, shrinking and runner contracts
remain Decision pending.

Consider tests that generate many inputs for a declared behavioral property and
reduce a failing input to a smaller counterexample. Use the existing test runner
and seeded pseudorandom generator; reuse 29's aggregate diagnostics. Express
properties with existing FPAS routines before considering compiler support.

Record the seed, generator configuration and actual counterexample. Repeated
runs with the same code, configuration and seed should reproduce generated
cases. Shrinking must preserve input validity and the observed failure within a
bounded budget; it need not establish a globally minimal counterexample.
Generated-input reproducibility does not reproduce ordinary concurrent task
ordering or external host responses.

Open questions: generator composition, valid-domain constraints, case counts,
size/work limits, rejected inputs, shrinking order, timeouts and report format.
Expected behavior must come from stated properties or independent examples;
copying the implementation into a test oracle provides no independent check.

Evaluation should cover encode/decode round trips, collection invariants and
state transitions such as a rejected purchase preserving currency and inventory.
Include fixed-seed reproduction, constrained inputs, counterexample reduction,
budget exhaustion and reuse of a counterexample as a regression test.

## 31. Bounded record and replay

Status: Agreed direction for evaluating a bounded first delivery. Priority: P2.
The [D2 record/replay wish](agent-wishes.md#d2-record-replay-and-query) owns the
detailed scope and feasibility decisions. No replay interface is agreed.

Start with a concrete failure that existing failure dumps, batch debugging,
tracepoints and controlled scheduling cannot explain adequately. Select the
smallest useful recorded input and host-result set for that case, such as UI
events, clock readings or random values. The first delivery must state its
supported effects and concurrency coverage; it is not general recording of all
native execution.

Define executable/source identity, recording size and work limits, task-order
coverage, incomplete recordings and the host effects replaced by recorded
responses during replay. Unsupported effects must be explicit rather than
silently using live inputs. Existing bounded session recording and seeded debug
scheduling do not already implement replay of ordinary execution.

Evaluation should record one supported failure, replay it headlessly and inspect
the same relevant state repeatedly. Cover changed executable identity, omitted
inputs, exhausted limits and unsupported effects. General time-travel queries
and broader effect capture remain later D2 decisions. Preserve existing JSONL
debugging and its supported commands.

## Other deferred scope

A general purity/effects system is Rejected; closed for FPAS 2. It will not be
implemented under this plan. Reconsideration would require a new explicit
decision and a demonstrated benefit. FPAS currently remains function-oriented
with explicit binding mutation, rather than a guarantee that every function is pure.

## Evaluation before implementation

For each selected topic, first record concrete failing or awkward examples,
the smallest replacement contract, migration scope, compiler/runtime/tooling
cost and tests that establish the improvement. Preserve recorded design agreements
and resolve technical choices autonomously; obtain user approval for FPAS syntax
changes before implementation. An agreed design still needs an explicit implementation decision. This
document is not a second package-completion index.

State observable behavior independently of the implementation, using concrete
input/output examples, state invariants or UI outcomes that can become executable
acceptance tests. A test that repeats the implementation's assumptions is not
independent evidence of the requested behavior.

Use existing distinct types, subranges, variants and private-field records to
separate domain values and restrict construction where that prevents mistakes.
Keep validation at explicit construction/input boundaries. These are design
rules for the selected work, not additional language features.
