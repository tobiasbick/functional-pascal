# P10 stack removal

Status: complete.

P10 removes the superseded stack architecture and leaves the P9 portable bytecode path as the only
compiler, linker, artifact, benchmark, and VM implementation. FPAS syntax, semantics, standard-
library signatures, project manifests, and portable program-format versions are unchanged.

## Removed architecture

The following implementation families were deleted rather than retained behind adapters:

- stack instructions, chunks, validation, and persistent-value conversion from `fpas-bytecode`;
- stack compiler emission, stack standard-call lowering, stack unit objects, and the differential
  compiler test tree from `fpas-compiler`;
- the stack interpreter, operand stack, stack worker helpers, shared stack state, old task/runtime
  tests, and the temporary `vm/register/` nesting from `fpas-vm`;
- chunk objects, chunk relocations, and the superseded linker implementation from `fpas-unit` and
  `fpas-linker`;
- temporary register-only benchmark execution and its six phase workloads.

The retained implementation now has ordinary ownership boundaries:

```text
crates/fpas-bytecode/src/
  instruction.rs, executable.rs, value/, validate/   - sole bytecode model and verifier
crates/fpas-compiler/src/
  lowering/, bytecode/, object/                      - sole compiler and object path
crates/fpas-linker/src/
  lib.rs, error.rs, relocation.rs, ...                - sole linker
crates/fpas-vm/src/vm/
  dispatch.rs, worker.rs, execute/, tasks/, hosted/   - sole VM
```

Temporary public and internal qualifiers were removed: `RegisterVm`, `RegisterExecution`,
`RegisterWorker`, `RegisterCallbackSession`, `RegisterShutdownHandle`, `RegisterLinkError`,
`RegisterBackend`, `compile_register_*`, `link_register_*`, and name-only function targets no longer
exist. Architecture terms such as register window and register count remain where they describe the
actual bytecode model rather than a temporary alternative.

## Runtime-value reconciliation

Records and data enums now have exactly one runtime representation backed by executable-owned
positional layouts. The named-field/named-variant compatibility representations and cross-
representation equality adapters were deleted. First-class functions always contain a numeric
`FunctionId`; the name is diagnostic metadata only.

Standard-library and hosted intrinsics construct records and data enums through
`fpas_std::AggregateFactory`. `RUNTIME_AGGREGATE_TYPES` is the explicit compiler/runtime contract for
layouts produced behind an intrinsic boundary, so object pruning cannot remove JSON, TOML, process,
Console, or Graph result layouts. Simple enums continue to use integer discriminants. Opaque hosted
resources use `Value::OpaqueHandle`, keeping the empty public `SavedRegion` record free of hidden
fields and keeping `Value` within the 16-byte size contract.

## Benchmark and dependency cleanup

`fpas-bench` has one runner: every suite row invokes the production `fpas run` executable. Its direct
compiler/bytecode/VM dependencies and the temporary register phase groups were removed. The current
unit-object and interface codecs still intentionally use `serde_json`; `fpas-bytecode` uses it only
as a development dependency for wire-format tests. No dead artifact dependency was retained.

## Documentation reconciliation

Current user documentation now points to `instruction.rs`, typed lowering, current task scheduling,
and `Vm`; it describes unsupported earlier artifacts without referring to a retained migration path.
Historical P0-P9 evidence under this future-plan directory remains a record of the phased cutover and
may name files that P10 intentionally deleted. P11 will remove the whole plan after final performance
and platform acceptance.

## Regression coverage

P10 keeps the existing verifier, compiler, linker, artifact, VM, CLI, hosted-runtime, and FPAS suites
and adds or strengthens these focused contracts:

- `object_retains_layouts_constructed_by_runtime_intrinsics` verifies that object pruning retains
  Graph record and JSON data-enum layouts;
- `headless_graph_open_size_and_close_are_deterministic` supplies explicit verified layouts instead
  of relying on an aggregate adapter;
- function-value equality and VM call tests use mandatory numeric targets;
- `runtime_value_stays_compact` covers the reconciled `Value` representation, including opaque host
  handles;
- the renamed bytecode, object, linker, compiler, and VM suites exercise only the surviving path.

## Removal and structure audit

The final focused searches returned no production hits for deleted types or APIs:

```text
rg --glob '!**/tests/**' --glob '!**/examples/**' \
  "\\b(Op|Chunk|PersistentValue|RegisterVm|RegisterExecution|RegisterWorker|RegisterObject)\\b" crates
rg "compile_register|link_register|register_object|stack_vm|stack VM|stack-VM" crates
rg "legacy stack|compatibility decoder|dormant alternate path" crates docs/pascal
```

Remaining `legacy` or `old` wording outside this architecture describes unrelated public APIs,
atomic-publication cleanup tests, or earlier artifact-version diagnostics. A changed-file line-count
audit found no changed production Rust file above 500 lines. `git diff --check` passed.

## Exit-gate evidence

All commands ran from the repository root:

- `cargo fmt --all -- --check` — passed;
- `cargo build --workspace` — passed;
- `cargo test --workspace` — passed;
- `cargo clippy --workspace --all-targets -- -D warnings` — passed;
- `fpas test tests/ --report json` — 385 passed, 1 skipped, 0 failed, 0 compile errors,
  0 runtime errors, 0 timeouts.

P11 performance and native-platform acceptance remain intentionally unstarted. P10 makes no speed or
cross-platform claim.
