# P2 register bytecode implementation

## Scope and production status

P2 adds the complete in-memory register executable contract and verifier to `fpas-bytecode`. It does
not switch the compiler, linker, program format, CLI, or VM to the new representation. Production
execution still uses `Chunk`, `Op`, and `BYTECODE_VERSION = 1`; the inactive register contract uses
`REGISTER_BYTECODE_VERSION = 2` until the artifact cutover in P9.

This is an internal runtime/tooling refactor. FPAS syntax, semantics, diagnostics, and current pages
under `docs/pascal/` are unchanged.

## Implemented ownership

```text
crates/fpas-bytecode/src/
  chunk_validate.rs                 temporary stack-Chunk verifier, moved without behavior changes
  instruction.rs                    packed Instruction(u64), Opcode, and checked form codecs
  instruction/                      decoded form and codec-error concerns
  operand.rs                        fixed-width register and table identifiers
  function.rs                       code ranges, frame metadata, returns, and flags
  executable.rs                     Executable candidate and VerifiedExecutable proof wrapper
  limits.rs                         shared resource limits for later builder/codec reuse
  metadata/                         constants, strings, globals, layouts, and sparse source maps
  validate/                         verifier coordinator and responsibility-specific checks

crates/fpas-bytecode/tests/
  register_bytecode.rs              integration-test root
  register_bytecode/support.rs      compiler-independent executable builder and fixtures
  register_bytecode/instruction.rs  packed form, inventory, boundary, and size tests
  register_bytecode/executable.rs   positive executable, source lookup, and constant identity tests
  register_bytecode/verifier.rs     malformed executable matrix
```

No new production Rust source or P2 integration-test file exceeds 500 lines. The pre-existing stack
modules remain available under their existing public exports until the later production cutover.

## Packed instruction contract

`Instruction` is `#[repr(transparent)]` over `u64`; tests assert its size is exactly eight bytes on
every test target. The low byte is an explicit `#[repr(u8)]` `Opcode`. The remaining bytes use one of
three opcode-declared forms:

| Form | Checked constructor | Checked accessor | Payload |
|---|---|---|---|
| ABC | `Instruction::abc` | `abc_operands` | `u16 A`, `u16 B`, `u16 C`, `u8 auxiliary` |
| ABx | `Instruction::abx` | `abx_operands` | `u16 A`, `u32 Bx` |
| Ax | `Instruction::ax` | `ax_operand` | one logical 48-bit value |

Packing uses shifts from widened fixed-width inputs. Decoding uses `to_le_bytes` plus
`from_le_bytes`, so no narrowing cast, native struct serialization, alignment assumption,
`transmute`, or `unsafe` is involved. `Instruction::from_word` deliberately creates an untrusted
candidate; unknown discriminants and the currently reserved Ax opcode are rejected by verification.

`Opcode::ALL` contains all 94 assigned discriminants. The exhaustive test independently decodes all
256 possible low bytes and compares the result with this table, so adding an enum discriminant
without inventory coverage fails the count/equality assertion.

## Operand and operation conventions

All cross-table and frame operands use transparent fixed-width newtypes. `Register` uses `u16`, with
`u16::MAX` reserved as `NO_REGISTER`; no valid register may encode that sentinel. Functions, record
types, record fields, enum types, enum variants, and intrinsic identifiers use `u16`. Constants,
strings, globals, instruction addresses, and source identifiers use `u32`. Collection-index
conversion uses `try_from` and returns `OperandError` instead of truncating.

The opcode inventory covers the scalar, control-flow, call/closure, global, aggregate,
Result/Option, intrinsic, and task operations required by P3 through P7. Operations that logically
produce a copy-on-write aggregate but need four register operands use an in-place bytecode
convention: the first operand is both input and result for `IndexSet`, `StoreField`, and
`UpdateRecord`. Later code generation first moves the source aggregate to the desired destination
when the destination differs. This preserves value semantics without limiting one register operand
to the eight-bit auxiliary field.

Unused fields have one canonical encoding and are verified as zero or `NO_REGISTER`. Calls,
closures, intrinsics, arrays, dictionaries, records, enums, and tasks use validated contiguous
register windows; checked arithmetic occurs before comparing their end with the frame size.

## Executable metadata

An untrusted `Executable` owns one instruction vector and dense ordered tables for functions,
constants, strings, globals, record layouts, enum layouts, and executable-wide enum variants.
Persistent constants retain exact `i64` or `f64` bit identity, including distinct NaN payloads and
signed zero. Runtime-only aggregates and host resources are absent from the new constant type.

Each `FunctionInfo` declares a nonempty half-open `CodeRange`, name ID, arity, capture count, register
count, return convention, and task-spawn flag. Function zero is the only accepted root entry.
Functions partition the instruction vector in dense ID order without overlap or gaps.

`SourceMap` stores source-path string IDs plus sorted `SourceRun` values. Lookup uses binary search via
`partition_point` and returns the closest preceding run. Every function boundary must begin a run,
even when its location repeats the preceding effective location.

`Executable::verify` is the only constructor for `VerifiedExecutable`. Its executable is immutable
through the proof wrapper, preventing a future VM entry point from accidentally accepting an
unchecked candidate.

## Verifier coverage

The verifier rejects the following deterministic categories with `ValidationError`, including the
function ID/name, instruction address, decoded opcode, operand role, actual value, and valid bound
whenever those contexts apply:

- global and per-function resource-limit violations;
- missing/nonzero entry, a non-Unit/parameterized/capturing root, empty/out-of-range/overlapping/gapped
  function ranges, oversized capture counts, and undersized frames;
- unknown or reserved opcodes and noncanonical unused operands;
- sentinel or out-of-frame registers;
- missing constant, string, global, function, record, field, enum, variant, or intrinsic references;
- overflowing/out-of-frame argument, capture, aggregate, and intrinsic windows;
- wrong direct-call arity and return destinations;
- branch targets outside the current function and reachable fallthrough past its end;
- return operands that disagree with function metadata;
- invalid layout ownership and positional field/variant slots;
- duplicate strings and invalid metadata string references;
- capturing or task-bound values in the non-capturing persistent function-constant form;
- unsorted, duplicate, out-of-code, zero-position, or invalid-source runs and missing function runs;
- task-spawn flags that disagree with emitted operations.

The test-only builder emits a fixture containing every active opcode without using the compiler. The
verifier accepts that complete fixture and the minimal fixture, while focused mutations exercise each
invalid operand category. Tests also assert `size_of::<Value>() <= 16` so the pre-existing compact
runtime representation remains an explicit cross-target gate.

## P2 verification commands

The phase gate uses:

```text
cargo fmt --all -- --check
cargo build -p fpas-bytecode --locked
cargo test -p fpas-bytecode --locked
cargo clippy -p fpas-bytecode --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
git diff --check
```

Because no `.fpas` file and no production VM/compiler path changes in P2, no new FPAS regression
program or benchmark is warranted. The full existing workspace and FPAS suites remain the regression
gate for proving that the side-by-side model does not change current behavior.
