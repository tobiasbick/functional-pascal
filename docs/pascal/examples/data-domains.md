# Data-domain examples

These complete programs are compiled independently and executed by the
[acceptance tests](../../implementation/comparisons-and-acceptance.md).
See [data domains](../language/data-domains.md)
for the specified identity, conversion, and default-value rules.

## Distinct identity and explicit unwrapping

```pascal
program Identities;

uses System.Console as Console;

type UserId = distinct long;
type PlayerName = distinct string;

begin
  const Id: UserId := new UserId(42L);
  const Name: PlayerName := new PlayerName('Mara');
  const Raw: long := cast<long>(Id);
  Console.WriteLine(Raw); // 42
  Console.WriteLine(cast<string>(Name)); // Mara

  const Missing: UserId? := null;
  Console.WriteLine(Missing.HasValue); // false
end;
end program;
```

Passing Id to a long parameter requires explicit unwrapping. A separately declared
distinct long type is not assignment-compatible with UserId. Nullable UserId is a
CLR Nullable<UserId>, not an Option supplied by the language.

## Checked bounds and valid default array elements

```pascal
program Ranges;

uses System as Sys, System.Console as Console;

type Percent = int range 0..100;
type PortNumber = ushort range 1..65535;

begin
  const Level: Percent := new Percent(50);
  const Increased: int := Level + 10;
  const Next: Percent := new Percent(Increased);
  Console.WriteLine(cast<int>(Next)); // 60

  const Ports: PortNumber[] := new PortNumber[2];
  Console.WriteLine(Ports[0].Value); // 1: the valid lower bound, not zero

  var Input: int := int.Parse('101');
  try
    const Invalid: Percent := new Percent(Input);
    Console.WriteLine(Invalid.Value);
  catch Failure: Sys.ArgumentOutOfRangeException do
    Console.WriteLine(Failure.Message);
  end try;
end;
end program;
```

The Parse call supplies a runtime value. A literal `new Percent(101)` is instead
a compile-time bound violation. Neither unchecked arithmetic nor byref access to
the underlying primitive can bypass range validation.
