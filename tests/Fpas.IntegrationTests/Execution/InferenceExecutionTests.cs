using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class InferenceExecutionTests
{
    [Fact]
    public Task Routines_record_methods_and_CLR_methods_infer_from_typed_arguments() => ProgramExecution.Output("""
        program Inference;
        uses System as Sys, System.Console as Console, System.Linq.Enumerable as Seq, System.Collections.Generic as C;
        function Id<T>(Value: T): T; begin return Value; end function;
        function First<T>(Left: T; Right: T): T; begin return Left; end function;
        function Map<T, U>(Items: C.IEnumerable<T>; Transform: function(T): U): array of U;
        begin return Seq.ToArray(Seq.Select(Items, Transform)); end function;
        function Compare<T>(Left: T; Right: T): int; where T: Sys.IComparable<T>;
        begin return Left.CompareTo(Right); end function;
        function Or<T>(Value: T?; Other: T): T; where T: struct; begin return Value ?? Other; end function;
        function Missing<T>(Value: T?): bool; where T: class; begin return Value = null; end function;
        function Head<T>(Values: array of T; Other: T): T; begin return Values[0]; end function;
        function Recursive<T>(Value: T; N: int): T;
        begin if N = 0 then return Value; else return Recursive(Value, N - 1); end if; end function;
        type Box<T> = record
          public Value: T;
          public function Map<U>(Transform: function(T): U): Box<U>; begin return new Box<U>(Transform(self.Value)); end function;
          public function Echo<U>(Value: U): U; begin return Value; end function;
        end record;
        begin
          Console.WriteLine(Id(42)); Console.WriteLine(Id('text'));
          const Number: double := First(1, 2.5); Console.WriteLine(Number = 1.0);
          const Items: array of string := Map([1, 2], function(N: int): string => N.ToString());
          for Item: string in Items do Console.WriteLine(Item); end for;
          Console.WriteLine(Compare(1, 2) < 0); Console.WriteLine(Recursive('recursive', 2));
          const Boxed: Box<int> := new Box<int>(7);
          Console.WriteLine(Boxed.Map(function(N: int): string => N.ToString()).Value);
          Console.WriteLine(Boxed.Echo('method'));
          Console.WriteLine(Seq.Count(Seq.Where([1, 2, 3], function(N: int): bool => N > 1)));
          const Empty: array of int := Map([], function(N: int): int => N * 2); Console.WriteLine(Empty.Length);
          const Wide: array of double := Map([1, 2], function(N: double): double => N * 2.0); Console.WriteLine(Wide[1] = 4.0);
          Console.WriteLine(Head([1], 2.0) = 1.0);
          const Maybe: int? := 9; Console.WriteLine(Id(Maybe).HasValue);
          const Text: string? := First('text', null); Console.WriteLine(Text ?? 'missing');
          Console.WriteLine(Or(Maybe, 1)); Console.WriteLine(Or(null, 1));
          const Absent: string? := null; Console.WriteLine(Missing(Absent));
        end;
        end program;
        """, "42", "text", "True", "1", "2", "True", "recursive", "7", "method", "2", "0", "True", "True", "True", "text", "9", "1", "True");

    [Fact]
    public Task Variant_payloads_expected_types_and_sibling_payloads_determine_type_arguments() => ProgramExecution.Output("""
        program Inference;
        uses System.Console as Console;
        type Tree<T> = variant Empty; Leaf(Value: T); Branch(Left: Tree<T>; Right: Tree<T>); end variant;
        type Choice<A, B> = variant First(Value: A); Second(Value: B); end variant;
        function Count<T>(Value: Tree<T>): int;
        begin return case Value of when Tree.Empty: 0; when Tree.Leaf(_): 1; when Tree.Branch(const L: Tree<T>, const R: Tree<T>): Count(L) + Count(R); end case; end function;
        function Wrap<T>(Value: T): Tree<T>; begin return Tree.Leaf(Value); end function;
        function FirstTree<T>(Values: array of Tree<T>): Tree<T>; begin return Values[0]; end function;
        begin
          const Empty: Tree<int> := Tree.Empty;
          const Leaf: Tree<int> := Tree.Leaf(42);
          const Branch: Tree<int> := Tree.Branch(Tree.Empty, Tree.Leaf(7));
          Console.WriteLine(Count(Empty)); Console.WriteLine(Count(Leaf)); Console.WriteLine(Count(Branch));
          Console.WriteLine(Count(Tree.Branch(Tree.Empty, Tree.Leaf(9)))); Console.WriteLine(Count(Wrap('text')));
          const Wide: Tree<double> := Tree.Leaf(1);
          Console.WriteLine(case Wide of when Tree.Leaf(const N: double): N = 1.0; when Tree.Empty, Tree.Branch(_, _): false; end case);
          const Null: Tree<string?> := Tree.Leaf(null);
          Console.WriteLine(case Null of when Tree.Leaf(const S: string?): S = null; when Tree.Empty, Tree.Branch(_, _): false; end case);
          const Phantom: Choice<int, string> := Choice.First(3);
          Console.WriteLine(case Phantom of when Choice.First(const N: int): N; when Choice.Second(_): 0; end case);
          const Many: array of Tree<int> := [Tree.Empty, Tree.Leaf(1)]; Console.WriteLine(Count(Many[1]));
          Console.WriteLine(Count(FirstTree([Tree.Empty, Tree.Leaf(2)])));
        end;
        end program;
        """, "0", "1", "1", "1", "1", "True", "True", "3", "1", "0");

    [Fact]
    public Task Inferred_named_payloads_evaluate_once_in_written_order() => ProgramExecution.Output("""
        program Evaluation;
        uses System.Console as Console;
        type Pair<T> = variant Both(Left: T; Right: T); end variant;
        function Trace<T>(Label: string; Value: T): T;
        begin Console.WriteLine(Label); return Value; end function;
        begin
          const Value: Pair<int> := Pair.Both(Right := Trace('right', 2), Left := Trace('left', 1));
          Console.WriteLine(case Value of when Pair.Both(const L: int, const R: int): L * 10 + R; end case);
        end;
        end program;
        """, "right", "left", "12");
}
