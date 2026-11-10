# Collections

Use CLR arrays and actual System.Collections.Generic collections. Assignment
shares reference objects; collection APIs keep their own mutation, ordering,
indexer, and exception behavior.

| Need | .NET API |
| --- | --- |
| Fixed-size indexed storage | `array of T` or rectangular CLR arrays |
| Growable ordered storage | List<T> |
| Key lookup | Dictionary<TKey, TValue>, including TryGetValue |
| Unique values | HashSet<T> |
| Projection/filtering/aggregation | Static Enumerable methods with typed delegates |
| Producer/consumer coordination | System.Threading.Channels APIs |

Import namespaces/types with aliases and use exact metadata names. List.Count
and Array.Length are properties. Indexer access uses Values[Index] or Map[Key].
A const binding can still reference a mutable collection. For-in follows the
collection's enumerable view and disposes an acquired disposable enumerator on
all exits; modification may invalidate it. The compiler does not insert a snapshot.

Extension methods are called in static form. The
[directory inventory example](../../../examples/io/directory-inventory/README.md)
uses actual LINQ Where/Select with captured state. The
[array/collection example](../language/types/arrays.md#example) demonstrates
aliasing, List, Dictionary, and initialized out arguments.

## See also

- [Arrays and collections](../language/types/arrays.md)
- [Parameter modes](../language/functions/parameters.md)
- [CLR calls and delegates](../interop/calls-and-delegates.md)
- [.NET collection reference](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic)
