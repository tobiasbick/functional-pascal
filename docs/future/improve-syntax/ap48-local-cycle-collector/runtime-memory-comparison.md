# Runtime memory management comparison

Source and documentation review of QuickJS, BEAM and Wren. No runtime change or
performance comparison was made. Downloaded sources remain under
`.temp-data/runtime-gc-review/`.

This supplements the [execution access investigation](execution-access-follow-up.md)
and its Lua comparison. The deferred [AP48.8 replacement design](08-memory-architecture.md)
builds on these findings. The findings below describe
the reviewed runtimes; they do not establish that either FPAS design is faster.

## QuickJS

Reviewed Bellard QuickJS, `VERSION` 2026-06-04, commit
`7d1c13c5d8f8c6b32893786ed35cdcebd88194d2`.

- Reference counting reclaims acyclic objects when their last owned reference is
  released. A separate cycle-removal pass handles unreachable cycles.
- The cycle collector subtracts internal graph edges from reference counts,
  identifies objects still reachable from external references, restores counts
  and frees the remaining cycles. Correct owned references provide the external
  liveness information without a separate C shadow-root list. Native classes
  must expose managed graph edges through their `gc_mark` callback.
- Refcount increments and decrements are ordinary integer operations; immediate
  primitive values skip reference counting. One runtime does not support
  concurrent execution. Contexts within it can share objects; independent
  runtimes cannot exchange ordinary heap objects.
- Captured variables initially refer to stack slots. Closing a variable detaches
  its shared reference into owned storage. Closure reads access that storage
  directly and duplicate the returned value as required by reference counting.
- Embedders must honor `JS_DupValue` / `JS_FreeValue` ownership. Finalizers cannot
  execute JavaScript. These restrictions are part of the memory model.

This is the closest comparison for the GC mechanism: reference counting plus
cycle collection is an established design. Trial deletion could be a separate
future investigation, but FPAS cannot simply substitute `Arc::strong_count` for
its root protocol. Administrative references, shared containers, native exports,
debugger roots and concurrent access all need consistent internal/external edge
accounting. QuickJS's non-atomic counts and direct accesses depend on its runtime
execution restriction.

Sources:

- [Runtime, embedding ownership and collector documentation](https://github.com/bellard/quickjs/blob/7d1c13c5d8f8c6b32893786ed35cdcebd88194d2/doc/quickjs.texi#L805-L880),
  [GC overview](https://github.com/bellard/quickjs/blob/7d1c13c5d8f8c6b32893786ed35cdcebd88194d2/doc/quickjs.texi#L1002-L1008).
- [Reference-count operations](https://github.com/bellard/quickjs/blob/7d1c13c5d8f8c6b32893786ed35cdcebd88194d2/quickjs.h#L705-L743),
  [cycle collection](https://github.com/bellard/quickjs/blob/7d1c13c5d8f8c6b32893786ed35cdcebd88194d2/quickjs.c#L6796-L6950).
- [Closing variable references](https://github.com/bellard/quickjs/blob/7d1c13c5d8f8c6b32893786ed35cdcebd88194d2/quickjs.c#L17824-L17848),
  [closure read/write opcodes](https://github.com/bellard/quickjs/blob/7d1c13c5d8f8c6b32893786ed35cdcebd88194d2/quickjs.c#L18923-L18963).

## BEAM

Reviewed Erlang/OTP 29.1.2, commit
`d199a49a01a2079afb6f2c5c9ae047cd4555e68d`.

- Each Erlang process has its own heap and stack. A generational copying
  collector traces live terms from registers, stacks and other process roots.
  Collection of one ordinary process heap does not require collecting all
  process heaps together.
- Erlang terms are immutable. Old terms cannot acquire new pointers into the
  young generation through ordinary language mutation. FPAS mutable capture
  cells cannot inherit this simplification.
- The scheduler holds the process main lock while running the process. Ownership
  belongs to the logical process, not permanently to an OS thread.
- Messages normally copy terms between process heaps. Reference-counted binaries
  and literals are important exceptions on the same node. Shared off-heap data
  still requires separate lifetime management.
- Closures carry an environment of captured terms on the process heap. They do
  not implement FPAS-style shared mutable captured variables.
- Process termination releases the local heap and its off-heap references.
  Shared data can remain alive for other owners. Function return does not have
  the same reclamation boundary as process termination.
- Native terms belong to an `ErlNifEnv`. Process-bound environments are valid
  only during their call on the supplied thread; retained terms need a suitable
  longer-lived environment. Mutable native resources and shared native state
  require explicit synchronization.

The useful lesson is coarse execution ownership and isolation between tasks.
A task-local heap with explicit copying or transfer could simplify FPAS access
rules, but would change sharing costs and representation. It is a larger design
alternative, especially with retained host values and mutable captures. BEAM
does not justify removing synchronization from currently shared FPAS cells.

Sources:

- [Collector design](https://github.com/erlang/otp/blob/d199a49a01a2079afb6f2c5c9ae047cd4555e68d/erts/emulator/internal_doc/GarbageCollection.md),
  [process heaps and message copying](https://github.com/erlang/otp/blob/d199a49a01a2079afb6f2c5c9ae047cd4555e68d/system/doc/efficiency_guide/eff_guide_processes.md).
- [Scheduler main lock](https://github.com/erlang/otp/blob/d199a49a01a2079afb6f2c5c9ae047cd4555e68d/erts/emulator/beam/erl_process_lock.h#L109-L118),
  [closure environment](https://github.com/erlang/otp/blob/d199a49a01a2079afb6f2c5c9ae047cd4555e68d/erts/emulator/beam/erl_fun.h#L60-L85).
- [Native concurrency restrictions](https://github.com/erlang/otp/blob/d199a49a01a2079afb6f2c5c9ae047cd4555e68d/erts/doc/references/erl_nif.md#L281-L292),
  [native environment lifetimes](https://github.com/erlang/otp/blob/d199a49a01a2079afb6f2c5c9ae047cd4555e68d/erts/doc/references/erl_nif.md#L556-L600).

## Wren

Reviewed upstream development commit
`99d2f0b8fc2686134b32b18166e037639f7e9f2c`.

- A nonmoving mark-and-sweep collector runs a complete collection synchronously.
  Allocation pressure triggers collection; the next threshold is based on live
  allocation size, configured growth and a minimum heap size.
- Roots include modules, temporary VM roots, the active fiber, persistent host
  handles and compiler state. The collector traces reachable objects and frees
  unmarked objects. Unreachable cycles need no separate algorithm.
- A persistent `WrenHandle` keeps its value alive. The host must explicitly
  release handles. Raw native pointers do not replace this rooting contract.
- Shared closure upvalues initially point into the stack and move into closed
  storage when their scope ends. Upvalue reads and writes directly access the
  value, without per-access refcounting, mutexes or incremental write barriers.
- Fibers are cooperative. Separate VMs can run on different OS threads; this is
  not a promise of safe concurrent access to one VM's heap.
- Foreign finalizers may release native resources but cannot freely re-enter
  the VM during collection.

Wren is the clearest small reference for explicit host handles, temporary roots
and cheap internal access with controlled collection points. FPAS already has
execution, host and debugger roots; this comparison can help audit their
boundaries. Replacing FPAS storage with Wren's heap would also change immediate
reclamation, copy-on-write and collection pauses, so it is not a small fix.

Sources:

- [Collector and allocation trigger](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/src/vm/wren_vm.c#L125-L240),
  [upvalue closing](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/src/vm/wren_vm.c#L287-L302),
  [upvalue opcodes](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/src/vm/wren_vm.c#L1094-L1107).
- [Host handles](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/doc/site/embedding/slots-and-handles.markdown#L240-L268),
  [independent VMs](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/doc/site/embedding/index.markdown#L140-L148),
  [fibers](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/doc/site/concurrency.markdown).
- [Finalizer restrictions](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/doc/site/embedding/storing-c-data.markdown#L150-L180).

## Implications for FPAS

A bounded internal access API remains an alternative within the current memory
model. Both that alternative and the replacement must prove that borrowed access
cannot escape, park, cross a collection point or re-enter arbitrary host code
without the required rooting and exclusion. API shapes may change; the underlying
memory-safety obligations remain.

Domain-access bookkeeping and the cell-storage mutex protect different things.
Avoiding a redundant domain guard does not itself justify removing the mutex;
shared snapshots, external access and destructor re-entry need their own audit.
Keep the access regression tests and GC allocation-balance tests as acceptance
criteria, then measure against the corrected implementation.

The preferred replacement combines a Wren-like simple tracing heap with explicit
task ownership inspired by BEAM. QuickJS remains evidence that reference counting
plus cycle collection is viable, so a rewrite must earn its additional effort
through a simpler complete design and measured tradeoffs. The source review
establishes design tradeoffs, not a predicted speedup or a proof that another
collector would avoid FPAS integration bugs.
