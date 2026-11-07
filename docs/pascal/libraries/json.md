# JSON serialization

Use `System.Text.Json.JsonSerializer` directly. Supply the source type explicitly
to generic calls such as `Serialize<Measurement>(Sample)`. FPAS records expose public
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

type Measurement = record
  [Serialization.JsonPropertyNameAttribute('sample_name')]
  public Name: string;
  public Value: int;
end record;

type Descending = record implements Generic.IComparer<int>
  public function Compare(Left: int; Right: int): int;
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

  const Sample: Measurement := new Measurement(Name := 'Alpha', Value := 10);
  Console.WriteLine(Json.JsonSerializer.Serialize<Measurement>(Sample));
end;
end program;
```

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
