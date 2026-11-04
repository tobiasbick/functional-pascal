# Debugging CLR programs

FPAS emits portable PDBs, source positions, local scopes, closure names and async
metadata for ordinary CLR debuggers. IDE debugging uses official VS Code with
Microsoft's C# extension (`ms-dotnettools.csharp`).

## Launch in VS Code

Build through the [project SDK](../../build/development.md#build-an-fpas-application)
or [compiler CLI](../program-structure/cli.md), retaining matching DLL, PDB,
runtime configuration, dependencies and source files. Use a Debug build for
source stepping.

Allow breakpoints in FPAS files in workspace settings:

```json
{ "debug.allowBreakpointsEverywhere": true }
```

Add `.vscode/launch.json`, replacing Program.dll with the assembly name:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "FPAS",
      "type": "coreclr",
      "request": "launch",
      "program": "${workspaceFolder}/bin/Debug/net10.0/Program.dll",
      "cwd": "${workspaceFolder}",
      "console": "integratedTerminal"
    }
  ]
}
```

Use the normal breakpoint, step, stack and variable controls. Program input
belongs in the integrated terminal. Debug output supports Microsoft's default
Just My Code/JIT settings; no custom debugger extension or C# file association is
required. Automatic project discovery and a Pascal expression evaluator are
separate features. Watch/evaluate uses the debugger's expression syntax.

## Values and stepping

Records and union cases have source-oriented displays and typed property views
with a Raw View. Distinct/subrange domains display their decoded Value. Nullable
values and arrays use standard CLR views. Captured mutable values are reachable
through `this` -> capture field -> Value; captures are not flat locals.

Async source stepping follows the continuation and exposes parameters/hoisted
locals after suspension. Generated helpers and technical storage are hidden from
ordinary stepping/expansion. Source display/proxy attributes take precedence.
Inspection may execute getters, display code or evaluated methods. See
[debug metadata](../../architecture/debug-information.md#debugger-attributes-and-value-views)
for representation and scope limits.

## Agent debugging with MCP

`fpas debug --netcoredbg <absolute-executable-path>` hosts a stdio MCP server for agents.
It uses the official [C# MCP SDK](https://github.com/modelcontextprotocol/csharp-sdk)
and NetCoreDbg. The VS Code workflow above does not require NetCoreDbg.

Prepare the external debugger:

1. Download a matching OS/architecture archive from the
   [official NetCoreDbg releases](https://github.com/Samsung/netcoredbg/releases).
   Extract the complete archive, keeping its dependencies and license files.
2. Locate `netcoredbg` on Unix or `netcoredbg.exe` on Windows. On Unix, ensure
   it is executable. Use its absolute path; FPAS neither bundles NetCoreDbg as a
   NuGet package nor downloads it automatically.
3. Make the program's .NET runtime available to the agent-host process and its
   children. A nonstandard .NET installation may require PATH and DOTNET_ROOT.
4. Build the FPAS program in Debug and retain the matching DLL, PDB,
   runtimeconfig.json, dependencies and source files.

Register the command as a **stdio MCP server** in the
agent host. For clients using an mcpServers configuration, the entry is:

```json
{
  "mcpServers": {
    "fpas-debug": {
      "command": "fpas",
      "args": ["debug", "--netcoredbg", "/path/to/netcoredbg/netcoredbg"]
    }
  }
}
```

Replace the placeholder path; use the host's configuration syntax and ensure
`fpas` is discoverable there (see [local deployment](../../build/development.md#deploy-the-local-release-compiler)).
The host starts the server and discovers its tools;
starting a process in a shell does not register tools with an agent.

On Linux install `setsid` from util-linux. The wrapper establishes an owned
process group; on Windows it uses a kill-on-close Job Object. NetCoreDbg
3.2.0-1092 with .NET 10 is covered by Linux integration tests using the official
MCP client. Windows and a real agent-host registration require separate acceptance.

Use the tools in this order:

1. `debug_launch` with absolute `program` DLL and `cwd` paths, plus optional
   `args` as individual strings. It stops at entry and returns `sessionId` and
   `stopId`. It does not build the program. Keep the original sources unchanged;
   DLL/PDB identity and source checksums are checked before launch.
2. `debug_set_breakpoints` with `sessionId`, absolute `source` and positive
   `lines`. An empty array removes that source's breakpoints.
3. `debug_continue` or `debug_step` with the current IDs. Step kinds are `over`,
   `into` and `out`. These acknowledge execution; use `debug_status` to observe
   the next stop. Set `afterEvent` to the returned `event` cursor to wait for
   changes, up to `waitMilliseconds=30000`. A running result is normal.
4. At a stop, `debug_threads` and `debug_stack` expose threads and source frames.
   Pass a returned `frameHandle` to `debug_variables` for scopes, then a returned
   `variablesHandle` to expand values. Pages use `start` and `count` (default 32,
   maximum 128). Every resume invalidates old handles and `stopId`.
5. `debug_evaluate` accepts a current frame and NetCoreDbg expressions. It does
   not implement Pascal expressions; getters/evaluation can execute target code.
   User-unhandled exception stops include available backend exception details.
6. `debug_output` returns stdout/stderr chunks with `sequence` numbers. Pass
   `nextSequence` as `afterSequence` for the next page. The 1 MiB ring reports
   expired cursors through `truncated`; pages contain at most 32768 UTF-8 bytes.
7. `debug_pause` suspends a running program. `debug_stop` terminates the owned
   target/backend and waits for cleanup; repeating it is safe. An ended session
   retains output/status until the next launch, which creates a new `sessionId`.

Results use MCP `structuredContent`; tool errors set `isError` with an `error`
object containing `code` and `message`. Use current IDs after `stale_reference`;
rebuild matching portable symbols for `missing_symbols`, or restore the exact
source for `missing_source`. Backend timeout/failure ends the affected session.
Request cancellation cannot undo target side effects. Idle stops do not expire.

MCP traffic owns
stdin/stdout; logs use stderr and program output comes through debug_output.
Target stdin is not available in the initial scope. See the
[tool contract](../../future/debugger/mcp-debug.md) for state, errors and limits.
Batch reports and a human CLI debugger are outside this command's scope.

## See also

- [Compiled programs](../program-structure/compiled-programs.md)
- [Exceptions](../language/error-handling/exceptions.md)
- [Portable symbol implementation](../../architecture/debug-information.md)
- [.NET debugging tools](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/)
