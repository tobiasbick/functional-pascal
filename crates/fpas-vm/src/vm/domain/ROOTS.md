# Domain root inventory

`fpas_bytecode::TraceRoots` enumerates direct edges. The shared value walker follows
nested aggregates, callable captures/receivers, cells, reference cells and dictionary
path keys. An unavailable provider returns an error; a partial view never authorizes
collection. The [execution protocol](EXECUTION.md) supplies execution exclusion, safe points,
request servicing and explicit native-wait publication. The
[local collector](COLLECTION.md) uses this complete root view for scheduled and
explicit maintenance requests. Running thresholds do not suppress root-release
requests in quiescent or retired domains.

| Holder | Root treatment and lifetime |
| --- | --- |
| `Worker.registers` | Every initialized register, including saved caller windows; uninitialized capacity is excluded |
| `TaskState` registers, completion, suspension, supervision and callback continuations | Same `TraceRoots` interface while saved; domain identity moves with the state |
| Shared session globals | `GlobalRoots` registered once by `HostValues` or `DebugTaskRuntime` in the original domain; synchronized reads report contention |
| `Worker.local_globals` | Enumerated directly by the owning worker |
| Worker pending completion and `OwnerCleanup.outcome` | Successful values remain execution roots until completion or cleanup commits |
| Channel send suspension and timeout | Retained send values are enumerated; task-transfer validation additionally rejects local graphs at publication |
| Supervised retry input | `SupervisedTask.function`, including captures and receiver, is enumerated throughout retries |
| Synchronous callback caller | Native root registration captures all caller execution edges before entering another worker |
| Cached callback worker | Successful cache retains register capacity only; faulted cache retains live frames for existing failure inspection and is recursively enumerated |
| Main and scheduled higher-order callbacks | Caller/callback frames and the continuation's pending result and complete operation state are execution roots; native registrations are unnecessary for individual elements |
| Native higher-order operation | One synchronized provider retains input cursors, partial arrays/dictionaries, reduction accumulators, cached sort keys and pending results; per-call arguments are registered before re-entry |
| Native callback result handoff | Native state accepts the result before another FPAS call; A temporary result registration protects the final return value before releasing execution access |
| Unused and claimed Select cases | Each case owns a registration for callback and send payload. Claim moves the registration intact; close, owner/attempt cleanup, cancellation and shutdown release it |
| Active Select callback | Callback invocation publishes caller and arguments; saved-task execution stores its callback continuation |
| Scheduler queues and timers | `ScheduledTask` enumerates the original direct or observed `TaskState`; no copied shadow registers |
| Failure catalog | Saved state is traced under its original owner mutex. The saved task relinquishes its catalog back-reference. Active workers remain execution owners; templates have no live registers |
| Failure responses, global capture and public reports | `ValueSnapshot` trees contain detached strings/scalars/metadata; no cell, function, reference or host resource owner survives capture |
| Debug task driver | Its workers and pending main result implement `TraceRoots`; published child results retain task-transfer checks |
| Inspection frames and globals | Bounded captured values and raw capture-source bindings have registered roots; writable closure-cell origins retain the cell even when the displayed payload is scalar |
| Inspection handles and prepared results | Registrations follow the exact retained child values and writable origins; generation invalidation releases handles |
| Debugger evaluation sandbox | Separate domain; registered construction map, temporary references/results and synchronized globals. Error drops all construction registrations |
| Evaluation handoff and mutation preparation | Successful output retains reachable sandbox cells plus bounded source inspection roots in an operation-scoped arena; outer API exit releases them on success and error |
| Read-only evaluation output | Local graphs detach for inspection in a fresh registered domain; code provenance survives. Retained children carry this domain after sandbox destruction. Transferable outputs need no cell copy |
| Debugger data-breakpoint comparison values | Registered in the original global domain; replacement publishes the new registration before dropping the old one |
| Live trace aliases | `LocalTraceState` enumerates inherited and parameter cell/reference aliases |
| Trace histories and frozen parameter leaves | Only primitive/string or fieldless enum observations; no enclosing aggregate or cell is retained |
| `Execution.value` and `DebugTermination.value` | Sealed `HostValue` registrations preserve domain and code identity without retaining worker/session/hosted state; independent clones own independent registrations |
| Host ingress batch | Source-domain guarded validation and copying apply Q41 before target execution; no raw local-value compatibility export |
| File lifecycle callback file value | Opaque resource identity only; callback ingress and session globals follow the same host boundary |
| Channel queue, pending rendezvous send and retained task results | Task-transfer validation excludes cells, task-bound callables and escaping references; these are not roots of a local cell graph |
| HTTP handle registry | Task-transfer validation covers retained response values; resource-shaped records contain identities, not hidden local graph owners |
| Dispatch results, restart/return/mutation preparation, render children and native arguments | Temporary values remain under their execution/inspection owner or registered evaluation/native scope; no FPAS call or eligible maintenance boundary may bypass these owners |

Root entries are weakly indexed. Each registration retains its domain; its entry
has only a weak domain back-reference. Deferred removals retain entries until the
outer external access exits, without a domain-to-domain ownership cycle. Every
independent registration removal requests a maintenance generation, including when
another registration still shares the same values. Host edge-removing mutations
request maintenance under the same guarded domain access.

Retired host exports permit guarded inspection, cell/root mutation and release.
They cannot invoke FPAS, reopen resources or keep the whole session alive. Session
shutdown rejects re-entry, including shutdown before a single-use VM call.

Live-image replacement validates all execution, registered, global and pending
main-result graphs before any commit. The existing instruction mapping cannot
rebind a retained callable graph atomically. Such graphs reject replacement,
including nested targets and Select callbacks; a numeric function ID never grants
new code provenance. Detached inspection copies preserve their original identity.
