# Domain execution and maintenance

Each logical execution has one movable `ExecutionRoots`. It moves through the
worker, saved task, catalog and retry boundary; a physical pool template owns none.
A fresh task entry holds only transferable values, so its first worker admits the
roots; spawning neither registers nor collects the new domain.
`ExecutionAccess` is thread-bound and reentrant on its owning thread. Its gate also
excludes external access and maintenance on other threads. No instruction acquires
a gate: dispatch acquires once per batch, with outer execution guards covering
admission, suspension restoration and terminal cleanup.

Execution providers are unavailable while registers change. A completed boundary
publishes initialized registers, continuations, supervision and suspension values.
The final returned value is included before a safe point. `RuntimeExecution` owns
a temporary registration until the public export is registered. Host ingress
likewise roots same-domain arguments before consuming the final incoming export.
A mismatched register/initialization bitmap defers maintenance.

Higher-order bytecode callbacks use frames on their owning main or scheduled worker.
Caller windows, callback captures, pending results and operation state are enumerated
by that worker's execution roots. Each dispatch batch follows the same publication
and safe-point protocol; individual elements need no native root registrations.

Before native callback re-entry the enclosing worker publishes and registers its native
caller roots. Native operation state retains intermediate results and arguments.
Every enclosing execution provider must be complete; an unavailable provider makes
maintenance defer. Returning invalidates the published caller before dropping its
native registration. A cached successful callback clears frames; a failed callback
retains roots for failure inspection.

## Permit and request handshake

The per-domain gate mutex protects owner/depth, parking, collection, retirement,
request recording and the final service/release decision. Atomic pending flags
provide the disabled fast path. Requests use a saturating generation plus a dirty
bit, so saturation does not lose work. A pass acknowledges only its complete
prepared view and repeats for requests/removals arriving during that pass. Final
acknowledgement and release share the gate mutex with request recording. A later
request sees the idle domain and services itself; an earlier request is handed to
the owner. A busy/poisoned provider or unwinding hook leaves work pending.

Public `DomainAccess` guards are always counted, also on the executing owner thread,
because they may outlive execution or span a safe point or native wait. A single
blocking cell read or replacement on that thread instead reuses the execution permit;
it cannot escape, park, reach a safe point or run FPAS/host code.
Same-thread nested external access postpones root removal until its outer release.
Another thread can commit root retirement during opaque execution without waiting
for that execution: an in-flight removal counter excludes maintenance until all
released values have been dropped. During maintenance the removal queues instead,
and remains a root until committed by the exclusive owner. None of these paths
waits for a permit already held by its own thread.

Lock order is domain ownership, then short registry/storage access, then individual
cell access. The gate mutex itself is never held while tracing, invoking a hook or
dropping values. Root registry locks are released before provider traversal or
request servicing. Cell replacement releases old values outside cell locks.
Scheduler/catalog operations defer servicing until their storage locks have gone;
task publication happens before enqueue. Native waits publish before acquiring
channel/notification locks and reacquire execution after those locks are released.
Maintenance uses a prepared root view without scheduler, channel or catalog locks.
Trusted maintenance hooks cannot enter FPAS, acquire external domain access, change
registrations or retain a strong domain/registration back-reference.

## Lifecycle

Normal return, failure and cancellation close unused owner cases before dropping
frames and logical execution roots. Saved states retain their logical domain.
Retries create the replacement owner before releasing the abandoned attempt, while
closing attempt cases and discarding old frames. Recoverable debugger failures keep
inspectable execution roots; completed/cancelled tasks retire them. Stopped debugger
commands acquire relevant runtime gates in stable task order, including peer frames
and main-domain globals. These commands never dispatch a stopped peer.

A callback session keeps an explicit session lease between invocations; cancel/drop
releases it. A one-shot VM retires after its final call/run. Its globals remain
registered until the VM object is destroyed. Independent host/debugger roots can
outlive execution and the VM without retaining hosted resources or workers. Retired
domains allow guarded graph access/mutation/release, but reject execution re-entry.
The requesting thread can service an idle or retired domain with surviving roots.

VM-created domains install the scheduled [local collector](COLLECTION.md).
Allocation/mutation pressure and completed-batch polls request safe service. A batch
records its poll before deciding whether to publish nested callback roots; unrelated
maintenance retries do not count as execution batches. Saved-state publication and
native parking request service regardless of running thresholds. Root release and
quiescent mutations retain synchronous service through the same generation handshake.

## Native blocking inventory

| Entry/path | State and servicing rule | Regression owner |
| --- | --- | --- |
| Main `Tasks.Send`, `Receive`, cancellation and timeout variants (`tasks/channel/`) | Published polling wait, including retained send payload; no queued task runs inline | `domain/protocol_tests.rs`, `tasks/channel/tests.rs` |
| Main `Tasks.Select` (`tasks/selection/polling.rs`) | Published notification wait; claimed case registrations stay alive | `tasks/selection/polling/tests.rs`, bytecode `roots/protocol_tests.rs` |
| Main `Tasks.Wait`, wait-all/any and cancellation probes (`tasks/scopes/mod.rs`, task wait modules) | Published notification wait through `wait_observing_scopes` | `tasks/channel/tests.rs`, `tests/task_scopes/blocking_waits.rs` |
| Normal scope exit and explicit group close (`tasks/scopes/mod.rs`, `tasks/groups/close.rs`) | Published polling wait, after resource probes release their locks | Scope/group close tests |
| Failure scope unwind and implicit terminal group teardown (`tasks/scopes/unwind.rs`, `tasks/groups/teardown.rs`) | Opaque cleanup wait; retained outcome stays rooted; service after return/unwind | Task-scope failure, group lifetime and supervision tests |
| Scheduled sleep, yield, task/channel waits, selection, group cleanup and retry backoff | Published saved task; wakeup can claim readiness but execution waits for maintenance | `domain/protocol_tests.rs`, scheduler wakeup and supervision tests |
| Main/callback `Time.Sleep` (`sleep_observing_scopes`) | Opaque cancellation-aware sleep; permit retained | Task-scope cancellation tests, bytecode opaque-provider tests |
| Console input/output and supplied text streams (`hosted/console.rs`) | Opaque host I/O, including user-supplied blocking writers | `domain/protocol_tests.rs`, host-value shutdown and console tests |
| File/directory open/read/write/flush/close and lifecycle callbacks (`hosted/fs/`) | Opaque host filesystem I/O; callback re-entry additionally publishes native caller roots | Filesystem and callback filesystem-lifecycle tests |
| Network DNS/connect/listen/accept/TLS/send/receive and close (`hosted/net/`) | Opaque native operation even where internal polling observes cancellation | Network establishment, transport and cancellation tests |
| HTTP request/response operations and handle I/O (`hosted/http_handles.rs`, hosted HTTP support) | Opaque network/filesystem operation | HTTP handle and HTTP intrinsic tests |
| Process launch/pipe polling/wait/cancel/cleanup (`hosted/proc/`) | Opaque native process lifecycle | Process engine/platform/failure tests |
| HTTP server lifecycle, handler joins and shutdown (`hosted/server/`) | Opaque service operation; callback sessions have their own domains | Server and callback failure tests |
| Runtime pool/timer shutdown, catalog finish and resource destructors | Opaque teardown; storage locks never invoke maintenance; execution roots retire at ownership boundary | Scheduler shutdown, catalog lifecycle and host shutdown tests |
| Other hosted intrinsics and OS calls (environment, clock, random, synchronous libraries) | Conservatively opaque for the whole call, including OS entropy/metadata; no prompt-reclamation guarantee | Existing intrinsic tests; shared opaque-return permit regression |

An operation falling back from publication because a provider is unavailable or an
external borrow is held remains opaque. Publication is proof of complete roots,
not merely registration of a few native arguments. Opaque operations service pending
work at the completed dispatch boundary or final unwind before another FPAS batch.
