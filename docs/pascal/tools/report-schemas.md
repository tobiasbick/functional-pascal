# CLI JSON report schemas

`fpas schema NAME` writes a JSON Schema 2020-12 document to stdout without
executing an application or evaluating a project. Redirect it to a file for a
schema-aware client. An unknown name or invalid argument returns one operation
failure with exit 2. Help/version remain normal CLI text.

```sh
fpas schema operation
fpas schema terminal-script
fpas schema test
```

| Name | Contract |
| --- | --- |
| `operation` | Profile, terminal, dump operations and schema argument failures |
| `terminal-script` | Controlled terminal-test input |
| `project` | Init, build, run, target, publish, clean and restore |
| `compile` | Standalone compiler diagnostics |
| `check` | Evaluated project/solution checking and corrections |
| `query` | Checked source queries |
| `test` | Native VSTest/MTP results and discovery |
| `debug` | Batch, tracepoints, runtime-failure observations and `run --failure-report` files |
| `format` | Source formatting |
| `doc` | API documentation lookup |
| `refactor` | Existing rename plans/application |
| `environment` | Toolchain identity/availability |

Schema identifiers are `urn:fpas:cli:NAME:1`. They describe the selected command's
actual serializer, member names, nullable values, nested collections and numeric
types. Nullable members omitted by a report's serializer remain optional.
Terminal input adds its accepted version, dimensions and resource bounds.
Existing report families keep their own shapes; do not apply one family to a
different command or to DAP/LSP messages.

The operation report has schemaVersion 1, command, success and status, with an
error code/message for operational failures; incomplete traces retain their
explicit loss/truncation fields. Depending on the command it can include an
artifact path, captured process result, decoded trace summary or terminal result.
It is bounded to 4 MiB and reports overflow instead of emitting partial JSON.
The owning command reference defines statuses, exit codes, native process status,
artifact completeness and optional sections. Schema validity alone does not prove
command success, complete coverage or useful diagnostic observations.

New commands must select and document their report family and behavior before
implementation. A new family or incompatible schema change needs an explicit
schema/version contract. Existing JSON fields do not define future commands.
Full command names are the public spelling; no additional short aliases are
selected. Use each command's `--help` for examples and accepted options.

## See also

- [Compiler CLI](../program-structure/cli.md)
- [Diagnostic commands](cli-diagnostics.md)
- [Terminal scripts](terminal-testing.md)
- [Native testing](testing.md)
