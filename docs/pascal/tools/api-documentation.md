# Look up declarations and API documentation

`fpas doc` looks up FPAS declarations and public .NET APIs in an explicit project
or solution context. It uses the evaluated reference assemblies and package
versions, including APIs the FPAS source has not used yet.

```sh
fpas doc examples/numerics/mandelbrot/Mandelbrot.fpasproj --name EscapeIterations
fpas doc examples/basics/hello/Hello.fpasproj --name System.Console.WriteLine --report json
fpas doc examples/basics/hello/Hello.fpasproj --name System.Collections.Generic.List --limit 50 --report json
```

## Names and scope

Pass an existing `.fpasproj` or `.slnx`. Names are case-sensitive simple or qualified
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
`--no-restore` skips restore only. The whole operation has a 120-second deadline;
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
