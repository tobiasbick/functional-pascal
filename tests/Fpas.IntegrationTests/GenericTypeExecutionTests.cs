namespace Fpas.IntegrationTests;

public sealed class GenericTypeExecutionTests
{
    [Fact]
    public Task Generic_records_keep_typed_fields_defaults_updates_equality_and_methods() => ProgramExecution.Output("""
        program Records;
        uses System.Console as Console;
        type Box<T> = record
          Value: T;
          Label: string := 'box';
          function Get(): T; begin return self.Value; end function;
          function Map<U>(Transform: function(T): U): Box<U>;
          begin return new Box<U>(Transform(self.Value)); end function;
          static function Make(Value: T): Box<T>; begin return new Box<T>(Value); end function;
        end record;
        function Wrap<T>(Value: T): Box<T>; begin return Box<T>.Make(Value); end function;
        begin
          const Value: Box<int> := Wrap<int>(42);
          Console.WriteLine(Value.Get()); Console.WriteLine(Value.Label);
          Console.WriteLine(Value = new Box<int>(42));
          const Copy: Box<int> := Value with Value := 7; end with;
          Console.WriteLine(Copy.Value); Console.WriteLine(Value.Value);
          const Text: Box<string> := Value.Map<string>(function(N: int): string => N.ToString());
          Console.WriteLine(Text.Get()); Console.WriteLine(Text.Label);
          const Empty: Box<string?> := new Box<string?>(null);
          Console.WriteLine(Empty.Value = null);
        end;
        end program;
        """, "42", "box", "True", "7", "42", "42", "box", "True");

    [Fact]
    public Task Generic_record_closures_distinguish_type_and_method_parameters() => ProgramExecution.Output("""
        program Closures;
        uses System.Console as Console;
        type Box<T> = record
          Value: T;
          function Reader(): function(): T; begin return function(): T => self.Value; end function;
          function Join<U>(Other: U; Combine: function(T, U): string): function(): function(): string;
          begin return function(): function(): string => function(): string => Combine(self.Value, Other); end function;
        end record;
        begin
          const Value: Box<int> := new Box<int>(42);
          Console.WriteLine(Value.Reader()());
          Console.WriteLine(Value.Join<string>('!', function(N: int; S: string): string => N.ToString() + S)()());
          const Get: function(): int := Value.Reader(); Console.WriteLine(Get());
        end;
        end program;
        """, "42", "42!", "42");

    [Fact]
    public Task Recursive_generic_unions_preserve_payload_types_exhaustive_matching_and_equality() => ProgramExecution.Output("""
        program Trees;
        uses System.Console as Console;
        type Tree<T> = union Empty; Leaf(Value: T); Branch(Left: Tree<T>; Right: Tree<T>); end union;
        function Count<T>(Value: Tree<T>): int;
        begin
          return case Value of
            when Tree.Empty: 0;
            when Tree.Leaf(const Item: T): 1;
            when Tree.Branch(const Left: Tree<T>, const Right: Tree<T>): Count<T>(Left) + Count<T>(Right);
          end case;
        end function;
        begin
          const A: Tree<int> := Tree<int>.Leaf(42);
          const B: Tree<int> := Tree<int>.Branch(A, Tree<int>.Empty);
          Console.WriteLine(Count<int>(B)); Console.WriteLine(A = Tree<int>.Leaf(42));
          const Text: Tree<string?> := Tree<string?>.Leaf(null);
          Console.WriteLine(case Text of when Tree.Empty: 'empty'; when Tree.Leaf(const Item: string?): Item ?? 'null'; when Tree.Branch(_, _): 'branch'; end case);
        end;
        end program;
        """, "1", "True", "null");
}
