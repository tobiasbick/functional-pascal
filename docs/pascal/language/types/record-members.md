# Record defaults, private state, static factories, and globals

This complete program is compiled and executed by the
[acceptance tests](../../../implementation/comparisons-and-acceptance.md). Records remain
immutable; private state is constructed inside the record and retained by updates.

```pascal
program RecordMembers;

uses System.Console as Console;

const DefaultLabel: string := 'Explorer';
var DefaultsEvaluated: int := 0;
var TotalCreated: int := 0;

function NextLabel(): string;
begin
  DefaultsEvaluated := DefaultsEvaluated + 1;
  return DefaultLabel;
end function;

type Counter = record
  private Value: int := 0;
  public Label: string := NextLabel();

  public static function Create(Initial: int := 0): Counter;
  requires Initial >= 0;
  ensures (Returned: Counter): Returned.Value >= 0;
  begin
    TotalCreated := TotalCreated + 1;
    return new Counter(Value := Initial);
  end function;

  public static procedure ResetStatistics();
  begin
    DefaultsEvaluated := 0;
    TotalCreated := 0;
  end procedure;

  private function NextValue(): int;
  begin
    return self.Value + 1;
  end function;

  public function Current(): int;
  begin
    return self.Value;
  end function;

  public function Increment(): Counter;
  begin
    return self with Value := self.NextValue(); end with;
  end function;
end record;

begin
  Counter.ResetStatistics();
  const First: Counter := Counter.Create(2);
  const Renamed: Counter := First with Label := 'Mara'; end with;
  const Next: Counter := Renamed.Increment();

  Console.WriteLine(First.Current()); // 2
  Console.WriteLine(Next.Current());  // 3
  Console.WriteLine(Next.Label);     // Mara
  Console.WriteLine(DefaultsEvaluated); // 1: with does not rerun defaults
  Console.WriteLine(TotalCreated);     // 1: the factory was called once
end;
end program;
```

Private Value makes the generated constructor private, even though it has a
default. Code outside Counter uses the public factory and cannot directly read
or update Value or call NextValue. The factory's postcondition can read Value
because it is lexically inside Counter; it does not read mutable global counters.

Label's default runs in its declaration scope only when omitted during construction.
The two with updates preserve all unmentioned fields. Static methods have no self;
instance methods receive it implicitly. Global counters are ordinary mutable CLR
static fields, with no automatic synchronization.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
