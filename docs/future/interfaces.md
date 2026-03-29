# Future: Interfaces

> Planned. Adds polymorphic dispatch to Functional Pascal without classical OOP inheritance.

## Motivation

Records with methods cover concrete types well, but there is no way to write a function that accepts "any type with a `Draw` procedure" or "any type that supports `Length`". Today the only workaround is storing function-typed fields inside a record (a manual vtable), which is verbose and error-prone.

Interfaces provide a first-class mechanism for **ad-hoc polymorphism**: a type declares that it satisfies a contract, and generic or polymorphic code can operate on any type that fulfills that contract.

## Proposed Syntax

### Declaring an Interface

```pascal
type
  Drawable = interface
    procedure Draw(Self: Drawable; X: integer; Y: integer);
    function Bounds(Self: Drawable): Rect;
  end;
```

Each method lists `Self` typed as the interface name. The actual type is substituted at call time.

### Implementing an Interface

A record opts in to an interface with an `implements` clause:

```pascal
type
  Button = record
    Label: string;
    X: integer;
    Y: integer;
    Width: integer;
    Height: integer;

    implements Drawable;

    procedure Draw(Self: Button; X: integer; Y: integer);
    begin
      GotoXY(X, Y);
      Write('[' + Self.Label + ']')
    end;

    function Bounds(Self: Button): Rect;
    begin
      return record X := Self.X; Y := Self.Y; W := Self.Width; H := Self.Height end
    end;
  end;
```

### Using an Interface as a Parameter Type

```pascal
procedure RenderAll(Items: array of Drawable);
begin
  for Item: Drawable in Items do
    Item.Draw(Item.Bounds().X, Item.Bounds().Y)
end;
```

### Interface Composition

An interface can extend another:

```pascal
type
  Interactive = interface extends Drawable
    function HandleEvent(Self: Interactive; E: Event): boolean;
  end;
```

## Constraints Integration

The existing built-in constraints (`Comparable`, `Numeric`, `Printable`) could be unified as compiler-known interfaces, making the constraint system a subset of the interface system.

```pascal
function Max<T: Comparable>(A: T; B: T): T;
begin
  if A > B then return A else return B
end;
```

## Type Erasure

Like generics, interfaces use type erasure at the VM level. The compiler emits dynamic dispatch through a method table; no monomorphization is required.

## Docs and Specs to Extend

- [05-types.md](../pascal/05-types.md): add interface declarations, `implements` clause on records
- [04-functions.md](../pascal/04-functions.md): interface-typed parameters
- [grammar.ebnf](../specs/grammar.ebnf): `InterfaceDecl`, `ImplementsClause` productions
- Generics docs ([05-types.md § Constraints](../pascal/05-types.md#constraints)): unify built-in constraints with interfaces
