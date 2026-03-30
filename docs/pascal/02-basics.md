# 2. Basics

## Primitive Types

| Type      | Description                  | Example              |
|-----------|------------------------------|----------------------|
| `integer` | 64-bit signed integer        | `42`, `-7`, `0`      |
| `real`    | 64-bit floating point        | `3.14`, `-0.5`       |
| `boolean` | Boolean                      | `true`, `false`      |
| `char`    | Single ASCII character       | `'A'`               |
| `string`  | Immutable text sequence      | `'Hello'`            |

`char` must be explicitly declared — a single-character string literal like `'A'` is `string` unless the variable is typed as `char`:

```pascal
var
  C: char := 'A';       { char }
  S: string := 'A';     { string }
```

Strings use single quotes with doubled apostrophes for escaping: `'It''s Pascal'`.

Strings may span multiple lines:

```pascal
var
  Poem: string := 'Roses are red
Violets are blue';
```

### Character Codes

Like FreePascal, the `#` prefix denotes a character by its ASCII code. These can be concatenated directly with string literals:

```pascal
var
  LineBreak: string := #13#10;                  { CR+LF }
  Greeting: string := 'Hello'#13#10'World';     { Hello\r\nWorld }
  Tab: char := #9;                               { tab character }
  Letter: char := #65;                           { 'A' }
```

## Variables

Variables are **immutable by default**. Use `mutable var` to allow reassignment. This works both as a declaration block and as an inline statement inside a `begin..end` block.

```pascal
var
  Name: string := 'Alice';       { immutable — cannot be reassigned }

mutable var
  Age: integer := 30;            { mutable — can be reassigned }
```

Reassigning an immutable variable is a compile-time error:

```pascal
var
  X: integer := 10;

begin
  X := 20;  { ERROR: cannot assign to immutable variable 'X' }
end.
```

Mutable variables can be reassigned freely:

```pascal
mutable var
  Count: integer := 0;

begin
  Count := Count + 1;  { OK }
end.
```

Inline mutable variables use the same syntax:

```pascal
begin
  mutable var Count: integer := 0;
  Count := Count + 1
end.
```

## Reference Assignment

`ref T` stores a shared reference to a heap-allocated record. Assigning a `ref` value copies the reference, so both variables observe the same underlying record.

```pascal
type
  Counter = record
    Value: integer;
  end;

mutable var
  Root: ref Counter := new Counter with
    Value := 1;
  end;

var
  Alias: ref Counter := Root;

begin
  Root.Value := 2;
  WriteLn(IntToStr(Alias.Value));  { 2 }
end.
```

Field and index access dereference a `ref` automatically. No explicit dereference operator is required.

The mutability check applies to the variable you write through. Writing through an immutable `ref` variable is rejected even though the target is heap-allocated:

```pascal
type
  Counter = record
    Value: integer;
  end;

var
  Root: ref Counter := new Counter with
    Value := 1;
  end;

begin
  Root.Value := 2;  { ERROR: Root is immutable }
end.
```

## Constants

Constants are declared with `const` and `:=`. Must have a value known at compile time:

```pascal
const
  Pi: real := 3.14159265;
  MaxSize: integer := 1024;
  Greeting: string := 'Hello';
```

`new` performs runtime allocation, so it is not valid in a `const` initializer.

## Number Literals

Integers support decimal and hexadecimal notation. Underscores are allowed as visual separators:

```pascal
var
  A: integer := 1_000_000;     { one million }
  B: integer := $FF;            { 255 hex }
  C: integer := $FF_FF;         { 65535 hex }
```

Real literals require digits on both sides of the decimal point. Scientific notation is supported:

```pascal
var
  X: real := 3.14;
  Y: real := 1.5e10;           { scientific notation }
  Z: real := 3.0E-4;           { 0.0003 }
  W: real := 0.5;              { OK — not .5 }
```

`.5` and `5.` are **not** valid — always write `0.5` or `5.0`.

Negative numbers are parsed as unary minus + literal: `-42` is `-(42)`.

## Type Aliases

Use `type` to define a new name for an existing type:

```pascal
type
  Name = string;
  Age = integer;
```

## Operators

### Arithmetic

| Operator | Description      | Example     |
|----------|------------------|-------------|
| `+`      | Addition         | `A + B`     |
| `-`      | Subtraction      | `A - B`     |
| `*`      | Multiplication   | `A * B`     |
| `/`      | Real division    | `A / B`     |
| `div`    | Integer division | `A div B`   |
| `mod`    | Modulo           | `A mod B`   |

### Comparison

| Operator | Description       | Example    |
|----------|-------------------|------------|
| `=`      | Equal             | `A = B`    |
| `<>`     | Not equal         | `A <> B`   |
| `<`      | Less than         | `A < B`    |
| `>`      | Greater than      | `A > B`    |
| `<=`     | Less or equal     | `A <= B`   |
| `>=`     | Greater or equal  | `A >= B`   |

### Logical / Bitwise

| Operator | Description                          | Example         |
|----------|--------------------------------------|------------------|
| `and`    | Logical AND / bitwise AND on integer | `A and B`       |
| `or`     | Logical OR / bitwise OR on integer   | `A or B`        |
| `not`    | Logical NOT / bitwise NOT on integer | `not A`         |
| `xor`    | Logical XOR / bitwise XOR on integer | `A xor B`       |
| `shl`    | Shift left (integer)                 | `A shl 2`       |
| `shr`    | Shift right (integer)                | `A shr 1`       |

### String Indexing

Individual characters can be read by 0-based integer index using bracket notation. The result type is `char`.

```pascal
var
  S: string := 'Hello';
  C: char := S[0];   { 'H' }
  L: char := S[4];   { 'o' }
```

Accessing an out-of-bounds index is a **runtime error**. The index must be an `integer`; non-integer indices are a compile-time error.

```pascal
{ iterate over characters }
mutable var I: integer := 0;
while I < Std.Str.Length(S) do begin
  WriteLn(S[I]);
  I := I + 1
end
```

### String Concatenation

```pascal
var
  Full: string := 'Hello' + ' ' + 'World';  { 'Hello World' }
```

## Comments

Three comment styles are supported. Comments do **not** nest.

```pascal
{ Brace comment — single or multi-line }

(* Parenthesis-star comment — single or multi-line *)

// Line comment — to end of line
```

`{ outer { inner } ← closes here` — the first `}` ends the comment.

## Local Variables

Variables can be declared inline inside `begin..end` blocks:

```pascal
function FullName(First: string; Last: string): string;
begin
  var Space: string := ' ';
  return First + Space + Last;
end;
```

## Arrays

Arrays are declared with the `array of` syntax:

```pascal
var
  Numbers: array of integer := [1, 2, 3, 4, 5];
  Names: array of string := ['Alice', 'Bob', 'Charlie'];
```

Accessing elements uses bracket notation (0-based index):

```pascal
var
  First: integer := Numbers[0];   { 1 }
  Second: string := Names[1];     { 'Bob' }
```


