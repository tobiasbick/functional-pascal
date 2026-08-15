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
| `continue` | stopped | none | resumes all tasks |
| `step_into`, `step_over`, `step_out` | stopped | optional `task_id` | resumes toward the selected task's next step stop |
| `pause` | running | none | cooperative pause request |
| `tasks` | stopped | optional `start`, `count` | bounded task catalog |
| `stack` | stopped | optional `task_id`, `start`, `count` | bounded frames and resolved `task_id` |
| `scopes` | stopped | `frame_id` | lexical scopes |
| `variables` | stopped | `variables_reference`; optional `start`, `count` | values or aggregate children |
| `evaluate` | stopped | `expression`; optional `frame_id` | rendered detached value and child reference |
| `variable.set` | stopped | `variables_reference`, `name`, `expression` | committed rendered value and fresh child reference |
| `expression.set` | stopped | `target`, `expression`; optional `frame_id` | committed rendered value and fresh child reference |
| `dictionary.insert` | stopped | `target`, `key`, `expression`; optional `frame_id` | committed dictionary and fresh child reference |
| `dictionary.remove` | stopped | `target`, `key`; optional `frame_id` | committed dictionary, removed value, and fresh child reference |
| `dictionary.replace_key` | stopped | `target`, `key`, `new_key`; optional `frame_id` | committed dictionary, old/new keys, and fresh child reference |
| `array.insert` | stopped | `target`, `index`, `expression`; optional `frame_id` | committed array, affected index, and fresh child reference |
| `array.remove` | stopped | `target`, `index`; optional `frame_id` | committed array, affected index, removed value, and fresh child reference |
| `string.replace_character` | stopped | `target`, `index`, `expression`; optional `frame_id` | committed string, affected index, and old/new characters |
| `frame.return` | stopped | `frame_id`; optional `expression` | completed callee result, unwind count, and the fresh caller frame |
| `variant.describe` | stopped | `target`; optional `frame_id` | canonical variants and declared fields for one wrapper target |
| `variant.construct` | stopped | `target`, `variant`, `fields`; optional `frame_id` | committed wrapper value, canonical variant, and fresh child reference |
| `storage.initialize` | stopped | `target`, `initializer`, `expression`; optional `frame_id` | committed descendant and complete-root summaries |
| `disconnect` | non-terminal | optional `terminate` | cleanup confirmation |

An omitted evaluation `frame_id` exposes globals only. A supplied frame and all
variable references belong to the current stop and expire on resume. Evaluation
returns `result`, `type_name`, `variables_reference`, `named_variables`, and
`indexed_variables`.

The task catalog returns stable `task_id`, `name`, `state`, and `inspectable`
fields. States are `runnable`, `running`, `waiting`, `sleeping`, `completed`,
`failed`, or `cancelled`. Omitted task selection resolves to the task that
caused the current stop, or main task `0` at entry. Frame and variable IDs are
qualified by their task snapshot and cannot alias across tasks. Completed or
cancelled tasks are catalog-visible for lifecycle reporting but reject stack
inspection.

`variable.set` addresses one child returned by `variables`. The reference and
child name must belong to the current stop. Mutable locals, parameters,
globals, closure cells, record fields, array elements, existing dictionary
values, named fields of the active data-carrying enum variant, and the `value`
child of `Result.Ok`, `Result.Error`, and `Option.Some` are supported.
Visible uninitialized mutable locals and globals accept one complete root
value through the same request; descendant selectors on empty storage are
rejected. `variable.set` does not expose inactive variants as children.
Replacement expressions use the same parser, detached
controlled-call policy, and resource limits as `evaluate`. Complete mutable enum,
`Result`, and `Option` values accept constructor expressions such as
`Choice.Pair(1, 2)`, `Choice.Empty`, `Ok(3)`, `Error('failed')`, `Some(4)`, and
`None`. Function-typed targets accept one visible binding that already holds a
compatible non-task-bound function value, for example `Backup`, one unique
non-capturing executable routine such as `AddTwo`, or a named nested routine
whose direct captures are immutable initialized values in the selected
lexical-owner frame, for example `AddBase` or `MakeAdder.AddBase`. A successful
result has the same five rendered fields as `evaluate`, refreshes inspection
state, and expires all earlier variable references. Any failure is atomic and
leaves the old references usable.

`expression.set` addresses a target without first requesting variable handles:

```json
{"type":"request","id":8,"command":"expression.set","arguments":{"frame_id":4294967296,"target":"State.Items[Selected].Value","expression":"Counter + 1"}}
{"type":"response","request_id":8,"command":"expression.set","success":true,"body":{"result":"42","type_name":"integer","variables_reference":0,"named_variables":0,"indexed_variables":0}}
```

The bounded target grammar is one visible name followed only by stored-field
selectors (`.Field`) and index selectors (`[expression]`). Field selectors
address record fields, active enum payload fields, wrapper `.value`, and an
explicit inactive single-payload variant suffix such as `Some.value`,
`Ok.value`, `Error.value`, or `Count.Value`. Parenthesized or
computed bases, calls as the root/path, properties, assignments, declarations,
and statements are rejected. A supplied current `frame_id` selects that
frame's task and lexical scope. An omitted frame searches globals only and
never falls back to the selected or main frame. Array indexes must be in range;
dictionary keys must already exist. An unqualified inactive field does not
select a variant. Fieldless and multi-field variants still require a complete
constructor on the binding or `variant.construct`. An uninitialized mutable local or global accepts
only the complete root name; field, index, and variant-transition selectors on
empty storage fail with `variable_path_unsupported`. Use `storage.initialize`
when a descendant below empty storage must be seeded from an explicit complete
root. Text indexes and aggregate structure changes are unsupported by
`expression.set`; use the explicit dictionary and sequence commands below.
Complete enum, `Result`, and `Option` values can also be replaced by assigning
a constructor expression to the complete target. A function-typed target
accepts one visible source binding that already holds a compatible
non-task-bound function value, or one statically resolved non-capturing
executable routine such as `AddTwo` or `Math.Transform`, or a named nested
routine whose direct captures are immutable initialized values in the selected
lexical-owner frame. A simple name uses
lexical lookup first and falls back to the executable catalog only after an
unknown name. Qualified identifier chains are catalog names only. Anonymous
closure syntax, mutable `Cell` or `EnclosingCell` captures, bound methods,
computed expressions, and
inactive-variant function payloads remain rejected. A task-typed target accepts
one visible source binding that already holds a compatible task handle, for
example `Current := Pending`. The request copies the exact runtime ID through
the standard `variable.set` / `expression.set` commands; it does not add a
capability flag or custom command. Numeric IDs, `<task N>` display text, calls,
Dynamic endpoints, and complete aggregates that contain task handles remain
rejected. Copied handles keep ordinary `Wait`/`WaitAll` lifetime rules. A write to an old
payload-child handle never selects a different variant.

Selectors run once from left to right and the replacement runs last, all
against the unchanged stopped snapshot and under one expression/call budget.
Only after target resolution, evaluation, portable type validation, and live
storage checks succeed does the debugger commit one root. Success refreshes all
task snapshots and expires old frame and variable IDs. Any failure performs no
write and preserves those IDs. The result uses the same five rendered fields
as `evaluate` and `variable.set`.

Dictionary structure commands target the dictionary container rather than one
entry value:

```json
{"type":"request","id":20,"command":"dictionary.insert","arguments":{"frame_id":4294967296,"target":"State.Scores","key":"'Ada'","expression":"42"}}
{"type":"request","id":21,"command":"dictionary.remove","arguments":{"frame_id":8589934592,"target":"State.Scores","key":"'Ada'"}}
{"type":"request","id":22,"command":"dictionary.replace_key","arguments":{"frame_id":12884901888,"target":"State.Scores","key":"'Grace'","new_key":"'Hopper'"}}
```

`target` uses the same bounded root/selector grammar as `expression.set`, but
must resolve to a mutable `dict of K to V`. Insert evaluates and validates a
missing `key` and `expression`, then appends the pair. Remove requires an
existing key and adds `removed` to the normal committed dictionary result. Key
replacement requires an existing old key plus a missing, different `new_key`,
preserves the value and pair position, and adds `old_key` and `new_key` to the
result. All selectors and operation expressions run left-to-right against one
unchanged snapshot and one shared budget before the single-root commit.

Success returns the committed dictionary in the normal five rendered fields,
refreshes every stopped task snapshot, and expires old IDs. Parse, evaluation,
type, target, collision, missing-key, no-op, limit, cancellation, and storage
failures write nothing and preserve old IDs. `initialize` advertises
`dictionary_insert`, `dictionary_remove`, and `dictionary_replace_key`.

Sequence commands target a complete array or string container:

```json
{"type":"request","id":23,"command":"array.insert","arguments":{"frame_id":17179869184,"target":"State.Items","index":"Selected","expression":"42"}}
{"type":"request","id":24,"command":"array.remove","arguments":{"frame_id":21474836480,"target":"State.Items","index":"0"}}
{"type":"request","id":25,"command":"string.replace_character","arguments":{"frame_id":25769803776,"target":"State.Label","index":"1","expression":"'é'"}}
```

Array insertion permits `0..=length`; removal permits `0..<length`. String
indexes count Unicode scalars, not UTF-8 bytes, and the replacement expression
must produce a one-scalar string different from the current character. Success
uses the normal five rendered fields and adds `index`; removal also adds
`removed`, while string replacement adds `old_character` and `new_character`.
All inputs run once under one shared budget before an atomic commit. Initialize
advertises `array_insert`, `array_remove`, and `string_replace_character`.

`frame.return` completes a selected ordinary callee without resuming the
program:

```json
{"type":"request","id":17,"command":"frame.return","arguments":{"frame_id":4294967297,"expression":"42"}}
{"type":"response","request_id":17,"command":"frame.return","success":true,"body":{"task_id":0,"result":"42","type_name":"integer","variables_reference":0,"named_variables":0,"indexed_variables":0,"unwound_frames":1,"frame":{"frame_id":8589934593,"name":"main","depth":0}}}
```

`frame_id` must be a current-generation frame for the task that caused the
current non-failure stop, and that frame must have a saved caller. Depth zero
unwinds one frame; a deeper selected frame unwinds that frame plus every
younger frame. Functions require `expression` evaluated in the selected frame;
procedures omit it. Convention and portable result type come from the selected
function. Success returns the rendered result fields, `unwound_frames` (selected
depth plus one), and the fresh caller `frame`, remains stopped, and refreshes
every stopped-task snapshot. Failure is atomic. Initialize advertises
`frame_return`. Procedures render as `()` with `type_name` `unit`.

`variant.describe` returns the constructible variants for one textual mutable
enum, `Result`, or `Option` target without changing live state:

```json
{"type":"request","id":20,"command":"variant.describe","arguments":{"frame_id":3,"target":"Selected"}}
{"type":"response","request_id":20,"command":"variant.describe","success":true,"body":{"target":"Selected","type_name":"Choice","variants":[{"name":"Choice.Empty","fields":[]},{"name":"Choice.Pair","fields":[{"name":"Left","type_name":"Integer"},{"name":"Right","type_name":"Integer"}]}]}}
```

`variant.construct` builds one complete variant and commits it atomically:

```json
{"type":"request","id":21,"command":"variant.construct","arguments":{"frame_id":3,"target":"Selected","variant":"Choice.Pair","fields":{"Left":"1","Right":"2"}}}
```

`fields` maps canonical or ASCII-case-insensitive field names to debugger
expression strings. A fieldless variant requires `{}`. Expressions are parsed
before any evaluation, then evaluated once in declaration order under one
shared budget. Success returns the ordinary five rendered fields plus canonical
`variant`. Discovery does not expire handles; successful construction does.
Initialize advertises `variant_describe` and `variant_construct`.

`storage.initialize` seeds one descendant below an empty mutable local or
global. `target` must include at least one selector. `initializer` is a normal
debugger expression that produces the declared complete root; `expression`
replaces the selected descendant. The debugger evaluates initializer, index
selectors, and replacement once under one shared budget, resolves the path
against the detached seed, and commits the rebuilt complete root atomically:

```json
{"type":"request","id":30,"command":"storage.initialize","arguments":{"frame_id":3,"target":"State.Items[0].Count","initializer":"MakeInitialState()","expression":"42"}}
{"type":"response","request_id":30,"command":"storage.initialize","success":true,"body":{"root":"State","target":"State.Items[0].Count","root_value":"Holder {...}","value":"42","type":"integer","variables_reference":0,"named_variables":0,"indexed_variables":0}}
```

The request accepts only `frame_id`, `target`, `initializer`, and `expression`.
The returned `target` substitutes evaluated indexes; string keys use escaped
FPAS string-literal syntax such as `Scores['it''s']`.
An already initialized root fails with `storage_already_initialized` and does
not become ordinary mutation. Failure leaves empty storage and inspection
handles unchanged. A later source initializer still overwrites the debugger
value. Initialize advertises `storage_initialize`.

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

Events are `initialized`, `breakpoint`, `output`, `task`, `stopped`,
`runtime_error`, `terminated`, and fatal `protocol_error`. Task events report
`started` or `exited` with a stable `task_id`. Every stopped event reports the
responsible `task_id` and `all_tasks_stopped: true`; runtime errors also carry
the responsible task. A stopped breakpoint event includes both the first
`breakpoint_id` and ordered `breakpoint_ids` for all logical breakpoints at
that sequence point.

V2 advertises source breakpoints, pause/continue/steps, pagination, inspection,
aggregate expansion, structured output, evaluation, controlled calls,
set-variable, set-expression, all three dictionary structure operations, all
three sequence structure operations, forced return, variant describe and
construct, empty-storage initialization, conditional breakpoints, hit
conditions, and logpoints. Attach, non-stop execution and reverse execution
remain false;
`task_threads` is true. `frame_return`, `variant_describe`,
`variant_construct`, and `storage_initialize` are true.

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
`variable_target_unknown`, `variable_target_expired`, `variable_not_mutable`,
`variable_path_unsupported`, `variable_uninitialized`, `variable_value_type`,
`variable_unavailable`, `dictionary_key_exists`, `dictionary_key_missing`,
`dictionary_key_unchanged`, `sequence_index_out_of_bounds`,
`string_character_required`, `string_character_unchanged`,
`frame_return_unsupported`, `frame_return_value_required`,
`frame_return_value_unexpected`, `frame_return_type`, `variant_unknown`,
`variant_field_set`, `storage_already_initialized`, `unknown_task`,
`timeout`, `instruction_limit`, and `output_limit`. Parse/validation failures
also include a stable code, UTF-8 byte offset and length, message, and help.
Textual target failures use `expression_target_parse` or
`expression_target_unsupported` before runtime target errors are considered.
