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

The debugger supports source breakpoints, pause, continue, step in/over/out,
stack frames, lexical scopes, variables, aggregate expansion, read-only
expression evaluation, conditional breakpoints, exact-hit conditions,
non-stopping logpoints, stopped-state variable mutation, output, and structured
runtime failures. Execution is bounded by `--timeout`, `--instruction-limit`,
and `--output-limit`. Programs may spawn retained and detached tasks. Attach,
non-stop task execution, control-flow manipulation, and reverse execution
remain unsupported.

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
a complete constructor on the binding. Index selectors and
the replacement are
ordinary debugger expressions and may use the same controlled detached calls
as `evaluate`. All selectors are evaluated once from left to right against the
unchanged pre-commit snapshot, followed by the replacement, under one shared
evaluation and call budget. Wrapper payloads accept only `.value`.
Handle-based `setVariable` does not advertise inactive variants as children.

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
evaluation-only results, function captures,
task values, function values, and opaque hosted values are not
writable. Uninitialized mutable locals and globals accept only a complete root
value; they have no writable fields, indexes, dictionary entries, or payload
descendants, and a qualified variant transition cannot synthesize that outer
storage. A write to an old payload-child handle never selects a different
variant; after a successful write the previous child handles are unavailable
and clients must request variables again. Existing `setVariable` and
`setExpression` operations still cannot insert/remove entries or change keys;
clients use the explicit dictionary operations instead. Standard `setVariable`
and `setExpression` still cannot resize arrays or address string characters;
clients use the explicit sequence operations instead. Mutation cannot invoke a
property setter or change control flow. If execution later reaches the source
initializer, that store overwrites the debugger-provided value.

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
