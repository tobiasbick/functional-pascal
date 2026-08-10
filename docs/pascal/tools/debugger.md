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
non-stopping logpoints, output, and structured runtime failures. Execution is
bounded by `--timeout`, `--instruction-limit`, and `--output-limit`. Reachable
task spawning is rejected. Mutation, debugger-side calls, attach, multiple task
threads, and reverse execution remain unsupported.

Evaluation is available only at a stable stop. It accepts FPAS literals,
visible names, parentheses, unary `-` and `not`, arithmetic, Boolean/bitwise,
shift, comparison and `in` operators, stored record/enum fields, and read-only
array, dictionary, or string indexes. Names are ASCII case-insensitive; the
innermost parameter/local/capture wins, then globals. Calls, properties,
methods, constructors, closures, `go`, `try`, mutation, and aggregate creation
are rejected before frame values are read. Aggregate results can be expanded
like ordinary variables and expire on resume.

One evaluation is limited to 4,096 UTF-8 source bytes, depth 64, 1,024
operations, 16 aggregate traversals, and 65,536 rendered bytes. A log template
is limited to 16,384 bytes and 64 interpolations; cumulative debugger log
output is limited to 1,048,576 bytes per session.

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
