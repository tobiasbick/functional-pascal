# Generic routines and constraints

Generic routines preserve CLR type parameters and validate their constraints.

```pascal fragment context=declarations
function Identity<T>(Value: T): T;
begin
  return Value;
end function;

function Compare<T>(Left: T; Right: T): int;
where T: Sys.IComparable<T>;
begin
  return Left.CompareTo(Right);
end function;
```

Generic arguments use `<...>` in declarations, types, and calls:
`Identity<int>(42)`. Calls may infer routine type arguments from supplied argument
types. Expected result type alone does not infer routine type arguments. See
[type inference](type-inference.md) for overload evidence and contextual variants.
Namespace-qualified and nested CLR generics are supported. Put each type's own
arguments on its name segment, for example `N.Outer<string>.Inner<int>`. An inner
type that introduces no parameters still inherits the enclosing arguments, as in
`Collections.Dictionary<string, int>.KeyCollection`. Enclosing arguments are not
repeated on the nested segment. Every segment must name an accessible type and
satisfy its arity and constraints; constraints on inner parameters may refer to
outer parameters. The same paths work in annotations, construction, casts,
`typeof`, and static member receivers.

Each `where T: ...;` follows the routine header or the type's `=` and precedes
the body/type definition. Constraints may include a base class, interfaces,
`class`, `class?`, `struct`, or `new()`. They follow CLR/C# combination rules,
with `new()` last. A record constraint example is:

```pascal fragment context=declarations
type Box<T> =
where T: class;
record
  public Value: T;
end record;
```

## Constraints

Where clauses follow the declaration header and precede begin. Each parameter
has at most one clause. Supported requirements are class, class?, struct, new(),
an accessible non-sealed CLR/source base class, imported instance interfaces,
and a non-cyclic type-parameter constraint such as `where T: U;`.
System.Enum constraints are supported, including `where T: struct, Sys.Enum;`.

Primary class/struct constraints must come first; a base/type-parameter
constraint precedes interfaces; new() must come last and cannot be combined
with struct. Duplicate, cyclic, or inconsistent constraints are errors. Nullable
reference annotations are retained on constraints. A struct-constrained
parameter cannot serve as another parameter's base constraint.

The same requirements are validated at source calls, when constructing CLR
generic types containing these parameters, and when forwarding them to CLR
methods. Constraints resolve instance members and static abstract/virtual members
of imported interfaces. For example, `where T: N.INumber<T>;` permits arithmetic,
`T.Zero`, and `T.CreateChecked(Value)` through the actual System.Numerics contract.
Static method groups such as `T.CreateChecked<int>` become ordinary CLR delegates.
Inherited contracts and default implementations retain CLR dispatch. Unrelated
applicable operators or same-name static properties are ambiguous, not selected
by constraint order. Unmanaged/byref-like constraints remain unsupported. There
are no built-in Numeric/Printable constraints or runtime reflection fallback.

```pascal
program GenericArithmetic;

uses System.Numerics as N, System.Console as Console;

function Sum<T>(Values: array of T): T;
where T: N.INumber<T>;
begin
  var Total: T := T.Zero;
  for Value: T in Values do
    Total := Total + Value;
  end for;

  return Total;
end function;

begin
  Console.WriteLine(Sum<int>([20, 22]));
end;
end program;
```

`new T()` requires struct or an explicit new() requirement and takes no arguments.
It calls the actual CLR Activator.CreateInstance<T>(), preserving the CLR's
value/reference constructor behavior. Constraints are also emitted for C# and
other consumers to enforce independently.

## Nullability and type-independent code

An unconstrained T can be a value type, a non-null reference, or a nullable
argument. Returning/passing the same T preserves that type. It does not let a
body return null as an unannotated T or assume a value is safe to dereference.
A stable null check establishes non-nullness before member use or conversion
to a non-null object. Boxing to object? is available without that check.

Class and non-null interface/base constraints prove non-nullness. A class?
constraint permits nullable reference arguments and requires validation before
dereferencing a value of T. T? requires a reference or struct constraint;
unconstrained T? syntax remains invalid. Reference T? emits nullable annotations,
while struct T? uses the actual Nullable<T> representation. Nullable annotations
remain aligned through nested generic return types and assembly boundaries.

Nonempty new T[Count] cannot initialize a non-null reference element for every
possible T, so it requires a struct element constraint or an explicitly nullable
reference element annotation. Empty arrays and initialized array literals remain
available. The pipeline example uses List<U>.ToArray() to construct fully
initialized results without weakening element annotations.

Arithmetic on an unknown T requires an applicable imported operator constraint;
the compiler does not infer one from its call sites. Runtime `is T`, explicit boxing/unboxing,
and generic null/coalescing checks retain CLR behavior. Generic resources and
enumerables use constrained interface calls and normal cleanup. Specific case
patterns on an otherwise unknown T can be expressed after boxing to object?;
the compiler does not infer a variant/constant domain for T from its call sites.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
