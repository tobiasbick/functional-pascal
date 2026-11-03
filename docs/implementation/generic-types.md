# Generic records and unions

This milestone extends [records](records.md), [unions](unions-and-matching.md),
and [generic routines](generic-routines.md) with source generic data types.
They emit actual CLR generic classes and retain their types across FPAS and C#
assembly boundaries. The [generic tree](../../examples/generic-tree/README.md)
is an executable multi-source example.

```pascal
type Box<T> = record
  Value: T;
  Label: string := 'box';

  function Get(): T;
  begin return self.Value; end function;

  function Map<U>(Transform: function(T): U): Box<U>;
  begin return new Box<U>(Transform(self.Value)); end function;
end record;

type Tree<T> = union
  Empty;
  Leaf(Value: T);
  Branch(Left: Tree<T>; Right: Tree<T>);
end union;
```

## Construction, constraints, and methods

Type parameters follow the type name. All bindings, fields, payloads, routine
parameters, and results still have explicit types. Construct with
`new Box<int>(42)` or `Tree<int>.Leaf(42)`; a fieldless case is `Tree<int>.Empty`.
Type references and record/CLR static type receivers keep explicit type arguments.
[Union-constructor inference](type-argument-inference.md) also accepts Tree.Leaf(42)
and derives Tree.Empty from a matching expected type.
Records and unions are invariant: Box<string> does not convert to Box<object>.

Where clauses follow the equals sign and precede record/union:

```pascal
type Factory<T> =
where T: class, new();
record
  Value: T := new T();
end record;
```

The constraints and nullable rules are those of [generic routines](generic-routines.md#constraints).
They are checked on construction, in field/signature types, and when a routine
forwards its own parameters. All source field shapes are declared before
validating constructor constraints, so a forward-declared record with required
fields cannot accidentally satisfy new(). Public type/method constraints cannot
expose private types. Type-parameter names cannot shadow enclosing type parameters,
source types, imported aliases, predefined types, self, or args.

An instance or static method may add its own parameters, as Map<U> does above.
Its body and constraints can use both T and U. It cannot redeclare or add a where
clause for the enclosing T. Calls may supply explicit method arguments or infer
them from supplied values.
Closures can capture self, generic values, and mutable local cells, including
nested closures that use both sets of parameters.

Defaults are checked once in declaration scope for every valid instantiation.
They may construct a constrained new T(), call source routines, or return lambdas.
Omitted defaults run only during construction; with updates keep existing values.
Private fields/factories and generated equality/hash behavior retain their
[record semantics](records.md). Box<string?> keeps a nullable field, and a
struct-constrained T? uses actual Nullable<T> storage.

## Recursive unions and matching

Generic unions can recursively reference themselves or other source types through
CLR references. A value never substitutes null for a fieldless case.

```pascal
function Count<T>(Value: Tree<T>): int;
begin
  return case Value of
    when Tree.Empty: 0;
    when Tree.Leaf(const Item: T): 1;
    when Tree.Branch(const Left: Tree<T>, const Right: Tree<T>):
      Count<T>(Left) + Count<T>(Right);
  end case;
end function;
```

Pattern case names use the ordinary union owner. The selector supplies that
owner's type arguments, including in nested payload patterns. Payload annotations
must match the substituted field types exactly; another instantiation or another
generic union cannot match accidentally. Existing exhaustiveness, guard,
alternative-binding, and unreachable-arm rules apply to each closed union.
The same checking works when the union comes from another FPAS assembly.

## CLR representation and verification

A source type Box<T> is a sealed nested ``Box`1`` class. Tree<T> is an abstract nested
``Tree`1`` class; its generated nested LeafCase and BranchCase classes inherit the
owner's parameters and derive from Tree<T>. C# names them Tree<T>.LeafCase,
without additional case-specific type arguments. Factories remain on Tree<T>.
Generated type parameters carry the same constraints and nullable metadata.

Type fields/signatures encode !n, while generic method parameters encode !!n.
Constructed owners use TypeSpec plus MemberRef signatures; method instantiations
add MethodSpec. Closures have their own combined generic parameter list and
translate both scopes into the appropriate !n slots. No field is erased to object;
dispatch uses ordinary CLR calls and no generated C#. Constrained new T() keeps
the existing Activator.CreateInstance<T>() lowering.

Version 1 of the existing Core record/union metadata contracts also describes
generic definitions. Readers substitute the owner's arguments before validating
property, copy/default helper, constructor, case base, and factory signatures.
Field order, readonly payloads, visibility, and nullable annotations remain checked.
Core still supplies metadata attributes only; the CLR executes all generic code.
See the [CLR mapping](../specs/clr-mapping.md#generic-data-types).

Tests inspect distinct CLR type/method slots and nested closure parameters, execute
records/unions with value/reference/nullable arguments, and exercise recursive
matching and equality. Mixed builds use a model library, a separate FPAS consumer,
and a C# host with nullable warnings treated as errors. They verify imported
defaults/copy helpers, generic method/constructor constraints, nested cases,
closures, and rejection of a nullable field returned as non-null string. The
tree example builds and runs through its actual FPAS project.

Fully general nested generic type paths in FPAS annotations, globals,
and LSP/query tooling remain pending.
[Primitive numbers, aliases, enums, and data domains](scalar-types.md) are implemented.
[XML documentation](xml-documentation.md) is implemented.
[Source overloads, record CLR interfaces, async, and portable PDBs](overloads-interfaces-debugging-and-async.md)
are implemented.
