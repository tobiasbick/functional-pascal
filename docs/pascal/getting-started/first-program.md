# First program

## A first taste

```pascal
program GreetingDemo;

uses Std.Console as Console;

function Greet(Name: string): string;
begin
  return ('Hello, ' + Name) + '!';
end function;

begin
  var Message: string := Greet('Pascal');
  Console.WriteLn(Message);
end program;
```

## Program structure

Every Functional Pascal program starts with a `program` declaration, optional `uses` clauses, then declarations and the main block:

```pascal
program MyApp;

uses Std.Console as Console;

// constant declarations

const MaxItems: integer := 100;
// variable declarations

var Counter: integer := 0;

// function declarations
function Add(A: integer; B: integer): integer;
begin
  return A + B;
end function;

// main block
begin
  Console.WriteLn(Add(3, 4));
end program;
```

The first segment `Std` in a unit name is reserved for the standard library. User-defined units use another root segment (for example `MyApp.Utils`); see [Units](../program-structure/units.md) and [Standard library](../std/README.md) for `uses` rules.

## See also

- [Functions](../language/functions/README.md)
- [Basics](../language/basics/README.md)
- [Hello world](hello-world.md)
