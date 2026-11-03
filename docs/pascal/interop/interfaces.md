# Implementing CLR interfaces

Declare the imported interfaces a record implements in its header, then provide
their required members. This allows an FPAS value to participate in .NET APIs
that accept an interface, such as a custom collection comparer.

```pascal fragment
type Descending = record implements Collections.IComparer<int>
  function Compare(Left: int; Right: int): int;
  begin
    return Right.CompareTo(Left);
  end function;
end record;
```

The record header lists imported interface types after `implements`, separated
by commas. Public instance methods satisfy compatible interface members. Public
read-only record properties may satisfy matching getter-only interface properties;
private members and static methods do not satisfy instance contracts.
Signatures match CLR names, types, generic arity, and parameter modes; parameter
labels need not match. Calling through the interface uses that interface's labels.

Every required abstract member must be implemented, including inherited members.
Existing default implementations may be used. Conflicting same-name members with
incompatible signatures produce an error; explicit-interface implementations are
not in the baseline. Setter properties, indexers, events, static abstract members,
and byref-like signatures cannot be authored as interface implementations in this
baseline. Such an interface can still be consumed where its signatures are
otherwise supported. There is no user-authored interface declaration syntax.

## See also

- [Complete comparer and JSON example](../libraries/json.md#example)
- [Records](../language/types/records.md)
- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
