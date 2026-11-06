# Debugger platform acceptance

Only unresolved verification belongs here. The [core reference](../architecture/debugger-core.md),
[batch contract](../pascal/tools/batch-debugging.md) and
[DAP contract](../pascal/tools/debugging.md) describe the implementation.
Core, batch and DAP share session control, source mapping and passive inspection.
Editor extensions and editor-specific configuration are outside this work.

## Required environments

| Environment | Debug tool build | Release tool build |
| --- | --- | --- |
| Native Windows x64, .NET 10 target | [ ] Complete matrix and deployment | [ ] Complete matrix and deployment |
| WSL2 Linux x64, .NET 10 target inside the distribution | [ ] Complete matrix and deployment | [ ] Complete matrix and deployment |

Each cell requires the matrix below against the same reviewed source revision.
Build WSL inputs in a Linux-local source copy with isolated outputs and package
caches. A Windows launcher under /mnt, a cloud Linux run, mocked native calls or
a skipped required case cannot establish WSL acceptance.

Store environment/version manifests, source snapshot identities, shim hashes,
TRX results and bounded logs under ignored `.temp-data/`. Record actual passes,
failures and skips. Do not commit machine-specific paths or identifiers.
Use the [test instructions](../../tests/README.md#own-coreclr-connection-tests)
for filters and the [development workflow](../build/development.md) for builds.
Do not rebuild toolchain binaries while their tests are running.

## Core matrix

| Area | Required observations and owning tests |
| --- | --- |
| Native startup/runtime | Suspended launch, callbacks, startup failure/cancellation, real exits and released handles. Validate missing, incompatible and wrong-architecture native/runtime assets with bounded failures. `NativeCoreTests`, `CoreSessionTests`, `EnvironmentCommandTests`. |
| Symbols and modules | Matching/missing/mismatched PDB, changed sources/modules, lexical scopes, recursive/library frames, equal filenames, Unicode/space paths and native path-case rules. `SymbolContextTests`, `ModuleContentIdentityTests`. |
| Breakpoints/control | Entry effects, replacement/removal, duplicates, pending/load/unload binding, rejected points, racing stops, pause and into/over/out through loops, recursion, hidden regions and exceptions. `SourceControlTests`. |
| Passive values | Exact scalar extremes, decimal/floating values, UTF-16, null, bounded pages, generic/static storage, arrays with actual bounds, aliases/cycles, source records/unions/domains and expired references. Sentinel getters, initializers, operators, conversions and display code must remain unexecuted. Scalar/Compound/Static/Capture/SourceProjection/ExpressionInspectionTests. |
| Async and exceptions | Real suspension, another CLR thread/instance, hoisted values, source stepping, logical callers, raised/default-unhandled policy and passive exception type/message/throw stack. `AsyncControlTests`, `ExceptionInspectionTests`. |
| Optimization | Debug and optimized Release targets preserve supported source stops and current values; unavailable storage is explicit. Do not disable target optimization to satisfy tests. `ScalarInspectionTests`, `AsyncControlTests`. |
| Lifetimes and bounds | Stale session/stop/frame/value identities, exit/stop races, worker crash/hang, callback failure, output floods, blocked I/O, deadlines/cancellation and clipping. Verify owned processes exit before harness cleanup and an unrelated process survives. `CoreSessionTests`, `TargetIoTests`. |

Tool-build configuration and target optimization are independent axes. Exercise
both Debug and optimized Release FPAS targets with each tool build.

## Batch and DAP matrix

- [ ] Run `BatchDebuggerTests` and `TargetIoTests` in every required cell. Check
  first-stop effects without continuing after capture, exact detached values,
  normal/nonzero exit, exception/missed-breakpoint/expression failure, stdin
  EOF/file bytes, blocked input, Unicode/quoted argv, limits and cancellation.
  Parse complete stdout as one bounded JSON object plus newline; no live handles
  may escape. Failures must preserve the documented outcome and exit code.
- [ ] Run `DapDebuggerTests` through its independent stdio client in every cell.
  Check framing, configuration order, capabilities, source paths/coordinates,
  inspection/control, passive expressions, stale handles, output and terminal
  events. Malformed/oversized/duplicate requests, cancellation, backpressure and
  client EOF must retain bounded responses and owned shutdown.
- [ ] Compare batch and DAP observations from the same source stop with
  independently expected values. Serialization must not alter binding,
  inspection or omission policy.
- [ ] Verify real Windows ConPTY Ctrl+C and Linux SIGINT separately from client
  EOF. Observe worker/target exit before closing harness ownership. A full or
  blocked ordinary request queue must still permit cancellation and shutdown.

## Delivered launcher matrix

- [ ] Run warning-free Debug/Release builds and the compiler, symbol, process,
  environment, debugger, packaging and documentation tests on both platforms.
- [ ] Run `LocalDeploymentTests`, which invokes `dist.ps1` or `dist.sh` and checks
  both batch and DAP through the delivered launcher with a fresh matching SDK
  consumer. Repeat from an unrelated cwd without source checkout/cache access.
  Invalid project inputs beside a built target must not cause build or restore.
- [ ] Check packaged native assets/licenses, offline DBI/DAC lookup and the
  same-executable worker. Verify `fpas env` passively reports the deployed
  debugger/runtime facts. Invalid arguments must fail before
  launch, and packages must contain only current debugger dependencies.

The [single-file/AOT distribution goal](dist-gap.md) is separate from ordinary
managed deployment. Delete this plan after all required cells pass; keep usable
contracts in their owning references rather than a completed-work ledger.
