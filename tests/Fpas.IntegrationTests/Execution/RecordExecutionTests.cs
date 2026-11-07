using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class RecordExecutionTests
{
    [Fact]
    public Task Records_have_value_equality_hashing_immutable_updates_and_instance_closures() => ProgramExecution.Output("""
        program Records;
        uses System.Console as Console, System.Collections.Generic as Collections, System.Double as Real;
        type Point = record
          public X: double;
          public Y: double := 0.0;
          public function Shift(Amount: double): Point;
          begin return self with X := self.X + Amount; end with; end function;
          public function Reader(): function(): double;
          begin return function(): double => self.X; end function;
        end record;
        type Box = record public Values: array of int; end record;
        begin
          const First: Point := new Point(X := 2.0);
          const Equal: Point := new Point(Y := 0.0, X := 2.0);
          const Changed: Point := First.Shift(3.0);
          Console.WriteLine(First = Equal);
          Console.WriteLine(First <> Changed);
          Console.WriteLine(First.Equals(cast<object>(Equal)));
          Console.WriteLine(First.Equals(cast<object?>('other')));
          Console.WriteLine(First.GetHashCode() = Equal.GetHashCode());
          const Seen: Collections.HashSet<Point> := new Collections.HashSet<Point>();
          discard Seen.Add(First); discard Seen.Add(Equal); discard Seen.Add(Changed);
          Console.WriteLine(Seen.Count);
          Console.WriteLine(First.X); Console.WriteLine(Changed.Reader()());
          const Missing: Point? := null;
          Console.WriteLine(Missing = null); Console.WriteLine(Missing <> First);
          const NaN: Point := new Point(Real.NaN);
          Console.WriteLine(NaN = new Point(Real.NaN));
          const Values: array of int := [1, 2];
          Console.WriteLine(new Box(Values) = new Box(Values));
          Console.WriteLine(new Box(Values) = new Box([1, 2]));
        end;
        end program;
        """, "True", "True", "True", "False", "True", "2", "2", "5", "True", "True", "True", "True", "False");

    [Fact]
    public Task Private_fields_factories_and_copy_helpers_preserve_state() => ProgramExecution.Output("""
        program PrivateRecords;
        uses System.Console as Console;
        type Counter = record
          private Value: int := 0;
          public Label: string := 'initial';
          public static function Create(Initial: int): Counter;
          begin return new Counter(Value := Initial); end function;
          private function Read(): int;
          begin return self.Value; end function;
          public function Current(): function(): int;
          begin return function(): int => self.Read(); end function;
          public function Increment(): Counter;
          begin return self with Value := self.Value + 1; end with; end function;
        end record;
        begin
          const First: Counter := Counter.Create(2);
          const Named: Counter := First with Label := 'renamed'; end with;
          Console.WriteLine(First.Current()());
          Console.WriteLine(Named.Increment().Current()());
          Console.WriteLine(Named.Label);
          const Factory: function(int): Counter := Counter.Create;
          Console.WriteLine(Factory(4).Current()());
        end;
        end program;
        """, "2", "3", "renamed", "4");

    [Fact]
    public Task Constructor_and_update_expressions_run_once_in_the_documented_order() => ProgramExecution.Output("""
        program Order;
        uses System.Console as Console;
        function Mark(Value: int): int;
        begin Console.WriteLine(Value); return Value; end function;
        type Pair = record
          public X: int := Mark(1);
          public Y: int := Mark(2);
        end record;
        function OrderValue(): int; begin return Mark(3); end function;
        function Make(): Pair;
        begin Console.WriteLine('receiver'); return new Pair(10, 20); end function;
        begin
          const Mark: int := 99;
          const First: Pair := new Pair(Y := OrderValue());
          const Second: Pair := new Pair();
          const Third: Pair := Make() with Y := OrderValue(); X := First.Y; end with;
          Console.WriteLine(Third.X); Console.WriteLine(Third.Y);
        end;
        end program;
        """,
        "3", "1", "1", "2", "receiver", "3", "3", "3");
}
