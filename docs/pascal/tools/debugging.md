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

CLI/DAP remains [open](../../future/debugger/cli-dap.md); no FPAS debug command is
supplied. Batch debugging is blocked.

## See also

- [Compiled programs](../program-structure/compiled-programs.md)
- [Exceptions](../language/error-handling/exceptions.md)
- [Portable symbol implementation](../../architecture/debug-information.md)
- [.NET debugging tools](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/)
