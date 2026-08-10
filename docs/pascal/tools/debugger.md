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

The complete wire contracts are documented in [JSONL protocol V1](debugger-jsonl.md)
and [Debug Adapter Protocol V1](debugger-dap.md).

Direct `.fpascp` debugging requires `--source-root`. Every portable source path
must remain below that root and its BLAKE3 identity must match the image. Stale
or escaping sources are rejected before execution.

The debugger supports source breakpoints, pause, continue, step in/over/out,
stack frames, lexical scopes, variables, aggregate expansion, output, and
structured runtime failures. Execution is bounded by `--timeout`,
`--instruction-limit`, and `--output-limit`. Reachable task spawning is rejected
in V1. Evaluation, mutation, attach, conditional breakpoints, logpoints,
multiple task threads, and reverse execution are intentionally unsupported.

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
