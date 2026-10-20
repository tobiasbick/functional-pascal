# FPAS 2 language directions

Status: proposal collection; delivered topics are recorded in their sections.
Topics are open proposals unless their section explicitly records an agreement. The
[typed operational error direction](#1-typed-operational-errors) and
[numeric design](#21-explicit-width-numeric-types-and-money) are agreed at the
scope stated in those sections, with their listed open points. Design agreement
does not change the current specification or authorize implementation.
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

This document owns shared language and runtime directions. The
[graphics application case](graphics.md) records concrete rendering needs and
keeps graphics-specific API, platform and deployment questions separate.

## 1. Typed operational errors

Status: Implemented. Native error declarations, typed causes, automatic output,
constructor values and explicit MapError are described in the current
[language handbook](../pascal/language/types/errors.md) and
[Std reference](../pascal/std/README.md). Functional acceptance includes source,
portable-image, editor and executable documentation checks. Elapsed-time
performance acceptance requires a controlled local host.

Channel closure, timeout and cancellation use `Tasks.TaskError` inside Result;
see [task APIs](../pascal/std/concurrency/task.md).
[Stdlib gap 9](stdlib-gap-review.md#direction-for-gap-9-structured-errors)
tracks the delivered native and source families. Selected Std interfaces support
typed decisions independent of diagnostic wording.

### Selected language direction

Native nominal error declarations and the Std FsError family are implemented:

```pascal
type ConfigError = error
  FileFailure(Cause: Fs.FsError) = 'Cannot load configuration';
  MissingSetting(Name: string) = 'Required setting is missing';
  InvalidSetting(Name: string; Reason: string) = 'Invalid setting';
end error;
```

Ordinary variant qualification, construction, inference and exhaustive matching
apply. The compiler supplies automatic output from variant labels and payload
metadata. A typed Cause retains the original error and is printed as the next
cause frame. There is no required ErrorContext/Describe/ErrorReport layer,
custom derivation framework or class inheritance. Optional literal messages add
readable labels without another interpolation language or formatter callback.

Error is also a built-in generic constraint implying Printable, not a storable
erased root type. Concrete native or application error types remain in Result.
One Message() operation and direct Printable output share the same renderer;
strict string formatting still requires a string. Full syntax, type, Cause and
output rules are owned by the [current handbook](../pascal/language/types/errors.md).

### Agreed MapError and try contract

```text
Value.MapError(F: function(Cause: E): E2): Result of (T, E2)
```

The receiver and callable expression evaluate once in normal order. Ok preserves
its success payload without invoking the mapper; Error invokes it once and wraps
the returned error payload. Callable capture construction has ordinary evaluation
semantics even on Ok. T is unchanged; general Result payloads remain supported.
Map transforms success, MapError transforms error, and OrElse supports recovery.

Payload variant constructors are ordinary routine values, consistently for enums
and errors. The selected use is:

```pascal
const Text := try Fs.ReadText(Path).MapError(ConfigError.FileFailure);
```

Extra context uses an ordinary captured callable, not partial application. Try
retains its [existing semantics](../pascal/language/error-handling/try.md): extract
Ok or propagate Error through a compatible enclosing error type. Incompatible
error families remain a static error; no automatic wrapper/subtype conversion is
added. Option propagation, panic, contracts, cleanup and task ownership remain
unchanged. Neither native errors nor MapError catch runtime failures.

### Delivery and scope

The [Std reference](../pascal/std/README.md) documents domain families and their
[shared native cause](../pascal/std/host/errors.md), normalization and explicit
mappings. Selected signatures and their consumers use those concrete families;
there is no parallel typed/string wrapper API. Source-authored application Result
errors remain general and require no runtime registration.

The [compiled-program handbook](../pascal/program-structure/compiled-programs.md)
documents artifact compatibility. Error metadata adds no polling to ordinary VM
instruction dispatch. Further language or library capabilities require their
own design and implementation decision.

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

Use the shared [binary-buffer design](#22-packed-numeric-data-and-binary-buffers)
for pipe payloads rather than defining a process-specific byte representation.

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

[Native event loops](#25-native-event-loops-and-selectable-host-events) add
main-thread ownership and host wakeups to this assessment. Releasing a VM worker
does not by itself integrate a window event loop or satisfy its thread rules.

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

Include [selectable host events](#25-native-event-loops-and-selectable-host-events)
when defining eligible sources. Event-loop integration must preserve whichever
priority or fairness contract is selected here.

Evaluation should cover multiple continuously ready sources, changing readiness,
recreated cases, competing selectors, cancellation/timer priority and callback
failure. Verify the selected starvation guarantee and preserved ownership using
controlled readiness rather than timing-dependent expectations.

## 18. Validation and typed decoders

[Std.Json.Fields](../pascal/std/text/json-fields.md) already provides strict typed
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

## 21. Explicit-width numeric types and money

Status: agreed numeric design, carried forward from the
[graphics application case](graphics.md#numeric-types); the open points below
still need decisions. No implementation is scheduled. These types are not FPAS 1
prerequisites, and graphics is only one consumer of the design.

FPAS offers exactly one way to express each numeric concept: every built-in
numeric type has one name, without alternative built-in aliases.

### Types

| Kind | Types |
| --- | --- |
| Signed integer | `integer8`, `integer16`, `integer32`, `integer64` |
| Unsigned integer | `unsigned8`, `unsigned16`, `unsigned32`, `unsigned64` |
| Binary floating point | `real32`, `real64` |
| Exact monetary amount | `money` |

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
| Integer to floating point | `real32(X)`, `real64(X)` | Replaces `Conv.IntToReal`. Large magnitudes may lose precision. |
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
  places. This is a fixed-scale amount, not a general-purpose Decimal type.
- Literals in a `money` context are parsed as exact decimals:
  `var Price: money := 19.99;`. `money(X)` converts integers exactly.
- A runtime `real64` becomes `money` only through an explicit rounding function
  in `Std.Math`.
- Addition and subtraction are exact.

### Runtime representation

Registers keep computing in 64-bit integers or 64-bit floats. The width of a
typed operation is an instruction operand, and the result is checked or narrowed
when it is stored. This keeps the opcode set small and the runtime value at 16
bytes. The width matters for storage only in packed arrays.

### Open points and evaluation

- The rounding rule for `money` multiplication and division, for example four
  places with ties to even, as `Math.Round` uses.
- The name of the rounding function from `real64` to `money`.

Before implementation, assess migration of source code, standard-library
signatures, compiler/runtime formats and editor tooling. Coordinate storage with
[packed numeric data](#22-packed-numeric-data-and-binary-buffers). Evaluation must
cover literal boundaries, mixed-type rejection, checked arithmetic and
conversions, shifts, `real32` rounding, exact money literals and the selected
money-rounding rules. Verify diagnostics for the removed type names.

## 22. Packed numeric data and binary buffers

Status: open proposal; no language form, library API or storage layout is agreed.

The [graphics application case](graphics.md#bulk-numeric-data) needs contiguous
vertex, index and pixel data without one tagged runtime value or conversion per
element. Binary files, codecs, audio, process pipes and native FFI share the need
for a canonical byte representation. The
[standard-library roadmap](std-roadmap.md#open-decisions) leaves that choice open.

Consider one shared design for byte buffers and typed contiguous numeric data,
using the [explicit-width types](#21-explicit-width-numeric-types-and-money).
Evaluate a language-level packed array against a standard-library buffer type
before choosing either. Native intrinsics should be able to access supported
layouts without per-element conversion. Include binary reads and writes in
`Std.Fs`; image decoding and GPU upload policy remain graphics-specific consumers.

Open questions: construction, indexing and slicing, value semantics and mutation,
borrowing across native calls, task sharing, bounds and allocation limits,
alignment, byte order and record padding. Decide which numeric types and
fixed-layout records are eligible and how layouts are described and checked.
Contiguous FPAS storage does not automatically establish C-ABI compatibility.

Evaluation should include binary-file round trips, embedded zero bytes, malformed
input, typed upload and download, bounds, aliasing and native-buffer lifetimes.
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

Status: open proposal; no handle type, open modes or byte API is agreed.

`Std.Fs` reads and writes whole files only. Turbo Pascal programs used
`Assign`/`Reset`/`Rewrite`/`Append`, `Seek`, `FilePos`, `FileSize`, `Eof`,
`BlockRead`/`BlockWrite`, `Truncate`, `Flush` and `Close` for large files, logs,
record files and binary formats. FPAS has no equivalent: a 64 MiB text file
cannot be processed line by line, and no file can be read or written at an offset.

Consider one opaque handle type in `Std.Fs` with an explicit open mode
(read, write, read-write, append; create, truncate or require existing), plus
`Seek`, `Position`, `Size`, `AtEnd`, `ReadLine`, `WriteText`, byte block reads and
writes, `Truncate`, `Flush` and `Close`. Byte operations depend on the canonical
byte representation from [packed numeric data and binary buffers](#22-packed-numeric-data-and-binary-buffers);
whole-file `ReadBytes` and `WriteBytes` should land with the same design.
Keep `Assign` out: opening returns a handle in one step.

Open questions: who closes a handle and when (explicit `Close`,
[scope-bound cleanup](#5-scope-bound-cleanup-actions) or both); behavior after
close; sharing a handle between tasks; positions as byte offsets versus
character positions for text; line-ending and UTF-8 boundary handling for
`ReadLine` and `Seek`; buffering and flush-on-close; Windows sharing modes;
[worker-releasing host I/O](#14-host-io-that-releases-vm-workers); and
[typed operational errors](#1-typed-operational-errors) for end of file,
missing files and permission failures. Typed record files (`file of T`) need a
fixed layout and are out of scope until packed records are decided.

Evaluation should cover streaming a file larger than the `ReadText` limit,
random reads and in-place updates at offsets, append from several tasks,
truncate and extend, use after close, partial reads at end of file, and
handle leaks on early `return` and panics.

## Other deferred scope

A general-purpose Decimal type remains deferred and is not part of the agreed
fixed-scale `money` design. It needs a separate use case and design decision.
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
