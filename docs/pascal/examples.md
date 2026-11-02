# Syntax examples

**These are proposed programs, not compiler-verified samples.** The
[implemented compiler subset](../implementation/first-program.md) does not yet
cover most of these programs. The [Mandelbrot program](../../examples/mandelbrot/README.md)
is executable and covered by integration tests. Each Pascal code fence below is a separate complete
program. Imports and helper declarations are included.
Programs are independent and must not be concatenated into one compilation.

The [topic documentation](README.md) defines their semantics. A two-file unit
example is in
[program structure](program-structure/README.md).
For a complete documented unit and its planned XML output, see
[source comments and API documentation](language/comments.md#complete-source-example).

## 1. Hello world and primitive types

```pascal
program Hello;

uses System.Console as Console;

begin
  const Name: string := 'Mara';
  const Health: int := 20;
  const WorldSeed: long := 4_000_000_000L;
  const Zoom: double := 1.5;
  const Price: decimal := 12.50M;
  const Initial: char := #77;
  const Ready: bool := true;

  Console.WriteLine('Hello, ' + Name);
  Console.WriteLine(Health);
  Console.WriteLine(WorldSeed);
  Console.WriteLine(Zoom);
  Console.WriteLine(Price);
  Console.WriteLine(Initial);
  Console.WriteLine(Ready);
end;
end program;
```

There are no predefined integer/real aliases. Library properties have their
actual names: `'Hello'.Length`, not `Length()`.

## 2. Free functions, generics, and captured state

```pascal
program Functions;

uses System.Console as Console;

function Identity<T>(Value: T): T;
begin
  return Value;
end function;

function Apply(Value: int; Transform: function(int): int): int;
begin
  return Transform(Value);
end function;

function MakeCounter(): function(): int;
begin
  var Count: int := 0;
  return function(): int begin
    Count := Count + 1;
    return Count;
  end function;
end function;

begin
  const Double: function(int): int := function(Value: int): int => Value * 2;
  const Counter: function(): int := MakeCounter();
  Console.WriteLine(Identity<string>('Functions first'));
  Console.WriteLine(Apply(21, Double)); // 42
  Console.WriteLine(Counter());        // 1
  Console.WriteLine(Counter());        // 2
end;
end program;
```

## 3. Immutable records, updates, and union matching

```pascal
program Shapes;

uses System.Console as Console, System.Math as Math;

type Point = record
  X: double;
  Y: double;
end record;

type Shape = union
  Dot;
  Circle(Radius: double);
  Rectangle(Width: double; Height: double);
end union;

function Area(Value: Shape): double;
begin
  return case Value of
    when Shape.Dot: 0.0;
    when Shape.Circle(const Radius: double): Math.PI * Radius * Radius;
    when Shape.Rectangle(const Width: double, const Height: double): Width * Height;
  end case;
end function;

begin
  const Start: Point := new Point(X := 1.0, Y := 2.0);
  const Moved: Point := Start with X := Start.X + 3.0; end with;
  Console.WriteLine(Start.X); // 1
  Console.WriteLine(Moved.X); // 4
  Console.WriteLine(Area(Shape.Rectangle(3.0, 4.0))); // 12
end;
end program;
```

## 4. Real CLR collections, aliasing, and out parameters

```pascal
program Collections;

uses System.Console as Console, System.Collections.Generic as Generic;

begin
  const Values: int[] := [1, 2, 3];
  const Shared: int[] := Values;
  Shared[0] := 9;
  Console.WriteLine(Values[0]); // 9: both bindings reference the same array

  const Names: Generic.List<string> := new Generic.List<string>();
  Names.Add('Mara');
  Names.Add('Keeper');
  for Name: string in Names do
    Console.WriteLine(Name);
  end for;

  const Scores: Generic.Dictionary<string, int> := new Generic.Dictionary<string, int>();
  Scores['Mara'] := 10;
  var Score: int := 0;
  if Scores.TryGetValue('Mara', out Score) then
    Console.WriteLine(Score);
  end if;

  var Parsed: int := 0;
  if int.TryParse('42', out Parsed) then
    Console.WriteLine(Parsed);
  end if;
end;
end program;
```

## 5. Nullable references and UTF-16 strings

```pascal
program NullableText;

uses System.Environment as Environment, System.Console as Console;

begin
  const OptionalName: string? := Environment.GetEnvironmentVariable('FPAS_PLAYER');
  const Name: string := OptionalName ?? 'Explorer';
  Console.WriteLine(Name);

  if OptionalName <> null then
    Console.WriteLine(OptionalName.Length);
  end if;

  const Text: string := 'A😀';
  Console.WriteLine(Text.Length); // 3 UTF-16 code units
  const First: char := Text[0];
  Console.WriteLine(First);       // A
end;
end program;
```

## 6. Gameplay outcomes as an application union

```pascal
program MoveOutcomes;

uses System.Console as Console;

type Position = record
  X: int;
  Y: int;
end record;

type MoveOutcome = union
  Moved(NewPosition: Position);
  Blocked(Reason: string);
end union;

function Move(Current: Position; DX: int; DY: int): MoveOutcome;
begin
  const Next: Position := Current with
    X := Current.X + DX;
    Y := Current.Y + DY;
  end with;
  if Next.X < 0 or Next.X >= 10 or Next.Y < 0 or Next.Y >= 10 then
    return MoveOutcome.Blocked('The edge of the map');
  end if;
  return MoveOutcome.Moved(Next);
end function;

begin
  const Start: Position := new Position(X := 0, Y := 0);
  case Move(Start, -1, 0) of
    when MoveOutcome.Moved(const Next: Position):
      Console.WriteLine(Next.X);
    when MoveOutcome.Blocked(const Reason: string):
      Console.WriteLine(Reason); // The edge of the map
  end case;
end;
end program;
```

Both cases are normal gameplay outcomes. The union is declared by this application
and handled explicitly. It does not throw, unwind, or return early on its own.
Arithmetic overflow still throws an ordinary CLR OverflowException.

## 7. Exceptions and deterministic disposal

```pascal
program ReadWorld;

uses System as Sys, System.IO as IO, System.Console as Console;

begin
  try
    using const Reader: IO.StreamReader := new IO.StreamReader('world.json');
    Console.WriteLine(Reader.ReadToEnd());
  catch Failure: IO.IOException do
    Console.WriteLine(Failure.Message);
  catch Failure: Sys.UnauthorizedAccessException do
    Console.WriteLine(Failure.Message);
  finally
    Console.WriteLine('Read attempt finished');
  end try;
end;
end program;
```

The file must exist for the success path. The acquired reader is disposed even
when reading fails. Failure to construct it does not dispose an unacquired reader.
The declaration belongs to the try-body scope, so disposal occurs before any
catch/finally body executes.

## 8. Async HTTP with cancellation

```pascal
async program DownloadPage;

uses System as Sys,
  System.Console as Console,
  System.Net.Http as Http,
  System.Threading as Threading,
  System.Threading.Tasks as Tasks;

async function Download(Address: string;
  Cancellation: Threading.CancellationToken): Tasks.Task<string>;
begin
  using const Client: Http.HttpClient := new Http.HttpClient();
  return await Client.GetStringAsync(Address, Cancellation);
end function;

begin
  using const Cancellation: Threading.CancellationTokenSource :=
    new Threading.CancellationTokenSource(Sys.TimeSpan.FromSeconds(5.0));
  try
    const Text: string := await Download('https://example.com/', Cancellation.Token);
    Console.WriteLine(Text.Length);
  catch Failure: Sys.OperationCanceledException do
    Console.WriteLine(Failure.Message);
  catch Failure: Http.HttpRequestException do
    Console.WriteLine(Failure.Message);
  end try;
end;
end program;
```

This program needs network access when eventually executed. Starting Download
does not imply a new thread. Its Task and CancellationToken are real .NET types.
Client is disposed before Download completes; Cancellation remains alive through
the try/catch and is disposed when the program entry body exits.

## 9. Attributes, JSON, and interface implementation

```pascal
program Interop;

uses System.Console as Console,
  System.Collections.Generic as Generic,
  System.Text.Json as Json,
  System.Text.Json.Serialization as Serialization;

type Player = record
  [Serialization.JsonPropertyNameAttribute('player_name')]
  Name: string;
  Score: int;
end record;

type Descending = record implements Generic.IComparer<int>
  function Compare(Left: int; Right: int): int;
  begin
    return Right.CompareTo(Left);
  end function;
end record;

begin
  const Numbers: Generic.List<int> := new Generic.List<int>();
  Numbers.Add(1);
  Numbers.Add(3);
  Numbers.Add(2);
  Numbers.Sort(new Descending());
  Console.WriteLine(Numbers[0]); // 3

  const Value: Player := new Player(Name := 'Mara', Score := 10);
  Console.WriteLine(Json.JsonSerializer.Serialize<Player>(Value));
end;
end program;
```

## 10. CLR event subscription with a retained delegate

```pascal
program Events;

uses System.Console as Console,
  System.Threading as Threading,
  System.Timers as Timers;

begin
  using const Timer: Timers.Timer := new Timers.Timer(100.0);
  const Handler: Timers.ElapsedEventHandler :=
    procedure(Sender: object?; Event: Timers.ElapsedEventArgs) begin
      Console.WriteLine('Tick');
    end procedure;

  subscribe Timer.Elapsed with Handler;
  Timer.Start();
  Threading.Thread.Sleep(350);
  Timer.Stop();
  unsubscribe Timer.Elapsed with Handler;
end;
end program;
```

Tick timing/count is nondeterministic. Unsubscription does not wait for an event
handler already dispatched by the CLR timer. This example illustrates event
binding, not a synchronization or exact-timing guarantee.

## 11. Mandelbrot calculation with CLR numeric types

```pascal
program Mandelbrot;

uses System.Console as Console;

function EscapeIterations(CReal: double; CImaginary: double; Limit: int): int;
begin
  var RealPart: double := 0.0;
  var ImaginaryPart: double := 0.0;
  var Iteration: int := 0;

  while Iteration < Limit and
    RealPart * RealPart + ImaginaryPart * ImaginaryPart <= 4.0 do
    const NextReal: double := RealPart * RealPart - ImaginaryPart * ImaginaryPart + CReal;
    ImaginaryPart := 2.0 * RealPart * ImaginaryPart + CImaginary;
    RealPart := NextReal;
    Iteration := Iteration + 1;
  end while;

  return Iteration;
end function;

begin
  const Width: int := 80;
  const Height: int := 24;
  const Limit: int := 50;

  for Y: int := 0 to Height - 1 do
    var Row: string := '';
    for X: int := 0 to Width - 1 do
      const CReal: double := -2.0 + 3.0 * cast<double>(X) / cast<double>(Width - 1);
      const CImaginary: double := -1.0 + 2.0 * cast<double>(Y) / cast<double>(Height - 1);
      const Count: int := EscapeIterations(CReal, CImaginary, Limit);
      Row := Row + if Count = Limit then '#' else ' ' end if;
    end for;
    Console.WriteLine(Row);
  end for;
end;
end program;
```

This exact source is the [executable Mandelbrot example](../../examples/mandelbrot/main.fpas).
Its MSBuild integration test verifies every pixel against a .NET Complex calculation.
It is a small console rendering to show numeric and loop syntax. It makes no
claim about interactive graphics, deep-zoom precision, or performance.

## 12. Generic unions, nullable values, enum fallback, and a CLR constraint

```pascal
program TypeRules;

uses System as Sys, System.Console as Console;

type Direction = enum : byte
  North = 0;
  South = 1;
end enum;

type Tree<T> = union
  Empty;
  Leaf(Value: T);
  Branch(Left: Tree<T>; Right: Tree<T>);
end union;

function Compare<T>(Left: T; Right: T): int;
where T: Sys.IComparable<T>;
begin
  return Left.CompareTo(Right);
end function;

begin
  const Port: int? := null;
  Console.WriteLine(Port ?? 8080);

  const TreeValue: Tree<int> := Tree.Branch(Tree.Leaf(1), Tree.Leaf(2));
  case TreeValue of
    when Tree.Empty:
      Console.WriteLine('Empty tree');
    when Tree.Leaf(const Value: int):
      Console.WriteLine(Value);
    when Tree.Branch(_, _):
      Console.WriteLine('Branch');
  end case;

  Console.WriteLine(Compare<int>(2, 3));
  const Facing: Direction := cast<Direction>(255);
  case Facing of
    when Direction.North:
      Console.WriteLine('North');
    when Direction.South:
      Console.WriteLine('South');
    else
      Console.WriteLine('An unnamed CLR enum value');
  end case;
end;
end program;
```

Nullable int, an application-defined union, and a CLR enum are distinct types.
A cast can construct an unnamed enum value, which is why the final case requires
a fallback. Tree is an ordinary generic domain type, not an implicit library type.

## 13. Preconditions and postconditions

```pascal
program Contracts;

uses System as Sys, System.Console as Console;

function AddPositive(Left: int; Right: int): int;
requires Left > 0;
requires Right > 0;
ensures (Returned: int): Returned > Left;
begin
  return Left + Right;
end function;

procedure ClampToZero(ref Value: int);
ensures Value >= 0;
begin
  if Value < 0 then
    Value := 0;
  end if;
end procedure;

begin
  Console.WriteLine(AddPositive(2, 3)); // 5
  var Value: int := -1;
  ClampToZero(ref Value);
  Console.WriteLine(Value); // 0

  try
    discard AddPositive(0, 3);
  catch Failure: Sys.InvalidOperationException do
    Console.WriteLine(Failure.Message); // The first requires condition failed
  end try;
end;
end program;
```

Checks also run in release builds. A typed ensures binding names the successful
return value. See [contracts](language/contracts.md) for cleanup, restricted
expressions, and async behavior.

## More complete examples

- [Distinct scalars and checked subranges](examples/data-domains.md)
- [Record defaults, private fields, static factories, and globals](examples/record-members.md)
