# FPAS debugger JSONL protocol V2

The external debugger protocol is UTF-8 JSON Lines: one complete JSON object
per physical line. Protocol stdout contains only responses and events; program
and logpoint output is carried by structured `output` events.

## Invocation and envelopes

```text
fpas debug <target> --protocol jsonl
fpas debug <target> --protocol jsonl --commands <path> --report jsonl
```

```json
{"type":"request","id":1,"command":"initialize","arguments":{"version":2}}
{"type":"response","request_id":1,"command":"initialize","success":true,"body":{"protocol":"fpas-debug-jsonl","version":2}}
```

Request IDs are unique positive integers. Malformed JSON, non-object records,
invalid envelopes, duplicate IDs, and missing arguments produce stable errors.
`initialize` accepts only version `2`; V1 is obsolete and is rejected without a
compatibility mode. A response precedes events caused by that request.

## Commands

| Command | State | Arguments | Result |
|---|---|---|---|
| `initialize` | created | optional `version` (default `2`) | capabilities and limits |
| `breakpoint.set` | initialized/stopped | `source`, `line`; optional `column`, `condition`, `hit_condition`, `log_message` | logical breakpoint and verification |
| `breakpoint.clear` | initialized/stopped | `breakpoint_id` | removal confirmation |
| `launch` | initialized | optional `stop_on_entry` | starts or stops at entry |
| `continue`, `step_into`, `step_over`, `step_out` | stopped | none | resumes execution |
| `pause` | running | none | cooperative pause request |
| `stack` | stopped | optional `start`, `count` | bounded frames |
| `scopes` | stopped | `frame_id` | lexical scopes |
| `variables` | stopped | `variables_reference`; optional `start`, `count` | values or aggregate children |
| `evaluate` | stopped | `expression`; optional `frame_id` | rendered detached value and child reference |
| `disconnect` | non-terminal | optional `terminate` | cleanup confirmation |

An omitted evaluation `frame_id` exposes globals only. A supplied frame and all
variable references belong to the current stop and expire on resume. Evaluation
returns `result`, `type_name`, `variables_reference`, `named_variables`, and
`indexed_variables`.

`evaluate` may call exact executable routines, record methods and readable
properties, visible first-class functions, and deterministic `Std.*`
intrinsics. Calls use a detached copy of globals, arguments, receivers,
captures, cells, and aggregates, so accepted writes never change the stopped
program. Host I/O, nondeterminism, blocking, task operations, opaque resources,
and unknown dynamic effects are denied. The accepted expression forms and
effect policy are documented in [Source debugger](debugger.md).

Conditions and log expressions use the same detached subset documented in
[Source debugger](debugger.md). Invalid syntax or unsupported constructs make
the breakpoint unverified. Hit conditions accept only a positive decimal `N`
and match exactly the Nth physical hit. Each logical breakpoint has an
independent saturating counter, including breakpoints sharing one sequence
point. Policy order is condition, hit test, then log-or-stop. Log templates use
`{expression}`, `{{`, and `}}`.

## Events and capabilities

Events are `initialized`, `breakpoint`, `output`, `stopped`, `runtime_error`,
`terminated`, and fatal `protocol_error`. A stopped breakpoint event includes
both the first `breakpoint_id` and ordered `breakpoint_ids` for all logical
breakpoints at that sequence point.

V2 advertises source breakpoints, pause/continue/steps, pagination, inspection,
aggregate expansion, structured output, evaluation, controlled calls,
conditional breakpoints,
hit conditions, and logpoints. Attach, task threads, set-variable, and reverse
execution remain false.

## Default limits

| Limit | Default |
|---|---:|
| Frames / variables per response | 256 / 256 |
| Value depth / retained handles | 16 / 16,384 |
| String characters / rendered variables bytes | 4,096 / 1,048,576 |
| Expression bytes / depth | 4,096 / 64 |
| Expression operations / traversals | 1,024 / 16 |
| Rendered evaluation bytes | 65,536 |
| Calls / nested call depth | 64 / 32 |
| Call instructions / detached values | 1,000,000 / 65,536 |
| Call timeout | 2 seconds |
| Log template bytes / interpolations | 16,384 / 64 |
| Cumulative log output bytes | 1,048,576 |
| Captured program output bytes | 1,048,576 |
| Instructions / resume timeout | 100,000,000 / 300 seconds |

Stable errors include `invalid_request`, `invalid_state`,
`unsupported_protocol_version`, `unsupported_capability`, `unknown_breakpoint`,
`unknown_frame`, `unknown_variables_reference`, `unknown_name`,
`uninitialized_value`, `evaluation_type`, `evaluation_domain`,
`evaluation_limit`, `unavailable_value`, `call_target_unknown`,
`call_ambiguous`, `call_arity`, `call_effect_forbidden`, `call_limit`,
`call_timeout`, `call_cancelled`, `call_runtime`, `limit_exceeded`,
`tasks_unsupported`,
`timeout`, `instruction_limit`, and `output_limit`. Parse/validation failures
also include a stable code, UTF-8 byte offset and length, message, and help.
