# Source debugger

`fpas debug` runs a program in the bounded source debugger. It accepts a
`.fpas` program, a program `.fpasprj`, a workspace with one program, or a
`.fpascp` image. No debugger operation changes FPAS language semantics.

```text
fpas debug app.fpasprj --protocol jsonl
fpas debug app.fpasprj --protocol jsonl --commands session.jsonl --report jsonl
fpas debug app.fpasprj --protocol dap
```

Program arguments follow `--`. JSONL live mode uses one UTF-8 JSON object per
line on stdin/stdout; script mode processes `--commands` deterministically.
DAP uses standard `Content-Length` framing. Protocol stdout never contains raw
program output; output is delivered as structured events.

The complete wire contracts are documented in [JSONL protocol V2](debugger-jsonl.md)
and the [Debug Adapter Protocol contract](debugger-dap.md).

Direct `.fpascp` debugging requires `--source-root`. Every portable source path
must remain below that root and its BLAKE3 identity must match the image. Stale
or escaping sources are rejected before execution.

The debugger supports source and function breakpoints, pause, continue, step in/over/out,
stack frames, lexical scopes, variables, aggregate expansion, read-only
expression evaluation, conditional breakpoints, exact-hit conditions,
non-stopping source logpoints, selectable runtime-failure stops, stopped-state variable mutation, explicit complete
construction of enum, `Result`, and `Option` variants, forced return from a
selected live frame or task entry, replacement of an unconsumed retained task
result, selected-frame restart, output, and structured runtime failures. Execution is
bounded by `--timeout`, `--instruction-limit`, and `--output-limit`. Programs
may spawn retained and detached tasks. Attach, non-stop task execution, reverse
execution, and arbitrary instruction-pointer changes remain unsupported. Frame
restart reconstructs a selected live frame at its function entry; it is not a
general `goto`.

Task debugging is deterministic and all-stop. The debug session runs the main
program and spawned FPAS tasks on one host execution lane, while normal
non-debug execution keeps using the concurrent worker pool. Each stop freezes
all live tasks at complete bytecode instruction boundaries. Task IDs are stable
for the session, and current runnable, waiting, sleeping, failed, and completed
states are available through the JSONL task catalog or DAP threads.

Continue and pause apply to the whole session. Step in, over, and out target the
selected task; when it is waiting, the driver may run dependencies in stable
task-ID order until the selected task becomes runnable. A breakpoint, runtime
failure, pause, or resource limit reached by any task takes precedence and
identifies that task. Instruction, output, and resume-time limits cover the
combined work of the entire session. Completed and cancelled tasks remain
briefly visible for lifecycle reporting but have no fabricated inspectable
stack.

Evaluation is available only at a stable stop. It accepts FPAS literals,
visible names, parentheses, unary `-` and `not`, arithmetic, Boolean/bitwise,
shift, comparison and `in` operators, stored record/enum fields, and read-only
array, dictionary, or string indexes. It also accepts controlled calls,
readable properties, instance and static record methods, array/dictionary/record/Result/Option construction, fully qualified enum
constructors, record updates, and `try`. Names are ASCII
case-insensitive; the innermost parameter/local/capture wins, then globals.
Visible first-class function values and closures may be called when their
captures can be detached safely. Aggregate results can be expanded like
ordinary variables and expire on resume. `go`, newly entered closure syntax,
statements, declarations, and assignments inside `evaluate` remain rejected.

`setVariable`, DAP `setExpression`, JSONL `variable.set`, and JSONL
`expression.set` are separate stopped-state operations. Handle-based mutation
addresses a child previously returned by `variables`. Textual mutation starts
with one visible binding and accepts stored record fields, active enum payload
fields, wrapper `.value`, and array or dictionary indexes, for example
`Counter`, `Origin.X`, `choice.count`, `result.value`, `optional.value`,
`State.Items[Selected].Value`, or `Scores['blue']`. Root and field lookup is
ASCII case-insensitive. Omitting a frame deliberately searches globals only;
a local, parameter, or capture requires a current frame and is resolved with
normal lexical shadowing.

Both forms accept mutable source locals, mutable parameters, mutable globals,
and mutable captures backed by an existing closure cell. A visible
source-declared mutable local or global that has not yet received a value can
be assigned one complete replacement through the same operations. Complete mutable enum,
`Result`, and `Option` values can be replaced with one constructor expression
such as `Choice.Pair(1, 2)`, `Choice.Empty`, `Ok(3)`, `Error('failed')`,
`Some(4)`, or `None`. Enum constructors must be fully qualified `Type.Variant`
names; short variant names are rejected. Record fields, array elements,
existing dictionary values, named fields of the currently active data-carrying
enum variant, and the `value` child of `Result.Ok`, `Result.Error`, and
`Option.Some` below those roots can be replaced, including nested combinations
such as `result.value.items[1].name` or `optional.value`. A textual target may
also name an inactive single-payload variant explicitly and supply that
payload, for example `Optional.Some.value`, `Outcome.Error.value`, or
`Selected.Count.Value`. The variant name is required and matched
case-insensitively against executable metadata; an unqualified inactive field
never selects a variant. Fieldless variants and multi-field payloads still use
a complete constructor on the binding, or the explicit variant-construction
commands below. Index selectors and
the replacement are
ordinary debugger expressions and may use the same controlled detached calls
as `evaluate`. All selectors are evaluated once from left to right against the
unchanged pre-commit snapshot, followed by the replacement, under one shared
evaluation and call budget. Wrapper payloads accept only `.value`.
Handle-based `setVariable` does not advertise inactive variants as children.
A mutable function-typed target can be replaced by copying one visible binding
that already holds a compatible first-class function value, for example
`Current := Backup`, or by assigning one statically resolved executable
routine such as `Current := AddTwo`, `Current := Math.Transform`, or a named
nested routine whose captures are the recorded immutable values and existing
mutable cells in the exact selected live lexical-owner frame, for example
`Current := AddBase`, `Current := MakeAdder.AddBase`, or `Current := AddCell`.
An instance method can be bound to one evaluated record receiver snapshot, for
example `Current := Receiver.Add`. The record layout supplies the exact method
mapping retained by the compiler; the debugger does not infer a method from a
rendered type or function name. The method's receiver parameter and remaining
portable signature must match the exact runtime record layout and destination
function type. Receiver graphs containing cells, task handles, opaque handles,
or task-bound functions are rejected before mutation.
Copying a binding shares the exact existing function and capture storage and
does not reconstruct its environment. An already materialized task-bound
function can be copied only within its selected owner task, onto a mutable
local or parameter register in the same selected frame. Assigning a capturing nested routine constructs a
new function value from verified capture provenance and the selected owner
frame; it does not search older, peer-task, or similarly named frames.
Immutable captures clone values. `Cell` and `EnclosingCell` captures clone the
existing cell handles without reading their payloads. A constructed function
that captures a cell is task-bound to the selected stopped task: later
invocation on another task fails before callee entry, and `go` rejects it.
Every capture source must be initialized and visible in that frame.
Task-bound construction and copying write only a source-declared mutable function-typed
local or parameter register in that same frame; globals, capture-cell roots,
aggregate descendants, and Dynamic endpoints remain rejected for those values.
A simple name uses ordinary lexical lookup first;
the executable catalog is consulted only after that lookup reports an unknown
name. For `Receiver.Method`, the receiver expression is evaluated once first;
an identifier-only chain falls back to the executable catalog only when its
receiver name is unknown. Matching is
ASCII-case-insensitive, an unqualified short name is accepted only when exactly
one executable routine has that final component, and the stored value uses
canonical spelling. Nested routines are stored under their enclosing-routine
path, so `AddBase` and `MakeAdder.AddBase` identify the same unique nested
function. The routine signature is proven from portable parameter and
result metadata; anonymous closure syntax, non-method computed function
expressions, Dynamic endpoints, foreign-task or escaping task-bound copies,
and inactive-variant function payloads remain rejected.
A mutable task-typed target can be replaced by copying one visible binding
that already holds a compatible task handle, for example `Current := Pending`.
The debugger copies the exact runtime task ID and does not spawn, retain,
resume, cancel, consume, or poll the scheduler. Source and destination must
both be declared `task` types whose result types match structurally under the
ordinary evaluation depth and value budgets. The source is one simple binding
name after parentheses are removed; literals, calls, selectors, numeric IDs,
and rendered `<task N>` display text are rejected. Copied pending handles keep
the ordinary `Wait`/`WaitAll` lifetime. Consumed or otherwise invalid handles
are not revived and are not rejected early. Dynamic endpoints and complete
aggregates that merely contain task handles remain non-assignable.

The debugger validates the complete replacement against portable FPAS type
metadata before committing one live root. A failed parse, evaluation, call,
type check, path check, resource check, or storage access leaves the stopped
program and its variable references unchanged. A successful update rebuilds
aggregate roots with normal copy-on-write value semantics, preserves closure
cell identity, refreshes every stopped task snapshot, and expires previous
frames and variable references. Clients must request stack, scopes, and
variables again after success. Failed mutation preserves every old reference.

Dictionary structure mutation is exposed through explicit JSONL commands, DAP
custom requests, and three Functional Pascal VS Code commands. Insert requires
a missing key and appends the pair. Remove requires an existing key, removes
that pair, and returns the removed value. Key replacement requires an existing
old key and missing, different new key; it preserves the value and iteration
position. Each operation addresses a complete dictionary container through the
same bounded textual target form, validates key and value expressions against
portable `dict of K to V` metadata, and commits one mutable root atomically.
The operations support locals, mutable parameters, globals, closure captures,
nested aggregate paths, and stopped task frames. Success expires all inspection
references; every failure preserves both live state and existing references.

Sequence structure mutation uses the same atomic target and evaluation model.
Explicit array insertion accepts indexes from zero through the current length;
array removal accepts an existing zero-based element index and returns the
removed value. String character replacement addresses zero-based Unicode
scalar indexes, requires an expression producing exactly one scalar, and
returns the old and new characters. These operations are available through
JSONL, matching DAP custom requests, and three Functional Pascal VS Code
commands. An unchanged character is rejected without writing.

Immutable bindings, compiler-hidden storage,
evaluation-only results, synthetic function children such as `receiver` and
`capture[i]`, function captures,
and opaque hosted values are not
writable. Function values are writable by copying an already
materialized, visible function binding, or by assigning a
unique executable routine — including a named nested routine whose captures
are immutable values or existing mutable cells from the selected lexical-owner
frame — onto a structurally
compatible mutable function-typed path. Constructed cell-capturing functions
are task-bound to the selected task. They can be copied only within that owner
task and may be stored only in a mutable local or parameter register of the
selected owner frame. Task handles are writable by copying
one visible initialized binding whose declared task result type matches the
destination; the copy preserves the runtime ID and does not change scheduler
ownership. Whole aggregates that contain task handles, Dynamic endpoints, and
numeric or `<task N>` input remain rejected. Uninitialized mutable locals and
globals accept only a complete root value; they have no writable fields,
indexes, dictionary entries, or payload
descendants, and a qualified variant transition cannot synthesize that outer
storage. Seeded descendant initialization is a dedicated command: the client
supplies both a complete root initializer and the descendant replacement, and
the debugger prepares the value off to the side before committing the complete
root once. JSONL `storage.initialize`, DAP `fpas/initializeStorage`, and the
VS Code command **Debug: Initialize Empty Storage** address a textual target
with at least one selector. The initializer must produce the declared complete
root type; every field, in-range array element, existing dictionary key, and
active payload in the path must already exist in that seed. Evaluation order is
initializer, index selectors, then replacement, under one shared budget.
Identity-bearing values anywhere in the seed are rejected. If the root is
already initialized, the command fails and points the client to `expression.set`
/ `setExpression`. When the compiler retained the exact declaration store and
that store is still pending in the selected live frame, successful debugger
initialization suppresses that one store. This applies equally to complete-root
`setVariable` / `setExpression` and seeded `storage.initialize`; initializer
expressions are not re-evaluated and unrelated stores still execute. The
suppression identity is the verified function, live frame, and exact `Move` or
`StoreGlobal` instruction, never a source-name, line, or register scan. A write
to an old payload-child handle never selects a different
variant; after a successful write the previous child handles are unavailable
and clients must request variables again. Existing `setVariable` and
`setExpression` operations still cannot insert/remove entries or change keys;
clients use the explicit dictionary operations instead. Standard `setVariable`
and `setExpression` still cannot resize arrays or address string characters;
clients use the explicit sequence operations instead. Mutation cannot invoke a
property setter or otherwise change control flow. Executables without exact
initializer metadata retain the conservative behavior: the mutation succeeds,
but a later ordinary source store can overwrite it. Parameters and captures do
not have suppressible declaration stores and remain unavailable while
uninitialized.

Explicit variant construction is a dedicated stopped-state command, not
`setVariable` and not a Variables-tree child. JSONL `variant.describe` /
`variant.construct`, DAP `fpas/variantDescribe` / `fpas/variantConstruct`, and
the VS Code command **Debug: Construct Variant** address the same textual
mutable target as `expression.set`. Discovery returns canonical variant names
and declared fields from portable metadata without mutating live state or
expiring handles. Construction requires exactly one expression per declared
field, evaluates those expressions once in declaration order under one shared
budget, and commits one complete enum, `Result`, or `Option` value atomically.
Enum names are fully qualified (`Choice.Pair`); `Result` and `Option` use `Ok`,
`Error`, `Some`, and `None`. Matching is ASCII-case-insensitive; responses use
canonical spelling. An uninitialized mutable root may receive the complete
value; descendants still require existing outer storage. Function, task, and
capture-cell fields, omitted or extra fields, inactive Variables children, and
writes to old payload-child handles remain rejected. Existing constructor
assignment through `setVariable` / `setExpression` is unchanged.

Forced return is a dedicated stopped-state command, not an assignment and not
`stepOut`. JSONL `frame.return`, DAP `fpas/forceReturn`, and the VS Code
command **Debug: Force Return** accept a current-generation frame from the task
that caused the current non-failure stop. Depth zero completes the active
callee. A deeper selected frame is removed together with every younger frame.
Selecting the oldest program or task entry frame completes that entry instead.
A function requires one expression evaluated
in the selected frame under the same detached policy and limits as `evaluate`.
A procedure omits the expression and returns `unit`. Convention and portable
result type come from the selected function. The value is checked before any
live frame changes. Success releases the selected register window and all
younger windows, restores the selected frame's caller, writes the result only
to that caller's saved destination, remains all-stop, and refreshes every
stopped-task snapshot once. The result includes the number of unwound frames,
which is the selected depth plus one. It does not run remaining selected or
younger instructions, dispatch a `Return` opcode, or add to the VM instruction
count. Completing the program entry terminates the session and cancels its
remaining tasks without dispatching them. Completing a spawned task entry
publishes its retained result when applicable, emits its task-exit event, and
keeps the session stopped on another inspectable task. Failure leaves frames,
registers, the current stop, instruction count, and inspection handles
unchanged. At a runtime-error stop, the same command explicitly replaces the
exact failed callee or entry result after all normal type checks. The original
runtime diagnostic remains in the event history; a successful callee recovery
returns to an all-stop caller and can be continued, while entry recovery uses
the normal program/task completion rules. A stale or different retained failure
is rejected atomically. Waiting, sleeping, or peer tasks, missing result
metadata, and Dynamic, first-class function, task, capture-cell, or opaque
results are rejected. Returning an ordinary older frame into an entry caller is
allowed.

Completed retained task results have a separate stopped-state replacement
operation. JSONL `task.result.replace`, DAP `fpas/replaceTaskResult`, and the VS
Code command **Debug: Replace Completed Task Result** take the stable runtime
task ID, an optional current evaluation frame, and a replacement expression for
a function task. Procedure tasks omit the expression. The task must have
completed successfully, have a retained handle, and still expose an unconsumed
result. The expression is evaluated and checked against the task entry
function's portable result type before the scheduler result changes. Replacing
the same result repeatedly before `Wait` is allowed; the last replacement is
what the waiter consumes. Consumed, pending, failed, detached, unknown, or
unsupported result categories are rejected atomically. The command executes no
program instruction and invalidates only stopped variable snapshots. Ordinary
callee results whose frames have already been removed have no retained
completion identity and cannot be replaced.

JSONL `frame.restart`, DAP `restartFrame`, and the VS Code **Restart Frame**
action reconstruct one selected live frame of the stop-owning task. Current
parameter and capture values are kept, including exact cell handles. Locals and
temporaries are cleared, younger frames are discarded, and the instruction
pointer returns to the selected function entry. The command executes no
bytecode; repeated side effects occur only after an explicit continue. Peer,
waiting, sleeping, failed, and completed tasks are rejected atomically.

Arbitrary instruction-pointer changes, including DAP `goto` / `gotoTargets` and
JSONL `instruction.set`, are rejected. Existing bytecode verification proves
the original control-flow graph from function entry, not safe interior jumps.
Temporary registers are reused, so a same-function sequence-point destination
cannot be shown to preserve initialization, operand types, or lexical state.
Use stepping, a source breakpoint, or frame restart instead.

Every call runs in a separate detached sandbox. Arguments, receivers, globals,
aggregates, and closure cells are deep-cloned while preserving sharing and
cycles inside the clone. Writes performed by an otherwise accepted function or
procedure are discarded when evaluation ends. The debugger derives transitive
effects from verified bytecode and permits only deterministic computation and
sandbox-local writes. Console, file, process, environment, graph, time, random,
task, blocking, opaque-resource, and unresolved dynamic-call effects fail
before their host implementation is invoked. No debugger call emits program
output or creates a nested stop.

One evaluation is limited to 4,096 UTF-8 source bytes, depth 64, 1,024
operations, 16 aggregate traversals, and 65,536 rendered bytes. A log template
is limited to 16,384 bytes and 64 interpolations; cumulative debugger log
output is limited to 1,048,576 bytes per session.

Controlled calls additionally share limits of 64 calls, depth 32, 1,000,000
dispatched instructions, 65,536 detached values, and two seconds per
expression. Timeout, cancellation, forbidden effects, bad arity, unknown or
ambiguous targets, sandbox runtime failures, and resource exhaustion are
reported separately. DAP cancel and disconnect requests cooperatively stop an
active call at an instruction boundary, retain the original stop, and leave it
inspectable unless the session was disconnected.

A breakpoint condition must produce Boolean `true` to stop. A hit condition is
one positive decimal integer such as `3`, meaning exactly the third physical
hit. Log messages interpolate `{expression}` and escape braces as `{{` and
`}}`; valid logpoints never stop execution. Runtime condition errors stop
safely, while runtime log interpolation errors report a bounded diagnostic and
continue.

A function breakpoint names executable routine metadata rather than a source
line. Matching is ASCII case-insensitive. A canonical selector such as
`Math.Transform` binds that exact routine; a short selector such as
`Transform` binds every executable routine whose final name component matches,
in executable order. One logical breakpoint ID covers all of those exact
function identities. A missing selector or a matching routine without an entry
sequence point remains visible as unverified. Function breakpoints support the
same Boolean conditions and exact positive hit counts as source breakpoints;
log messages remain source-breakpoint behavior.

Runtime failures stop for inspection by default. A client may instead replace
the session filter with exact advertised diagnostic codes such as `F4001`, or
with the single filter `all`. An empty selection stops on no runtime failures.
A nonmatching failure is still reported as a structured diagnostic, then the
session terminates unsuccessfully without exposing a stopped state. Unknown,
reserved, duplicate, mixed `all`/code, or excessive selections are rejected
atomically.

Pause and execution-limit checks are cooperative at VM instruction boundaries.
A blocking host intrinsic already in progress cannot be interrupted; the pause
or limit is observed at the next source/instruction boundary after that call
returns.

VS Code-compatible editors use the contributed `fpas` debug type. A minimal
`launch.json` entry is:

```json
{
  "type": "fpas",
  "request": "launch",
  "name": "Debug Functional Pascal",
  "program": "${file}",
  "cwd": "${workspaceFolder}",
  "stopOnEntry": false
}
```

The Functional Pascal VS Code extension supplies its bundled source standard
library to the debugger automatically. Source and project targets using
`Std.*` units therefore do not need a machine-wide FPAS installation.
