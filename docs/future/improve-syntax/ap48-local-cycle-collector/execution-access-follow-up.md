# Execution access follow-up

Status: permit reuse for blocking cell reads and replacements is implemented.
The Lua findings and access-safety requirements also apply to the
[AP48.8 redesign](08-memory-architecture.md).

See also the [QuickJS, BEAM and Wren comparison](runtime-memory-comparison.md)
for additional source-based findings and their limits for FPAS.

## Delivered permit reuse

The exclusion fix counts every public `DomainAccess`, including guards the
executing owner acquires, because such a guard may outlive execution or span a
safe point or native wait. Cell operations took the same counted path, so each
captured read or write locked the domain gate twice. `mutable_capture` rose from
234 ms to about 342 ms.

`MemoryDomain::operation_access` (`crates/fpas-bytecode/src/value/roots/access.rs`)
returns no guard when the calling thread is the published executing owner, and a
counted guard otherwise. It is visible only inside the value modules and used by
`OwnedCell::read_snapshot`, `read_integer_or_value` and blocking `replace`. Those
operations never escape the guard, park, reach a safe point or run FPAS/host code;
maintenance requested during them defers because execution is active. Nonblocking
variants and `update`, whose closure is caller code, keep counted access.

The lifecycle regressions in `crates/fpas-bytecode/src/value/roots/protocol_tests/lifecycle.rs`
still cover collection at a safe point, published waits and an external access that
outlives execution. After the change, `mutable_capture` measures about 200 ms
(local release build, 3,000,000 iterations).

## Lua comparison

Reviewed Lua 5.5.1 at upstream commit
`7579fc9d7ed90240487251dfb69168f8e64e9294`, including its reference manual and
implementation. The upstream GitHub repository supplied the sources because
direct access to lua.org was blocked by this cloud environment's network policy.

- Lua traces its managed heap, with incremental mark-and-sweep and generational
  collection modes. Ordinary value assignment copies a tagged value without
  per-assignment reference counting. Unreachable cycles are ordinary GC input.
- Returning from a function releases its stack window; it does not immediately
  reclaim all heap objects used by that function. An open closure upvalue points
  into the stack. Closing it moves the value into the existing upvalue object's
  storage, so all closures sharing that upvalue retain its identity.
- `OP_GETUPVAL` reads directly through the upvalue pointer. `OP_SETUPVAL` stores
  directly and applies a GC write barrier when needed. The barrier first checks
  whether the stored value is collectable; integer/Boolean values do not require
  a heap-edge barrier. These opcode paths have no per-access mutex.
- Lua coroutines are cooperative execution contexts inside a Lua state, not OS
  threads executing the same heap concurrently. Core-entry lock hooks are no-ops
  in the default build. An embedding application must arrange serialization for
  concurrent access to one state; independent states have separate heaps. This
  differs from FPAS parallel tasks, shared transferable values and retained host
  and debugger exports.
- Lua's C embedding keeps managed objects reachable through the Lua stack or
  registry (for example via `luaL_ref`). An arbitrary C pointer is not a GC root.
- A local `<close>` variable runs its `__close` method at scope exit. This is
  deterministic resource cleanup, not automatic destruction of all referenced
  heap objects. A retained reference can keep the closed object alive.

The applicable idea is to make execution ownership and GC entry points explicit,
then keep bounded internal accesses cheap. Copying Lua's raw-pointer operations
without its execution and rooting constraints would reproduce the access bug.
Replacing FPAS reference counting with a complete Lua-style tracing heap would
also require redesigning transfer, copy-on-write and host-value lifetime handling;
it is a separate architectural change, not this investigation.

Four small lifetime checks were run in each of Lua's two GC modes: delayed
reclamation after function return, shared mutable captures, unreachable
table/closure cycles, and `<close>` with a retained reference. All eight passed.
These are semantic checks, not Lua/FPAS performance measurements. Sources, build
log, script and output remain under `.temp-data/lua-gc-review/`.

Sources (pinned upstream version):

- [Manual: memory management](https://github.com/lua/lua/blob/v5.5.1/manual/manual.of#L680-L810),
  [scope cleanup](https://github.com/lua/lua/blob/v5.5.1/manual/manual.of#L1742-L1787),
  [state and coroutine model](https://github.com/lua/lua/blob/v5.5.1/manual/manual.of#L2566-L2582).
- [Upvalue operations](https://github.com/lua/lua/blob/v5.5.1/lvm.c#L1293-L1305),
  [closing upvalues](https://github.com/lua/lua/blob/v5.5.1/lfunc.c#L194-L210),
  [tagged value copies](https://github.com/lua/lua/blob/v5.5.1/lobject.h#L118-L121).
- [Default lock hooks](https://github.com/lua/lua/blob/v5.5.1/lapi.h#L29-L36),
  [GC entry and write barriers](https://github.com/lua/lua/blob/v5.5.1/lgc.h#L233-L253),
  [stack tracing](https://github.com/lua/lua/blob/v5.5.1/lgc.c#L695-L717).
