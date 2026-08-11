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
and `--output-limit`. Reachable task spawning is rejected. Attach, multiple task
threads, control-flow manipulation, and reverse execution remain unsupported.

Evaluation is available only at a stable stop. It accepts FPAS literals,
visible names, parentheses, unary `-` and `not`, arithmetic, Boolean/bitwise,
shift, comparison and `in` operators, stored record/enum fields, and read-only
array, dictionary, or string indexes. It also accepts controlled calls,
readable properties, instance and static record methods, array/dictionary/
record/Result/Option construction, record updates, and `try`. Names are ASCII
case-insensitive; the innermost parameter/local/capture wins, then globals.
Visible first-class function values and closures may be called when their
captures can be detached safely. Aggregate results can be expanded like
ordinary variables and expire on resume. `go`, newly entered closure syntax,
statements, declarations, and assignments inside `evaluate` remain rejected.

`setVariable` and JSONL `variable.set` are separate stopped-state operations.
They accept mutable source locals, mutable parameters, mutable globals, and
mutable captures backed by an existing closure cell. Record fields, array
elements, and existing dictionary values below those roots can be replaced,
including nested combinations. The replacement is an ordinary debugger
expression and may use the same controlled detached calls as `evaluate`.

The debugger validates the complete replacement against portable FPAS type
metadata before committing one live root. A failed parse, evaluation, call,
type check, path check, resource check, or storage access leaves the stopped
program and its variable references unchanged. A successful update rebuilds
aggregate roots with normal copy-on-write value semantics, preserves closure
cell identity, refreshes the variable snapshot, and expires previous variable
references. Clients must request scopes and variables again after success.

Immutable or uninitialized bindings, compiler-hidden storage, dictionary keys,
evaluation-only results, function captures, enum or `Result`/`Option` payload
descendants, task values, function values, and opaque hosted values are not
writable. Mutation cannot insert dictionary entries, resize arrays, change
control flow, or initialize a binding before normal execution does so.

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
