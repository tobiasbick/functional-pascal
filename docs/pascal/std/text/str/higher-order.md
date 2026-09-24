# Higher-order string operations

`Std.Str.Map`, `Filter`, and `Reduce` process a string eagerly, one Unicode scalar at a time from left to right. Each callback receives the current scalar as a one-scalar `string`. A scalar is not necessarily a user-perceived grapheme cluster: `e` followed by a combining accent invokes the callback twice. These functions do not enable `for ... in` iteration over strings.

Import `Std.Str` and use qualified names when another imported unit exports `Map`, `Filter`, or `Reduce`.

## `Map(S: string; F: function(C: string): string): string`

Calls `F` once per scalar and returns the mapped scalars in order. Every callback result must contain **exactly one Unicode scalar**. An empty or multi-scalar result raises a runtime error naming `Std.Str.Map`; the result is not returned. The scalar length of the result equals that of `S`. Use `Filter` to remove scalars; `Map` does not expand one scalar into several.

```pascal
function ReplaceStar(C: string): string;
begin
  if C = '*' then return '★';
  return C
end;

var ResultText: string := Std.Str.Map('a*b', ReplaceStar);  // 'a★b'
```

## `Filter(S: string; F: function(C: string): boolean): string`

Calls `F` once per scalar and retains those for which it returns `true`, preserving their order.

```pascal
function NotSpace(C: string): boolean;
begin
  return C <> ' '
end;

var Compact: string := Std.Str.Filter('a b c', NotSpace);  // 'abc'
```

## `Reduce(S: string; Init: U; F: function(Acc: U; C: string): U): U`

Starts with `Init` and passes the current accumulator and scalar to `F` in left-to-right order. The callback must return the accumulator type, which is inferred from `Init`.

```pascal
function CountNonSpaces(Acc: integer; C: string): integer;
begin
  if C = ' ' then return Acc;
  return Acc + 1
end;

var Count: integer := Std.Str.Reduce('a b c', 0, CountNonSpaces);  // 3
```

For empty `S`, `Map` and `Filter` return `''` and `Reduce` returns `Init`; no callback runs. Arguments are evaluated once in the usual left-to-right order. If a callback fails, the operation stops at that scalar and propagates the error without returning a partial result. Side effects from callbacks already run remain visible.

## See also

- [`Std.Str` reference](README.md)
- [Array higher-order operations](../../collections/array/higher-order.md)
- [Text and parsing index](../README.md)
