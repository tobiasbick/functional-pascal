# P0 contract and baseline record

This is the reproducible P0 work record. It freezes the verified pre-register-VM behavior and
records only non-identifying environment facts needed to interpret same-machine measurements.

## Scope and contract check

- Revision: `3b35614f9be4a0108ea243ab09a0850465fb1c17` on
  `codex/portable-register-vm-plan` before P0 changes.
- Runtime toolchain: Rust `1.97.1`, FPAS `0.0.1`.
- Measurement host class: Windows 11 x86-64, balanced power scheme. Measurements were made in one
  interactive session; no result below is a portability or speed claim.
- `rg -ni "cranelift|jit|aot" Cargo.toml Cargo.lock crates -g Cargo.toml -g '*.rs'` produced no
  references. No native backend dependency was added.
- `git diff -- docs/pascal/language` was empty before and during P0. No language document, syntax,
  semantic rule, evaluation order, CLI workflow, or standard-library API changed.

## Current implementation inventory

The P0 commands required by the implementation contract were run against the checkout:

```text
rg "\\bChunk\\b" crates
rg "functions\\(\\)|GetGlobal|SetGlobal|FieldGet|FieldSet|IsVariant|MakeEnum" crates
```

The results confirm that the current execution path is the stack `Op`/`Chunk` model:

- `crates/fpas-bytecode/src/op.rs` owns every current stack opcode.
- `crates/fpas-bytecode/src/chunk.rs` owns `Vec<Op>`, parallel locations, constants, and a
  name-keyed function table.
- Compiler, linker, program image, CLI test image, and VM consume `Chunk`.
- `fpas-vm` resolves function, global, record-field, and enum identities through names on ordinary
  execution paths.

The complete variant-by-variant successor inventory is in
[Traceability](traceability.md#p0-current-opcode-migration-inventory). It is the P0 contract for
later opcode work; every row remains planned until its named phase has tests and implementation.

## Added isolating benchmarks

The initial suite did not contain equivalent isolated signals for dense globals, known record fields,
captured direct calls, alternating branches, or generic numeric dispatch. P0 therefore added and
registered the following final-suite members before recreating the snapshot:

| ID | Program | Primary signal |
|---|---|---|
| `global_access` | `examples/pascal/vm/global_access_benchmark.fpas` | dense global read/write path |
| `record_field_access` | `examples/pascal/vm/record_field_access_benchmark.fpas` | known-field get/set path |
| `closure_call` | `examples/pascal/vm/closure_call_benchmark.fpas` | captured first-class invocation |
| `branch_dispatch` | `examples/pascal/vm/branch_dispatch_benchmark.fpas` | alternating conditional dispatch |
| `dynamic_numeric` | `examples/pascal/vm/dynamic_numeric_benchmark.fpas` | `Numeric`-constrained generic arithmetic |

Each program accepts `ITERATIONS` and optional `MAX_MILLIS`, starts timing inside FPAS code, prints a
checksum/final value, and is registered in `docs/bench/suite.toml` under `vm`. The new sources passed
the FPAS formatter and individual `fpas check` runs. `dynamic_numeric` uses the existing documented
`T: Numeric` constraint; it does not introduce a language feature.

## Release baseline

Commands:

```text
cargo build --release -p fpas-cli
cargo bench-fpas save register-vm-before
cargo bench-fpas run --group vm
cargo bench-fpas run --group vm
```

The final snapshot is local and gitignored at `.temp-data/bench/register-vm-before.json`; it has all
16 current suite rows. The snapshot results were:

| Benchmark | Elapsed ms | Throughput |
|---|---:|---:|
| integer_loop | 12143 | 4117598 iters/s |
| global_access | 1011 | 4945598 global updates/s |
| record_field_access | 2807 | 4275026 field accesses/s |
| closure_call | 631 | 4754358 closure calls/s |
| branch_dispatch | 4559 | 4386926 branches/s |
| dynamic_numeric | 1199 | 4170141 dynamic numeric ops/s |
| array_push | 209 | 9569377 pushes/s |
| array_length | 79 | 6329113 lengths/s |
| string_concat | 3612 | 1384274 concats/s |
| string_length | 77 | 6493506 lengths/s |
| function_call | 1095 | 5479452 calls/s |
| array_callbacks | 1249 | 7686148 callbacks/s |
| record_update | 715 | 1398601 updates/s |
| unicode_char_at | 1303 | 2302379 chars/s |
| task_spawn_wait | 1005 | 99502 tasks/s |
| tui_headless | 10182 | 49 frames/s |

The two required VM repetitions produced materially different absolute timings. This is expected
machine noise evidence, not a comparison: no performance result or supported-platform claim is made
from P0. All later comparisons must use this saved final-suite shape on the same power configuration.

## Artifact behavior snapshot

The existing regression coverage establishes the current stack-image behavior before the format
cutover:

| Behavior | Evidence |
|---|---|
| deterministic envelope, unsupported format version, truncation, trailing bytes, digest | `cargo test -p fpas-program --test format` — 10 passed |
| bytecode version, malformed constant operand, source-id bounds, relative/absolute source paths, decoded execution | `cargo test -p fpas-program --test roundtrip` — 8 passed |
| malformed executable operands, calls, control flow, and names | `cargo test -p fpas-bytecode --test executable_validation` — 18 passed |
| direct source-less `fpas run file.fpascp` path | `cargo test -p fpas-cli run_cli_executes_compiled_program_without_project_sources` — 1 passed |
| host-native bundle without project files | `cargo test -p fpas-cli built_native_application_runs_without_project_files` — 1 passed |

## Verification record

The pre-change checkout passed `cargo fmt --all -- --check`, `cargo build`, `cargo test --workspace`,
`cargo clippy --all-targets --all-features --locked -- -D warnings`, and `fpas test tests/` (385
passed, 1 skipped, 0 failed). The initial `fpas fmt --check examples/ tests/ apps/` reported 52
already committed noncanonical FPAS files. P0 formatted those files mechanically.

The P0 exit-gate rerun passed all required commands:

```text
cargo fmt --all -- --check
cargo build
cargo test --workspace
cargo clippy --all-targets --all-features --locked -- -D warnings
fpas fmt --check examples/ tests/ apps/
fpas test tests/                         # 385 passed, 1 skipped, 0 failed
git diff --check
```

P0 is therefore complete. The legacy stack VM remains the only production path; P1 may add only
test-reachable typed-IR code while keeping that path unchanged.

No benchmark history entry was recorded: P0 establishes a baseline and does not claim a settled win.
