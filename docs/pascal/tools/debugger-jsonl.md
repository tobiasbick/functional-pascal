# FPAS debugger JSONL protocol V1

The external debugger protocol is UTF-8 JSON Lines. Each physical line is one
complete JSON object. Protocol stdout contains no banners, progress text, ANSI
sequences, or raw program output.

## Invocation

```text
fpas debug <target> --protocol jsonl
fpas debug <target> --protocol jsonl --commands <path> --report jsonl
```

`<target>` accepts the same source, program-project, and workspace forms as
`fpas run`. Direct `.fpascp` input additionally requires `--source-root` and
verified source identities. Program arguments follow `--`.

Live mode reads requests from stdin and writes responses and events to stdout.
Script mode reads the same requests from `--commands`, reaches a terminal state,
and exits without prompting.

## Envelopes

Request:

```json
{"type":"request","id":1,"command":"initialize","arguments":{}}
```

Successful response:

```json
{"type":"response","request_id":1,"command":"initialize","success":true,"body":{"protocol":"fpas-debug-jsonl","version":1}}
```

Failed response:

```json
{"type":"response","request_id":1,"command":"continue","success":false,"error":{"code":"invalid_state","message":"The session is not stopped.","help":"Wait for a stopped event before continuing execution."}}
```

Event:

```json
{"type":"event","event":"stopped","body":{"reason":"breakpoint","thread_id":1,"location":{"source":"src/main.fpas","line":12,"column":3}}}
```

Rules:

- Request IDs are positive JSON integers and are echoed unchanged.
- Unknown top-level fields are ignored for forward compatibility.
- Missing required fields, duplicate request IDs, invalid UTF-8, malformed JSON,
  and non-object lines produce stable errors when a request ID can be recovered;
  otherwise the session emits a protocol error event and terminates.
- Protocol version is negotiated by `initialize`. V1 accepts only version `1`.
- A response precedes events caused by the accepted request, except fatal
  transport errors for which no response can be written.

## Commands

| Command | Allowed state | Required arguments | Result |
|---|---|---|---|
| `initialize` | created | optional `version` (default `1`) | protocol version, capabilities, limits |
| `breakpoint.set` | initialized or stopped | `source`, `line`; optional `column` | stable breakpoint ID and verified location |
| `breakpoint.clear` | initialized or stopped | `breakpoint_id` | removal confirmation |
| `launch` | initialized | optional `stop_on_entry` | accepts launch; later emits stopped/terminated |
| `continue` | stopped | none | accepts execution; later emits stopped/terminated |
| `pause` | running | none | requests cooperative pause |
| `step_into` | stopped | none | accepts execution; later emits stopped/terminated |
| `step_over` | stopped | none | accepts execution; later emits stopped/terminated |
| `step_out` | stopped | none | accepts execution; later emits stopped/terminated |
| `stack` | stopped | optional `start`, `count` | bounded frames and total frame count |
| `scopes` | stopped | `frame_id` | parameters, locals, captures, globals references |
| `variables` | stopped | `variables_reference`; optional `start`, `count` | bounded variables and child counts |
| `disconnect` | any non-terminal state | optional `terminate` (default `true`) | cleanup confirmation |

Read-only `stack`, `scopes`, and `variables` requests are deterministic and safe
to retry while the session remains at the same stop. Continuing invalidates all
frame and variable references.

## Events

| Event | Body |
|---|---|
| `initialized` | empty object |
| `breakpoint` | breakpoint ID, verified flag, requested and bound locations, optional message |
| `output` | `category = stdout`, text, monotonically increasing output sequence |
| `stopped` | reason, thread ID `1`, location, instruction, call depth, optional breakpoint ID |
| `runtime_error` | structured FPAS diagnostic and location |
| `terminated` | exit reason, exit code, instruction count |
| `protocol_error` | stable error object when no request response is possible |

V1 stop reasons are `entry`, `breakpoint`, `pause`, `step`, and
`runtime_error`. V1 has one logical thread with ID `1`.

## Capabilities

`initialize` reports booleans for source breakpoints, pause, continue, step
into, step over, step out, stack pagination, scope inspection, variable
pagination, aggregate expansion, and structured output. It explicitly reports
false for attach, task threads, evaluate, set-variable, conditional
breakpoints, hit conditions, logpoints, data breakpoints, function breakpoints,
restart, reverse execution, and hot reload.

## Default limits

| Limit | Default |
|---|---:|
| Stack frames per response | 256 |
| Variables per response | 256 |
| Value nesting depth | 16 |
| String characters | 4,096 |
| Retained variable handles | 16,384 |
| Rendered bytes per variables response | 1,048,576 |
| Captured program output bytes | 1,048,576 |
| Instructions per session | 100,000,000 |
| Resume timeout | 300 seconds |

The CLI exposes `--timeout`, `--instruction-limit`, and `--output-limit` for execution limits. Inspection limits are fixed in V1. Truncation is explicit in response metadata.

## Stable V1 error codes

```text
invalid_request
invalid_state
unsupported_protocol_version
unsupported_capability
unknown_breakpoint
unknown_frame
unknown_variables_reference
limit_exceeded
tasks_unsupported
timeout
instruction_limit
output_limit
```
