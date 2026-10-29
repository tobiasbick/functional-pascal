# Rust test and build structure

Status: Planned. Implementation has not started. The current authorization covers
this plan only.

The goal is a dependable development cycle: build the FPAS tools, compile the
Rust tests, then run tests that finish or report an actionable failure. Reduce
repeated linking, debug artifacts and memory pressure while preserving the
contracts the tests protect.

Rust remains the implementation language. Internal test layout, fixtures,
profiles and tooling may be replaced. FPAS syntax and language behavior remain
unchanged; changing either requires explicit user approval. FPAS 2 point 14
continues under its [owning plan](fpas2-language-directions.md#14-host-io-that-releases-vm-workers).

## Starting evidence

These findings identify work to investigate. They do not establish how much
each issue contributed to the recent resource peaks.

| Finding | Consequence for this plan |
| --- | --- |
| `fpas-debug/tests/` currently has 73 root Rust files, with Cargo's automatic test discovery enabled. | Each entry produces a separate integration executable. Group related cases where process isolation is unnecessary. |
| [The HTTPS certificate test](../../crates/fpas-cli/src/main_tests/network_tls.rs) joins a server using blocking accept/read before checking the CLI result. | A client failure can leave the fixture waiting and hide its diagnostics. The observed host run stopped here; the underlying client error is still unknown. |
| [Timeout tests](../../crates/fpas-cli/tests/test_timeout.rs) wait through `Command::output()` before checking elapsed time. | The test's supervisor needs a deadline independent of the timeout mechanism being tested. |
| [Server fixtures](../../crates/fpas-cli/src/main_tests/network_server.rs) obtain an unused port and release it before starting the server. | Port selection has a race. Fixture ownership and readiness need a reliable handoff. |
| [GC memory measurements](../../crates/fpas-runtime/tests/gc_memory/measurement.rs) re-execute exact test names under a process-wide allocator. | Grouping must preserve child selection, completion markers and allocation isolation. |

The reported development program build took about 3 minutes 10 seconds. Its
cache state was not recorded, so this is context rather than a benchmark target.
The interrupted optimized test build and the default debug test builds compiled
different workloads and cannot serve as a profile comparison.

Windows runs also encountered a linker file-open failure and sandbox-denied
loopback/hard-link operations. Host hard-link creation succeeded. Separate these
environment conditions from code failures; neither the linker failure's cause
nor the hard-link fallback's share of disk activity has been measured.

## Delivery and tracking

Execute steps in order after implementation is authorized. Each step should
leave a reviewable result and its own verification evidence. Internal design
choices do not require another approval round.

Mark a checkbox `[x]` only after its stated result is delivered and verified.
Record the revision, commands, platform, result and evidence location beside the
completed step. Keep raw machine-specific logs out of committed documentation.
Commit and push only when authorized, following the repository's commit review.

### 1. Inventory and comparable measurements

- [ ] Inventory test executables and their protected contracts. Identify shared
  process state, global allocators, environment changes, terminal ownership,
  subprocesses and exact-name child invocations before proposing groups.
- [ ] Record production dependency boundaries from the
  [Rust architecture](../rust-architecture.md#tests-and-dependency-checks).
  Distinguish normal dependencies from test-only compiler/runtime dependencies.
- [ ] Measure program builds, test compilation and test execution separately,
  using the pinned toolchain, locked dependencies and identical cache conditions
  for each before/after comparison. Bound known hanging reproductions externally.
- [ ] Record cold compilation, an unchanged repeat, and a rebuild after one
  representative runtime source change. Record whether dependency downloads and
  FPAS artifact-cache invalidation are included. Use Cargo `--timings` for build
  phases; run one Cargo operation at a time.
- [ ] Record elapsed time, peak process-tree memory, system memory/commit
  headroom, executable/link counts and artifact sizes. Distinguish logical file
  sizes from allocated disk space and hard-link copies. Describe the measurement
  method and any unavailable counters.
- [ ] Set numerical target counts, artifact/resource budgets and an acceptable
  timing variation from this baseline before changing layout or profiles. Record
  the reason for each target; avoid treating the reported 3:10 as a deadline.

Acceptance: the inventory accounts for every current test target and isolation
exception. A reproducible measurement table and explicit improvement targets
exist. An incomplete baseline run records its failure or deadline rather than
being counted as a pass.

### 2. Bounded fixtures and useful failures

- [ ] Reproduce the HTTPS failure with an independent deadline and capture the
  client exit status, stdout, stderr and server result. Fix the actual failure
  at its owning boundary, without weakening certificate rejection.
- [ ] Replace unbounded fixture accept/read/join paths with owned, cancellable
  lifecycles. Apply the same design to related streaming and hardening fixtures.
  Cleanup must release listeners, threads, child processes and temporary files
  on success, assertion failure and timeout. Preserve the original error if
  cleanup also fails.
- [ ] Make the TLS negative test prove that certificate validation was reached.
  A generic TLS initialization failure must fail the test. Retain a successful
  control that exercises the relevant verifier initialization path.
- [ ] Give timeout tests an outer watchdog independent of the FPAS timeout under
  test. Inspect process/pipe fixtures for failure paths that bypass kill/wait or
  leave readers blocked. Supervise descendant cleanup as part of that lifecycle.
- [ ] Replace released-port guesses and fixed startup sleeps with owned endpoint
  allocation and readiness reporting. Keep support at fixture or real public
  boundaries; avoid production APIs used only by tests.

Acceptance: controlled cases for no connection, an early client failure, a
stalled peer and a stuck child all finish within documented outer deadlines and
report the original failure. They leave no owned processes, threads, listeners
or scratch files. Regressions fail before their fix for the intended reason and
pass afterward; record any unavailable pre-fix proof.

### 3. Fewer integration executables

- [ ] Design coherent test entry points with case modules below topic
  directories. Start with `fpas-debug`, then apply the same review to CLI and
  runtime targets that repeatedly link the same dependency graph.
- [ ] Record an old-to-new case mapping, expected executable counts and the
  reason for every standalone target. Keep user-facing contract coverage intact;
  test deletion counts are not an optimization goal.
- [ ] Move cases and share setup within the owning test boundary. Remove
  obsolete entry files and manifest declarations so cases run exactly once.
  Document how to select a single case or group after moving it.
- [ ] Preserve GC allocator isolation, child-process exact-name dispatch and
  completion-marker checks. Verify that a renamed filter actually executes its
  intended child case, rather than reporting success with zero cases.
- [ ] Verify changed groups with ordinary harness concurrency as well as focused
  case selection. Give process-global state explicit ownership or its own target
  instead of relying on the whole suite running serially.

Acceptance: the target counts reach the step 1 targets, every old contract has
an identified owner, and each retained case is discovered once. Allocation and
process-lifecycle measurements retain their original isolation guarantees.
Grouping introduces no normal production dependency on test support.

### 4. Profiles suited to their workload

- [ ] Keep `dev-fast` with Thin-LTO for development program builds and Fat-LTO
  for release, benchmarks and dist. Evaluate Rust test compilation as a separate
  workload with normal test assertions and overflow checks enabled.
- [ ] Compare reduced test debug information, starting with `line-tables-only`,
  against the baseline. Evaluate dependency debug symbols separately. Keep the
  smallest setting that provides useful failure backtraces on Windows and Linux.
- [ ] Provide a documented opt-in profile for full native Rust debugging if the
  default test profile reduces symbols. Verify breakpoints and local inspection
  in that profile, and distinguish Rust debugging from FPAS source debugging.
- [ ] Check that profile changes preserve artifact staging and test execution
  behavior. Remove superseded configuration and avoid machine-specific flags or
  paths. Measure any extra profile rebuilds as part of the total developer cost.

Acceptance: the chosen profiles meet the recorded artifact and memory targets,
retain useful backtraces, and preserve release/dist settings. Test runtime and
program rebuild times stay within the variation agreed in step 1. Profile
selection and its rebuild cost are explicit.

### 5. Commands, scheduling and contributor documentation

- [ ] Document these separate operations in the
  [build guide](../bench/build-times.md), with exact commands for the selected
  profiles: program build, test compilation without execution, focused tests and
  the complete workspace run. Reuse Cargo artifacts between compilation and
  execution; avoid a second build with different flags or profiles.
- [ ] Keep Cargo's default build parallelism. Explain separately what Cargo
  build jobs, harness test threads and fixture/VM worker pools control. Preserve
  serial test execution as a diagnostic option rather than a blanket policy.
- [ ] Compare measured resource use with the budgets after steps 3 and 4. If
  test execution still needs automatic admission, select concurrency from
  available CPU/memory and measured case costs, with a conservative fallback
  and a manual override. Account for child processes and nested workers. Record
  an evidence-based decision if no additional scheduler is needed.
- [ ] Document required loopback, subprocess and filesystem capabilities, how
  to recognize a denied capability, and which execution environment was used.
  Report hard-link copy fallback without treating cached Cargo diagnostics as a
  new failure. Keep security settings unchanged by build/test tooling.
- [ ] Align the README, CONTRIBUTING, AGENTS and Rust architecture pointers with
  the owning build guide. Keep helper tooling small; new helper scripts follow
  the repository's FPAS dogfooding rules and record any fallback gap.

Acceptance: a contributor can reproduce the build and full test workflow from
the documented commands. Build parallelism and test scheduling remain separate.
Resource decisions cover both smaller and larger developer machines without a
hard-coded machine identity or a universal fixed worker count.

### 6. Windows and Linux acceptance

- [ ] Run focused regressions for the changed fixtures, grouping and native
  debugging support, followed by formatting and the relevant lint checks.
- [ ] Complete the full Rust workspace suite on Windows and Linux with the
  documented profiles and scheduling. Record passed/failed/ignored counts and
  account for platform-specific discovery differences. Any ignored child-only
  fixture must still be exercised through its owning test.
- [ ] Run the unchanged FPAS regression suite and the dependency-boundary
  checks. Keep language, protocol, debugger, memory and process-cleanup contracts
  covered; do not skip failing tests to complete this plan.
- [ ] Verify the development program build on both systems. Verify real dist
  output and standard-library staging wherever profile, build or staging changes
  affect them; run the corresponding script checks if those scripts change.
- [ ] Repeat the step 1 measurements under the same workload and cache
  conditions. Meet the recorded target counts, resource budgets and timing
  bounds, including complete Windows test compilation without exhausting system
  commit. Report any unmet target and keep this step open.
- [ ] Update implemented contributor documentation and record the final results.
  Remove this plan once its scope is delivered and the owning docs and tests
  cover it, following the [future-doc rules](README.md#rules).

Acceptance: both platform results are complete, every contract is accounted for,
resource improvements are measured, and all checkboxes are backed by evidence.
A successful program build alone does not complete test acceptance.

## Separate follow-ups

VM scheduling, per-process async runtimes, ACP codec code generation and compiler
artifact-identity granularity need their own measurements before a redesign.
Reopen those areas only if evidence from this work identifies them as a remaining
bottleneck. This plan does not implement the remaining FPAS 2 point 14 features.

## Cargo references

- [Integration targets and grouping](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests)
- [Reducing generated debug information](https://doc.rust-lang.org/cargo/guide/build-performance.html#reduce-amount-of-generated-debug-information)
- [Profiles and inheritance](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Build timings](https://doc.rust-lang.org/cargo/reference/timings.html)
