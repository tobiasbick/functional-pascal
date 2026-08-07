# Traceability and acceptance matrix

This matrix is the authoritative completion map. Implementation agents replace `planned` with concrete
paths and test names as work lands. An item is complete only when code, verification, and evidence all
exist.

## Architecture requirements

| ID | Requirement | Primary owner | Required evidence | State |
|---|---|---|---|---|
| PVM-ARCH-001 | Typed target-independent CFG IR | `crates/fpas-ir` | IR positive/negative validation tests | planned |
| PVM-ARCH-002 | Exactly 8-byte packed instruction | `fpas-bytecode/instruction.rs` | size + all-form round-trip tests | planned |
| PVM-ARCH-003 | One exhaustive opcode dispatch | `fpas-vm/vm/dispatch.rs` | opcode inventory + VM tests | planned |
| PVM-ARCH-004 | Per-function register windows | bytecode function metadata + VM frames | calls/recursion/window edge tests | planned |
| PVM-ARCH-005 | Deterministic linear-scan allocation | `fpas-compiler/bytecode/allocation.rs` | deterministic and max-register tests | planned |
| PVM-ARCH-006 | No final stack compiler/VM path | compiler/bytecode/VM crates | zero old-symbol search hits | planned |
| PVM-ARCH-007 | Cranelift absent/deferred | workspace manifests | no Cranelift dependencies or native backend code | planned |
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
| PVM-PERF-001 | Trustworthy pre-change baseline | `cargo bench-fpas save register-vm-before` | complete final suite shape | planned |
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
| PVM-QUAL-001 | Focused module/file layout | file-size and responsibility review | planned |
| PVM-QUAL-002 | Public Rust documentation complete | rustdoc/lint inspection | planned |
| PVM-QUAL-003 | Structured errors, no production panic for inputs | negative tests + source review | planned |
| PVM-QUAL-004 | No dead compatibility path | dependency/symbol/file search | planned |
| PVM-QUAL-005 | Current user docs reconciled | `docs/pascal/` diff + link search | planned |
| PVM-QUAL-006 | Full Rust verification | fmt/build/test/clippy commands | planned |
| PVM-QUAL-007 | Full FPAS verification | fmt check + `fpas test tests/` | planned |
| PVM-QUAL-008 | Privacy preserved | repository diff inspection | planned |
| PVM-QUAL-009 | Future plan removed after completion | current docs/tests contain durable truth | planned |

## Current-opcode migration inventory template

Populate this table in P0 with every current `Op` variant. Grouping is allowed only when operand and
error semantics are identical. Do not delete a row until its new code and tests exist.

| Current family | Required register successor | Special preservation concern | State |
|---|---|---|---|
| constants and Unit | destination-based constant load | exact persistent identity | planned |
| Pop/Dup | normally eliminated; explicit move where needed | no hidden side effects | planned |
| local/enclosing access | register/local/cell operations | capture and mutation semantics | planned |
| globals | numeric global load/store | initialization and task synchronization | planned |
| typed integer/real/string/bool ops | three-address typed operations | all numeric/error edge cases | planned |
| dynamic generic ops | three-address dynamic operations | type-erased behavior | planned |
| jumps/conditions | function-local terminators/branches | short-circuit and loop targets | planned |
| direct/value calls | numeric direct/dynamic register calls | arity, frames, captures | planned |
| closures/cells | numeric closure and cell operations | task-bound mutable captures | planned |
| arrays/dictionaries | register aggregate operations | order, COW, indexing errors | planned |
| records | layout-slot operations | defaults, properties, events, formatting | planned |
| Result/Option | destination-based tagged operations | unwrap diagnostics | planned |
| enums | numeric type/variant operations | associated data and matching | planned |
| intrinsics/print | uniform intrinsic register ABI | host behavior and callbacks | planned |
| tasks/yield | register task operations | scheduling and saved state | planned |
| Halt/Return/Panic | root/function terminators | entry policy and diagnostics | planned |

## Final sign-off

Before declaring completion, every row above must be `complete`, `not applicable` with a precise
reason, or `unverified` only for a genuinely unavailable native platform. Performance threshold
exceptions require explicit user acceptance.
