# 1. Overview

Functional Pascal is a modern, function-first programming language built on Pascal's readable syntax. It runs on a managed virtual machine — no pointers, no manual memory management, no unsafe operations.

## Hello World

```pascal
program Hello;

uses
  Std.Console;

begin
  WriteLn('Hello, World!');
end.
```

## Design Philosophy

- **Function first** — Functions are the primary building block. No classical classes.
- **Immutable by default** — All bindings are immutable unless declared `mutable`.
- **Explicit types** — Every variable and parameter declares its type.
- **Safe by design** — The VM manages memory. No pointers, no manual allocation.
- **Familiar syntax** — Pascal's `begin`, `end`, `:=`, `downto` and other well-known keywords.
- **Case-insensitive** — Keywords and identifiers are case-insensitive, following Pascal tradition.

## A First Taste

```pascal
program Greet;

uses
  Std.Console,
  Std.Str;

function Greet(Name: string): string;
begin
  return Concat('Hello, ', Name, '!');
end;

begin
  var Message: string := Greet('Pascal');
  WriteLn(Message);
end.
```

## Program Structure

Every Functional Pascal program starts with a `program` declaration, optional `uses` clauses, then declarations and the main block:

```pascal
program MyApp;

uses
  Std.Console;

{ constant declarations }
const
  MaxItems: integer := 100;

{ variable declarations }
var
  Counter: integer := 0;

{ function declarations }
function Add(A: integer; B: integer): integer;
begin
  return A + B;
end;

{ main block }
begin
  WriteLn(Add(3, 4));
end.
```

The first segment `Std` in a unit name is reserved for the implementation-defined standard library. When user-defined units exist, their names must not start with `Std.`; see the standard library section for valid `Std.*` units and `uses` rules.

## Keywords

All keywords are case-insensitive, following traditional Pascal convention:

```
program   uses      const     var       mutable
function  procedure begin     end       return
if        then      else      case      of
for       to        downto    in        do        while
repeat    until     and       or        not
xor       div       mod       shl       shr
true      false     type      record    enum
array     forward   panic     break     continue
result    option    ok        error     some
none      try
```
