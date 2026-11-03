# CLR value and reference semantics

Imported classes have reference semantics; imported structs have value semantics.
The compiler must preserve value-type receiver mutation and storage rules rather
than accidentally mutate a boxed copy. `const` bindings do not prohibit mutation
of referenced objects. A mutating method on a struct in a read-only receiver
position follows CLR/C# defensive-copy behavior; assigning the receiver's fields
directly still requires writable storage.

Imported inheritance and interface assignability participate in normal conversions
and overload resolution. Supporting imported class hierarchies does not require
user-authored inheritance syntax. A virtual call dispatches virtually; an
interface call dispatches through that interface. Boxing and unboxing follow
the rules in [types](../language/types/README.md).

Nullable annotations are read as described in [nullable types](../language/types/nullable-types.md).
Foreign code may violate its annotations; the compiler does not add a second
presence wrapper to method results. `CancellationToken`, `DateTime`, `Guid`, and other
structs are actual CLR values, not FPAS handles.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
