# Toolchain environment

Use `fpas env` to inspect the running compiler, .NET installation, cached FPAS
SDKs, the packaged debugger and the stdio language server. The command does not evaluate projects, build, restore
packages or download tools. Missing components remain visible in the report.

## Commands

```sh
fpas env
fpas env --json
fpas env --directory examples/mandelbrot --json
```

Without `--json`, output is an English summary. JSON owns stdout and contains one
report, including argument errors. `--directory` selects the working directory
for .NET SDK discovery; it defaults to the current directory. Normal `global.json`
selection applies. It does not select or analyze an FPAS project.

`--timeout` accepts 1..3600 seconds and defaults to 30 for the complete inspection.
Version probes use owned processes with closed stdin and bounded captured
streams. Cancellation or a deadline terminates and joins those probes.

## What is inspected

- `fpas`: paths, informational versions and SHA-256 hashes of the running CLI,
  compiler and Core assemblies, plus the expected FPAS SDK package version.
  A CLI extracted from an SDK uses that package's manifest version.
- `runtime`: the framework, operating system and process architecture running
  the CLI. This is separate from the SDK selected for future builds.
- `dotnet`: the host selected by `DOTNET_HOST_PATH`, then `DOTNET_ROOT`, then
  `PATH`, following build/run host selection. Reports installed SDKs/runtimes
  and the result of `dotnet --version` in the selected directory. A broken
  override is reported without silently choosing another host.
- `sdkCache`: cached `Fpas.NET.Sdk` versions in the active NuGet global-packages
  directory. `NUGET_PACKAGES` takes precedence; otherwise the selected host
  queries NuGet's configured cache location. Compiler/Core content hashes
  distinguish `matching`, `different` and `incomplete` packages, including stale
  files under an unchanged package version.
- `debugger`: passive deployment facts: status, managed/shim versions, selected
  RID and shim path/hash, required runtime and available `batch`/`dap` protocols.
  It does not load the native shim or start a worker/target.
- `languageServer`: deployment status, managed assembly path/version/SHA-256,
  protocol `3.17`, transport `stdio` and command `fpas lsp`. Inspection neither
  starts the server nor creates a workspace. Capabilities are negotiated when
  a client initializes a [language-server session](language-server.md).

The cache is an installation inventory, not proof of which SDK an arbitrary
project will resolve. Other package versions may contain different compilers;
only a mismatch in the expected version is an error. An uncached expected SDK
is a warning: it may still be available from the matching distribution feed.
Follow the [deployment workflow](../../build/development.md#deploy-the-local-release-compiler)
before resolving that SDK for a project.

A missing packaged shim or unsupported platform is an error. No target program
or debugger session is launched. The report includes tool paths and versions, not environment-variable
dumps or package-source credentials.

## Results and exits

JSON schema version 3 provides `success`, `status`, `directory`, the component
records above, `issues` and `truncated`. Each issue has severity, code, message
and an optional path. Component inspection can fail while other evidence remains
available. Invalid arguments, cancellation and deadlines return a failure report;
deadline/cancellation reports do not promise partial component data.

| Exit | Meaning |
| --- | --- |
| 0 | Inspection finished without error-severity issues; warnings may remain. |
| 1 | Component errors, inspection failure, deadline or result limit. |
| 2 | Invalid arguments. |
| 130 | Cancellation. |

Inventories are bounded to 128 entries each. The expected FPAS SDK is always
checked, including in a larger cache. Omitted cache versions produce a warning,
`sdkCache.status: partial` and `truncated: true`; they do not hide a mismatch in
the expected version. A .NET inventory beyond its bound is an error. Hashing is
bounded to 64 MiB per assembly, probe output to 1 MiB and the final JSON to 1 MiB.

## See also

- [Tools](README.md)
- [Project commands](../program-structure/project-commands.md)
- [Debugging](debugging.md)
