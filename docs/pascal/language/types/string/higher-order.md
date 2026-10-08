# Higher-order string operations

String `Map`, `Filter`, and `Reduce` process a string eagerly, one Unicode scalar at a time from left to right. Each callback receives the current scalar as a one-scalar `string`. A scalar is not necessarily a user-perceived grapheme cluster: `e` followed by a combining accent invokes the callback twice. These functions do not enable `for ... in` iteration over strings.

No import is required. Calls such as `'a*b'.Map(ReplaceStar)` select the
string catalog operation by type and name, independently of visible free routines.

## `Text.Map(F: function(C: string): string): string`

Calls `F` once per scalar and returns the mapped scalars in order. Every callback result must contain **exactly one Unicode scalar**. An empty or multi-scalar result raises a runtime error naming String `Map`; the result is not returned. The scalar length of the result equals that of `S`. Use `Filter` to remove scalars; `Map` does not expand one scalar into several.

```pascal
function ReplaceStar(C: string): string;
begin
  if C = '*' then
    return '★';
  end if;

  return C;
end function;

const ResultText: string := 'a*b'.Map(ReplaceStar); // 'a★b'
```

## `Text.Filter(F: function(C: string): boolean): string`

Calls `F` once per scalar and retains those for which it returns `true`, preserving their order.

```pascal
function NotSpace(C: string): boolean;
begin
  return C <> ' ';
end function;

const Compact: string := 'a b c'.Filter(NotSpace);  // 'abc'
```

## `Text.Reduce(Init: U; F: function(Acc: U; C: string): U): U`

Starts with `Init` and passes the current accumulator and scalar to `F` in left-to-right order. The callback must return the accumulator type, which is inferred from `Init`.

```pascal
function CountNonSpaces(Acc: integer; C: string): integer;
begin
  if C = ' ' then
    return Acc;
  end if;

  return Acc + 1;
end function;

const Count: integer := 'a b c'.Reduce(0, CountNonSpaces); // 3
```

For empty `S`, `Map` and `Filter` return `''` and `Reduce` returns `Init`; no callback runs. Arguments are evaluated once in the usual left-to-right order. If a callback fails, the operation stops at that scalar and propagates the error without returning a partial result. Side effects from callbacks already run remain visible.

## See also

- [`String operations` reference](README.md)
- [Array higher-order operations](../array/higher-order.md)
- [Text and parsing index](../../../std/text/README.md)
