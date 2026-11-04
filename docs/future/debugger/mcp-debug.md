# fpas debug: command and MCP contract

Status: **Implemented; Windows/agent-host acceptance pending**. Delivery is tracked in the [implementation checklist](implementation.md).
No FPAS syntax change or new debugger engine is required.

## Components and dependencies

- `fpas debug` hosts an MCP **server** using the official
  [C# SDK](https://github.com/modelcontextprotocol/csharp-sdk). Use its hosting,
  stdio transport, tool schemas, structured results and cancellation support.
- Use the `ModelContextProtocol` NuGet package, pinned centrally in
  Directory.Packages.props; version 2.3.1 is the selected stable version. Add
  compatible hosting dependencies required by the SDK. Do not add AspNetCore,
  HTTP hosting, a product MCP client or a second JSON-RPC implementation.
- A thin production DAP client launches the user-supplied NetCoreDbg executable
  with `--interpreter=vscode`. It correlates responses/events and owns process
  lifetime. NetCoreDbg implements breakpoints, stepping, frames, values and evaluation.
- Keep protocol/session/tools in a focused `Fpas.Debugging` library, referenced
  by Fpas.Cli. It depends on the MCP SDK and BCL, not the compiler or Fpas.Core.
  CLI argument binding remains in Fpas.Cli; Fpas.Core gains no debugger dependency.

NetCoreDbg is an external executable, not a NuGet dependency or FPAS download.
The user selects an upstream release for the target OS/architecture, extracts it,
retains its license/notices and grants execute permission where necessary.
The selected release must be tested with the project's .NET runtime.

## Command and connection

Canonical command: `fpas debug --netcoredbg <executable>`.

The agent host registers this as an MCP stdio server and starts it as a child
process. Running the command in an ordinary shell does not provide a human REPL.
The executable option is required: no implicit PATH/environment fallback,
network download, IDE debugger substitution or backend installation.

Only stdio MCP is exposed. No `--mcp`, transport-selection or raw-DAP mode is
needed. stdin/stdout belong exclusively to MCP; logs go to stderr. NetCoreDbg's
DAP pipes and target output stay separate. The program is selected by the launch
tool, not a second CLI launch path. No compilation/build is implicit.

Require an absolute backend executable path. Validate it before accepting a debug launch. Missing or
unusable configuration gives an actionable stderr error and nonzero startup exit.
MCP tools validate program/PDB/source inputs, cwd and individual argument strings.
The DLL needs its matching runtime configuration, dependencies and portable PDB.
Program and cwd paths are absolute; responses return canonical source paths.
Source identifiers remain FPAS.

## Session and tools

One active debuggee per MCP server. Starting another while one is active returns
an explicit state error. Each launch creates a new sessionId; stale session and
stopped-frame handles never silently select another program or stop.
Launch sets stopAtEntry=true so the agent can inspect the entry and set source
breakpoints before continuing. Configure user-unhandled exception stops during
launch; include backend exception details in the stopped result when supported.
Subsequent interaction is through MCP tools:

| Tool | Input and result |
| --- | --- |
| `debug_launch` | program, cwd, args; starts the backend/target and returns sessionId plus the initial stopped state |
| `debug_set_breakpoints` | sessionId, source, lines; replaces that source's breakpoint set and reports pending/bound locations |
| `debug_continue` | sessionId, stopId; resumes and returns acknowledged current state |
| `debug_step` | sessionId, stopId, kind=over/into/out; steps the thread identified by the current stop |
| `debug_pause` | sessionId; requests suspension of a running target |
| `debug_status` | sessionId, afterEvent, waitMilliseconds; returns state/current stop, breakpoint updates and an event cursor, with bounded waiting |
| `debug_threads` | sessionId, stopId; lists backend threads and identifies the stopped thread |
| `debug_stack` | sessionId, stopId, optional threadId, paging; returns source frames and frame handles |
| `debug_variables` | sessionId, stopId, frame handle or variables handle, paging; returns scopes/values and expandable handles |
| `debug_evaluate` | sessionId, stopId, frame handle, expression; forwards to the backend evaluator |
| `debug_output` | sessionId, afterSequence, page limit; returns categorized stdout/stderr and truncation/cursor information |
| `debug_stop` | sessionId; ends the session and waits for owned target/backend cleanup; repeated stop is harmless |

Use SDK-generated input/output schemas and structured content. Every tool has a
short purpose, legal states and a usage description. Reuse typed result objects;
do not make agents parse terminal transcripts or a JSON string inside a result.

States are starting, running, stopped, exited and failed. A stopped state includes
stopId, reason, thread and available source location. Results report the actual
state after the backend acknowledgement, even when a stop/exit races the request.
Every new stop has a fresh stopId. Inspect/evaluate/step reject handles from an
older stop; running state never presents cached locals as current values.

Continue/step do not wait indefinitely for the next breakpoint. The agent uses
debug_status to wait or inspect progress. A dedicated DAP reader dispatches
responses/events while pause, stop and MCP cancellation remain available; do not
hold a global tool lock while waiting for execution to stop.

## Limits, errors and output

- Startup deadline: 30 seconds; ordinary DAP response deadline: 10 seconds;
  shutdown deadline: 5 seconds. debug_status waits up to 30 seconds, defaulting
  to 1 second, and returns running when that wait expires. No whole-session or
  breakpoint-idle deadline is imposed.
- Stack and variable pages default to 32, maximum 128 items. Target output uses
  a bounded 1 MiB ring buffer and pages of at most 32 KiB UTF-8. There are at most
  64 concurrent DAP requests, with 1 MiB messages and 8 KiB
  headers. Status returns the complete current breakpoint set (up to 4096) and a
  monotonic change cursor, rather than an event backlog. Report truncation or
  expired cursors; never silently omit data or reuse invalid references.
- Return meaningful MCP tool errors with stable codes such as invalid_state,
  stale_reference, missing_symbols, backend_timeout, backend_exited and
  unsupported_capability. MCP protocol errors remain distinct from tool failures.
- Target output is tool data. It must never appear as protocol stdout. stderr
  diagnostics must not dump inherited environment variables or credentials.
- Evaluate uses NetCoreDbg expression syntax, not a Pascal interpreter. Getters,
  displays/proxies and evaluation may execute code; annotations must not falsely
  describe these operations as side-effect-free. Cancellation cannot undo effects.
- Program input is through args/cwd and application fixtures in this first scope.
  The selected NetCoreDbg DAP path does not provide target stdin/runInTerminal;
  no input tool or fictitious stdin support is exposed. A blocked input program
  can be paused/stopped. Interactive target input needs a separate scope decision.

## Ownership and shutdown

The MCP server owns its launched backend/target and never attaches to an unrelated
process. Establish an OS-appropriate ownership boundary before launching: it must
cover a target even if the adapter dies before reporting its PID. Retain process
identities/handles and use normal DAP disconnect/terminate before bounded fallback
cleanup. PID guessing or killing unrelated processes is unacceptable.

On MCP EOF, server cancellation/shutdown, failed launch or backend failure, close pipes,
finish outstanding requests deterministically and clean up owned processes. A tool
cancellation must leave a known state; if backend responsiveness cannot be restored,
end that session. Preserve terminal exit state/output until the next launch or
server shutdown. A new launch does not inherit old handles or output cursors.

Prove containment with disposable fixtures before deliberate adapter crash/hang
checks. If ownership cannot be implemented reliably on a target platform, report
that as a blocker rather than claiming successful cleanup.

## Acceptance boundaries

A real MCP client using the official SDK must discover/call these tools and debug
actual FPAS programs through unmodified NetCoreDbg. Tests cover source controls,
updated locals, captures, real async suspension, ordinary exceptions, bounded
output, invalid/stale references, cancellation and every shutdown path.

Verify the documented agent-host setup on Linux and Windows, with explicit
backend/runtime versions. IDE debugging does not need to be reopened.
Agent-driven sequences of session tools are normal debugging. One-shot batch
reports, command-file runners, custom debugger/evaluation engines, backend forks,
HTTP APIs, MCP Tasks extensions and new editor integrations are outside this scope.

References: [SDK hosting example](https://github.com/modelcontextprotocol/csharp-sdk/blob/main/docs/concepts/getting-started.md),
[SDK tools](https://github.com/modelcontextprotocol/csharp-sdk/blob/main/docs/concepts/tools/tools.md),
[NetCoreDbg](https://github.com/Samsung/netcoredbg),
[compiler debug information](../../architecture/debug-information.md).
