# JSON serialization

Use `System.Text.Json.JsonSerializer` directly. Supply the source type explicitly
to generic calls such as `Serialize<Player>(Value)`. FPAS records expose public
properties that the serializer can inspect; .NET serialization attributes control
their JSON names.

This example implements a CLR comparer for a list and serializes a record with a
renamed property. The collection and JSON operations use the original .NET APIs.

## Example

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

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
