# P3 scalar/control-flow implementation

P3 is complete. It provides the first end-to-end register pipeline while deliberately leaving the
production compiler, CLI, artifact path, and stack VM unchanged. No FPAS syntax, semantics, or
language document changed.

## Implemented boundary

The development API accepts a functionless root program without imports or top-level declarations.
Within the root body it supports:

- Unit completion plus integer, real, boolean, and string constants;
- immutable and mutable scalar locals, assignment, lexical scopes, and expression temporaries;
- typed integer, real, boolean, and string operations, mixed integer/real conversion, and the
  generic-erased dynamic numeric operations already represented by P1/P2;
- `if`, scalar `case` labels/ranges/guards/bindings/`else`, `while`, `repeat`, `for` in both
  directions, `break`, and `continue`;
- root `return` without a value and `panic` with the existing diagnostic contract.

Imports, globals, calls, functions, closures, aggregates, intrinsics, hosted runtimes, tasks, and
persistent register artifacts remain assigned to P4 and later phases. An attempted unsupported
lowering returns a structured compiler diagnostic; there is no public CLI flag that exposes the
partial backend.

## Compiler pipeline

`fpas-compiler` now exposes two Rust development entry points:

1. `lower_register_subset` runs the existing semantic analysis and lowers its expression types and
   scalar-case binding metadata to `fpas-ir`.
2. `compile_register_subset` validates that IR, computes deterministic reverse-postorder blocks,
   allocates registers, selects packed instructions, builds sparse metadata, and returns only a
   `VerifiedExecutable`.

The implementation is split by concern:

```text
crates/fpas-compiler/src/
  lowering/
    mod.rs          analysis boundary and root construction
    context.rs      CFG, scope, local, and loop state
    types.rs        compact semantic-to-IR scalar types
    expr.rs         typed scalar expressions
    stmt.rs         declarations, assignment, return, panic
    control_flow.rs if/while/repeat/for and loop control
    case.rs         scalar case labels, ranges, guards, bindings
  bytecode/
    mod.rs          validated executable construction
    blocks.rs       deterministic block layout
    allocation.rs   local pinning and linear-scan temporary reuse
    selection.rs    IR operation to opcode selection
    metadata.rs     deterministic constants, strings, source runs
```

Explicit FPAS locals receive stable pinned registers. Temporary live intervals are allocated in
deterministic value order to the lowest free register. A temporary is released after its last use,
including branch and terminator operands. Block addresses are computed from selected instruction
widths before emission, so branch encoding is deterministic and independent of hash iteration.

Constants and strings are interned by first deterministic encounter. The emitted source map contains
only changed source runs. The compiler records statement-level locations for diagnostic terminators,
but ordinary dispatch never resolves a source path or line.

## Register interpreter

`fpas-vm::RegisterVm` accepts only `VerifiedExecutable`. Each VM owns a fresh register window while
the immutable executable may be shared through `Arc`. Instances are single-use, matching the
existing VM lifecycle boundary.

The implementation has one exhaustive `Opcode` match in `vm/register/dispatch.rs`. It decodes each
instruction once and passes decoded ABC or ABx operands to focused scalar, comparison, and dynamic
handlers. The worker stores the current packed instruction address on every dispatch. Only error
construction performs the sparse source-map lookup.

P3 opcodes execute directly. An opcode assigned to a later phase can be verifier-valid but produces
an internal invariant diagnostic if it reaches this deliberately partial interpreter. Malformed
registers, constants, branch targets, functions, and source metadata cannot enter through the public
API because `VerifiedExecutable` is the admission type.

## Preservation and regression evidence

The compiler regressions under `crates/fpas-compiler/src/tests/register_subset/` run the same parsed
program through the production stack compiler/VM and the inactive register compiler/VM. They cover:

- scalar locals, temporaries, assignments, conversions, wrapping integer edges, strings, and
  boolean/bitwise operations;
- nested `while`, `repeat`, `for to`, `for downto`, `break`, and `continue`;
- scalar case values, ranges, guards, bindings, and `else`;
- matching success or matching diagnostic code/message/help/line/column;
- deterministic IR and bytecode, verifier admission, temporary reuse, and a fixed small-program
  instruction count.

Direct interpreter tests under `crates/fpas-vm/src/vm/register/tests/` do not depend on compiler
selection. They cover all P3 typed scalar families, dynamic mixed integer/real arithmetic and
ordering, left/right destination aliasing, dispatched-instruction counts, division/modulo failure,
dynamic type failure, later-phase opcode rejection, shared-image isolation, and the single-use
lifecycle. `fpas-ir` validation adds positive and negative unary operand/result cases.

Focused results at implementation close:

```text
cargo test -p fpas-ir --locked                                      19 passed
cargo test -p fpas-compiler register_subset --locked               11 passed
cargo test -p fpas-vm vm::register::tests --locked                  9 passed
cargo clippy -p fpas-ir -p fpas-bytecode -p fpas-compiler \
  -p fpas-vm --all-targets --locked -- -D warnings                  passed
```

The phase-close gate also ran `cargo fmt --all -- --check`, `cargo build --workspace --locked`,
`cargo test --workspace --locked`, workspace all-target/all-feature clippy, FPAS source formatting,
and `fpas test tests/`. All passed.

## Dispatch viability measurement

`crates/fpas-vm/examples/register_dispatch_micro.rs` is a deterministic register-only decrement loop.
It reports dispatched instructions and wall time without writing a benchmark snapshot or claiming a
production speedup. Reproduce it with:

```text
cargo run -p fpas-vm --example register_dispatch_micro --release --locked -- 10000000
```

One implementation-close run dispatched 20,000,003 instructions in 210.184 ms, or 95.155 million
instructions per second. This single-machine sample establishes that the safe packed dispatch path
is viable; it is not comparable to the production benchmark baseline because the CLI does not use
the register path yet. Consequently `docs/bench/history.md` remains unchanged.

## Portability and production status

The slice adds no native backend, executable memory, pointer-bearing bytecode, target triple, or
host-sized serialized field. It relies on the fixed-width, verifier-backed P2 executable and safe
Rust operations, so it preserves the planned Rust/crate target envelope. Cross-host `.fpascp`
interchange is still unimplemented because P9 owns the register artifact codec and CLI cutover.

The current production path remains the stack compiler, linker, artifact encoding, and stack VM.
P4 can begin by adding calls, frame windows, closures, and callbacks to this validated pipeline.
