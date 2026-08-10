# FPAS Debug Adapter Protocol contract V1

The adapter uses standard DAP framing over stdin/stdout and delegates all
runtime behavior to the same debugger session used by the JSONL protocol.

## Advertised capabilities

V1 advertises:

- configuration done requests;
- pause;
- source breakpoints for `.fpas`;
- step into, next, and step out;
- stack-frame and variable pagination;
- delayed stack loading;
- terminate-on-disconnect for an adapter-owned launch.

V1 does not advertise:

- attach;
- conditional breakpoints, hit conditions, or logpoints;
- function, instruction, data, or exception breakpoints;
- evaluate, completions, or set-variable;
- restart, terminate request, goto, step back, reverse continue, or hot reload;
- multiple FPAS task threads;
- disassembly or raw register inspection.

Unsupported requests receive a failed response with an actionable message. They
are never accepted and ignored.

## Supported request mapping

| DAP request | Debugger operation |
|---|---|
| `initialize` | negotiate V1 capabilities |
| `launch` | load the target and create the session |
| `setBreakpoints` | replace breakpoints for one canonical source path |
| `configurationDone` | stop at entry or begin execution |
| `threads` | return logical thread `1` |
| `stackTrace` | return bounded logical FPAS frames |
| `scopes` | return parameters, locals, captures, and globals |
| `variables` | return bounded values or aggregate children |
| `continue` | continue to breakpoint, failure, pause, or exit |
| `pause` | cooperatively request a stop |
| `next` | step over |
| `stepIn` | step into |
| `stepOut` | step out |
| `source` | return verified source content when available |
| `disconnect` | terminate owned execution and release session state |

The adapter emits standard `initialized`, `output`, `stopped`, `exited`, and
`terminated` events. Runtime failures first emit `stopped` with
reason `exception`, remain inspectable, and terminate after the next continue
or disconnect.

## Launch configuration

```json
{
  "type": "fpas",
  "request": "launch",
  "name": "Debug Functional Pascal",
  "program": "${workspaceFolder}/app.fpasprj",
  "cwd": "${workspaceFolder}",
  "args": [],
  "stopOnEntry": false
}
```

`program` is required unless the extension can resolve one unambiguous selected
program project or workspace. `cwd`, when omitted, is the target's owning
directory. `args` is a JSON string array. Direct `.fpascp` launch additionally
requires `sourceRoot`.

## Breakpoint and stepping rules

- A source breakpoint binds to the first sequence point at or after the
  requested line within the same source declaration region.
- A line with no reachable sequence point remains unverified; it never binds to
  an unrelated later function.
- Multiple requested breakpoints that resolve to the same sequence point share
  execution behavior but retain their DAP breakpoint IDs.
- A breakpoint at the current stopped sequence point is ignored once when
  continuing, preventing an immediate no-progress stop.
- `stepIn` stops at the next sequence point regardless of call depth.
- `next` stops at the next sequence point whose call depth is not deeper than
  the starting depth.
- `stepOut` stops after returning below the starting depth.
- Instructions emitted for compiler prologues or diagnostic-only locations are
  not step targets.

For two source statements on one line, distinct columns may produce distinct
sequence points. A line-only breakpoint binds to the first. Repeated loop
execution may stop at the same sequence point again after at least one
instruction has executed.
