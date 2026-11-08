# .NET interoperability

The compiler reads supplied assembly metadata and emits ordinary CIL calls,
constructors, fields/properties, and interface dispatch. Imports resolve the
actual API rather than a handwritten wrapper catalog.

| Topic | Description |
| --- | --- |
| [Construction and members](construction-and-members.md) | New, fields, properties, and indexers |
| [Value and reference semantics](value-semantics.md) | Classes, structs, receiver storage, and boxing |
| [Calls and delegates](calls-and-delegates.md) | Generic calls, overloads, parameters, and extension methods |
| [Protocols and interfaces](interfaces.md) | Declare CLR interfaces with `protocol` and implement them using records |
| [Events](events.md) | Subscription/removal and retained delegate identity |
| [Attributes](attributes.md) | Source annotations and actual CLR metadata targets |
| [Supported signatures](supported-signatures.md) | Explicit exclusions and precise diagnostics |

Use IDisposable/IAsyncDisposable through the language's
[resource declarations](../language/error-handling/README.md). Await supported
[task APIs](../language/concurrency/README.md) directly. CLR identity and lowering
are specified in [CLR mapping](../../specs/clr-mapping.md).

## See also

- [Using .NET libraries](../libraries/README.md)
- [Projects and references](../program-structure/projects.md)
- [Types](../language/types/README.md)
