# Future: Generics Extensions

> Deferred. Core generics (functions, records, enums, type aliases, type inference) are implemented — see the main docs.

## Open

- **Constraints / bounds** — e.g., `<T: Comparable>` or `where T: Printable`
- **Higher-kinded types** — e.g., `<F<_>>`
- **Variance annotations** — covariance / contravariance
- **Generic methods on non-generic types** — currently all type params must be at the type level
- **Default type arguments** — e.g., `<T = integer>`
- **Specialization** — alternative implementations for specific types

## Examples

### Linked List

```pascal
type
  List<T> = enum
    Nil;
    Cons(Head: T; Tail: List of T);
  end;

function Prepend<T>(L: List of T; Value: T): List of T;
begin
  return List.Cons(Value, L)
end;

function ListLength<T>(L: List of T): integer;
begin
  case L of
    List.Nil: return 0;
    List.Cons(_, Rest): return 1 + ListLength(Rest)
  end
end;

begin
  var L: List of integer := List.Nil;
  var L2: List of integer := Prepend(Prepend(L, 1), 2);
  WriteLn(ListLength(L2));  { 2 }
end.
```

### Generic Pair with Map

```pascal
type
  Pair<A, B> = record
    First: A;
    Second: B;
  end;

function MapFirst<A, B, C>(
  P: Pair of A, B;
  F: function(X: A): C
): Pair of C, B;
begin
  return record
    First := F(P.First);
    Second := P.Second
  end
end;

begin
  var P: Pair of integer, string := record First := 42; Second := 'hello' end;
  var Q: Pair of string, string := MapFirst(P,
    function(X: integer): string begin return IntToStr(X) end);
  WriteLn(Q.First);  { '42' }
end.
```
