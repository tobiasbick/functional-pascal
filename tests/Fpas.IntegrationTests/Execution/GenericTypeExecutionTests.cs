// [Documentation](../../../docs/pascal/language/types/generics.md)
using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class GenericTypeExecutionTests
{
    [Fact]
    public Task Generic_records_keep_typed_fields_defaults_updates_equality_and_methods() => ProgramExecution.Output("""
        program Records;
        uses System.Console as Console;
        type Box of T = record
          public Value: T;
          public Label: string := 'box';
          public function Get(): T; begin return self.Value; end function;
          public function Map of U(Transform: function(T): U): Box of U;
          begin return new Box of U(Transform(self.Value)); end function;
          public static function Make(Value: T): Box of T; begin return new Box of T(Value); end function;
        end record;
        function Wrap of T(Value: T): Box of T; begin return (Box of T).Make(Value); end function;
        begin
          const Value: Box of int := Wrap of int(42);
          Console.WriteLine(Value.Get()); Console.WriteLine(Value.Label);
          Console.WriteLine(Value = new Box of int(42));
          const Copy: Box of int := Value with Value := 7; end with;
          Console.WriteLine(Copy.Value); Console.WriteLine(Value.Value);
          const Text: Box of string := Value.Map of string(function(N: int): string => N.ToString());
          Console.WriteLine(Text.Get()); Console.WriteLine(Text.Label);
          const Empty: Box of string? := new Box of string?(null);
          Console.WriteLine(Empty.Value = null);
        end;
        end program;
        """, "42", "box", "True", "7", "42", "42", "box", "True");

    [Fact]
    public Task Generic_record_closures_distinguish_type_and_method_parameters() => ProgramExecution.Output("""
        program Closures;
        uses System.Console as Console;
        type Box of T = record
          public Value: T;
          public function Reader(): function(): T; begin return function(): T => self.Value; end function;
          public function Join of U(Other: U; Combine: function(T, U): string): function(): function(): string;
          begin return function(): function(): string => function(): string => Combine(self.Value, Other); end function;
        end record;
        begin
          const Value: Box of int := new Box of int(42);
          Console.WriteLine(Value.Reader()());
          Console.WriteLine(Value.Join of string('!', function(N: int; S: string): string => N.ToString() + S)()());
          const Get: function(): int := Value.Reader(); Console.WriteLine(Get());
        end;
        end program;
        """, "42", "42!", "42");

    [Fact]
    public Task Recursive_generic_variants_preserve_payload_types_exhaustive_matching_and_equality() => ProgramExecution.Output("""
        program Trees;
        uses System.Console as Console;
        type Tree of T = variant Empty; Leaf(Value: T); Branch(Left: Tree of T; Right: Tree of T); end variant;
        function Count of T(Value: Tree of T): int;
        begin
          return case Value of
            when Tree.Empty: 0;
            when Tree.Leaf(const Item: T): 1;
            when Tree.Branch(const Left: Tree of T, const Right: Tree of T): Count of T(Left) + Count of T(Right);
          end case;
        end function;
        begin
          const A: Tree of int := (Tree of int).Leaf(42);
          const B: Tree of int := (Tree of int).Branch(A, (Tree of int).Empty);
          Console.WriteLine(Count of int(B)); Console.WriteLine(A = (Tree of int).Leaf(42));
          const Text: Tree of string? := (Tree of string?).Leaf(null);
          Console.WriteLine(case Text of when Tree.Empty: 'empty'; when Tree.Leaf(const Item: string?): Item ?? 'null'; when Tree.Branch(_, _): 'branch'; end case);
        end;
        end program;
        """, "1", "True", "null");
}
