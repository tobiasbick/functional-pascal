# Traceability and acceptance matrix

This matrix is the authoritative completion map. Implementation agents replace `planned` with concrete
paths and test names as work lands. An item is complete only when code, verification, and evidence all
exist.

## Architecture requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-ARCH-001 | Typed target-independent CFG IR | `crates/fpas-ir` | `crates/fpas-ir/tests/validation.rs`: 19 focused positive, negative, and boundary tests, including P3 unary typing, loop backedges, semantic source spans, and maximum IDs; `cargo test -p fpas-ir` passed | complete |
| PVM-ARCH-002 | Exactly 8-byte packed instruction | `fpas-bytecode/instruction.rs` | `register_bytecode::instruction`: 94-opcode exhaustive inventory, ABC/ABx/Ax round trips, malformed forms, and `size_of::<Instruction>() == 8` | complete |
| PVM-ARCH-003 | One exhaustive opcode dispatch | `fpas-vm/vm/register/dispatch.rs` | exhaustive P5 opcode match; direct register-VM tests cover scalar/control-flow, calls, closures, callbacks, globals, layouts, aggregates, wrappers, limits, diagnostics, and lifecycle | complete through P5 |
| PVM-ARCH-004 | Per-function register windows | bytecode function metadata + VM frames | register VM direct/recursive/limit and aggregate-window cases plus compiler contiguous call-window selection | complete through P5 |
| PVM-ARCH-005 | Deterministic linear-scan allocation | `fpas-compiler/bytecode/allocation.rs` | `register_subset::structure` proves deterministic allocation; P5 reserves contiguous constructor, update, and call windows | complete through P5 |
| PVM-ARCH-006 | No final stack compiler/VM path | compiler/bytecode/VM crates | zero old-symbol search hits | planned |
| PVM-ARCH-007 | Cranelift absent/deferred | workspace manifests | P0 `rg` found no Cranelift/JIT/AOT manifest or Rust-source reference; continued enforcement required | complete |
| PVM-ARCH-008 | Safe Rust execution/codec | bytecode/program/VM | P2 model/verifier and P3-P5 compiler/interpreter use no `unsafe`, `transmute`, unchecked narrowing, or production panic for input; artifact codec proof remains in P9 | complete through P5 |

## Runtime lookup requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-ID-001 | Direct calls use `FunctionId` | compiler/linker/VM | compiler function differential tests and VM direct/wrong-ID tests | complete through P4 |
| PVM-ID-002 | First-class functions retain numeric target | bytecode value + VM | named/anonymous closure, callback, task-bound, and capture tests | complete through P4 |
| PVM-ID-003 | Globals use dense `GlobalId` slots | linker + shared runtime | compiler global differential test; direct dense/immutable register-global tests; `register_global_access` | complete through P5 development path |
| PVM-ID-004 | Record fields use layout slots | layouts + aggregate runtime | defaults/get/set/update/nested/shared-layout/COW tests; invalid-slot tests; `register_record_update` | complete through P5 development path |
| PVM-ID-005 | Enum type/variant tests use IDs | layouts + enum runtime | simple/data enum construction, case match, destructuring, associated-field, and invalid-slot tests | complete through P5 development path |
| PVM-ID-006 | Intrinsics use validated IDs and register ABI | compiler/bytecode/VM | `Intrinsic::all()`, explicit ID/uniqueness tests, canonical compiler-catalog coverage, verifier rejection, and one ABC register-window convention | complete through P6 development path |
| PVM-ID-007 | Names remain diagnostic metadata only | linker/runtime formatting | P5 positional operations carry IDs/slots; names are shared once for diagnostics/display; equality/display tests cover legacy and positional values | complete through P5 development path |

## Semantic preservation requirements

| ID | Requirement | Primary test families | Required evidence | State |
|---|---|---|---|---|
| PVM-SEM-001 | FPAS syntax accepted/rejected unchanged | parser/sema/compiler suites | no parser/lexer/grammar or `docs/pascal/language/` change; existing syntax and semantic suites retained | complete through P5 |
| PVM-SEM-002 | Evaluation order unchanged | compiler + FPAS effect tests | stack/register differential tests include nested properties and receiver-before-value event/property effects | complete through P5 subset |
| PVM-SEM-003 | Integer and real behavior unchanged | VM numeric tests | wrapping integer edges, mixed numeric conversion, typed/dynamic arithmetic, divide/modulo diagnostics, comparisons, and aliasing covered directly and differentially | complete through P3 subset |
| PVM-SEM-004 | Functions/procedures/methods unchanged | compiler/VM function tests | P4 call coverage plus P5 instance/static/generic methods, properties, events, and bound-method values | complete through P5 subset |
| PVM-SEM-005 | Closure/capture semantics unchanged | closure and nested routine tests | named/anonymous, immutable/mutable cell, nested, capture-order, and task-bound cases | complete through P4 subset |
| PVM-SEM-006 | Aggregate value/COW semantics unchanged | array/dict/record/enum tests | stack/register differential collection and record tests; direct shared-layout/COW tests; positional/legacy equality and display | complete through P5 subset |
| PVM-SEM-007 | Result/Option behavior unchanged | compiler/VM/FPAS tests | construction/equality, test, unwrap payload/error, pattern, and `try` success/early-return cases | complete through P5 subset |
| PVM-SEM-008 | Std and hosted APIs unchanged | intrinsic/Std/Graph/TUI suites | borrowed stack/register shared decoder, compiler differential intrinsic/callback tests, direct Console/Test/headless Graph tests, existing Std and production FPAS suites | complete through P6 development path |
| PVM-SEM-009 | Task scheduling/results unchanged | concurrency/pool/runtime suites | stress, wait, sleep, shutdown, panic | planned |
| PVM-SEM-010 | Diagnostic codes/locations/help preserved | compiler/VM/CLI negative tests | prior differential diagnostics plus P5 missing array/dictionary index codes, immutable globals, wrong unwrap variant, and malformed layout admission | complete through P5 subset |

## Artifact and portability requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-FMT-001 | Explicit fixed-width little-endian codec | `fpas-program/format` | wire and endian tests | planned |
| PVM-FMT-002 | No pointer-width/host metadata | object/program codecs | schema review + 32/64-bit compile evidence | planned |
| PVM-FMT-003 | Bounded section decoder | program format | truncation/mutation/limit tests | planned |
| PVM-FMT-004 | Deterministic bytes across hosts | compiler/linker/program | canonical digest and producer digests | planned |
| PVM-FMT-005 | Sparse source map | bytecode/program/VM diagnostics | P2 sparse-map validation plus P3 metadata run coalescing and diagnostic-only lookup; direct VM failures resolve line 41/column 7 while ordinary dispatch does not query metadata | complete through P3 |
| PVM-FMT-006 | Verifier before VM | bytecode/program/VM constructors | `compile_register_subset` returns `VerifiedExecutable`; `RegisterVm` accepts no unverified image; compiler, call-window, function-range, closure, and direct VM admission tests pass | complete through P4 |
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
| PVM-QUAL-001 | Focused module/file layout | P5 adds responsibility-named aggregate lowering/selection/execution, layout, bound-method, wrapper-validation, and CFG-state modules; every changed/new Rust file is at or below 500 lines | complete through P5 |
| PVM-QUAL-002 | Public Rust documentation complete | Public compiler, value-layout, benchmark-engine, and register VM APIs have `///` documentation | complete through P5 |
| PVM-QUAL-003 | Structured errors, no production panic for inputs | P3-P5 reject unsupported constructs, malformed aggregate types/layouts, runtime bounds, and resource failures through diagnostics; no new production `unsafe`, `unwrap()`/`expect()`, `panic!`, `todo!`, `unimplemented!`, `transmute`, or unchecked narrowing | complete through P5 |
| PVM-QUAL-004 | No dead compatibility path | dependency/symbol/file search | planned |
| PVM-QUAL-005 | Current user docs reconciled | no language or production behavior changed; `docs/pascal/` remains unchanged and P5 truth is recorded only under `docs/future/` | complete through P5 |
| PVM-QUAL-006 | Full Rust verification | P5 targeted and full-gate results are recorded in `p5-globals-aggregates.md` | complete through P5 |
| PVM-QUAL-007 | Full FPAS verification | FPAS formatting and regression gates retained; three new benchmark sources are formatter-checked | complete through P5 |
| PVM-QUAL-008 | Privacy preserved | P5 docs, fixtures, metadata, and benchmark output contain no host-identifying metadata | complete through P5 |
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
| `SpawnTask` | `SpawnTask(dst, callee, arg_base, arg_count)` | P7; spawn/wait/scheduler tests | planned |
| `SpawnDetachedTask` | `SpawnDetachedTask(callee, arg_base, arg_count)` | P7; detached task tests | planned |
| `Yield` | `Yield` scheduler operation | P7; timeslice/yield tests | planned |

## Final sign-off

Before declaring completion, every row above must be `complete`, `not applicable` with a precise
reason, or `unverified` only for a genuinely unavailable native platform. Performance threshold
exceptions require explicit user acceptance.
