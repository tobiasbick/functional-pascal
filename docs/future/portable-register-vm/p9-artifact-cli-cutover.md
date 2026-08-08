# P9 artifact and CLI cutover

P9 makes the verified register executable the sole production `.fpascp` payload and runtime path.
The FPAS language, standard-library API, project model, and command syntax are unchanged.

## Implemented production path

```text
FPAS source
  -> semantic analysis and typed IR
  -> relocatable register .fpascu objects
  -> deterministic numeric linker
  -> verified register executable
  -> bounded sectioned .fpascp
  -> RegisterVm
```

`fpas build`, `run`, `check`, and `test`, the test child process, native bundle publication, and the
thin `fpas-runner` now share this path. There is no CLI backend switch. The runner contains only the
bundle/program decoder, verifier, register VM, and runtime support required to execute the embedded
program.

The legacy stack compiler and VM are retained only for P10 deletion and temporary internal regression
coverage. Production build and execution do not select them.

## Portable program format

`PROGRAM_FORMAT_VERSION` and `BYTECODE_VERSION` are both 10 at the cutover. The format uses an
explicit little-endian header followed by exactly ten ordered, non-overlapping, contiguous sections:

1. strings;
2. constants;
3. sources;
4. globals;
5. record layouts;
6. enum layouts;
7. functions;
8. packed instructions;
9. sparse source runs;
10. entry metadata.

All persisted counts, IDs, offsets, lengths, flags, and packed instructions have fixed widths. The
image contains no pointer-width values, native pointers, target triple, native ABI layout, object
code, or host-endian data. The decoder checks the total payload limit, section topology and per-table
limits before allocation, validates the payload digest, rejects malformed values and instructions,
then passes the reconstructed executable through the bytecode verifier before it can reach the VM.

Old stack-format program images are not migrated. A project-backed command rebuilds incompatible
derived artifacts; direct execution of a source-less old `.fpascp` returns the observed format and
bytecode versions together with an actionable rebuild instruction.

## Artifact and CLI regression coverage

- `crates/fpas-program/tests/format.rs`: deterministic complete-table round trip, canonical bytes,
  all truncated prefixes, version failures, digest/trailing data, malformed section directories,
  invalid UTF-8/opcodes/booleans, and deterministic mutation rejection without panic.
- `crates/fpas-program/tests/roundtrip.rs`: decoded register execution without sources and exact
  floating-point bit preservation.
- `crates/fpas-build/tests/program_artifact.rs`: cold/warm reuse, relinking inputs, corruption rebuild,
  atomic preservation on failed rebuild, and non-program rejection.
- `crates/fpas-cli/src/main_tests/projects/run.rs`: project execution, stale rebuild, direct source-less
  `.fpascp`, corruption diagnostics, and old-format rebuild help.
- `crates/fpas-bundle/tests/{format,publication}.rs`: validated embedded register image, format limits,
  corruption forwarding, deterministic golden bytes, and atomic publication.
- `crates/fpas-cli/tests/native_executable.rs`: a host-native application runs after project files,
  sources, manifests, sidecars, standalone `.fpascp`, source standard library, and separate runner are
  absent.

Compiler and runtime regressions added while completing the production corpus cover imported layouts
and aliases, generic/captured values, contextual record defaults, standard-constant shadowing,
positional Console/Graph records, Json/Toml positional enums, TUI palette values, and queued Graph
events transferred into the opened register session.

## Platform evidence

P9 was verified natively on Windows x86-64. That host covers the complete workspace, FPAS suite,
canonical image digest, direct source-less `.fpascp`, and source-less native application.

Linux x86-64/ARM64, Windows ARM64, macOS x86-64/ARM64, and FreeBSD x86-64/ARM64 were unavailable in
this run and remain **unverified**. The wire format is designed to be host-neutral, but this is not a
claim of native execution proof. When native hosts are available, use the producer/consumer procedure
from `bytecode-and-portability.md`: preserve the producer image unchanged, record its digest, execute
it with the consumer host's `fpas run`, and compare stdout, stderr, exit status, and diagnostics. A
host-native bundle remains executable only on the host family for which its runner was built.

## Exit-gate verification

The committed P9 result requires these commands to pass on the recorded Windows x86-64 host:

```text
cargo fmt --all -- --check
cargo build --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo run --locked -p fpas-cli -- fmt --check examples/ tests/ apps/
cargo run --locked -p fpas-cli -- test tests/
git diff --check
```

Focused register compiler, TUI, Graph, artifact, bundle, CLI, and native-application tests are also
run before the full gates. No performance claim belongs to P9; release benchmark acceptance remains
P11.

The final Windows x86-64 run passed every command above. The direct FPAS corpus reported 385 passed,
1 explicitly skipped, 0 not run, and 0 failed across 386 tests. The workspace run additionally passed
the two source-less native-application tests and all Rust unit, integration, and documentation tests.
