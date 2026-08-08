# Traceability and acceptance matrix

This matrix is the authoritative completion map. Implementation agents replace `planned` with concrete
paths and test names as work lands. An item is complete only when code, verification, and evidence all
exist.

## Architecture requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-ARCH-001 | Typed target-independent CFG IR | `crates/fpas-ir` | `crates/fpas-ir/tests/validation.rs`: 19 focused positive, negative, and boundary tests, including P3 unary typing, loop backedges, semantic source spans, and maximum IDs; `cargo test -p fpas-ir` passed | complete |
| PVM-ARCH-002 | Exactly 8-byte packed instruction | `fpas-bytecode/instruction.rs` | `register_bytecode::instruction`: 94-opcode exhaustive inventory, ABC/ABx/Ax round trips, malformed forms, and `size_of::<Instruction>() == 8` | complete |
| PVM-ARCH-003 | One exhaustive opcode dispatch | `fpas-vm/src/vm/dispatch.rs` | exhaustive opcode match; direct, compiler, CLI, and full FPAS tests cover production execution | complete through P10 |
| PVM-ARCH-004 | Per-function register windows | bytecode function metadata + VM frames | register VM direct/recursive/limit and aggregate-window cases plus production compiler contiguous call-window selection | complete through P9 |
| PVM-ARCH-005 | Deterministic linear-scan allocation | `fpas-compiler/bytecode/allocation.rs` | deterministic structure tests plus cold/warm artifact byte equality | complete through P9 |
| PVM-ARCH-006 | No final stack compiler/VM path | compiler/bytecode/VM crates | P10 deletion inventory and zero production old-symbol search hits in `p10-stack-removal.md` | complete |
| PVM-ARCH-007 | Cranelift absent/deferred | workspace manifests | P0 `rg` found no Cranelift/JIT/AOT manifest or Rust-source reference; continued enforcement required | complete |
| PVM-ARCH-008 | Safe Rust execution/codec | bytecode/program/VM | P2 model/verifier, P3-P9 compiler/interpreter, and bounded P9 artifact codec use no `unsafe`, `transmute`, unchecked narrowing, or production panic for input | complete through P9 |

## Runtime lookup requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-ID-001 | Direct calls use `FunctionId` | compiler/linker/VM | compiler tests, P8 symbolic import relocation, VM direct/wrong-ID tests, and P9 production execution | complete through P9 |
| PVM-ID-002 | First-class functions retain numeric target | bytecode value + VM | named/anonymous closure, callback, task-bound, and capture tests | complete through P4 |
| PVM-ID-003 | Globals use dense `GlobalId` slots | linker + shared runtime | compiler test, cross-object relocation, dense/immutable runtime tests, `register_global_access`, and P9 production execution | complete through P9 |
| PVM-ID-004 | Record fields use layout slots | layouts + aggregate runtime | layout import/compatibility and relocation plus defaults/get/set/update/nested/shared-layout/COW, invalid-slot, and production tests | complete through P9 |
| PVM-ID-005 | Enum type/variant tests use IDs | layouts + enum runtime | enum/variant import and relocation plus construction, matching, destructuring, associated-field, invalid-slot, and production tests | complete through P9 |
| PVM-ID-006 | Intrinsics use validated IDs and register ABI | compiler/bytecode/VM | exhaustive IDs, compiler catalog, verifier rejection, ABC register windows, and P9 production FPAS suite | complete through P9 |
| PVM-ID-007 | Names remain diagnostic metadata only | linker/runtime formatting | positional operations carry IDs/slots; names are shared only for diagnostics/display; production equality/display tests pass | complete through P9 |

## Semantic preservation requirements

| ID | Requirement | Primary test families | Required evidence | State |
|---|---|---|---|---|
| PVM-SEM-001 | FPAS syntax accepted/rejected unchanged | parser/sema/compiler suites | no parser/lexer/grammar or `docs/pascal/language/` change; full workspace and FPAS suites retained | complete through P9 |
| PVM-SEM-002 | Evaluation order unchanged | compiler + FPAS effect tests | stack/register differential tests include nested properties and receiver-before-value event/property effects | complete through P5 subset |
| PVM-SEM-003 | Integer and real behavior unchanged | VM numeric tests | wrapping integer edges, mixed numeric conversion, typed/dynamic arithmetic, divide/modulo diagnostics, comparisons, and aliasing covered directly and differentially | complete through P3 subset |
| PVM-SEM-004 | Functions/procedures/methods unchanged | compiler/VM function tests | P4 call coverage plus P5 instance/static/generic methods, properties, events, and bound-method values | complete through P5 subset |
| PVM-SEM-005 | Closure/capture semantics unchanged | closure and nested routine tests | named/anonymous, immutable/mutable cell, nested, capture-order, and task-bound cases | complete through P4 subset |
| PVM-SEM-006 | Aggregate value/COW semantics unchanged | array/dict/record/enum tests | stack/register differential collection and record tests; direct shared-layout/COW tests; positional/legacy equality and display | complete through P5 subset |
| PVM-SEM-007 | Result/Option behavior unchanged | compiler/VM/FPAS tests | construction/equality, test, unwrap payload/error, pattern, and `try` success/early-return cases | complete through P5 subset |
| PVM-SEM-008 | Std and hosted APIs unchanged | intrinsic/Std/Graph/TUI suites | compiler intrinsic/callback tests, direct Console/Test/headless Graph tests, TUI regressions, and full production FPAS suite | complete through P9 |
| PVM-SEM-009 | Task scheduling/results unchanged | concurrency/pool/runtime suites | retained/detached, Wait/WaitAll, cooperative sleep, nested-frame/live-aggregate timeslice, task-bound capture, cancellation, VM, and production FPAS tests | complete through P9 |
| PVM-SEM-010 | Diagnostic codes/locations/help preserved | compiler/VM/CLI negative tests | prior differential diagnostics plus P5 missing array/dictionary index codes, immutable globals, wrong unwrap variant, and malformed layout admission | complete through P5 subset |

## Artifact and portability requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-FMT-001 | Explicit fixed-width little-endian codec | `fpas-program/format` | header/section wire tests and canonical byte digest | complete |
| PVM-FMT-002 | No pointer-width/host metadata | object/program codecs | P8 objects and P9 ten-section program image persist fixed-width fields only | complete |
| PVM-FMT-003 | Bounded section decoder | program format | exhaustive truncation, deterministic mutation, section topology, UTF-8/opcode/boolean, and configured-limit tests | complete |
| PVM-FMT-004 | Deterministic bytes across hosts | compiler/linker/program | cold/warm build equality and canonical `.fpascp` digest; additional native hosts remain unverified | complete on Windows x86-64; cross-host unverified |
| PVM-FMT-005 | Sparse source map | bytecode/program/VM diagnostics | P2 sparse-map validation plus P3 metadata run coalescing and diagnostic-only lookup; direct VM failures resolve line 41/column 7 while ordinary dispatch does not query metadata | complete through P3 |
| PVM-FMT-006 | Verifier before VM | bytecode/program/VM constructors | artifact decode returns `VerifiedExecutable`; `Vm` accepts no unverified image; compiler and VM admission tests pass | complete |
| PVM-FMT-007 | Old artifacts rejected/rebuilt | build/CLI | old `.fpascu` rebuild/replacement plus direct old `.fpascp` version and actionable rebuild-help tests | complete |
| PVM-FMT-008 | Source-less `.fpascp` execution | CLI/runner | decoded-program and CLI source/manifest removal tests | complete |
| PVM-FMT-009 | Native bundles remain host-specific | bundle/CLI | Windows x86-64 source-less native application tests; other hosts unverified | complete on Windows x86-64; other hosts unverified |
| PVM-FMT-010 | Windows `.fpascp` runs on Linux | program/CLI | cross-host fixture output/exit match | unverified; no Linux host in P9 run |
| PVM-FMT-011 | x86/ARM artifact interchange | program/CLI | native cross-architecture pair | unverified; no ARM host in P9 run |
| PVM-FMT-012 | macOS/FreeBSD remain portable targets | workspace/runtime crates | native matrix or explicit unverified status | unverified; no macOS or FreeBSD host in P9 run |

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
| PVM-QUAL-001 | Focused module/file layout | P10 file inventory in `p10-stack-removal.md`; no changed production Rust file exceeds 500 lines | complete through P10 |
| PVM-QUAL-002 | Public Rust documentation complete | public artifact, build, compiler, runner, bytecode, aggregate-factory, and VM APIs retain complete `///` documentation | complete through P10 |
| PVM-QUAL-003 | Structured errors, no production panic for inputs | P3-P10 reject unsupported constructs, malformed operands/layouts, runtime bounds, task boundaries, object/import/relocation/initializer failures, and artifact resource/format errors through diagnostics | complete through P10 |
| PVM-QUAL-004 | No dead compatibility path | dependency/symbol/file search | P10 deletion inventory and focused symbol searches in `p10-stack-removal.md` | complete |
| PVM-QUAL-005 | Current user docs reconciled | compiled-program and concurrency implementation pages describe the current artifact/VM paths; language/Std APIs unchanged | complete through P10 |
| PVM-QUAL-006 | Full Rust verification | P10 fmt, build, workspace-test, and all-target Clippy results are recorded in `p10-stack-removal.md` | complete through P10 |
| PVM-QUAL-007 | Full FPAS verification | P10 full FPAS regression result is recorded in `p10-stack-removal.md` | complete through P10 |
| PVM-QUAL-008 | Privacy preserved | P10 docs, fixtures, artifact metadata, and test output contain no host-identifying metadata | complete through P10 |
| PVM-QUAL-009 | Future plan removed after completion | current docs/tests contain durable truth | planned |

## P3 opcode implementation overlay

P3 supplies inactive, verifier-gated successors for the following migration groups. This overlay is
the phase-completion state; the P0 inventory below keeps `planned` in its final-migration column until
the old production instruction is removed at cutover.

| P3 group | Register implementation | Evidence | State |
|---|---|---|---|
| Constants, Unit, locals, discard/copy elimination | `LoadConstant`, `LoadUnit`, allocated value registers, pinned local registers | compiler differential tests plus fixed instruction-count and temporary-reuse assertions | complete |
| Integer scalar operations | add/subtract/multiply/divide/remainder/negate, shifts, bitwise operations, six comparisons | direct VM opcode-family and domain-edge tests; compiler wrapping/bitwise differential case | complete |
| Real scalar operations | add/subtract/multiply/divide/negate, six comparisons, integer conversion | direct VM opcode-family tests; compiler mixed numeric differential case | complete |
| Dynamic numeric operations | add/subtract/multiply/divide/negate, equality and four ordered comparisons | direct VM mixed integer/real, type mismatch, and domain-edge tests | complete |
| String and boolean operations | concatenation, six string comparisons, boolean equality/inequality/not/and/or | direct VM opcode-family tests; compiler string/boolean differential case | complete |
| Scalar control flow | jump, true/false branches, `if`, scalar `case`, while/repeat/for, break/continue | nested compiler differential case and dispatched-instruction count | complete |
| Root completion and panic | Unit return and panic terminators | success plus code/message/help/line/column differential tests | complete |

## P5 opcode implementation overlay

Like the P3 overlay, this records the inactive implementation. The P0 inventory below remains
`planned` until the production stack operation is removed at cutover.

| P5 group | Register implementation | Evidence | State |
|---|---|---|---|
| Dense globals | `LoadGlobal` and `StoreGlobal` with `GlobalId`; `RwLock<Vec<Option<Value>>>` | compiler differential global mutation; direct initialization/immutable-store tests; register benchmark | complete |
| Arrays and dictionaries | `MakeArray`, `MakeDictionary`, `IndexGet`, `IndexSet`, and `Contains` | construction/index/update/membership differential tests; COW and missing index/key direct tests; register array benchmark | complete |
| Record layouts | `MakeRecord`, `LoadField`, `StoreField`, and `UpdateRecord` with numeric slots | anonymous/named/default/generic/nested/COW/member differential tests; direct shared-layout test; verifier negatives; register record benchmark | complete |
| Enum layouts | `MakeEnum`, `TestVariant`, and `LoadEnumField` with numeric variants/fields | simple/data/generic enum construction, matching, destructuring, associated fields, and invalid-slot tests | complete |
| Result and Option | `MakeOk`, `MakeError`, `MakeSome`, `MakeNone`, tests, and unwrap operations | equality, pattern, `try`, payload, wrong-wrapper, and wrong-variant tests | complete |
| Record members | numeric direct calls plus generated bound-receiver closures | instance/static/generic method, property, event, bound method/event, and evaluation-order differential tests | complete |

## P0 current-opcode migration inventory

P0 read `crates/fpas-bytecode/src/op.rs` and recorded every current `Op` variant. No rows are
grouped: even closely related instructions retain their own successor and preservation contract. The
listed test families are the current owner coverage to preserve and extend. Its `State` column tracks
final production migration and old-op deletion, not the inactive phase overlay above.

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
| `Call` | `CallDirect(dst, FunctionId, arg_base, arg_count)` | compiler function differential tests; VM direct/recursive/arity tests | complete through P4 |
| `CallValue` | `CallValue(dst, callee, arg_base, arg_count)` | named first-class and closure differential/direct VM tests | complete through P4 |
| `MakeClosure` | `MakeClosure(dst, FunctionId, capture_base, capture_count)` | anonymous/nested capture and task-bound tests | complete through P4 |
| `MakeCell` | `MakeCell(dst, src)` | mutable anonymous and direct cell tests | complete through P4 |
| `CellGet` | `CellRead(dst, cell)` | mutable repeated-call tests | complete through P4 |
| `CellSet` | `CellWrite(cell, src)` | mutable repeated-call tests | complete through P4 |
| `Return` | `Return(src)` terminator | root, early-return, procedure, function, recursion tests | complete through P4 |
| `GetEnclosing` | capture/cell read using resolved capture slot | nested and escaping closure tests | complete through P4 |
| `SetEnclosing` | capture/cell write using resolved capture slot | enclosing cell representation and mutable closure tests | complete through P4 |
| `MakeArray` | `MakeArray(dst, value_base, count)` | P5; array construction/COW tests | planned |
| `IndexGet` | `IndexGet(dst, collection, index)` | P5; array/dict/string index tests | planned |
| `IndexSet` | `IndexSet(dst, collection, index, value)` | P5; aggregate mutation tests | planned |
| `Contains` | `Contains(dst, value, collection)` | P5; array/dict/string membership tests | planned |
| `MakeDict` | `MakeDict(dst, pair_base, pair_count)` | P5; dictionary order/equality tests | planned |
| `MakeRecord` | `MakeRecord(dst, RecordTypeId, value_base)` | P5; record defaults/layout tests | planned |
| `FieldGet` | `LoadField(dst, record, RecordFieldId)` | P5; record field tests and `record_field_access` | planned |
| `FieldSet` | `StoreField(dst, record, RecordFieldId, value)` | P5; record field mutation tests | planned |
| `UpdateRecord` | `UpdateRecord(dst, record, override_base, count)` | P5; record-update/COW tests and `record_update` | planned |
| `Print` | `Intrinsic(dst, PrintId, arg_base, arg_count)` | P6; Console output and shared hosted-state tests | complete through P6 development path |
| `PrintLn` | `Intrinsic(dst, PrintLnId, arg_base, arg_count)` | P6; Console output and shared hosted-state tests | complete through P6 development path |
| `Intrinsic` | `Intrinsic(dst, IntrinsicId, arg_base, arg_count)` | P6; exhaustive decoder/list/catalog inventory, verifier and register VM tests | complete through P6 development path |
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
| `SpawnTask` | `SpawnTask(dst, callee, arg_base, arg_count)` | retained spawn, Wait/WaitAll, sleep, timeslice, capture-boundary tests | complete through P7 development path |
| `SpawnDetachedTask` | `SpawnDetachedTask(callee, arg_base, arg_count)` | compiler differential detached-pool test | complete through P7 development path |
| `Yield` | `Yield` scheduler operation | main yield direct test and automatic spawned-task timeslice suspension | complete through P7 development path |

## Final sign-off

Before declaring completion, every row above must be `complete`, `not applicable` with a precise
reason, or `unverified` only for a genuinely unavailable native platform. Performance threshold
exceptions require explicit user acceptance.
