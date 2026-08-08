# P6 intrinsics and hosted runtimes

Status: complete on the inactive register-development path. Production CLI execution remains on the
stack VM until P9.

## Delivered contract

P6 routes every existing `Std.*` call to the packed register instruction
`Intrinsic(destination, intrinsic_id, argument_base, argument_count)`. The numeric ID is the stable
`u16` wire value owned by `fpas-bytecode::Intrinsic`; the verifier rejects unknown IDs, invalid
destinations, and argument windows outside the active frame before execution.

Semantic analysis records the canonical `Std.Unit.Member` dispatch name at the source call site.
The compiler catalog converts that name to one existing intrinsic ID, lowers arguments left to right
into one contiguous window, and emits one ABC instruction. Source-level variadic `Std.Console.Write`
and `WriteLn` calls expand into ordered single-value intrinsics so expression side effects retain
the stack VM's output order; empty `Write` is a no-op and empty `WriteLn` emits a blank line.
`Std.Str.Format` remains variadic in IR: its synthetic format-argument count occupies the final
register and validation checks the fixed prefix plus repeated dynamic tail. No runtime lookup hashes
or canonicalizes a standard routine name.

`fpas-std::run_intrinsic_borrowed` decodes a borrowed `&[Value]` from right to left, matching the
legacy stack convention without removing or cloning input registers. Scalar and read-only aggregate
operations borrow their inputs. Implementations acquire ownership only for returned values,
copy-on-write mutation, formatting storage, or platform APIs that require an owned value. The legacy
stack entrypoint is now a compatibility adapter over the same decoder, so both development paths
share standard-library behavior until cutover.

## Runtime ownership

The register VM owns one isolated hosted state shared with nested numeric callbacks:

- process arguments;
- captured/streamed Console state, line input, key and unified event input;
- Graph session, backbuffer, host event coalescer, handlers, and headless-test lifecycle.

Console includes text I/O, CRT state, raw/alternate-screen modes, unified events, color records,
deferred frames, cell/rectangle/region operations, Unicode display-width helpers, and interactive
terminal acquisition. Graph includes native and headless opening, close/size/redraw, upload and
drawing operations, configuration, host handler registration, event/redraw dispatch, the hosted run
loop, and deterministic test-key injection. Screen assertions and queued `ReadLn` input use this same
state. Platform calls remain in `fpas-std`; register bytecode contains no OS branch or host ABI data.

Higher-order Array, Dict, Result, Option, and hosted Graph operations invoke a first-class value only
through its numeric `FunctionId`. A nested worker shares immutable verified code, dense globals,
layout metadata, hosted state, and closure capture cells. Legacy name-only function values are
rejected on this path.

`Std.Task.Wait` and `Std.Task.WaitAll` remain deliberately assigned to P7. Their IDs are selected and
verified, but execution returns the existing VM-only diagnostic rather than silently entering a
partial scheduler.

## Completeness guards

Three independent inventories close the maintenance gap:

1. `Intrinsic::all()` is derived from the authoritative decoder over the complete `u16` space.
2. Bytecode tests compare the explicit all-intrinsics list with that decoder and verify global wire-ID
   uniqueness.
3. Compiler catalog tests resolve every stable intrinsic ID from a canonical source call, including
   Graph nesting and the four typed `Std.Test.AssertEquals` variants.

Direct register-VM tests cover argument isolation, Console/Test shared state, deterministic headless
Graph lifecycle, and the explicit P7 boundary. Compiler differential tests execute borrowed scalar,
string, numeric, test, collection, numeric callback, and variadic Console calls on both VMs.
Standard-runtime tests cover wrong count/type errors and prove borrowed aggregates are neither
consumed nor mutated.
An intrinsic-inside-loop regression also verifies that block-address calculation counts argument
moves and the optional Unit materialization; otherwise branches could target the middle of an
expanded intrinsic call.

The phase-compatible non-concurrency source corpus is the complete
`fpas-compiler::tests::register_subset` suite. It covers the language subset through P6 and runs
through the new compiler and VM, with differential stack-VM assertions where observable behavior is
involved. Repository tests that require independently compiled user units cannot use the new path
before the P8 object/linker phase; this is a phase dependency, not an intrinsic fallback.

## Verification

The P6 implementation is accepted only after these commands pass from the repository root:

```text
cargo fmt --all -- --check
cargo build
cargo test --workspace
cargo clippy --all-targets --all-features --locked -- -D warnings
target/debug/fpas fmt --check examples/ tests/ apps/
target/debug/fpas test tests/
git diff --check
```

Focused evidence includes `cargo test -p fpas-bytecode`, `cargo test -p fpas-ir`,
`cargo test -p fpas-sema`, `cargo test -p fpas-std`,
`cargo test -p fpas-vm vm::register::tests`, and
`cargo test -p fpas-compiler register_subset`.

`cargo bench-fpas save register-p6-complete --group register-p6` records two release diagnostic
workloads under `.temp-data/bench/`: `register_string_length` and `register_array_callbacks`. The
snapshot proves both workloads execute entirely on the inactive path. It is not compared with the
production stack group and therefore is not a speedup claim.

## Documentation classification

This is an internal compiler/runtime development-path change. FPAS syntax, semantics, standard API,
diagnostic contract, and production CLI behavior are unchanged, so `docs/pascal/` is unchanged.
