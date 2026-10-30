# Test and build structure validation

Implementation record, 2026-10-30. Baseline revision: `d7c9269ee`; results apply
to the structure changes described below. The [build guide](build-times.md) owns
the commands and profile policy. Full suites are supplemented by focused reruns after the final
fixture-only lint and startup refinements.

## Structure and contracts

Integration executables decreased from 225 to 116. The debugger has five targets
instead of 73, the CLI six instead of 20, and the language service four instead
of 31. All previous case modules remain registered exactly once. The mappings
and isolation decisions live beside the [debugger](../../crates/fpas-debug/tests/README.md),
[CLI](../../crates/fpas-cli/tests/README.md), and
[language-service](../../crates/fpas-language-service/tests/README.md) tests.

Runtime allocator, terminal and lifecycle programs retain their isolated
executables. Exact-name child selection and completion-marker assertions remain
covered. Shared fixture support is confined to tests; production compiler and
LSP dependencies exclude the VM, ACP driver/codec, MCP server and ACP client SDK.
The runner depends on execution code and excludes compiler/editor code.

The test profile retains assertions and overflow checks, uses workspace line
tables and omits external dependency symbols. `test-debug` provides full native
symbols. Development programs retain Thin-LTO; release, benchmark and dist
configuration retain Fat-LTO. Cargo jobs and normal harness concurrency remain
unchanged. FPAS syntax and language contracts remain unchanged.

Network fixtures own their listeners, streams and server threads, use bounded
I/O, and report client failures before joining servers. Servers report their
actual bound endpoint. Process fixtures own their groups/jobs, capture bounded
output, observe SUT cleanup before fallback teardown, and independently time out
stuck children. Linux fallback pins observed descendant identities with pidfds;
it cannot contain an unobserved daemon double-fork.

## Verification

Rust/Cargo 1.99.1 and locked dependencies were used on Windows/MSVC and native
Linux filesystems inside WSL. Compilation and execution were separate, with one
active Cargo/build/test workload at a time. No dependency downloads were included
in the recorded measurements.

| Check | Windows | Linux |
| --- | --- | --- |
| Complete Rust workspace, normal harness concurrency | 5,651 passed; 12 ignored; no failures | 5,685 passed; 13 ignored; no failures |
| Unchanged FPAS suite | 593 passed; one intended skip; no failures | 593 passed; one intended skip; no failures |
| Clippy, all targets and features, `-D warnings` | Passed | Passed |
| Development workspace program build | Passed, Thin-LTO/default jobs | Passed, Thin-LTO/default jobs |
| Real standard-library staging | 126 compiled, zero reused | 126 compiled, zero reused |
| Reduced-symbol backtrace with functions and source lines | Verified | Verified |
| Full native-debug breakpoints, arguments and locals | Not exercised | Verified with GDB in numeric tests |

Full Rust runs include native executable/ACP distribution contracts. Release
profile settings and dist scripts were unchanged; an additional Fat-LTO release
build was not run. Staging used disposable output directories and the actual
development binaries.

Ignored entries are invoked child/terminal fixtures and one opt-in local
profiling case. The FPAS skip is the existing runner skip-contract program.
No failing case was disabled to complete validation.

After the final fixture refinements, Windows reruns passed all 44 storage-mutation,
14 failure-dump and 10 host-I/O cases. Linux reruns passed all 14 failure-dump,
10 host-I/O and three process-pipe cases. Child-only ignored entries were still
invoked through their owners. All reruns used normal harness concurrency.

Platform discovery differs by 34 passed cases and one ignored terminal fixture:

| Linux minus Windows | Cases |
| --- | ---: |
| Bundle execution permissions | +1 |
| CLI platform-specific filename, publication and process cases | +1 |
| CLI language/process/terminal cases | +6 |
| CLI tool signals, FIFO, worker and source-query cleanup | +11 |
| Debugger path case sensitivity | +1 |
| Language-service Windows path/Unicode cases | -3 |
| Project-loading Windows case alias | -1 |
| Runtime Unix pipes/signals/FIFO/paths versus Windows drive/UNC cases | +16 |
| GC Unix ACP file jobs | +1 |
| Real Linux controlling terminal | +1 |

Linux additionally discovers the ignored `foreground::inherited_terminal_host`.
Counts use each Cargo executable's final result, excluding nested child-test
summaries.

## Resource and timing measurements

The pre-change debugger build exhausted available memory and was terminated
with its owned descendants. Completed cold/warm/source-change baseline timings
are unavailable; repeating that unsafe layout was excluded. Its 302 cached PDBs
contained 63,134,744,576 logical bytes, including stale entries. This does not
measure one clean build or uniquely allocated storage.

The link-program budget was fewer than 125 integration targets. Validation
reserved at least 8 GiB of available RAM and system commit headroom. The new
layout met both budgets with default build jobs and normal test concurrency.

| Workload | Cargo time | Supervised elapsed | Minimum host available RAM / commit headroom |
| --- | ---: | ---: | ---: |
| Windows development workspace build | 221 s | 222.304 s | 38.6 / 42.5 GiB |
| Linux development workspace build | 154 s | 159.784 s | 35.4 / 37.0 GiB |
| Windows workspace test compilation | 91 s | 92.938 s | 30.8 / 24.4 GiB |
| Linux workspace test compilation | 77 s | 82.473 s | 24.4 / 26.5 GiB |
| Windows runtime-source test rebuild | 24.58 s | 25.457 s | 30.8 / 29.0 GiB |
| Windows complete Rust execution | Already compiled | 899.485 s | 40.4 / 44.0 GiB |
| Linux complete Rust execution, final run | Already compiled | 340.603 s | 32.7 / 36.8 GiB |
| Windows complete FPAS suite | Development binary | 289.404 s | 41.3 / 45.5 GiB |
| Linux complete FPAS suite | Development binary | 56.799 s | 36.0 / 38.7 GiB |

Windows compilation reused some dependency artifacts. Linux test compilation
started in a fresh test-profile tree after its separate development build.
Linux used native filesystem storage after a mounted-source diagnostic exposed
substantial filesystem overhead. These platform rows are separate measurements,
not controlled comparisons of operating systems.

Unchanged Windows test-compilation repeats took 0.40 and 0.37 seconds of Cargo
time, a 7.5 percent difference, with no dirty fingerprints. Linux initial repeats
took 1.24 and 0.13 seconds; a later consecutive series took 1.18, 0.14 and 0.14
seconds, again without dirty units. Priming and steady repetitions are recorded
separately; the subsecond rows do not establish a comparable cold-build speedup.
WSL startup and host querying also contribute to outer supervision time.

The source-change measurement replaced the TCP polling-duration literal with
an equivalent duration, compiled the workspace tests, restored the literal and
compiled again before execution. FPAS execution/cache population is included in
execution rows and excluded from `--no-run` measurements.

An artifact snapshot selected 145 test executables from Cargo output: their
logical sizes totaled 2,454,116,352 bytes, and their PDBs 7,310,700,544 bytes.
Stale cache entries were excluded. The dependency-free Linux numeric test binary
was 7,736,576 bytes with full symbols and 7,441,768 bytes with reduced symbols;
this small-crate comparison does not predict workspace PDB savings.

Memory counters are sampled system minima. Process-tree peak RSS, individual
linker-process launches and uniquely allocated disk bytes were not collected.
Free-memory deltas do not substitute for those counters. No before/after
speedup or reclaimed-disk percentage is claimed against the incomplete baseline.

## Failures repaired and controls retained

- Bounding the HTTPS fixture exposed Winsock 10022 before TLS. TCP connection
  setup now waits for Mio writable readiness before checking/configuring the
  socket. The original case passes with an actual certificate-rejection alert;
  verifier initialization has a successful control.
- Missing connections, stalled reads and early client failures finish within
  fixture deadlines. Cancellation interrupts `read_exact` through bounded I/O
  slices, and cleanup preserves the original client failure.
- Process liveness recognizes EOF and connection reset after termination while
  rejecting live peers, unexpected data and missing connections. Five real TCP
  controls cover those outcomes. Linux cleanup continues killing known pinned
  descendants after an earlier probe or stop error.
- The SUT timeout regression requires a real descendant's startup receipt and
  host-owned TCP closure before supervisor fallback. Temporarily disabling
  production termination made the live-socket assertion fail; restoring it made
  the case pass. The separate blocking-sleep timeout remains one second.
- Project loading replaces quadratic ownership comparisons with canonical
  membership sets. Alias, skipped-program and transitive-owner contracts remain
  covered. The original mounted-source diagnostic changed from a 300-second
  timeout to a pass in 121.50 seconds. Its 4.23-second native-filesystem run is a
  separate filesystem measurement.

The final watchdog control has one 20-second overall budget. After startup and
readiness, remaining work gets at most three/eight seconds; narrowing the phase
never extends that overall deadline. Its helper delays its first connection
and readiness by four seconds; work/cleanup timing begins after real readiness.
This repairs the observed startup failures without increasing the stuck-work
budget. The complete affected host-I/O group passed on Windows in 7.69 seconds
and Linux in 7.01 seconds, including the delayed-start control.

## Scheduling decision and limits

Complete execution with normal harness concurrency stayed above the validation
memory reserve on both platforms, so an additional execution scheduler was not
introduced. CPU-based Cargo/harness defaults remain available on different
machines. The FPAS guard derives a reserve from currently available RAM, checks
the platform memory budget, and supports a manual reserve override. It passed
successful-child, independent-timeout and preflight-reserve-rejection controls.

The guard samples rather than imposing a hard memory quota. A smaller or busy
machine can require more headroom or an explicit manual concurrency adjustment;
guard cancellation never counts as a completed validation. Native Linux use is
limited to compilation because new process groups/sessions escape its outer
group; test execution relies on fixture-local deadlines and descendant cleanup.
Build/test tooling changes no firewall, antivirus or system memory settings.
