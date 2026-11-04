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

## Agent debugging with MCP (planned)

`fpas debug` is a planned MCP server for agents. The command is not implemented
yet; the [plan](../../future/debugger/README.md) defines its tools and delivery.
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

After implementation, register the command as a **stdio MCP server** in the
agent host. For clients using an mcpServers configuration, the planned entry is:

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

The planned sequence is debug_launch with absolute program/cwd paths and an
argument array, set breakpoints using returned source paths, continue/step, inspect stack/variables,
then debug_stop. Use the returned sessionId and current stopId. MCP traffic owns
stdin/stdout; logs use stderr and program output comes through debug_output.
Target stdin is not available in the initial scope. See the
[tool contract](../../future/debugger/mcp-debug.md) for state, errors and limits.
Batch reports and a human CLI debugger are outside this command's scope.

## See also

- [Compiled programs](../program-structure/compiled-programs.md)
- [Exceptions](../language/error-handling/exceptions.md)
- [Portable symbol implementation](../../architecture/debug-information.md)
- [.NET debugging tools](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/)
