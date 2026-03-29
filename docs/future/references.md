# Future: Reference Types

> Planned. Adds indirection for tree structures and shared state without manual memory management.

## Motivation

All records are value types — assignment copies the entire value. This makes it impossible to build **recursive data structures with shared nodes** (e.g., a tree where a parent holds children and children can locate their parent) or to share mutable state between components efficiently.

Enums with associated data already support recursion (`List<T> = enum Nil; Cons(Head: T; Tail: List of T); end`), but they are immutable values. For mutable tree structures where nodes are updated in place, a reference or handle mechanism is needed.

## Proposed Design

### Option A: `ref` Type Constructor

A built-in `ref` modifier creates a heap-allocated, reference-counted value:

```pascal
type
  Node = record
    Value: string;
    Children: array of ref Node;
    Parent: Option of ref Node;
  end;
```

```pascal
var Root: ref Node := new Node with
  Value := 'root';
  Children := [];
  Parent := None;
end;
```

Assignment of `ref` values shares the reference (not a deep copy):

```pascal
var Alias: ref Node := Root;   { both point to the same node }
```

Dereferencing is implicit for field access:

```pascal
WriteLn(Root.Value);           { no explicit ^ or * needed }
```

### Option B: Arena / Handle Pattern

A stdlib-based approach using an integer handle into a managed arena:

```pascal
uses Std.Arena;

var A: Arena of Node := Arena.Create();
var RootId: Handle := Arena.Add(A, record Value := 'root'; ... end);
var ChildId: Handle := Arena.Add(A, record Value := 'child'; ... end);
Arena.Get(A, RootId).Children := [ChildId];
```

This keeps the language simple (no new type constructor) but is more verbose.

### Recommendation

Option A (`ref`) is more ergonomic and aligns with the language's goal of safety without manual memory. The VM already manages all allocations; adding reference counting or tracing for `ref` values is a natural extension.

## Mutability

A `ref` value follows existing mutability rules:

```pascal
var R: ref Node := new Node with Value := 'x'; ... end;   { immutable ref }
mutable var R2: ref Node := R;                              { can reassign R2 }
```

To mutate the referenced value's fields, the `ref` target must be declared mutable — exact semantics TBD.

## Docs and Specs to Extend

- [05-types.md](../pascal/05-types.md): `ref` type constructor, value vs. reference semantics
- [02-basics.md](../pascal/02-basics.md): assignment semantics for `ref` values
- [grammar.ebnf](../specs/grammar.ebnf): `RefType`, `NewExpr` productions
- VM implementation: reference counting or GC for `ref` allocations
