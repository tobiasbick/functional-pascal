# Rust builds and distribution

## Build profiles

Program builds use two optimized profiles:

| Use | Cargo option | LTO | Codegen units | Output directory |
| --- | --- | --- | --- | --- |
| Release, benchmarks, and dist | `--release` | Fat | 1 | `target/release/` |
| Development | `--profile dev-fast` | Thin | 16 | `target/dev-fast/` |

`dev-fast` inherits the release settings and overrides LTO and codegen units
to reduce build time. Build the development binaries with:

```sh
cargo build --profile dev-fast -p fpas-cli -p fpas-lsp -p fpas-runner
```

Run the CLI with `cargo run --profile dev-fast -p fpas-cli -- <arguments>`.
Cargo keeps custom profiles' artifacts separate. Plain `cargo build` retains
Cargo's default development settings. Rust tests use their own settings:

| Use | Cargo option | Debug information | Output directory |
| --- | --- | --- | --- |
| Rust tests | default `cargo test` | Workspace line tables; no external dependency symbols | `target/cargo-build/debug/` |
| Native Rust debugging | `--profile test-debug` | Full symbols, types and locals | `target/cargo-build/test-debug/` |

Both test modes retain development assertions and overflow checks. The default
test profile reduces repeated symbol generation and linking across integration
executables. `test-debug` inherits `dev` and is an explicit rebuild when native
Rust breakpoints and local-variable inspection are needed. FPAS source debugging
uses FPAS executable metadata and does not require full native Rust symbols.

### Checkout isolation and library inputs

The workspace sets Cargo's `build.build-dir` to
`{workspace-root}/target/cargo-build`. Intermediate objects, fingerprints, test
executables and build-script outputs stay inside their checkout even when
`CARGO_TARGET_DIR` selects a shared final output directory. The paths in the
program profile table above still locate delivered programs. The first build
after changing this setting creates a fresh intermediate cache. Do not override
`CARGO_BUILD_BUILD_DIR` with a directory shared by different checkouts.
See [Cargo's build-directory configuration](https://doc.rust-lang.org/cargo/reference/config.html#buildbuild-dir).

`fpas-stdlib`, the build dependency rooted at `lib/`, tracks the library manifest
and `.fpas` sources through Cargo's package include list. Source additions,
changes and removals update the delivered library; `.fpascu` sidecars, locks and
temporary artifacts do not invalidate the CLI. Its build script deliberately
uses Cargo's default package change detection instead of a recursive directory
watch. Only FPAS source and project files are copied into delivered libraries.

Cargo-built CLIs and test executables select the library copy in their own
intermediate profile directory. This keeps compiler and library paired when
Cargo restores a cached executable into a shared final directory without
rerunning its build script. Installed or copied CLIs use their adjacent `lib/`;
`--std-lib` remains an explicit override. When retaining multiple toolchains
for comparison, separate final target directories also preserve each executable
and its adjacent library. Builds using one final output directory must run
sequentially, and each executable must be used before another checkout replaces it.
Worker processes launched by Rust tests use the delivered Cargo profile directory,
rather than inferring a CLI location from the intermediate test directory.

`crates/fpas-build/tests/cargo_build_cache.rs` verifies checkout round trips with
old source timestamps, installed library lookup, sidecar freshness and real
source addition/change/removal using small Cargo workspaces.

## Rust test workflow

Build tools, compile tests and execute tests as separate operations:

```sh
cargo build --workspace --locked --profile dev-fast
cargo test --workspace --locked --no-run --timings
cargo test --workspace --locked
```

The last two commands share the same profile and flags, so execution reuses
compiled test artifacts. For focused work, select a test executable and filter:

```sh
cargo test --locked -p fpas-debug --test evaluation call_evaluation
cargo test --locked -p fpas-cli --test host_io test_timeout
cargo test --locked -p fpas-debug --profile test-debug --test evaluation --no-run
```

Mappings from previous target names are documented beside the
[debugger tests](../../crates/fpas-debug/tests/README.md),
[CLI tests](../../crates/fpas-cli/tests/README.md), and
[language-service tests](../../crates/fpas-language-service/tests/README.md).
Runtime allocator, terminal and process-lifecycle tests retain separate targets
where the executable itself is part of the fixture.

Cargo build jobs control compilation/linking. Libtest `--test-threads` controls
cases within an executable; VM pools and fixture children have their own workers.
Keep program build jobs at Cargo's default. Serial case execution is available
for diagnosis by appending `-- --test-threads=1` to `cargo test`.

### Resource guard

On Windows, where test compilation or execution risks exhausting memory, use
the FPAS supervisor with an already built development CLI:

```sh
target/dev-fast/fpas run tools/resource-guard.fpas -- 900000 cargo test --workspace --locked --no-run --timings
target/dev-fast/fpas run tools/resource-guard.fpas -- 1800000 cargo test --workspace --locked
```

The first argument is a positive process-tree timeout in milliseconds. The guard
samples available memory while owning the command and its descendants, and
cancels its owned process tree if the reserve is reached. It reserves one quarter
of initially available RAM, bounded between 512 MiB and 8 GiB. Override the byte
reserve with `FPAS_RESOURCE_RESERVE_BYTES`; `--help` shows usage. Windows samples
physical availability and system commit headroom through a bounded CIM query.
Linux samples available memory and swap from `/proc/meminfo`. These are sampled
limits, not a hard memory quota; a sudden allocation can outrun a sample. Native
hardware-query gaps are recorded in [dogfooding gaps](../future/dogfooding-gaps.md#available-memory-and-commit-headroom).

On Linux, cgroup v2 ancestor memory limits also constrain available memory, and
strict kernel overcommit mode constrains the commit budget. Use this outer guard
for compilation (`cargo test --no-run`), then execute tests normally. FPAS process
cancellation owns an inherited Unix process group; test workers that create new
groups or sessions can escape that outer group. Linux execution therefore relies
on each fixture's independent deadline and identity-checked descendant cleanup,
rather than a claim that the FPAS outer guard contains those separate groups.

The supervisor captures output with a 16 MiB bound per stream and prints it when
the command completes. A nonzero child exit, deadline, query failure or exhausted
reserve produces a nonzero supervisor exit. Record full logs in ignored scratch
storage and commit only portable summaries. It changes neither Cargo jobs nor
test thread counts.

When rebuilding the development CLI itself, run the guard from another existing
profile, such as `target/debug/fpas`, so Windows can replace `target/dev-fast/fpas`.
Choose the outer timeout for the complete command, including compilation and
fixture cleanup; it is independent of the timeout a test exercises.

The [Windows/Linux validation record](test-build-structure.md) found no need for
an additional execution scheduler with the grouped targets and reduced symbols.
Cargo and the harness keep their CPU-based defaults; the guard calculates its
reserve from available memory. A reserve-triggered cancellation is a failed run
that needs more headroom or a deliberate manual concurrency adjustment.

Loopback fixtures require binding and connecting to local sockets. Process tests
need child execution; Cargo incremental caches benefit from same-volume hard
links. A restricted environment can deny these independently of the code under
test. Report denied capabilities and the execution environment, then verify in
an authorized environment with those capabilities. Build/test tooling must not
change firewall, antivirus, sandbox or filesystem security settings. A repeated
Cargo warning from a cached build is not evidence of a new hard-link failure.

## Distribution

`dist.ps1` and `dist.sh` run one locked release build for `fpas-cli`,
`fpas-lsp`, and `fpas-runner`. The resulting compiler then runs:

```sh
target/release/fpas stdlib stage --source target/release/lib --output bin/lib
```

Use `fpas stdlib stage --help` for the staging contract and `--dry-run` to inspect
resolved paths. Staging deletes old compiled artifacts from its disposable
source tree and recompiles every standard-library unit with the running
compiler. Only a successful compilation replaces the destination. Do not point
`--source` at the repository's `lib/` directory.

The scripts stop on failure and publish the three binaries after library
staging. They use the workspace release profile with Fat-LTO and one codegen
unit. Parallel jobs remain at Cargo's default.

The [Rust architecture](../rust-architecture.md) separates portable code and
standard-library declarations from the executing runtime. Compiler and editor
builds do not depend on the VM or native operation implementations.

## ACP dependencies

- `fpas-acp-model` owns raw values, errors, the pinned schema and declaration
  catalog. Semantic analysis, `fpas-builtins`, and `fpas-runtime` depend on it.
- `fpas-acp-codec` owns typed wire conversions and SDK normalization. It depends
  on the model and SDK, without the connection driver or MCP server.
- `fpas-acp` owns connections, transport, interaction state and the MCP bridge.
  It re-exports the model and codec for runtime callers.

The catalog codec uses static dispatch. Schema features and SDK versions remain
pinned; splitting the crates does not change protocol fields or wire behavior.
The model participates in the compiler artifact identity. Runtime connection
changes do not change this identity.

Check the production dependency graph with `cargo tree -p fpas-lsp -e normal`.
It should include `fpas-acp-model`, but not `fpas-runtime`, `fpas-acp`, `fpas-acp-codec`,
`agent-client-protocol` or `fpas-mcp`.

## Measuring

Record the revision, profile, exact command, cache state and any other active build.
Measure an unchanged distribution build separately from a runtime-source
rebuild and a fresh target-directory build. They answer different questions.
Avoid running Cargo commands concurrently: lock waits are not compilation time.

For per-crate build timings, run
`cargo build --locked --release -p fpas-cli -p fpas-lsp -p fpas-runner --timings` and inspect
`target/cargo-timings/`. A few busy cores near the end can be expected: dependency
ordering and final optimization/linking limit parallelism.

For development timings, replace `--release` with `--profile dev-fast` in the
same command. Compare profiles using the same packages and cache state.

Compare runtime with `cargo bench-fpas save <name>` and
`cargo bench-fpas compare <name>` on a quiet machine. Repeat suspicious rows,
including `--group protocols`, before interpreting single-run differences.
Keep builds and test suites out of the timed workload interval.
