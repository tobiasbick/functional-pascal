# Traceability and acceptance matrix

This matrix is the authoritative completion map. Implementation agents replace `planned` with concrete
paths and test names as work lands. An item is complete only when code, verification, and evidence all
exist.

## Architecture requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-ARCH-001 | Typed target-independent CFG IR | `crates/fpas-ir` | `crates/fpas-ir/tests/validation.rs`: 14 focused positive, negative, and boundary tests; `cargo test -p fpas-ir` passed | complete |
| PVM-ARCH-002 | Exactly 8-byte packed instruction | `fpas-bytecode/instruction.rs` | size + all-form round-trip tests | planned |
| PVM-ARCH-003 | One exhaustive opcode dispatch | `fpas-vm/vm/dispatch.rs` | opcode inventory + VM tests | planned |
| PVM-ARCH-004 | Per-function register windows | bytecode function metadata + VM frames | calls/recursion/window edge tests | planned |
| PVM-ARCH-005 | Deterministic linear-scan allocation | `fpas-compiler/bytecode/allocation.rs` | deterministic and max-register tests | planned |
| PVM-ARCH-006 | No final stack compiler/VM path | compiler/bytecode/VM crates | zero old-symbol search hits | planned |
| PVM-ARCH-007 | Cranelift absent/deferred | workspace manifests | P0 `rg` found no Cranelift/JIT/AOT manifest or Rust-source reference; continued enforcement required | complete |
| PVM-ARCH-008 | Safe Rust execution/codec | bytecode/program/VM | no new `unsafe`; lint/test gates | planned |

## Runtime lookup requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-ID-001 | Direct calls use `FunctionId` | compiler/linker/VM | direct-call and wrong-ID tests; profile | planned |
| PVM-ID-002 | First-class functions retain numeric target | bytecode value + VM | closure/callback/task tests | planned |
| PVM-ID-003 | Globals use dense `GlobalId` slots | linker + shared runtime | initialization/read/write/concurrency tests | planned |
| PVM-ID-004 | Record fields use layout slots | layouts + aggregate runtime | get/set/update/default/COW tests | planned |
| PVM-ID-005 | Enum type/variant tests use IDs | layouts + enum runtime | construction/match/destructure tests | planned |
| PVM-ID-006 | Intrinsics use validated IDs and register ABI | compiler/bytecode/VM | exhaustive intrinsic inventory | planned |
| PVM-ID-007 | Names remain diagnostic metadata only | linker/runtime formatting | search/profile and formatting tests | planned |

## Semantic preservation requirements

| ID | Requirement | Primary test families | Required evidence | State |
|---|---|---|---|---|
| PVM-SEM-001 | FPAS syntax accepted/rejected unchanged | parser/sema/compiler suites | no grammar change; full suite | planned |
| PVM-SEM-002 | Evaluation order unchanged | compiler + FPAS effect tests | differential output/files | planned |
| PVM-SEM-003 | Integer and real behavior unchanged | VM numeric tests | edge diagnostics and bit cases | planned |
| PVM-SEM-004 | Functions/procedures/methods unchanged | compiler/VM function tests | recursion/return/arity coverage | planned |
| PVM-SEM-005 | Closure/capture semantics unchanged | closure and nested routine tests | mutable/immutable/task-bound cases | planned |
| PVM-SEM-006 | Aggregate value/COW semantics unchanged | array/dict/record/enum tests | clone/mutate/equality/order/display | planned |
| PVM-SEM-007 | Result/Option behavior unchanged | compiler/VM/FPAS tests | wrap/unwrap/combinator/error cases | planned |
| PVM-SEM-008 | Std and hosted APIs unchanged | intrinsic/Std/Graph/TUI suites | exhaustive intrinsic and headless tests | planned |
| PVM-SEM-009 | Task scheduling/results unchanged | concurrency/pool/runtime suites | stress, wait, sleep, shutdown, panic | planned |
| PVM-SEM-010 | Diagnostic codes/locations/help preserved | compiler/VM/CLI negative tests | differential structured diagnostics | planned |

## Artifact and portability requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-FMT-001 | Explicit fixed-width little-endian codec | `fpas-program/format` | wire and endian tests | planned |
| PVM-FMT-002 | No pointer-width/host metadata | object/program codecs | schema review + 32/64-bit compile evidence | planned |
| PVM-FMT-003 | Bounded section decoder | program format | truncation/mutation/limit tests | planned |
| PVM-FMT-004 | Deterministic bytes across hosts | compiler/linker/program | canonical digest and producer digests | planned |
| PVM-FMT-005 | Sparse source map | bytecode/program/VM diagnostics | compression and lookup edge tests | planned |
| PVM-FMT-006 | Verifier before VM | bytecode/program/VM constructors | malformed executable rejection | planned |
| PVM-FMT-007 | Old artifacts rejected/rebuilt | build/CLI | direct error + project rebuild tests | planned |
| PVM-FMT-008 | Source-less `.fpascp` execution | CLI/runner | sources/manifests removed run test | planned |
| PVM-FMT-009 | Native bundles remain host-specific | bundle/CLI | Windows and Linux native tests | planned |
| PVM-FMT-010 | Windows `.fpascp` runs on Linux | program/CLI | cross-host fixture output/exit match | planned |
| PVM-FMT-011 | x86/ARM artifact interchange | program/CLI | native cross-architecture pair | planned |
| PVM-FMT-012 | macOS/FreeBSD remain portable targets | workspace/runtime crates | native matrix or explicit unverified status | planned |

## Performance requirements

| ID | Requirement | Workload/evidence | Acceptance | State |
|---|---|---|---|---|
| PVM-PERF-001 | Trustworthy pre-change baseline | `cargo bench-fpas save register-vm-before` | P0 final 16-row snapshot in `.temp-data/bench/register-vm-before.json`, with two VM repeats | complete |
| PVM-PERF-002 | VM geometric mean improvement | full `vm` group, repeated | >= 1.5x | planned |
| PVM-PERF-003 | Integer loop improvement | `integer_loop`, repeated | >= 1.5x; 2x stretch | planned |
| PVM-PERF-004 | Direct-call improvement | `function_call`, repeated | >= 1.5x; 2x stretch | planned |
| PVM-PERF-005 | Record operation improvement | record workload, repeated | >= 1.25x | planned |
| PVM-PERF-006 | No collateral regression | full suite, repeated | no row worse than 10% | planned |
| PVM-PERF-007 | Low-end Linux evidence | Chromebook runs | measured or explicitly unverified | planned |
| PVM-PERF-008 | Settled history record | `docs/bench/history.md` | one honest final entry | planned |

## Quality and completion requirements

| ID | Requirement | Evidence | State |
|---|---|---|---|
| PVM-QUAL-001 | Focused module/file layout | P1 `fpas-ir` uses responsibility-named IR files plus `validate/operands/` concern slices; every new Rust source and test file is below 500 lines | complete for P1 |
| PVM-QUAL-002 | Public Rust documentation complete | `fpas-ir` uses `#![deny(missing_docs)]`; `cargo build -p fpas-ir` and crate clippy gate passed | complete for P1 |
| PVM-QUAL-003 | Structured errors, no production panic for inputs | `ValidationError`/`ValidationErrorKind` plus negative `fpas-ir` validation tests; no `unsafe`, `unwrap()`/`expect()` calls, or `panic!` in production `fpas-ir` sources | complete for P1 |
| PVM-QUAL-004 | No dead compatibility path | dependency/symbol/file search | planned |
| PVM-QUAL-005 | Current user docs reconciled | `docs/pascal/` diff + link search | planned |
| PVM-QUAL-006 | Full Rust verification | P1: `cargo fmt --all -- --check`; `cargo build -p fpas-ir`; `cargo test -p fpas-ir`; `cargo clippy -p fpas-ir --all-targets --locked -- -D warnings`; `cargo test --workspace` | complete for P1 |
| PVM-QUAL-007 | Full FPAS verification | fmt check + `fpas test tests/` | planned |
| PVM-QUAL-008 | Privacy preserved | P1 diff inspection found no host-identifying metadata; P1 is target- and host-independent | complete for P1 |
| PVM-QUAL-009 | Future plan removed after completion | current docs/tests contain durable truth | planned |

## P0 current-opcode migration inventory

P0 read `crates/fpas-bytecode/src/op.rs` and recorded every current `Op` variant. No rows are
grouped: even closely related instructions retain their own successor and preservation contract. The
listed test families are the current owner coverage to preserve and extend; the `State` remains
`planned` until the named implementation phase supplies the new code and tests.

| Current `Op` | Required register successor | Owner/tests to preserve | State |
|---|---|---|---|
| `Constant` | `LoadConstant(dst, ConstantId)` | P3; bytecode constant identity, compiler literals, VM scalar tests | planned |
| `Unit` | `LoadUnit(dst)` | P3; compiler basics and root-return tests | planned |
| `Pop` | eliminated by value liveness; explicit discard only | P3; compiler statement/expression tests | planned |
| `Dup` | `Move(dst, src)` only when liveness requires a second value | P3; alias and evaluation-order tests | planned |
| `GetLocal` | `ReadLocal(dst, LocalId)` then allocated register | P3; compiler local and VM local tests | planned |
| `SetLocal` | `WriteLocal(LocalId, src)` while preserving live source | P3; compiler mutable-local tests | planned |
| `SetLocalPop` | `WriteLocal(LocalId, src)` with dead source | P3; compiler mutable-local tests | planned |
| `IncLocal` | `AddInt` plus `WriteLocal` | P3; `for` loop regression family | planned |
| `DecLocal` | `SubInt` plus `WriteLocal` | P3; `downto` loop regression family | planned |
| `GetGlobal` | `LoadGlobal(dst, GlobalId)` | P5; VM globals and `global_access` | planned |
| `SetGlobal` | `StoreGlobal(GlobalId, src)` | P5; VM globals and concurrency globals | planned |
| `GlobalIndexSet` | global-slot aggregate path ending in `StoreGlobal` | P5; global array indexing tests | planned |
| `AddInt` | `AddInt(dst, left, right)` | P3; VM numeric wrapping tests | planned |
| `SubInt` | `SubInt(dst, left, right)` | P3; VM numeric wrapping tests | planned |
| `MulInt` | `MulInt(dst, left, right)` | P3; VM numeric tests | planned |
| `DivInt` | `DivInt(dst, left, right)` | P3; divide-by-zero and overflow tests | planned |
| `ModInt` | `ModInt(dst, left, right)` | P3; modulo-by-zero and overflow tests | planned |
| `AddReal` | `AddReal(dst, left, right)` | P3; VM real-operation tests | planned |
| `SubReal` | `SubReal(dst, left, right)` | P3; VM real-operation tests | planned |
| `MulReal` | `MulReal(dst, left, right)` | P3; VM real-operation tests | planned |
| `DivReal` | `DivReal(dst, left, right)` | P3; real divide-by-zero tests | planned |
| `NegateInt` | `NegateInt(dst, src)` | P3; minimum-integer negation test | planned |
| `NegateReal` | `NegateReal(dst, src)` | P3; VM real-operation tests | planned |
| `AddDyn` | `AddDyn(dst, left, right)` | P3; generic numeric compiler/VM tests and `dynamic_numeric` | planned |
| `SubDyn` | `SubDyn(dst, left, right)` | P3; generic numeric compiler/VM tests | planned |
| `MulDyn` | `MulDyn(dst, left, right)` | P3; generic numeric compiler/VM tests | planned |
| `DivDyn` | `DivDyn(dst, left, right)` | P3; generic division error tests | planned |
| `NegateDyn` | `NegateDyn(dst, src)` | P3; dynamic negation overflow test | planned |
| `EqDyn` | `EqDyn(dst, left, right)` | P3; dynamic aggregate equality tests | planned |
| `NeqDyn` | `NeqDyn(dst, left, right)` | P3; dynamic aggregate equality tests | planned |
| `LtDyn` | `LtDyn(dst, left, right)` | P3; dynamic ordering diagnostic tests | planned |
| `GtDyn` | `GtDyn(dst, left, right)` | P3; dynamic ordering diagnostic tests | planned |
| `LeDyn` | `LeDyn(dst, left, right)` | P3; dynamic ordering diagnostic tests | planned |
| `GeDyn` | `GeDyn(dst, left, right)` | P3; dynamic ordering diagnostic tests | planned |
| `ConcatStr` | `ConcatStr(dst, left, right)` | P3; string compiler and VM tests | planned |
| `Shl` | `ShlInt(dst, left, right)` | P3; shift-bound diagnostics | planned |
| `Shr` | `ShrInt(dst, left, right)` | P3; shift-bound diagnostics | planned |
| `BitAnd` | `BitAndInt(dst, left, right)` | P3; bitwise regression tests | planned |
| `BitOr` | `BitOrInt(dst, left, right)` | P3; bitwise regression tests | planned |
| `BitXor` | `BitXorInt(dst, left, right)` | P3; bitwise regression tests | planned |
| `EqInt` | `EqInt(dst, left, right)` | P3; typed comparison tests | planned |
| `NeqInt` | `NeqInt(dst, left, right)` | P3; typed comparison tests | planned |
| `LtInt` | `LtInt(dst, left, right)` | P3; typed comparison tests | planned |
| `GtInt` | `GtInt(dst, left, right)` | P3; typed comparison tests | planned |
| `LeInt` | `LeInt(dst, left, right)` | P3; typed comparison tests | planned |
| `GeInt` | `GeInt(dst, left, right)` | P3; typed comparison tests | planned |
| `EqReal` | `EqReal(dst, left, right)` | P3; IEEE-754 comparison tests | planned |
| `NeqReal` | `NeqReal(dst, left, right)` | P3; IEEE-754 comparison tests | planned |
| `LtReal` | `LtReal(dst, left, right)` | P3; IEEE-754 comparison tests | planned |
| `GtReal` | `GtReal(dst, left, right)` | P3; IEEE-754 comparison tests | planned |
| `LeReal` | `LeReal(dst, left, right)` | P3; IEEE-754 comparison tests | planned |
| `GeReal` | `GeReal(dst, left, right)` | P3; IEEE-754 comparison tests | planned |
| `EqStr` | `EqStr(dst, left, right)` | P3; Unicode/string comparison tests | planned |
| `NeqStr` | `NeqStr(dst, left, right)` | P3; Unicode/string comparison tests | planned |
| `LtStr` | `LtStr(dst, left, right)` | P3; string ordering tests | planned |
| `GtStr` | `GtStr(dst, left, right)` | P3; string ordering tests | planned |
| `LeStr` | `LeStr(dst, left, right)` | P3; string ordering tests | planned |
| `GeStr` | `GeStr(dst, left, right)` | P3; string ordering tests | planned |
| `EqBool` | `EqBool(dst, left, right)` | P3; boolean comparison tests | planned |
| `NeqBool` | `NeqBool(dst, left, right)` | P3; boolean comparison tests | planned |
| `Not` | `NotBool(dst, src)` | P3; boolean coercion tests | planned |
| `And` | `AndBool(dst, left, right)` | P3; short-circuit lowering tests | planned |
| `Or` | `OrBool(dst, left, right)` | P3; short-circuit lowering tests | planned |
| `IntToReal` | `IntToReal(dst, src)` | P3; conversion tests | planned |
| `Jump` | function-local `Jump(target)` terminator | P3; loop and branch CFG tests | planned |
| `JumpIfFalse` | `BranchIfFalse(condition, target)` terminator | P3; `if` and short-circuit tests | planned |
| `JumpIfTrue` | `BranchIfTrue(condition, target)` terminator | P3; `if` and short-circuit tests | planned |
| `JumpIfLocalGt` | `GtInt` plus branch terminator | P3; `for to` bound tests | planned |
| `JumpIfLocalLt` | `LtInt` plus branch terminator | P3; `for downto` bound tests | planned |
| `Call` | `CallDirect(dst, FunctionId, arg_base, arg_count)` | P4; function/recursion/arity tests and `function_call` | planned |
| `CallValue` | `CallValue(dst, callee, arg_base, arg_count)` | P4; first-class function tests | planned |
| `MakeClosure` | `MakeClosure(dst, FunctionId, capture_base, capture_count)` | P4; closure capture-order/task-bound tests | planned |
| `MakeCell` | `MakeCell(dst, src)` | P4; mutable capture tests | planned |
| `CellGet` | `CellRead(dst, cell)` | P4; mutable capture tests | planned |
| `CellSet` | `CellWrite(cell, src)` | P4; mutable capture tests | planned |
| `Return` | `Return(src)` terminator | P3/P4; root, early-return, and function tests | planned |
| `GetEnclosing` | capture/cell read using resolved capture slot | P4; nested closure tests | planned |
| `SetEnclosing` | capture/cell write using resolved capture slot | P4; nested mutable capture tests | planned |
| `MakeArray` | `MakeArray(dst, value_base, count)` | P5; array construction/COW tests | planned |
| `IndexGet` | `IndexGet(dst, collection, index)` | P5; array/dict/string index tests | planned |
| `IndexSet` | `IndexSet(dst, collection, index, value)` | P5; aggregate mutation tests | planned |
| `Contains` | `Contains(dst, value, collection)` | P5; array/dict/string membership tests | planned |
| `MakeDict` | `MakeDict(dst, pair_base, pair_count)` | P5; dictionary order/equality tests | planned |
| `MakeRecord` | `MakeRecord(dst, RecordTypeId, value_base)` | P5; record defaults/layout tests | planned |
| `FieldGet` | `LoadField(dst, record, RecordFieldId)` | P5; record field tests and `record_field_access` | planned |
| `FieldSet` | `StoreField(dst, record, RecordFieldId, value)` | P5; record field mutation tests | planned |
| `UpdateRecord` | `UpdateRecord(dst, record, override_base, count)` | P5; record-update/COW tests and `record_update` | planned |
| `Print` | `Intrinsic(dst, PrintId, arg_base, arg_count)` | P6; console output tests | planned |
| `PrintLn` | `Intrinsic(dst, PrintLnId, arg_base, arg_count)` | P6; console output tests | planned |
| `Intrinsic` | `Intrinsic(dst, IntrinsicId, arg_base, arg_count)` | P6; all-intrinsics inventory | planned |
| `ArrayPushLocal` | local/capture array mutation plus write-back | P5; local array mutation tests | planned |
| `ArrayPopLocal` | local/capture array mutation plus result | P5; local array mutation tests | planned |
| `Halt` | root `Return(Unit)` terminator | P3/P9; entry completion tests | planned |
| `Panic` | `Panic(src)` terminator | P3; runtime diagnostic tests | planned |
| `MakeOk` | `MakeOk(dst, src)` | P5; Result tests | planned |
| `MakeErr` | `MakeErr(dst, src)` | P5; Result tests | planned |
| `MakeSome` | `MakeSome(dst, src)` | P5; Option tests | planned |
| `MakeNone` | `MakeNone(dst)` | P5; Option tests | planned |
| `IsResultOk` | `IsResultOk(dst, src)` | P5; Result tests | planned |
| `IsOptionSome` | `IsOptionSome(dst, src)` | P5; Option tests | planned |
| `UnwrapOk` | `UnwrapOk(dst, src)` | P5; Result error tests | planned |
| `UnwrapErr` | `UnwrapErr(dst, src)` | P5; Result error tests | planned |
| `UnwrapSome` | `UnwrapSome(dst, src)` | P5; Option error tests | planned |
| `MakeEnum` | `MakeEnum(dst, EnumVariantId, value_base)` | P5; enum construction/match tests | planned |
| `IsVariant` | `TestVariant(dst, value, EnumVariantId)` | P5; enum type/variant tests | planned |
| `EnumField` | `LoadEnumField(dst, value, field)` | P5; enum associated-data tests | planned |
| `SpawnTask` | `SpawnTask(dst, callee, arg_base, arg_count)` | P7; spawn/wait/scheduler tests | planned |
| `SpawnDetachedTask` | `SpawnDetachedTask(callee, arg_base, arg_count)` | P7; detached task tests | planned |
| `Yield` | `Yield` scheduler operation | P7; timeslice/yield tests | planned |

## Final sign-off

Before declaring completion, every row above must be `complete`, `not applicable` with a precise
reason, or `unverified` only for a genuinely unavailable native platform. Performance threshold
exceptions require explicit user acceptance.
