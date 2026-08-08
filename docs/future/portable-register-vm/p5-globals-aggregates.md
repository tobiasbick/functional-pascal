# P5 globals and aggregates

P5 is complete on the inactive register-development path. The production compiler, CLI, artifacts,
and stack VM remain unchanged until the later cutover phase. No FPAS syntax, semantics, or current
language documentation changed.

## Numeric metadata and runtime state

Semantic analysis now exposes declared named types in deterministic name order. The register
lowerer uses that metadata to build record and enum layouts even when a declaration is not first
encountered through an expression. Generic type parameters remain dynamically typed at this phase,
while their surrounding record, enum, collection, Result, Option, and function shapes remain typed.

Top-level variables receive dense `GlobalId` values in declaration order. Register execution owns an
`RwLock<Vec<Option<Value>>>`; immutable slots accept their initializer once, mutable slots retain
normal assignment visibility, and separate VM instances remain isolated. Global instructions carry
only the numeric slot. Names remain shared diagnostic metadata.

Record layouts store one shared type name and ordered field-name table. Enum metadata similarly
stores one enum name, dense variant IDs, and ordered associated-field names. Register values use
`PositionalRecordValue` and `PositionalEnumValue`: ordinary construction, reads, writes, updates,
tests, and destructuring use numeric positions without a field or variant name search. The existing
legacy aggregate values remain for the production stack VM during migration.

`RegisterVm` builds and validates runtime layout objects once per verified executable and shares
them with callbacks. Aggregate clones share immutable layout metadata and copy-on-write bodies.
Mutation detaches only the changed body, preserving value semantics, order, equality, and display
formatting. No profiling evidence justified replacing the existing `Arc`-based storage.

## Lowering and execution coverage

The typed IR, validator, allocator, selector, verifier, and interpreter now cover:

- global initialization, reads, writes, and aggregate write-back;
- array and dictionary construction, indexing, membership, and copy-on-write update;
- Unicode string indexing and string membership with existing diagnostics;
- named and anonymous record construction, defaulted fields, field reads/writes, nested writes, and
  `with` updates;
- simple enums, data-enum construction, variant tests, associated-field loads, and case-pattern
  destructuring;
- Result and Option construction, tests, unwrap operations, and `try` early return;
- record instance/static/generic methods, properties, event assignment/testing/raising, chained
  receivers, and bound method values.

Bound record methods lower to generated numeric thunks. A thunk captures the receiver once and calls
the already resolved method `FunctionId`; invoking the resulting value performs no method-name
lookup. Property and event lowering preserves receiver-before-value evaluation order.

The implementation remains split by responsibility:

```text
crates/fpas-compiler/src/
  lowering/
    aggregates.rs                 collection, record, enum, Result, and Option lowering
    members.rs                    methods, properties, and events
    closures/bound_methods.rs     numeric bound-method thunks
    context/blocks.rs             CFG and loop state split from lowering context
  bytecode/selection/
    aggregates.rs                 P5 opcode selection
crates/fpas-ir/src/validate/operands/
  p5.rs                           aggregate operand and layout validation
  p5/wrappers.rs                  Result and Option validation
crates/fpas-vm/src/vm/register/
  layouts.rs                      executable layout materialization
  execute/aggregates.rs           numeric aggregate execution
```

Every listed production file is below 500 lines.

## Error and preservation evidence

Compiler differential tests run the same source through the production stack path and inactive
register path. They cover anonymous and named records, defaults, qualified/case-insensitive names,
generic record and enum values, nested aggregate copy-on-write mutation, collections, Result/Option,
simple and data enums, destructuring, member calls, bound methods, events, and receiver/value order.

Direct register-VM tests independently cover dense and immutable globals, positional record/enum
slots, shared layouts with detached values, collection bounds and missing keys, Result/Option
payloads, wrong-variant unwrap errors, and existing runtime diagnostic codes. Typed-IR negative tests
reject wrong collection values, string updates, mismatched wrapper operations, and unknown record or
enum slots. The bytecode verifier rejects invalid global, layout, field, and variant references.
Existing semantic tests continue to reject missing and extra record fields before register lowering.

Positional and legacy record/enum values have cross-representation equality and identical display
output during the migration. `Std.Str` formatting recognizes both representations. This lets the
inactive path preserve observable formatting without changing the production representation.

## Register-path benchmark gate

`cargo bench-fpas` now accepts an explicit per-row `engine = "register"`; omitted rows still use the
production CLI and stack VM. The separate `register-p5` group compiles each source before timing and
measures only one verified `RegisterVm` execution. Each source performs a fixed workload and checks a
checksum without requiring P6 intrinsics:

```text
cargo bench-fpas run --group register-p5

register_global_access   1167 ms   59,982,876 instructions/s
register_array_access    1339 ms   53,771,491 instructions/s
register_record_update    786 ms   34,351,184 instructions/s
```

These single-machine phase-close samples prove that the three required workloads execute entirely
on the new path. They are not a production-stack comparison or an accepted speedup claim. The P5
implementation does not change the production hot path, so the saved production baseline remains
the comparison reference and `docs/bench/history.md` remains unchanged.

The required production comparison found one misleading saved single-sample result:
`integer_loop` was 19-23% slower than the saved 5567 ms value, while the other 13 VM benchmarks
were between -5.5% and +3.7%. A clean P4 build was therefore measured alternately with P5 under the
same load. P4 took 6653 and 6666 ms; P5 took 6810 and 6710 ms, a 0.7-2.4% difference. This confirms
that the saved integer-loop sample was an environmental outlier rather than a P5 stack-VM
regression. No production performance claim or history entry is made from either single-machine
check.

## Verification

The phase-close gates completed successfully:

```text
cargo fmt --all -- --check
cargo build --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
scripts/format-fpas-sources.ps1 -Check
target/debug/fpas test tests/
cargo bench-fpas compare p5-before --group vm
cargo bench-fpas run --group register-p5
```

The explicit FPAS suite reported 385 passed, one skipped, and zero failed. Targeted owner suites
also cover bytecode verification and values, typed-IR validation, semantic analysis, register
lowering, direct register execution, and the benchmark CLI. The full workspace gate includes those
owner suites and all existing integration and documentation tests.

## Portability and remaining boundary

P5 adds safe Rust data structures and fixed-width bytecode IDs only. It adds no native backend,
executable memory, target triple, host pointer, or host-width serialized field. The intended Rust and
crate target envelope therefore remains unchanged. Actual Windows/Linux/ARM/macOS/FreeBSD `.fpascp`
interchange still depends on the P9 codec and CLI cutover and is not claimed here.

Imports and standard-library intrinsics remain outside the development subset. P6 begins by routing
those hosted calls through the register ABI; tasks, unit objects, persistence, and public CLI
selection remain assigned to their later phases.
