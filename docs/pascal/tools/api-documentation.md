# Look up declarations and API documentation

`fpas doc` looks up public .NET APIs for a standalone source, project or solution.
Projects and solutions also provide checked FPAS declarations and evaluated
reference assemblies, including APIs the FPAS source has yet to use.

```sh
fpas doc examples/numerics/mandelbrot/Mandelbrot.fpasproj --name EscapeIterations
fpas doc examples/basics/hello/main.fpas --name System.IO.File.OpenRead --report json
fpas doc examples/basics/hello/Hello.fpasproj --name System.Console.WriteLine --report json
fpas doc examples/basics/hello/Hello.fpasproj --name System.Collections.Generic.List --limit 50 --report json
```

## Names and scope

An existing `.fpas` path selects the same .NET 10 reference pack and `Fpas.Core`
as [single-file check](../program-structure/cli.md#check-one-source-file). The
lookup reads metadata and XML independently of the program's contents, so it
also works while the program has errors. `--usage` checks the template against
those same references. Coverage in this mode describes the searched metadata,
and the result lists the searched assemblies with an empty `searchedProjects`.
Narrow broad searches by name or assembly to stay within 1,000 matches.

Names are case-sensitive simple or qualified
declaration names. `--match exact` is the default; prefix and contains are literal
alternatives. An exact type/container match also includes its directly declared
members. Inherited members remain under their declaring types. A method name
returns the matching overloads. Same-named declarations remain separate entries.

Use source-qualified names for FPAS and namespace/type/member names for metadata.
Nested CLR types use dots; generic type names can omit the metadata arity suffix
or include it, such as ``List`1``. Generic instantiation and expression parsing are
not part of name matching. Constructors have the simple name .ctor.

`--project` restricts source owners and reference lookup to one analyzed FPAS
project. `--assembly` selects an exact assembly simple name without .dll.
Solution configuration/platform mappings and dependency preparation follow
[check/query](../program-structure/cli.md). Configuration defaults to Debug;
`--no-restore` skips restore only. `--existing-inputs` also skips dependency builds
and preparation hooks, using existing build inputs. The operation has a 120-second deadline;
`--timeout` accepts 1..3600 seconds. Cancellation joins owned preparation processes.

Lookup uses checked regions of the selected FPAS scope. Inspect coverage.complete
and coverage.semantics before treating source matches or totals as exhaustive.
Partial searches can return checked signatures and documentation despite errors
elsewhere. An empty partial search is ok with incomplete coverage; not_found
establishes absence only in a complete search. If no checked source scope is
available, lookup fails explicitly. Text output identifies incomplete coverage;
the shared library result also carries it. The command does not emit the selected
root's assembly. Restore/dependency builds can write ordinary bin/obj files and
execute build tasks.

## Signatures and documentation

| Origin | Signature | Documentation |
| --- | --- | --- |
| source | Checked FPAS signature and declaration location | Attached Markdown, using the existing inspection service |
| metadata | CLR metadata rendering, assembly path and .NET XML documentation ID | Matching member from the adjacent assembly-named .xml file |

FPAS dependencies available as analyzed source use that source view without a
second metadata copy. Narrowing source owners can expose a dependency through
its public binary API instead. C#/F# source is not indexed; those dependencies
are described from their actual reference metadata.

Source Markdown uses the same comment preparation as editor documentation:
normalize CRLF and CR to LF, remove the `//` prefix and at most one following
ASCII space, and retain all remaining indentation and Markdown syntax.

External signatures identify types, constructors, methods, properties/indexers,
fields, events, generic parameters/constraints, parameter modes and metadata
defaults. Accessor methods are represented by their properties/events. Only
public types with public enclosing types and public members are included.
The CLR rendering is not FPAS source and does not certify that every signature
is callable from FPAS. Reference nullability annotations and all attribute-derived
signature conventions are not reconstructed; use the actual compiler's
[supported-signature rules](../interop/supported-signatures.md) when writing calls.

XML content is preserved, including parameter/return sections and references.
Text output renders it for reading; JSON retains the member XML. Includes and
inheritdoc are preserved rather than resolved or fetched. Lookup never loads
reference assemblies into the runtime or invokes initializers, getters or
attribute constructors.

Documentation availability is explicit: available, no_documentation for source,
file_missing, member_missing, or a budget omission. Missing text does not hide
the declaration. Malformed selected XML or a mismatched assembly identity fails
with documentation_invalid. External XML entities and DTDs are prohibited.
Only the adjacent sidecar is searched; there is no network or package-wide scan.

## Generate checked FPAS usage

Look up the declaration with `--name` and pass its exact documentation ID to
`--usage`. Quote the ID in the shell. Use an `M:` ID for methods and constructors,
a `P:` ID for properties and indexers, or an `F:` ID for fields.

```sh
fpas doc examples/basics/hello/Hello.fpasproj --name System.IO.File.OpenRead
fpas doc examples/basics/hello/main.fpas --usage 'M:System.IO.File.OpenRead(System.String)' --report json
```

For this overload the generated unit imports `System.IO as IO`, takes
`Path: string` and opens the file with
`using const Resource: IO.FileStream := IO.File.OpenRead(Path);`.
The caller supplies the path. The lookup never opens a file through the selected API.

For an instance call, supply the receiver as the first input. For example,
`M:System.String.Substring(System.Int32)` generates `Receiver.Substring(StartIndex)`
with a `string` receiver. A value-type receiver is passed by `ref` so mutations
affect the caller's variable. Interface and virtual calls are checked against
the selected declaration. The actual receiver determines runtime dispatch.

When two call inputs share a type, the template labels all arguments with their
CLR parameter names in declaration order. For example,
`M:System.String.Substring(System.Int32,System.Int32)` generates
`Receiver.Substring(startIndex := StartIndex, length := Length)`. The comparison
uses the concrete input types after generic substitution and treats nullable
reference annotations alike. A label keeps the metadata spelling, including
case and FPAS keyword escaping. FPAS record constructors use the field names.
A checked positional call is used when names
are missing, duplicated or unusable in FPAS, or when labels change overload
resolution. Indexers keep their bracket syntax.

For a constructor, use the overload's `#ctor` ID. For example,
`M:System.Text.StringBuilder.#ctor(System.Int32)` generates
`new Text.StringBuilder(Capacity)`. Only constructor arguments appear in `inputs`.

For a generic declaration, repeat `--type-arg` in declaration order, starting
with the outermost declaring type and ending with the method. Use FPAS types
such as `int`, `array of string` or a fully qualified
`System.Collections.Generic.List<string>`. The compiler resolves these against
the project's references and checks the constraints before verifying the call.

```sh
fpas doc examples/basics/hello/Hello.fpasproj --usage 'M:System.Array.Empty``1' --type-arg int
```

Each result identifies the assembly and its owning project when applicable. Use
`--project` and `--assembly` to narrow a solution lookup. The compiler checks
the generated unit in isolation against that project's references and verifies
that the expression binds to the selected method, property accessor or field
definition. It never runs the unit.

| `entry.usage.status` | Meaning |
| --- | --- |
| `template` | `source` is a checked FPAS unit. Supply the typed parameters listed in `inputs` when calling its `Use` routine. |
| `example` | `source` is a checked FPAS unit whose `Use` routine needs no arguments. |
| `unavailable` | `reason` explains why this selection could not produce a checked call. No source is offered. |

The generator supports closed generic types in signatures, nullable metadata,
`ref`/`out`/`in`, and arrays for supported by-value span parameters. Ordinary
results are returned. Standard task results can be awaited, and disposable
resources can be scoped with `using` or `await using` when the API transfers
ownership to the caller. Check the API contract for borrowed resources. A
wrapper with by-reference inputs returns the task or async-only disposable
to its caller for awaiting or cleanup. A task constructor returns the new task.
If a span adaptation would
select an array overload, the result explains the failed verification.
Missing or unsuitable type arguments, operators and direct accessor method IDs report
`unavailable`. Unknown IDs follow the normal lookup coverage
and `not_found` rules.

### Read or write a member

Properties, indexers and fields default to `--access read`. The generated
function returns the member value directly, including borrowed resources.
With `--access write`, the generated procedure takes the new value as its
last input. Index arguments follow the receiver in declaration order.

```sh
fpas doc examples/basics/hello/Hello.fpasproj --usage 'P:System.String.Length'
fpas doc examples/basics/hello/Hello.fpasproj --usage 'P:System.Text.StringBuilder.Chars(System.Int32)' --access write
fpas doc examples/basics/hello/Hello.fpasproj --usage 'F:System.Int32.MaxValue'
```

These generate `return Receiver.Length;`, `Receiver[Index] := Value;` and a
read of the static constant. A write requires a public setter or a writable
field. A private or init-only setter, a constant or a read-only field gives
`unavailable` with a reason for the requested write.

## Paging, consistency and reports

`--limit` defaults to 20 entries and accepts 1..100. `--offset` starts a later page.
Entries sort by owning project, source before metadata, qualified name, then source
position or metadata location. Results contain totalCount, offset, nextOffset,
truncated and omittedDocumentationCount. A missing name returns not_found; an
offset beyond an existing result returns an empty page with its actual total.

Combine pages only when snapshotId matches. The identity covers the evaluated
source/reference context and adjacent XML contents or absence. Inputs are
reverified before publication; changed inputs produce inputs_changed.

Documentation text shares a 64 KiB UTF-8 budget per page. Blocks that do not fit
are omitted whole with text_budget; source inspection can also report source_budget.
Identity/signature entries remain. XML input is bounded to 64 MiB bytes and
16 MiB characters. JSON output is bounded to 4 MiB; excess output is a result_limit
failure rather than partial JSON.

`--report json` owns stdout with one document, including errors. It contains
success/status, context/configuration, snapshotId, analysis coverage/preparation,
diagnostics, result entries and error. Human results go to stdout; diagnostics
go to stderr. Exit codes are 0 for success/not_found/help, 1 for analysis,
documentation, setup, deadline or limit failures, 2 for invalid arguments and
130 for user cancellation.

## See also

- [Source queries](../program-structure/cli.md)
- [Comments and XML documentation](../language/basics/comments.md)
- [.NET interoperability](../interop/README.md)

## Source files

<details>
<summary>Browse 29 source files</summary>

- `src/Fpas.Cli/Documentation/`
  - [DocCommand.cs](../../../src/Fpas.Cli/Documentation/DocCommand.cs)
  - [DocReport.cs](../../../src/Fpas.Cli/Documentation/DocReport.cs)
  - [StandaloneDocumentationTests.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/StandaloneDocumentationTests.cs)
- `src/Fpas.Compiler/Analysis/Usage/`
  - [ApiUsageTemplates.cs](../../../src/Fpas.Compiler/Analysis/Usage/ApiUsageTemplates.cs)
  - [UsageNames.cs](../../../src/Fpas.Compiler/Analysis/Usage/UsageNames.cs)
  - [UsageSource.cs](../../../src/Fpas.Compiler/Analysis/Usage/UsageSource.cs)
  - [UsageTypeArguments.cs](../../../src/Fpas.Compiler/Analysis/Usage/UsageTypeArguments.cs)
  - [UsageUnit.cs](../../../src/Fpas.Compiler/Analysis/Usage/UsageUnit.cs)
- `src/Fpas.Workspaces/Documentation/`
  - [BoundDocumentation.cs](../../../src/Fpas.Workspaces/Documentation/BoundDocumentation.cs)
  - [DocumentationInputs.cs](../../../src/Fpas.Workspaces/Documentation/DocumentationInputs.cs)
  - [DocumentationMarkdown.cs](../../../src/Fpas.Workspaces/Documentation/DocumentationMarkdown.cs)
  - [DocumentationQuery.cs](../../../src/Fpas.Workspaces/Documentation/DocumentationQuery.cs)
  - [DocumentationResult.cs](../../../src/Fpas.Workspaces/Documentation/DocumentationResult.cs)
  - [MetadataCatalog.cs](../../../src/Fpas.Workspaces/Documentation/MetadataCatalog.cs)
  - [MetadataSignatureTypes.cs](../../../src/Fpas.Workspaces/Documentation/MetadataSignatureTypes.cs)
  - [MetadataSignatures.cs](../../../src/Fpas.Workspaces/Documentation/MetadataSignatures.cs)
  - [XmlDocumentation.cs](../../../src/Fpas.Workspaces/Documentation/XmlDocumentation.cs)
- `src/Fpas.Workspaces/Queries/`
  - [SourceQuery.Documentation.cs](../../../src/Fpas.Workspaces/Queries/SourceQuery.Documentation.cs)
- `tests/Fpas.Compiler.Tests/Metadata/`
  - [DocumentationTests.cs](../../../tests/Fpas.Compiler.Tests/Metadata/DocumentationTests.cs)
- `tests/Fpas.IntegrationTests/ApiDocumentation/`
  - [DocumentationBoundsTests.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/DocumentationBoundsTests.cs)
  - [DocumentationCliTests.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/DocumentationCliTests.cs)
  - [DocumentationFixture.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/DocumentationFixture.cs)
  - [DocumentationScopeTests.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/DocumentationScopeTests.cs)
  - [MemberAccessUsageTests.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/MemberAccessUsageTests.cs)
  - [MetadataDocumentationTests.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/MetadataDocumentationTests.cs)
  - [NamedArgumentUsageTests.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/NamedArgumentUsageTests.cs)
  - [UsageTemplateTests.cs](../../../tests/Fpas.IntegrationTests/ApiDocumentation/UsageTemplateTests.cs)
- `tests/Fpas.IntegrationTests/Build/`
  - [MsBuildTests.Documentation.cs](../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Documentation.cs)
  - [MsBuildTests.DocumentationIds.cs](../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.DocumentationIds.cs)
- `tests/Fpas.IntegrationTests/Cli/`
  - [DocumentationCliTests.cs](../../../tests/Fpas.IntegrationTests/Cli/DocumentationCliTests.cs)

</details>
