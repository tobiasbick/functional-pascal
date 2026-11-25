# CLI JSON report schemas

`fpas schema NAME` writes a JSON Schema 2020-12 document to stdout without
executing an application or evaluating a project. Redirect it to a file for a
schema-aware client. An unknown name or invalid argument returns one operation
failure with exit 2. Help/version remain normal CLI text.

```sh
fpas schema operation
fpas schema terminal-script
fpas schema test
fpas schema commands
fpas schema commands query find
fpas schema query-batch-input
```

| Name | Contract |
| --- | --- |
| `operation` | Profile, terminal, dump operations and schema argument failures |
| `terminal-script` | Controlled terminal-test input |
| `workspace` | Evaluated project inventory, project references and source impact |
| `project` | Init, build, run, target, publish, clean and restore |
| `compile` | Standalone compiler diagnostics |
| `check` | Standalone source checking, evaluated project/solution checking, corrections and draft analysis |
| `lint` | Semantic lint findings and source-analysis failures |
| `lint-baseline` | Existing errors covered by an explicitly selected lint baseline |
| `check-overlay-input` | New and replacement source texts supplied to `check`, `query batch` or `refactor draft` with `--overlay` |
| `query` | Checked source queries, completion and signature help |
| `query-outline` | Syntax-only file outline and parser diagnostics |
| `query-batch` | Ordered source-query results from a shared snapshot |
| `query-batch-input` | Requests accepted on stdin by `query batch` |
| `test` | Native VSTest/MTP results and discovery |
| `debug` | Batch, tracepoints, runtime-failure observations and `run --failure-report` files |
| `format` | Source formatting |
| `doc` | API documentation lookup |
| `refactor` | Checked refactoring plans and application reports |
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

The `lint` report has schema version 1. Its status is `ok`, `findings`, `failed`
or `cancelled`. Findings carry `rule`, `key`, `severity`, `message`, `project`,
`location` and `related` source locations. Cycle findings also record their
dependency edges. The optional `baseline` contains the selected path, whether
it was written, the number of covered findings and resolved entries.
Compiler diagnostics and preparation issues accompany analysis failures.
An operational failure has an `error` record. Responses above
4 MiB produce a `result_limit` error. See the [lint command](../program-structure/cli.md#lint-a-project-or-solution).

`fpas schema commands` writes a versioned command catalogue as JSON. Its
`commands` entries identify each registered public command by its full command
path. Each entry lists positional arguments and options with their descriptions,
types, required status, arity and registered defaults when available.
`maxValues: null` indicates unbounded arity. Recursive options are included where
accepted. `reportSchema` identifies the associated report contract.
`reportOption` gives the JSON opt-in flag when available. The catalogue has
`schemaVersion: 1`. Append a command path to return just that command in the
same format. A command entry can list its immediate `subcommands`. Options
with registered value restrictions include `allowedValues`, `minimum` and
`maximum` as applicable. The `rules` array describes registered option
relationships with a `kind` and ordered `options`. For `requires` the first
option depends on the second. Other rule kinds are `excludes` and `exactlyOne`.
`inputSchema` identifies input contracts such as `query-batch-input`.
A file option for that contract is identified by `inputOption`.
`optionInputSchemas` maps additional file options to their schemas when the
command also accepts a stdin contract.

`fpas query batch CONTEXT` reads a JSON array of queries from stdin. A batch
report contains one `snapshotId` and one `results` array in request order.
Each entry reports its operation, status and typed result or error. See
[batch queries](../program-structure/cli.md#batch-queries) for the input contract.
An `inspect` or `refs` entry with `symbol` and `project` additionally requires
`--expected-snapshot` on the invoking command.

## Recovery details

Analysis errors and project preparation issues can expose `project`, the
exact `inputPath` when available, and the machine-readable `nextAction`.
An issue includes `inputProject` when the input belongs to another project.
A failed snapshot precondition includes `expectedSnapshotId` and
`actualSnapshotId`.
Display the accompanying message to explain the particular failure.

For `run_output_changed`, the project report's `error.nextArguments` contains
the argument array for a separate FPAS invocation. The first element is the
subcommand. A `target` follow-up refreshes the identity of an existing output.
A `build` follow-up prepares a missing output before identity lookup.

| Action | Follow-up |
| --- | --- |
| `restore_project` | Restore the indicated project to produce its assets file. |
| `build_project` | Build the project that owns the missing or stale dependency output. |
| `prepare_source` | Generate the named source or correct the evaluated source list. |
| `review_project` | Inspect the reported project or SDK evaluation failure. |
| `refresh_snapshot` | Repeat the read against current inputs and discard stale result coordinates. |
| `recreate_plan` | Prepare and review a fresh edit plan. |
| `check_sources` | Check the current sources after edits were already written. |

## See also

- [Compiler CLI](../program-structure/cli.md)
- [Diagnostic commands](cli-diagnostics.md)
- [Terminal scripts](terminal-testing.md)
- [Native testing](testing.md)

## Source files

- [OperationReport.cs](../../../src/Fpas.Cli/Reporting/OperationReport.cs)
- [CommandConstraints.cs](../../../src/Fpas.Cli/CommandConstraints.cs)
- [QueryBatchSchema.cs](../../../src/Fpas.Cli/Reporting/QueryBatchSchema.cs)
- [CommandCatalogue.cs](../../../src/Fpas.Cli/Reporting/CommandCatalogue.cs)
- [SchemaCommand.cs](../../../src/Fpas.Cli/Reporting/SchemaCommand.cs)
- [ReportSchemaTests.cs](../../../tests/Fpas.IntegrationTests/Cli/ReportSchemaTests.cs)
