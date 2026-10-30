# GC memory regressions

ACP host jobs add 24 exact-balance authentication sessions with gated native
providers, cyclic callback captures, scope exit and writer-triggered VM shutdown.
Eight Unix FIFO sessions additionally cover host file reads/writes, mixed native
filesystem peers, oversized payloads and SDK connection cleanup. Both use default
and 1/1/1 collection schedules. Separate tests bound unfinished SDK constructors,
signal shutdown during construction, cancel busy terminal lock waits and close
undelivered connection/terminal results.

Native Console boundaries add 24 exact-balance VM sessions for gated output and
text input, suspended nested callbacks with cyclic captures, and retained frames
and regions. Normal completion, scope exit and writer-triggered VM shutdown run
with default and 1/1/1 schedules. The gates require progress from separate input
and output jobs; all jobs and their native payloads must be joined and released.
Separate unit tests require one-worker progress, preserve resource issue order,
queue calls at native-job capacity and discard unpublished saved regions.
File and network regressions also retain undelivered handles while a native
callback returns before joining; final retirement closes those handles and
releases their job permits.

Native process boundaries add 24 exact-balance VM sessions covering a gated terminal
provider, `RunInteractive`, process `Start`/`Close`, cyclic captures and unused cases.
Normal completion, scope exit and VM shutdown run with default and 1/1/1 schedules.
The existing 72 owned-process teardown sessions also cover native startup, reaping,
stderr receivers and callback-triggered shutdown; the shutdown callback must return
without a native-thread self-join panic. Native retirement is joined before VM teardown
completes. TLS close unit tests separately force socket backpressure, preserve all
accepted plaintext through close_notify, and abort an abandoned detached transport.

Native socket configuration adds 24 exact-balance VM sessions for timeout changes
and listener creation inside cyclic callbacks. Normal completion, scope exit and
VM shutdown run with default and 1/1/1 schedules. Separate native/unit fixtures
force a busy connection lock, load TLS certificates over a Unix FIFO on one worker
and in the debugger, close unpublished TCP/TLS listeners and join abandoned PEM I/O.

Native path-job teardown adds 24 exact-balance VM sessions for whole-file I/O,
directory/tree operations, queries and temporary resources in nested callbacks.
Normal completion, scope exit and VM shutdown run with both collection schedules.
Separate unit tests prove cleanup of unpublished temporary paths, queue plain
queries at native-job capacity, retain atomic-write targets on cancellation and
use a Unix FIFO to require progress from two FPAS tasks on one VM worker.

Native file-job teardown covers normal handle I/O inside nested callbacks,
pre-cancelled operations, scope exit and callback-triggered VM shutdown. Large
native byte/text payloads, cyclic captures and unused WaitCase callbacks return
to the exact allocation and byte baseline in 32 first/repeated VM sessions under
default and 1/1/1 collection schedules. Separate gated runtime tests force a busy
file alias and prove that a second FPAS task runs on the only VM worker; native
unit tests check cancellation while locks remain held and close undelivered opens.

Selectable process-pipe regressions retain cyclic callbacks and dense ByteBuffer write
payloads through normal completion, a cancellation case, a timer, scope exit and VM
shutdown. Binary duplex traffic exercises selected partial reads/writes and EOF;
discarded cases must have no pipe effects. Writes preserve the constructor snapshot
after the caller mutates its buffer. Four runs with each collection schedule
require the exact pre-session allocation and byte baseline after every VM teardown,
including the first session. Independent host observers verify child termination.

Run the allocation tests and pool bounds with:

```sh
cargo test -p fpas-runtime --test gc_memory -- --nocapture
cargo test -p fpas-runtime --lib value::managed_heap
```

The integration test binary installs a process-wide `stats_alloc` wrapper around
the system allocator. Each case launches itself in a separate process with one
test thread, so unrelated tests cannot change its measurements. Allocations,
deallocations and reallocation growth/shrinkage on all runtime threads contribute
to the balance. The production allocator is unchanged.

Every measured round must return to one fixed baseline in both requested bytes
and outstanding allocation count. No per-round allowance or baseline reset can
hide cumulative growth. The fixtures also require substantial allocation traffic.
A control case retains 256 KiB allocated on another thread, grows and shrinks it,
and then checks that releasing it on a third thread restores the baseline.

## Coverage

| Case | Required behavior |
| --- | --- |
| Aggregate cycles | Two-cell cycles through arrays, dictionary keys/values, records, enums, Option, both Result variants, captures and bound receivers release their backing storage. Payloads include a 64 KiB string and an 8192-element array. Weak observations and allocated/released cell counts verify actual collection before thread exit. |
| Seeded graph/root model | An independent integer-adjacency oracle checks generated graph mutations, seven root kinds, root clones/replacement/push/removal and session retirement. Every collection preserves exactly the reachable cells, aliases and payloads. Failed candidate/marking budgets and rejected foreign/reference assignments preserve existing values. |
| VM | Large captured closure cycles and live aliases behave correctly with default and 1/1/1 collection thresholds. Both successful execution and runtime failure release memory. |
| Tasks | Concurrent child tasks release cycles on completion and on failure with scope cancellation. The balance includes pool/timer threads and their teardown. |
| Callback cycles | 32,000 self-referential closures created by `Map` callbacks, each with a private 2 KiB payload, keep sampled peak growth below 16 MiB. A control run with collection thresholds disabled must exceed that bound, so the check detects reclamation deferred to VM teardown. |
| Mixed live waits | A single running VM records 512 allocation checkpoints while combining task results, channel payloads, winning/losing Select callbacks, timers, timeouts, explicit case close and group cancellation. Long-lived aliases remain usable. Default and 1/1/1 collection schedules keep growth below fixed byte and allocation-count envelopes. |
| Mixed wait teardown | Completion, intentional live retention and selected-callback panic each return exactly to the same byte/allocation baseline after joined threads exit. Cases run with both collection schedules and include unused cases at failure. |
| Pending task teardown | Explicit idempotent VM shutdown, root failure and early scope exit release parked task frames, pending operations, unused/claimed selection callbacks, channel buffers and timers. Eleven wait modes run with default and 1/1/1 schedules; every first and repeated run returns exactly to its byte/allocation baseline. |
| Native socket waits | Pending accepts and reads retain cyclic captures and unused selection cases across cancellation, listener close, timeout, scope exit and VM shutdown. Native jobs join and every first/repeated run returns exactly to its allocation baseline, with default and 1/1/1 schedules. Cancelled or abandoned established connections remain usable. |
| Native connects and DNS | Plain and cancellation-aware connects resolve a loopback hostname, exchange bytes through nested callbacks and release frame payloads. Joined VM, socket and completed resolver workers return every first/repeated run exactly to its byte/allocation baseline, with default and 1/1/1 schedules. |
| Native entropy | OS bytes, integer sampling, explicit randomization and automatic PRNG seeding retain cyclic callbacks and unused cases across completion, scope exit and writer-triggered VM shutdown. All 96 first/repeated VM sessions return exactly to their byte/allocation baseline with default and 1/1/1 schedules. |
| MCP executor initialization | First HTTP publication through nested callbacks retains cyclic captures and unused cases. Completion and scope abandonment join the native executor, retire publications and release every allocation in 16 first/repeated VM sessions, with default and 1/1/1 schedules. |
| Tokenizer ownership | Concurrent tasks reuse VM-owned BPE tables through nested callbacks with cyclic captures and unused cases. Eight first/repeated VM sessions require exact byte/allocation balance with default and 1/1/1 schedules. No tokenizer or VM is warmed up. |
| Native callable receivers | Bound native operations retain arrays whose callable elements close cycles through the adapter itself. Sixteen first/repeated VM sessions, including runtime failures, return every receiver, cycle and large captured payload exactly to the allocation baseline under default and 1/1/1 schedules. |
| Owned process waits | All four Run variants retain cyclic captures and unused cases while child/peer gates require concurrent progress. Duplex stdin/stdout/stderr, owned descendants, cancellation, scope exit and VM or stderr-receiver shutdown return exactly to the first and repeated allocation baselines under default and 1/1/1 schedules. Independent host sockets prove children and descendants have exited. |
| Callbacks | A persistent session repeatedly exports aliases, invokes them and handles failed callbacks without memory growth. An independent closure remains alive; its last host root is released after session destruction. |
| Debugger | Repeated breakpoint stops, variable expansion and detached evaluation of a cyclic captured value have stable memory use. Disconnecting a stopped session releases its current payload and inspection/helper roots. |
| Debugger frame restart | Repeatedly restarting an older frame releases its discarded locals, younger frames and inspection roots. Caller and parameter aliases remain usable. Invalid and stale frame/variable handles are rejected; both continued execution and disconnect release all allocations. |
| Debugger forced return | Replacing an older frame's result releases unwound frames and their cyclic captures. Large replacement arrays remain expandable and usable by the caller. Repeated type and output-limit failures preserve the live frames without retaining detached preparation values. |
| Debugger mutation | Repeated variable and expression assignments release replaced cyclic closures and large arrays while preserving capture aliases. Failed type and output-limit checks leave values and handles unchanged; stale handles cannot mutate a later stop. |

Live-session measurements warm up for at least 64 rounds and require four
consecutive identical footprints within 256 rounds. They then check 32 more rounds
against the unchanged baseline while the session remains alive. This allows bounded
thread-local pools and registry capacities to fill before checking stability.
Debugger restart, forced-return and mutation cases run with both default and
1/1/1 collection thresholds. The return and mutation cases first fill the
weak-root registry with live-variable inspection handles, then invalidate them.
This prevents bounded HashMap growth from tombstone churn at a persistent stop
from being mistaken for retained payloads. Rejected return and mutation
preparations have their own fixed-baseline loops while the same frames remain
stopped: a successful resume or edit cannot hide storage retained by a failed
operation. Large arrays contain 8192 elements, above the pool's recycling limit.
Teardown measurements include the first session and require exact release after
its thread exits.

The mixed live workload records checkpoints after each complete round, before
the VM or its worker pool shuts down. Recording uses preallocated storage.
After 64 warm-up rounds, one fixed baseline bounds every remaining checkpoint:
growth must stay below 1 MiB and 2048 outstanding allocations. Small bounded
pools and asynchronous worker bookkeeping can still vary within this envelope.
The payload arrays exceed the pool's 4096-element recycling limit, so pooled
large buffers cannot hide delayed release. A control retains a distinct string
larger than 64 KiB per round and must exceed the byte envelope during execution.
The separate teardown test also accepts that control's final zero balance:
end-only measurements would miss its live retention.

The pending-task cases use the real worker pool and timer thread. A host writer
holds the root and any nested leaf at a barrier while the existing task-park
counter confirms the intended worker reached its pending operation. Nested
leaves cannot park before that barrier is released, so their activity cannot
satisfy this check. No fixed sleep is used to guess whether a task is waiting.
The matrix covers send/receive with and without cancellation, Sleep, a Select
with four unready sources, Wait, WaitAny, WaitAll, scope join and group close.
Four runs per configuration include the first session, for 264 VM executions.
Each waiting worker retains an oversized array and a cyclic closure with a
64 KiB string; root aliases remain valid after early scope exit. Inactive tokens
and one-hour deadlines keep operations pending until the test triggers cleanup.
Completion waits have a 30-second deadlock guard, not a speed assertion.

The seeded state machine runs 32 fixed ChaCha8 seeds with both collection
schedules and two repeats: 128 traces with 12,288 transitions, plus initial and
final root-release checks. Inputs include shared subgraphs, duplicate edges,
self-cycles, separate cyclic islands and up to four nested edge wrappers.
The model uses only node IDs and adjacency lists; it never calls the production
graph walker to calculate reachability. Weak observations, per-domain cell
counters, root-kind counts, root aliases and payload/edge contents are checked
after each transition. The driver retains no administrative strong cell handles
between transitions. Large strings and oversized arrays test actual backing
storage release; exact post-thread allocation balances include every trace,
including the first. Halfway through each trace the session retires while roots
remain alive, and subsequent graph access must preserve their values.

Candidate-limit and partial-marking failures alternate and each retain a fresh
unrooted cycle until a successful collection. Foreign-domain and escaping-reference
assignments must fail without changing the target. Failures print the seed and
transition index; replay one decimal seed with:

```sh
FPAS_GC_GRAPH_SEED=0 cargo test -p fpas-runtime --test gc_memory seeded_graph_roots_and_mutations_match_the_reference_model -- --nocapture
```

This is a bounded deterministic corpus, not exhaustive coverage of all graphs.

These tests assert typed outcomes, callback counts, payload contents and alias
behavior as well as memory. They use actual timer expiry and cancellation
without assertions about wall-clock speed. The envelope is a regression bound
for this workload, not a guarantee for arbitrary programs or all thread schedules.

Completed-thread measurements initialize Rust's thread machinery with a no-op
thread, then check every FPAS round, including the first, after all spawned threads
have joined. Thread-local pools are destroyed at that point; their cached buffers
are part of the required release, rather than an allowance subtracted from the
balance. A child completion marker also ensures a mistaken test filter cannot
silently pass without executing the requested case.

The tokenizer case also initializes regex-automata's process-global CPU-count
metadata directly before its baseline. It checks that this costs at most one
`usize` allocation (eight bytes on a 64-bit host), and a second query allocates
nothing. This does not construct a regex, tokenizer or FPAS session. Every table,
regex instance and callback allocation made by the first/repeated VM runs must
still be released, with no byte allowance.

The MCP executor case initializes an empty native Tokio I/O runtime separately
before its baseline. Feature-unified builds may retain Tokio's process-global
signal registry. A second empty runtime must leave the same footprint. Neither
step constructs an MCP server or FPAS session; every first/repeated MCP VM run
still requires an exact allocation balance.

The value pool tests separately saturate value, dictionary and string buffers
to exercise byte and bucket limits independently. They check take/recycle
accounting, oversized-buffer rejection, and the payload-box count limit. Recycled
payload boxes must contain only Unit and release a nested cell with a large string.

## Measurement boundary

These are balances of requested allocations through Rust's global allocator.
They do not measure allocator metadata, native thread stacks, external native
allocators or process resident memory. In particular, returning memory to the
system allocator does not require a decrease in RSS. Compiled images intentionally
retained by the host remain outside the execution balance.
