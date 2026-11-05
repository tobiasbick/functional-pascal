# Construction and member access

Construct .NET objects with `new` and use their public members by their metadata
names. Import each library namespace or type with an explicit alias.

```pascal
program ConstructionAndMembers;

uses System as Sys, System.Net.Http as Http;

begin
  using const Client: Http.HttpClient := new Http.HttpClient();
  Client.Timeout := Sys.TimeSpan.FromSeconds(10.0);
end;
end program;
```

`new Type(Arguments)` constructs an accessible CLR class, struct, language record,
or constrained generic type. Constructor overload resolution is the same as for
methods. Creating a struct with `new T()` uses its accessible parameterless
constructor where one exists; otherwise it uses permitted CLR value initialization.
The compiler does not expose arbitrary reference-type `default(T)` creation.

Access members by their exact metadata names. `Value.Property` invokes the getter;
`Value.Property := Expression` invokes an accessible setter. Read-only properties
cannot be assigned. Static members use the type name. Indexers use `Value[Index]`
or `Value[X, Y]`; array indexing is distinct from indexer overload resolution.

Source aliases and named arguments do not rename metadata. Named arguments use
the selected API's actual parameter names. Most interop examples use positional
arguments to avoid depending on third-party parameter naming conventions.

## See also

- [Calls and delegates](calls-and-delegates.md)
- [Value and reference semantics](value-semantics.md)
- [File reader example](../libraries/files-and-streams.md#example)
- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
