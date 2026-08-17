# FPAS Debug Adapter Protocol contract

The adapter uses standard DAP `Content-Length` framing and delegates runtime
work to the same protocol-neutral session as JSONL V2.

## Capabilities and requests

The adapter advertises configuration-done, pause, source and function breakpoints,
conditional breakpoints, hit conditions, logpoints, evaluate-for-hover,
step-in/next/step-out, stack and variable pagination, delayed stack loading,
cancel, set-variable, set-expression, and terminate-on-disconnect for an owned
launch. It also advertises exception-breakpoint filters for `all` and every
allocated runtime diagnostic code. It explicitly advertises `supportsAttach: false`,
`supportsDataBreakpoints: true`,
`supportsDisassembleRequest: false`, `supportsReadMemoryRequest: false`, and
`supportsStepBack: false`.
It does not advertise completions, instruction
breakpoints, restart, reverse execution, replay, hot reload, non-stop execution, or raw
register access. It explicitly advertises
`supportsSingleThreadExecutionRequests: false` because every stop freezes all
FPAS tasks. Per-task holds use custom requests `fpas/pauseTask` and
`fpas/resumeTask` with a known `threadId`; they do not change continue or
pause into single-thread execution. It advertises `supportsRestartFrame: true` and
`supportsGotoTargetsRequest: false`.

Supported requests are `initialize`, `launch`, `setBreakpoints`,
`setFunctionBreakpoints`, `setExceptionBreakpoints`, `dataBreakpointInfo`,
`setDataBreakpoints`,
`configurationDone`, `threads`, `stackTrace`, `scopes`, `variables`,
`evaluate`, `setVariable`, `setExpression`, `fpas/dictionaryInsert`,
`fpas/dictionaryRemove`, `fpas/dictionaryReplaceKey`, `fpas/arrayInsert`,
`fpas/arrayRemove`, `fpas/stringReplaceCharacter`, `fpas/forceReturn`,
`restartFrame`, `fpas/replaceTaskResult`, `fpas/pauseTask`, `fpas/resumeTask`, `fpas/cancelTask`, `fpas/createTask`, `fpas/restartTask`, `fpas/input`,
`fpas/eof`, `fpas/cancelInput`, `fpas/variantDescribe`,
`fpas/variantConstruct`, `fpas/initializeStorage`, `fpas/locationDescribe`, `fpas/recordingDescribe`, `fpas/record`, `cancel`, `continue`,
`pause`, `next`, `stepIn`, `stepOut`, `source`, and `disconnect`. `goto` and
`gotoTargets` fail with `instruction_change_unsupported`. `attach`,
`stepBack`, `reverseContinue`, `disassemble`, `readMemory`, and `writeMemory`
fail as unsupported. Other unsupported requests fail explicitly.

`threads` maps main task `0` to DAP thread `1` and assigns stable positive DAP
IDs to spawned FPAS tasks. `stackTrace.threadId`, `next`, `stepIn`, and
`stepOut` select that task. Unknown or expired DAP thread IDs fail explicitly
instead of falling back to main. Continue and pause remain whole-session
operations even when a `threadId` or `singleThread` argument is supplied; a
continue response reports `allThreadsContinued: true`. Pause is cooperative at
VM instruction boundaries; an in-progress host intrinsic finishes before the
pause stop is reported. A paused task remains in
`threads` with a `[paused]` name suffix until `fpas/resumeTask`. DAP `threads` omits
completed and cancelled tasks after their thread-exit event; JSONL `tasks`
still lists those identities for lifecycle reporting.

`evaluate` accepts contexts `watch`, `repl`, `hover`, and `variables`. A
supplied `frameId` must belong to the current stop; omitting it deliberately
evaluates globals only. Successful responses include `result`, `type`,
`variablesReference`, `namedVariables`, and `indexedVariables`. Evaluated
aggregates expand through `variables`; all references expire when execution
resumes and are not data-breakpoint identities. `fpas/locationDescribe` names a
durable location from a current-stop `variablesReference` child. `dataBreakpointInfo`
returns a persistable `dataId` only for executable globals (`g:<index>`) with
`accessTypes: ["write"]`. Frame registers and capture cells return a null
`dataId`. `setDataBreakpoints` maps those IDs onto JSONL
`data_breakpoints.replace`. Optional `assign` on `setBreakpoints` and
`setDataBreakpoints` items forwards the same global identity and expression as
JSONL. VS Code uses the standard variable data-breakpoint
UI; the adapter does not add a second watchpoint command or an assign editor
command. `fpas/record` starts capturing all-stop events and queued `Read`/`ReadLn`
lines without resuming or enabling reverse playback. `fpas/recordingDescribe`
names versioned program identity, portable sources, whether capture is on, and
captured events. The accepted expression subset and limits are documented in
[Source debugger](debugger.md). Controlled calls execute asynchronously in a
detached sandbox so standard DAP `cancel` and `disconnect` requests can reach
an active evaluation. Cancellation returns an evaluation failure and preserves
the current stop; disconnect cancels, waits for bounded cleanup, and terminates
without an orphan worker.

`setVariable` accepts a current-stop `variablesReference`, the exact returned
child `name`, and an FPAS expression in `value`. It supports mutable source
locals, parameters, globals, closure cells, record fields, array elements,
existing dictionary values, named fields of the active data-carrying enum
variant, the `value` child of `Result.Ok`, `Result.Error`, and
`Option.Some`, and complete mutable enum, `Result`, and `Option` values.
It does not advertise inactive variants as virtual children. Visible
uninitialized mutable locals and globals accept one complete root
value. Complete-value replacements use constructor expressions such as
`Choice.Pair(1, 2)`, `Ok(3)`, `Error('failed')`, `Some(4)`, and `None`.
Fieldless and multi-field variants can also be built through
`fpas/variantDescribe` and `fpas/variantConstruct` without writing a
constructor expression.
Function-typed targets accept one visible binding that already holds a
compatible function value, or one statically resolved
non-capturing executable routine such as `AddTwo` or `Math.Transform`, or a
named nested routine whose captures are immutable values or existing mutable
cells in the selected lexical-owner frame. Constructed cell-capturing
functions are task-bound to the selected task and may be stored only in a
mutable local or parameter register of that owner frame. An already
materialized task-bound function may be copied only within that selected owner
task and frame; the copy preserves its exact function and cell handles. A
simple name uses lexical lookup first. `Receiver.Method` binds one evaluated
record snapshot through the compiler-retained record-method mapping after
exact receiver-layout and visible-signature validation. Receiver graphs with
live or opaque identities are rejected atomically. Anonymous closure syntax, escaping or
foreign-task task-bound copies, synthetic function children such as `receiver`
and `capture[i]`, and inactive-variant function payloads remain rejected.
Task-typed targets accept one visible binding that already holds a compatible
task handle. Standard `setVariable` / `setExpression` copy the exact runtime
ID; they do not add a DAP capability or custom request. Numeric IDs, `<task N>`
display text, Dynamic endpoints, and aggregates that contain task handles
remain rejected. Copied handles keep ordinary `Wait`/`WaitAll` lifetime rules.
The response contains the rendered `value`,
`type`, a fresh `variablesReference`, and exact named/indexed child counts.
Non-default `format` options are rejected because value-formatting negotiation
is not implemented. The adapter advertises both `supportsSetVariable: true`
and `supportsSetExpression: true`.

`setExpression` maps standard DAP `expression`, `value`, and optional
`frameId` fields to the shared textual mutation operation. Its target is one
visible binding followed by stored record fields, active enum payload fields,
wrapper `.value`, an explicit inactive single-payload variant suffix such as
`Some.value` or `Count.Value`, or array/existing-dictionary
indexes. Complete enum, `Result`, and `Option` targets also accept a constructor
expression as `value`. Function-typed targets accept one visible source binding
that already holds a compatible function value, one
statically resolved non-capturing executable routine, or a named nested routine
whose captures are immutable values or existing mutable cells in the selected
lexical-owner frame. Constructed cell-capturing functions are task-bound to
the selected task and may be stored only in a mutable local or parameter
register of that owner frame. Task-bound sources may be copied only within the
selected owner task and frame. Task-typed targets accept
one visible source binding that already holds a compatible task handle.
Uninitialized
mutable roots accept the complete binding name only. Omitting `frameId`
searches globals only. A supplied frame selects its exact FPAS task and
lexical scope; stale or foreign frames fail without falling back to main. The response has the same rendered value, type, fresh aggregate
reference, and exact child counts as `setVariable`. Non-default `format` is
rejected consistently.

The three FPAS custom requests provide dictionary structure mutation because
DAP has no standard equivalent. They use `frameId`, `target`, and `key`;
insertion additionally uses `value`, while key replacement uses `newKey`.
`target` selects a complete mutable dictionary using the same bounded target
grammar as `setExpression`. Insert requires a missing key, remove requires an
existing key, and replacement requires an existing old key and missing,
different new key. Successful bodies contain the committed dictionary as
`value`, `type`, `variablesReference`, `namedVariables`, and
`indexedVariables`. Remove adds `removed`; replacement adds `oldKey` and
`newKey`. The custom requests map directly to the shared JSONL operations and
do not define separate mutation behavior or capability flags.

The sequence custom requests use `frameId`, `target`, and an FPAS `index`
expression. `fpas/arrayInsert` and `fpas/stringReplaceCharacter` additionally
use `value`. Array insertion accepts zero through the current length; removal
requires an existing zero-based element index. String indexes count Unicode
scalars and `value` must evaluate to exactly one different scalar. Successful
bodies use the standard mutation fields and add `index`; removal adds
`removed`, and string replacement adds `oldCharacter` and `newCharacter`.
They map directly to the corresponding JSONL operations.

`fpas/forceReturn` is the DAP mapping of JSONL `frame.return`. Arguments are
`frameId` and optional `expression`. A successful body contains `value`,
`type`, `variablesReference`, `namedVariables`, `indexedVariables`,
`unwoundFrames`, `taskId`, and the fresh caller `frame`. `unwoundFrames` is the
selected depth plus one. The adapter does not advertise a DAP capability flag
for this custom request. After a successful forced return, clients that
initialized with `supportsInvalidatedEvent: true` receive one `invalidated`
event whose `areas` are `stacks` and `variables`. Failure emits no invalidation
and leaves the current stop unchanged.

`restartFrame` is the DAP mapping of JSONL `frame.restart`. Arguments are
`frameId`. A successful body contains `taskId` and `discardedFrames`. Clients
that initialized with `supportsInvalidatedEvent: true` receive one
`invalidated` event whose `areas` are `stacks` and `variables`. The adapter
advertises `supportsRestartFrame`.

`goto` and `gotoTargets` map onto JSONL `instruction.set` and always fail with
`instruction_change_unsupported` while stopped. The adapter advertises
`supportsGotoTargetsRequest: false`. Use `next`, `stepIn`, `stepOut`, a source
breakpoint, or `restartFrame`.

`fpas/replaceTaskResult` is the DAP mapping of JSONL `task.result.replace`.
Arguments are `taskId`, optional `frameId`, and optional `expression`. A
successful body contains `taskId`, `value`, `type`, `variablesReference`,
`namedVariables`, and `indexedVariables`. Clients that initialized with
`supportsInvalidatedEvent: true` receive one `invalidated` event whose `areas`
are `variables`. The adapter does not advertise a DAP capability flag for this
custom request.

`fpas/pauseTask` and `fpas/resumeTask` map a known `threadId` onto JSONL
`task.pause` and `task.resume`. A successful body contains `taskId` and
`paused`. Continue and pause remain whole-session; these requests only change
the hold flag. `fpas/cancelTask` maps a known `threadId` onto JSONL
`task.cancel`. A successful body contains `taskId` and `state`, and the adapter
emits `thread`/`exited`. `fpas/createTask` and `fpas/restartTask` map onto the
JSONL rejections. `fpas/input` maps onto JSONL `io.input` with required `text`
and returns `bytes` plus `sessionBytes`. `fpas/eof` and `fpas/cancelInput` map
onto `io.eof` and `io.cancel`. Protocol stdin is never debuggee stdin. The adapter does not advertise
`supportsSingleThreadExecutionRequests`. After the next `threads` request, a
held task's name includes `[paused]`; cancelled tasks are omitted from
`threads` while remaining in the JSONL catalog.

`fpas/variantDescribe` and `fpas/variantConstruct` map `frameId`, `target`,
`variant`, and `fields` onto JSONL `variant.describe` and `variant.construct`.
Describe returns `target`, `typeName`, and `variants` with `name` plus
`typeName` on each field. Construct returns the standard mutation fields plus
`variant`. The adapter does not advertise DAP capability flags for these custom
requests. After a successful construct, clients that initialized with
`supportsInvalidatedEvent: true` receive one `invalidated` event whose `areas`
are `variables`. Discovery and failures emit no invalidation.

`fpas/initializeStorage` maps `frameId`, `target`, `initializer`, and
`expression` onto JSONL `storage.initialize`. A successful body contains
`root`, `target`, `rootValue`, `value`, `type`, `variablesReference`,
`namedVariables`, and `indexedVariables`. The adapter does not advertise a DAP
capability flag for this custom request. After a successful request, clients
that initialized with `supportsInvalidatedEvent: true` receive one
`invalidated` event whose `areas` are `variables`. Failure emits no
invalidation and retains current handles.

After any successful value, dictionary, or sequence mutation, clients that initialized with
`supportsInvalidatedEvent: true` receive an `invalidated` event for the
`variables` area after the response. No invalidation event is sent after a
failure or to clients that did not negotiate it. Successful mutation expires
earlier frames, scopes, and variable references for every stopped task, so
clients must refetch them. Failure preserves the current stop and all
references.

Failures returned by the shared debugger core retain their machine-facing
JSONL details in the DAP `body.error` object: `code` is the stable error code,
`format` is the user-facing message, and `help` is the actionable hint. Local
DAP request-shape failures that never reach the core provide the standard
`format` and `showUser` fields only.

`setBreakpoints` forwards DAP `condition`, `hitCondition`, `logMessage`, and
optional `assign` unchanged to the shared breakpoint policy. A condition stops only on Boolean
`true`. A hit condition is one positive decimal `N` and matches exactly the Nth
physical hit. Log messages use `{expression}`, with `{{` and `}}` for literal
braces, and never cause a user-visible stop. After condition and hit tests, an
optional `assign` object `{ identity, expression }` writes one executable
global through the same mutation transaction as `setVariable`. Invalid
expressions, hit syntax, or templates return an unverified DAP breakpoint with
an actionable message. An invalid `assign` identity or unparsable replacement
fails the request without creating the breakpoint.

Runtime condition errors emit a debugger diagnostic and fail closed by
stopping. Runtime log interpolation errors emit a rate-limited stderr output
event and continue. When normal breakpoints and logpoints share a sequence
point, log output is emitted in request order before the combined stop.

`setFunctionBreakpoints` uses standard DAP replace-all semantics. Each `name`
is matched case-insensitively against executable metadata. Canonical names such
as `Math.Transform` bind exactly; a short name such as `Transform` binds every
routine with that final component while retaining one logical breakpoint ID.
The standard response reports verified or unverified state and an actionable
message. `condition` and positive-decimal `hitCondition` use the shared source
breakpoint policy. Function logpoints, assignments, and adapter-local name
inference are not implemented.

`setExceptionBreakpoints` accepts the advertised `all` filter by itself or
exact advertised codes such as `F4001`; an empty array selects no runtime
failure stops. The default `all` selection preserves inspectable exception
stops. For a nonmatching code the adapter emits diagnostic `output`, then
`exited` with code `1` and `terminated`, without a `stopped` event. Invalid,
reserved, duplicate, mixed, or excessive selections fail atomically.

## Launch and stepping

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

`program` accepts `.fpas`, a program project/workspace, or `.fpascp`.
Compiled images additionally require `sourceRoot`. The adapter emits standard
`initialized`, `thread`, `output`, `stopped`, `exited`, and `terminated`
events. Spawned tasks emit one ordered thread-start and thread-exit event.
Stopped events identify the responsible thread and report
`allThreadsStopped: true`. A selected runtime failure stops with reason
`exception`, remains inspectable, and terminates on the next continue or
disconnect. A filtered-out runtime failure follows the non-stopping failed-exit
sequence above.

A source breakpoint binds to the first reachable sequence point at or after
the requested line within the same declaration region. Unreachable lines stay
unverified. Multiple logical breakpoints may share a sequence point while
retaining independent IDs and counters. Continue ignores the current point
once to prevent a no-progress loop. `stepIn` stops at the next sequence point;
`next` does not enter deeper frames; `stepOut` stops after returning below the
starting depth. If the selected task waits, the deterministic debug scheduler
may progress its dependencies; a breakpoint or failure in any progressed task
wins the stop. Distinct columns on one line may identify distinct points.
