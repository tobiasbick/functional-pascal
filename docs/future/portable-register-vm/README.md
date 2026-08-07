# Portable register VM rewrite

Status: approved implementation direction; P0 through P3 implemented, production cutover not started.

This directory is the implementation contract for replacing the current stack bytecode and stack
interpreter with a portable register VM. It is deliberately prescriptive so another coding agent can
execute the work without inventing architecture along the way.

The project is a hobby project. Internal compiler, linker, artifact, VM, and runtime structures may
be replaced completely. FPAS syntax, semantics, diagnostics contracts, and the user-facing language
specification must not change without explicit user approval.

## Fixed decisions

These decisions are already made. An implementation agent must not reopen them unless current-checkout
evidence proves one impossible.

1. The first fast backend is a portable register interpreter written in safe Rust.
2. Cranelift, JIT compilation, AOT compilation, machine-code caching, and executable-memory management
   are deferred. Do not add Cranelift crates during this plan.
3. `.fpascp` remains portable VM code. It must not contain native pointers, native-endian values,
   target triples, object code, or host ABI layouts.
4. A Windows-produced `.fpascp` must be consumable by a compatible Linux, macOS, or FreeBSD `fpas`
   runtime, including on ARM, when the repository and its external crates build on that target.
5. The old stack bytecode does not need backward compatibility. Increment format and bytecode versions
   at cutover and return a precise rebuild diagnostic for old artifacts.
6. The final implementation has one compiler path and one interpreter path. Temporary side-by-side
   code is allowed only inside the development branch and must be removed before completion.
7. Link-time names become deterministic numeric IDs. Runtime lookup by function, global, record-field,
   enum-type, or enum-variant string is not allowed on ordinary execution paths.
8. Source locations are sparse metadata and are resolved only for diagnostics, tracing, or explicit
   debugging. The dispatch loop must not fetch a source location for every instruction.
9. Runtime values remain semantically value-based. Existing copy-on-write aggregate behavior and
   concurrency semantics are preserved unless measurements and tests justify an internal replacement.
10. Optimizations are accepted only with same-machine release measurements. No speedup may be claimed
    from instruction counts, code inspection, debug builds, or a single noisy run.

## Required reading order

An implementation agent must read these files in order before editing code:

1. This file.
2. [Target architecture](target-architecture.md).
3. [Implementation phases](implementation-phases.md).
4. [Bytecode and portability](bytecode-and-portability.md).
5. [Compiler and linker](compiler-and-linker.md).
6. [Interpreter and runtime](interpreter-and-runtime.md).
7. [Benchmarks](benchmarks.md).
8. [Testing](testing.md).
9. [Terra runbook](terra-runbook.md).
10. [Traceability](traceability.md).

Completed phase evidence is recorded in [P0 contract and baseline](p0-contract-baseline.md) and
[P2 register bytecode implementation](p2-register-bytecode.md). P1 evidence lives directly beside
the typed IR tests and in the traceability matrix. The first end-to-end inactive compiler and
interpreter slice is recorded in [P3 scalar/control-flow implementation](p3-scalar-control-flow.md).

The repository-level `AGENTS.md` and the relevant project skills remain mandatory. In particular,
performance work follows `.agents/skills/fpas-bench/SKILL.md`, and behavior work follows
`.agents/skills/fpas-change-checklist/SKILL.md`.

## Current implementation snapshot

Revalidate these facts before implementation because file names can move:

- [`fpas-bytecode::Chunk`](../../../crates/fpas-bytecode/src/chunk.rs) stores `Vec<Op>`, a `Value`
  constant pool, one `SourceLocation` per instruction, and a name-keyed function `HashMap`.
- [`Op`](../../../crates/fpas-bytecode/src/op.rs) is a stack instruction enum. Calls, globals, records,
  enums, and closures carry constant-pool indices that resolve to strings.
- [`Compiler`](../../../crates/fpas-compiler/src/compiler/mod.rs) lowers the analyzed AST directly to
  stack bytecode and tracks locals as stack slots.
- [`Worker::exec_one`](../../../crates/fpas-vm/src/vm/execute/mod.rs) performs a top-level opcode match,
  then many category helpers match the opcode a second time.
- [`Worker`](../../../crates/fpas-vm/src/vm/worker.rs) owns a value stack and a separate call stack.
- [`SharedState`](../../../crates/fpas-vm/src/vm/shared.rs) stores globals in a name-keyed `HashMap`
  behind an `RwLock`.
- Record field access in
  [`execute/aggregates/records.rs`](../../../crates/fpas-vm/src/vm/execute/aggregates/records.rs)
  performs linear string search.
- [`fpas-linker`](../../../crates/fpas-linker/src/lib.rs) rebases stack instruction addresses and
  constant indices while retaining name-keyed callable metadata.
- [`fpas-program`](../../../crates/fpas-program/src/image/payload.rs) wraps a strict JSON instruction
  payload in a bounded binary `.fpascp` envelope.
- `Value` is already constrained to at most 16 bytes and compound values already use shared or
  copy-on-write storage. Preserve that useful baseline until a measurement disproves it.
- [`fpas-ir`](../../../crates/fpas-ir/src/lib.rs) now owns the validated target-independent typed IR
  introduced in P1.
- [`fpas-bytecode::Executable`](../../../crates/fpas-bytecode/src/executable.rs) and its verifier now
  own the inactive register representation introduced in P2; production still uses `Chunk`.
- [`compile_register_subset`](../../../crates/fpas-compiler/src/lib.rs) lowers the functionless P3
  scalar/control-flow subset through typed IR, deterministic allocation, bytecode selection, and
  verification without exposing a CLI backend switch.
- [`RegisterVm`](../../../crates/fpas-vm/src/vm/register/mod.rs) executes only a
  `VerifiedExecutable` through one exhaustive packed-opcode dispatch loop; production still uses
  the stack VM.

## Desired pipeline

```text
FPAS source
  -> parser (unchanged language)
  -> semantic analysis (unchanged meaning)
  -> target-independent typed control-flow IR
  -> deterministic register allocation and bytecode selection
  -> relocatable register objects (.fpascu)
  -> numeric-ID linker
  -> portable register executable (.fpascp)
  -> safe Rust register interpreter
```

Cranelift may later consume the typed IR, but it is not a deliverable or dependency of this rewrite.

## Success definition

The rewrite is complete only when all of the following are true:

- The parser and accepted FPAS syntax are unchanged.
- Existing language, standard-library, CLI, project, artifact, bundle, VM, hosted runtime, and
  concurrency tests pass.
- The old `Chunk`, stack `Op`, stack compiler emission, and stack interpreter are deleted.
- Direct calls, globals, record fields, enum variants, and intrinsic calls use validated numeric IDs.
- The executable verifier checks every register, ID, range, jump, function, source-map, and entry
  invariant before execution.
- `.fpascp` encoding is deterministic, bounded, explicitly little-endian, and pointer-width neutral.
- Cross-host artifact tests and the platform matrix in [Testing](testing.md) are satisfied or honestly
  marked unverified; unsupported external crates are reported rather than hidden.
- The same-machine VM benchmark geometric mean is at least 1.5x the saved baseline, `integer_loop`
  and `function_call` are each at least 1.5x, and no accepted suite row regresses by more than 10%.
- A low-end Linux x86-64 Chromebook measurement is recorded when that hardware is available. The
  stretch target is at least 2x for integer loops and direct calls on that device.
- Current behavior is documented under `docs/pascal/`, benchmark history is recorded, and this
  completed future directory is removed.

## Non-goals

- No FPAS syntax or semantic changes.
- No garbage collector rewrite unless profiling after the register cutover identifies it as the next
  dominant cost.
- No dictionary redesign; that has its own future decision.
- No new package manager, registry, global artifact cache, or remote compilation service.
- No GitHub Actions or other CI configuration.
- No native executable cross-compilation. Portable `.fpascp` and host-native `--executable` remain
  separate concepts.
- No interpreter abstraction trait merely to reserve a Cranelift slot. The later backend can be added
  when it exists.

## External design references

The design borrows narrowly from established VMs:

- Lua 5.0 uses registers allocated in per-call activation records, reducing value push/pop traffic:
  [The Implementation of Lua 5.0](https://www.lua.org/doc/jucs05.pdf).
- CPython's adaptive interpreter shows that call, global, and attribute lookup are valuable
  specialization targets and that quickening can remain separate from immutable portable bytecode:
  [PEP 659](https://peps.python.org/pep-0659/). FPAS should first exploit its static type information
  and numeric IDs; adaptive specialization is optional later work, not phase-one scope.
- Bytecode Alliance's Pulley design explicitly separates portable interpreter bytecode from
  Cranelift-supported native targets:
  [Pulley RFC](https://github.com/bytecodealliance/rfcs/blob/main/accepted/pulley.md).

These are implementation references, not language-design authorities.
