# CLI integration tests

Cargo builds six integration executables. Automatic discovery is disabled in
the manifest, so root case files remain at their existing relative include paths
without also becoming independent test executables.

| Target | Case files | Contract |
| --- | --- | --- |
| `language` | `acp_types`, `calendar`, `collection_options`, `contract_policy`, `error_mapping`, `fs_handles`, `json_streams`, `proc_options`, `text_diagnostics` | Nominal types, diagnostics and configuration across source, cached units and program images |
| `tools` | `batch_debug`, `debug_network`, `profile_report`, `source_queries` | Public debugger, profiler and source-query process boundaries |
| `host_io` | `console_eof`, `mcp_workers`, `test_timeout` | Stream EOF, MCP lifecycle and independent timeout supervision |
| `distribution` | `acp`, `native_executable` | Distributed declarations and executable bundles without source files |
| `proc_pipes` | `proc_pipes` | Raw interactive bytes, process ownership and DAP stdin/reader lifecycle |
| `failure_dump` | `failure_dump` and its existing child modules | Failure capture, publication, limits and native cleanup barriers |

Every former root entry belongs to exactly one row. Existing assertions and
submodule paths remain in their original case files. For a focused case, use
`cargo test -p fpas-cli --test language calendar::` or select another target and
module prefix from this table.

`proc_pipes` stays standalone because its FPAS programs re-execute the current
test binary with exact root names `pipe_child` and `owned_child`. `failure_dump`
keeps the exact child name `native::blocked_process_fixture`. These ignored
fixture cases are exercised by their owning parent tests.

Shared process fixtures live in `../test_support/process.rs`. Their deadline
supervises the real process independently of the FPAS timeout under test. Each
fixture owns an OS process group or Windows job, captures bounded output while
the process runs, and terminates the owned tree and reaps its direct child before
joining readers.
Cleanup assertions inspect the SUT's descendants before fallback teardown can
hide a leak. Unix leader observation preserves the unreaped process identity;
Windows retains the owned process/job handles. The outer-watchdog control
starts a descendant in a separate Linux session or nested Windows job.

The SUT timeout regression requires the real descendant's startup receipt and
checks its host-owned TCP connection before fallback cleanup. Its five-second
execution budget includes native-child startup; the separate blocking-sleep
regression retains the one-second budget.

Watchdog controls have one 20-second setup budget, then narrow the remaining
deadline to at most three or eight seconds after real readiness. The original
overall deadline is never extended. A control deliberately waits four seconds
before connecting to keep startup separate from the three-second stuck-work
deadline. Cleanup timing starts after setup.

Linux fallback also tracks observed descendants through `/proc` PID/start-time
identities, pins them with pidfds, and signals them before the leader. Unix pipe
readers use nonblocking I/O and explicit cancellation, so a missed descendant
cannot leave the fixture stuck joining a reader. Tracking is best effort for
descendants that daemonize before any ancestry sample; it is not a sandbox or a
replacement for the SUT's process ownership contract. Windows jobs provide the
outer containment for nested jobs. Linux tracking requires pidfd support.
