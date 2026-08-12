# FPAS Debug Adapter Protocol contract

The adapter uses standard DAP `Content-Length` framing and delegates runtime
work to the same protocol-neutral session as JSONL V2.

## Capabilities and requests

The adapter advertises configuration-done, pause, source breakpoints,
conditional breakpoints, hit conditions, logpoints, evaluate-for-hover,
step-in/next/step-out, stack and variable pagination, delayed stack loading,
cancel, set-variable, set-expression, and terminate-on-disconnect for an owned
launch. It does not advertise attach, completions, data/function/instruction
breakpoints, restart, reverse execution, hot reload, non-stop execution, or raw
register/disassembly access. It explicitly advertises
`supportsSingleThreadExecutionRequests: false` because every stop freezes all
FPAS tasks.

Supported requests are `initialize`, `launch`, `setBreakpoints`,
`configurationDone`, `threads`, `stackTrace`, `scopes`, `variables`,
`evaluate`, `setVariable`, `setExpression`, `fpas/dictionaryInsert`,
`fpas/dictionaryRemove`, `fpas/dictionaryReplaceKey`, `fpas/arrayInsert`,
`fpas/arrayRemove`, `fpas/stringReplaceCharacter`, `cancel`, `continue`,
`pause`, `next`, `stepIn`, `stepOut`, `source`, and `disconnect`. Unsupported
requests fail explicitly.

`threads` maps main task `0` to DAP thread `1` and assigns stable positive DAP
IDs to spawned FPAS tasks. `stackTrace.threadId`, `next`, `stepIn`, and
`stepOut` select that task. Unknown or expired DAP thread IDs fail explicitly
instead of falling back to main. Continue and pause remain whole-session
operations; a continue response reports `allThreadsContinued: true`.

`evaluate` accepts contexts `watch`, `repl`, `hover`, and `variables`. A
supplied `frameId` must belong to the current stop; omitting it deliberately
evaluates globals only. Successful responses include `result`, `type`,
`variablesReference`, `namedVariables`, and `indexedVariables`. Evaluated
aggregates expand through `variables`; all references expire when execution
resumes. The accepted expression subset and limits are documented in
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
expression as `value`. Uninitialized mutable roots accept the complete binding
name only. Omitting `frameId` searches globals only. A supplied frame selects its
exact FPAS task and lexical scope; stale or foreign frames fail without falling
back to main. The response has the same rendered value, type, fresh aggregate
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

After any successful value, dictionary, or sequence mutation, clients that initialized with
`supportsInvalidatedEvent: true` receive an `invalidated` event for the
`variables` area after the response. No invalidation event is sent after a
failure or to clients that did not negotiate it. Successful mutation expires
earlier frames, scopes, and variable references for every stopped task, so
clients must refetch them. Failure preserves the current stop and all
references.

`setBreakpoints` forwards DAP `condition`, `hitCondition`, and `logMessage`
unchanged to the shared breakpoint policy. A condition stops only on Boolean
`true`. A hit condition is one positive decimal `N` and matches exactly the Nth
physical hit. Log messages use `{expression}`, with `{{` and `}}` for literal
braces, and never cause a user-visible stop. Invalid expressions, hit syntax,
or templates return an unverified DAP breakpoint with an actionable message.

Runtime condition errors emit a debugger diagnostic and fail closed by
stopping. Runtime log interpolation errors emit a rate-limited stderr output
event and continue. When normal breakpoints and logpoints share a sequence
point, log output is emitted in request order before the combined stop.

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
`allThreadsStopped: true`. Runtime failures stop with reason `exception`,
remain inspectable, and terminate on the next continue or disconnect.

A source breakpoint binds to the first reachable sequence point at or after
the requested line within the same declaration region. Unreachable lines stay
unverified. Multiple logical breakpoints may share a sequence point while
retaining independent IDs and counters. Continue ignores the current point
once to prevent a no-progress loop. `stepIn` stops at the next sequence point;
`next` does not enter deeper frames; `stepOut` stops after returning below the
starting depth. If the selected task waits, the deterministic debug scheduler
may progress its dependencies; a breakpoint or failure in any progressed task
wins the stop. Distinct columns on one line may identify distinct points.
