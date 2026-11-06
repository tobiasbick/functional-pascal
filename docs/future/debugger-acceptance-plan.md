# Own debugger acceptance

Accept the [core](future-debugging.md) and [DAP/batch frontends](debugger-frontends-plan.md)
against real FPAS DLL/PDB/source sets. Both platforms are required. This plan
contains open acceptance work, not evidence that the replacement already works.

## Required matrix and evidence

| Environment | Required execution |
| --- | --- |
| Native Windows x64 | Windows `fpas` supervisor/worker and .NET 10 target; Job Object containment and real console cancellation. |
| WSL2 Linux x64 | Linux `fpas` supervisor/worker and .NET 10 target inside the distribution; Unix process/session containment and real SIGINT. |

- [ ] Record OS/distribution, architecture, .NET SDK/target runtime, debugger and
  shim versions/hashes for each run. Keep machine-specific paths and logs in
  ignored `.temp-data/` artifacts. Put portable commands and assertions in tests
  and references. Repeat required runs in Debug and Release tool builds.
- [ ] Build Linux test inputs in a WSL-local source copy with isolated outputs
  and NuGet cache. Use the same reviewed source revision or explicitly hashed
  working-tree snapshot as Windows. Keep a manifest of the tested source when
  the work is not committed. A `/mnt/...` Windows launcher, mocked runtime or
  skipped native case cannot
  satisfy Linux acceptance.
- [ ] Add focused tests through the real core and frontend interfaces. Use
  ordinary compiled FPAS fixtures and existing process ownership helpers.
  Exercise failure state with disposable workers/targets; do not add production
  hooks solely to inject test outcomes.

Every row below must pass on both environments. Adapt existing NetCoreDbg/MCP
fixtures to the own debugger where their assertions still apply. Retain metadata
and compiler-emission tests separately; passing them cannot establish native
debugger behavior.

| Area | Required observations |
| --- | --- |
| Native lifecycle | Suspended launch, startup registration, callback delivery, first stop, continue, process exit, startup failure and complete reference/handle cleanup. |
| Runtime assets | Correct RID/version selection; missing, wrong-architecture and incompatible shim/DBI/DAC give bounded actionable failures without executing user statements. |
| Symbols/source | Matching and missing/mismatched PDB, altered source checksum, hidden points, lexical scopes, multiple source files, Unicode/spaces and duplicate filenames. Windows drive/case rules and Linux case-sensitive paths are independent cases. |
| Breakpoints | Requested/actual position, non-executable line, replacement/removal, duplicate requests, pending dependency module binding, missed breakpoint and exception/breakpoint races. |
| Control/stepping | Continue/pause, into/over/out, loops, recursion, multiple statements per line, generated helpers, exception during step and responsive cancellation. |
| Values | Scalar extremes, exact long/ulong, decimal, floating special values, strings/surrogates, null, arrays, cycles, generic fields, FPAS records/unions/domains, shadowing and scope expiry. |
| Captures/async | Mutable and nested captures, hoisted locals before/after a real suspension, continuation on a different CLR thread and source stepping over/into await. Assertions must prove suspension, not only await a completed task. |
| Passive inspection | Sentinel getters, static initializers, overloaded operators, user conversions, display attributes, proxies and methods must not execute during expansion or expressions. Allowed primitive/field/index expressions have independently asserted values. |
| Exceptions | Handled raised exception under opt-in policy, unhandled exception under default policy, type/message/source/stack and exception object lifetime without calling target ToString. |
| Optimized targets | Debug and optimized Release targets retain supported locations and explicitly mark unavailable/optimized-out locals. Never return a cached local as current. |
| Session identities | Old session/stop/frame/value references fail after resume/relaunch; exit/stop races preserve actual state and emit terminal facts once. |
| Bounds | Oversized messages, output flood, large arrays/strings/stacks/thread counts, cycles, report escaping and invalid pages preserve limits and expose omissions. Requested observation loss cannot be reported as success. |
| Ownership | Normal stop, failed launch, target crash, private worker crash/hang, callback failure, stdin/output blockage, EOF and cancellation leave no owned descendants. Observe exit before harness fallback cleanup. An unrelated sentinel process must survive. |

Run focused rows after every [implementation step](future-debugging.md#implementation-order)
and the complete applicable matrix before product cutover. Tool-build Debug and
Release are separate from target optimization: exercise both Debug and optimized
Release FPAS targets, without forcing the latter into an unoptimized JIT mode.
Record exact passes, failures and skips; a skipped required row remains open.

## Batch acceptance

- [ ] Assert that a launch with source breakpoints stops before the selected
  statement, captures independently known values once and does not continue
  after capture. Confirm intended target side effects before the stop and their
  absence after it. Terminating that target after capture must preserve the
  fulfilled observation and identify debugger-requested termination.
- [ ] Check normal zero/nonzero exit, selected exception, missed breakpoint,
  individual expression failure, malformed request, deadline, output limit,
  cancellation and worker failure against the exact report/exit contract.
- [ ] Parse complete stdout as exactly one JSON object and a newline. Verify
  stdout/stderr/ANSI data remain report data, truncation markers are correct and
  no protocol/session handles escape into the detached snapshot.
- [ ] Verify stdin defaults to EOF; explicit file bytes arrive exactly once and
  are followed by EOF. A target that never reads input must still time out and
  be reaped. Do not use a real interactive terminal as program input.
- [ ] Run from an unrelated cwd with quoted Unicode/space paths and individual
  argv elements. Invalid options, help, version and an absent target remain
  non-interactive and do not launch a worker.
- [ ] Use a valid already-built DLL beside deliberately invalid project inputs.
  Confirm debugging neither builds/restores nor rewrites project/target/source
  files. Verify no external debugger executable is present or invoked.

## DAP acceptance

- [ ] Drive the product stdio adapter from an independent DAP client. Verify
  framing, initialize capabilities, launch/configuration ordering, breakpoints,
  every supported inspection/control request, output and terminal event order.
- [ ] Test split headers/bodies, malformed/oversized frames, unknown requests,
  duplicate/cancelled requests, backpressure and abrupt client EOF. Running or
  blocked native work must not prevent stop/cancel handling by the supervisor.
- [ ] Compare DAP and batch observations from the same source stop against
  independently expected values. Different serialization must not change core
  binding, source mapping, passive evaluation or truncation policy.
- [ ] Check native Windows Ctrl+C through real ConPTY and Linux SIGINT separately.
  Confirm worker/target exit before closing the harness's process owner. Test
  transport EOF independently from console cancellation.
## Packaged acceptance and removal gate

- [ ] Run warning-free Debug/Release builds and relevant compiler, symbol,
  ownership, environment and debugger tests on both platforms. Keep meaningful
  metadata/value/async/exception assertions when removing old MCP/backend tests.
  Check actual test counts and skips; required native rows may not be skipped.
- [ ] Run `dist.ps1` on Windows and `dist.sh` inside WSL. From each delivered
  launcher, use a fresh matching SDK consumer and repeat batch plus the DAP
  launch/inspect/step/stop sequence. Do not substitute a development test host.
- [ ] Verify native assets, licenses and worker launch from a clean deployment
  directory without the compiler checkout, Samsung NetCoreDbg, an MCP host or
  debug-time downloads. Run from an unrelated cwd and report runtime mismatch.
- [ ] Check `fpas env` exposes own-debugger deployment facts without launch or
  restore and no longer selects NetCoreDbg. Confirm obsolete MCP/JSONL/backend
  CLI options fail as specified and debugger-only MCP dependencies are absent.
- [ ] Verify removal of the known `fpas-debug` MCP registration while preserving
  unrelated host configuration. Recheck docs links, help examples, test coverage
  and packaging manifests against the delivered implementation.
- [ ] Move accepted behavior into the debugging, architecture and development
  references. Remove closed items across future plans and delete fully completed
  documents. Do not retain a completed-work ledger or obsolete MCP walkthrough.

Product cutover requires the complete core/frontend matrix and both deployment
rows. Until then, keep failures and remaining work explicit; an SDK unit test or
a previous NetCoreDbg acceptance cannot close an own-debugger requirement.
