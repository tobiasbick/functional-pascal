# Using .NET libraries

Use actual .NET types and packages. Project/package references supply assemblies;
source imports provide explicit aliases. The compiler's supported signatures and
the referenced package version determine which overloads are available.

| Task | APIs and examples |
| --- | --- |
| Arrays, lists, dictionaries, and projections | [Collections](collections.md) |
| Unicode, encodings, and regular expressions | [Text and regex](text-and-regex.md) |
| Files, paths, streams, and disposal | [Files and streams](files-and-streams.md) |
| Child processes, arguments, bounded capture, timeout, and cancellation | [Process execution](processes.md) |
| Environment configuration, instants, deadlines, and elapsed time | [Environment and time](environment-and-time.md) |
| Parsing, rounding, bounds, and random generators | [Numbers and random values](numbers-and-random.md) |
| Typed JSON and source attributes | [JSON serialization](json.md) |
| HTTP, cancellation, and parallel coordination | [Async APIs](../language/concurrency/cancellation-and-coordination.md) |
| Delegates, events, and interface adapters | [CLR interop](../interop/README.md) |

These are task guides, not a replacement API catalog. Consult the selected .NET
version's reference documentation for complete overloads and behavior. Fpas.Core
contains language metadata support rather than a parallel standard library.

## See also

- [Import rules](../program-structure/imports.md)
- [Project references and packages](../program-structure/projects.md)
- [Supported signatures](../interop/supported-signatures.md)
- [.NET API reference](https://learn.microsoft.com/en-us/dotnet/api/)
