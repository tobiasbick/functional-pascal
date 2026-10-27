# Rust builds and distribution

`dist.ps1` and `dist.sh` run one locked release build for `fpas-cli` and
`fpas-lsp`. This includes `fpas-runner`. The resulting compiler then runs:

```sh
target/release/fpas stdlib stage --source target/release/lib --output bin/lib
```

Use `fpas stdlib stage --help` for the staging contract and `--dry-run` to inspect
resolved paths. Staging deletes old compiled artifacts from its disposable
source tree and recompiles every standard-library unit with the running
compiler. Only a successful compilation replaces the destination. Do not point
`--source` at the repository's `lib/` directory.

The scripts stop on failure and publish the three binaries after library
staging. They preserve the release profile: fat LTO and one codegen unit.

## ACP dependencies

- `fpas-acp-model` owns raw values, errors, the pinned schema and declaration
  catalog. Semantic analysis and `fpas-std` depend on this crate.
- `fpas-acp-codec` owns typed wire conversions and SDK normalization. It depends
  on the model and SDK, without the connection driver or MCP server.
- `fpas-acp` owns connections, transport, interaction state and the MCP bridge.
  It re-exports the model and codec for runtime callers.

The catalog codec uses static dispatch. Schema features and SDK versions remain
pinned; splitting the crates does not change protocol fields or wire behavior.
The model participates in the compiler artifact identity. Runtime connection
changes do not change this identity.

Check the production dependency graph with `cargo tree -p fpas-lsp -e normal`.
It should include `fpas-acp-model`, but not `fpas-acp`, `fpas-acp-codec`,
`agent-client-protocol` or `fpas-mcp`.

## Measuring

Record the revision, exact command, cache state and any other active build.
Measure an unchanged distribution build separately from a runtime-source
rebuild and a fresh target-directory build. They answer different questions.
Avoid running Cargo commands concurrently: lock waits are not compilation time.

For per-crate build timings, run
`cargo build --locked --release -p fpas-cli -p fpas-lsp --timings` and inspect
`target/cargo-timings/`. A few busy cores near the end can be expected: dependency
ordering and final optimization/linking limit parallelism. This change does not
relax release optimization to improve that phase.

Compare runtime with `cargo bench-fpas save <name>` and
`cargo bench-fpas compare <name>` on a quiet machine. Repeat suspicious rows,
including `--group protocols`, before interpreting single-run differences.
Keep builds and test suites out of the timed workload interval.
