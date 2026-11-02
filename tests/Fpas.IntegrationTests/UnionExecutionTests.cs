namespace Fpas.IntegrationTests;

public sealed class UnionExecutionTests
{
    [Fact]
    public Task Generated_case_types_support_CLR_type_tests_and_payload_access() => ProgramExecution.Output("""
        program CaseTypes;
        uses System.Console as Console;
        type Result = union Success(Value: int); Missing; end union;
        begin
          const Value: Result := Result.Success(42);
          Console.WriteLine(Value is Result.SuccessCase);
          Console.WriteLine(case cast<object>(Value) of when is Result.SuccessCase as Found: Found.Value; else 0; end case);
        end;
        end program;
        """, "True", "42");

    [Fact]
    public Task Union_factories_equality_hashing_and_payload_patterns_execute() => ProgramExecution.Output("""
        program Actions;
        uses System.Console as Console, System.Collections.Generic as Collections;
        type Outcome = union
          Idle;
          Moved(X: int; Y: int);
          Blocked(Reason: string);
        end union;
        function Describe(Value: Outcome): string;
        begin
          return case Value of
            when Outcome.Idle: 'idle';
            when Outcome.Moved(Y := const Y: int, X := const X: int): X.ToString() + ',' + Y.ToString();
            when Outcome.Blocked(const Reason: string): Reason;
          end case;
        end function;
        begin
          const First: Outcome := Outcome.Moved(Y := 2, X := 1);
          const Equal: Outcome := Outcome.Moved(1, 2);
          Console.WriteLine(Describe(First)); Console.WriteLine(Describe(Outcome.Idle));
          Console.WriteLine(Describe(Outcome.Blocked('wall')));
          Console.WriteLine(First = Equal); Console.WriteLine(First <> Outcome.Blocked('wall'));
          Console.WriteLine(Outcome.Idle = Outcome.Idle);
          Console.WriteLine(First.Equals(cast<object>(Equal)));
          Console.WriteLine(First.GetHashCode() = Equal.GetHashCode());
          const Seen: Collections.HashSet<Outcome> := new Collections.HashSet<Outcome>();
          discard Seen.Add(First); discard Seen.Add(Equal); discard Seen.Add(Outcome.Idle);
          Console.WriteLine(Seen.Count); Console.WriteLine(First.Tag);
          const Missing: Outcome? := null;
          Console.WriteLine(Missing = null); Console.WriteLine(Missing <> First);
        end;
        end program;
        """, "1,2", "idle", "wall", "True", "True", "True", "True", "True", "2", "1", "True", "True");

    [Fact]
    public Task Nested_patterns_guards_alternatives_and_arm_closures_execute_in_order() => ProgramExecution.Output("""
        program Patterns;
        uses System.Console as Console;
        type Tree = union Empty; Leaf(Value: int); Branch(Left: Tree; Right: Tree); end union;
        function Read(Value: Tree): function(): int;
        begin
          return case Value of
            when Tree.Leaf(const N: int) if N > 0: function(): int => N;
            when Tree.Branch(Tree.Leaf(const N: int), Tree.Empty), Tree.Branch(Tree.Empty, Tree.Leaf(const N: int)):
              function(): int => N * 2;
            else function(): int => -1;
          end case;
        end function;
        function Select(): Tree;
        begin Console.WriteLine('selector'); return Tree.Branch(Tree.Empty, Tree.Leaf(3)); end function;
        begin
          Console.WriteLine(Read(Tree.Leaf(2))()); Console.WriteLine(Read(Tree.Leaf(-1))());
          case Select() of
            when Tree.Branch(const Left: Tree, const Right: Tree): Console.WriteLine(Read(Right)());
            else Console.WriteLine('other');
          end case;
          Console.WriteLine(Read(Tree.Branch(Tree.Leaf(3), Tree.Empty))());
        end;
        end program;
        """, "2", "-1", "selector", "3", "6");

    [Fact]
    public Task Primitive_ranges_nulls_CLR_type_patterns_and_single_evaluation_execute() => ProgramExecution.Output("""
        program PrimitivePatterns;
        uses System.Console as Console, System as Sys;
        function Classify(Value: object?): string;
        begin
          return case Value of
            when null: 'null';
            when 42: 'answer';
            when is string as Text if Text.Length > 0: Text;
            when is Sys.Exception as Failure: Failure.Message;
            else 'other';
          end case;
        end function;
        begin
          Console.WriteLine(Classify(null)); Console.WriteLine(Classify(42));
          Console.WriteLine(Classify('text')); Console.WriteLine(Classify(new Sys.Exception('failure')));
          Console.WriteLine(Classify(''));
          const Number: int? := 3;
          Console.WriteLine(case Number of when null: -1; when 0..5: 1; else 2; end case);
          Console.WriteLine(case true of when true: 'yes'; when false: 'no'; end case);
          Console.WriteLine(case #65 of when #65..#90: 'upper'; else 'other'; end case);
          Console.WriteLine(cast<object>(42) is int);
          Console.WriteLine(cast<object>('text') is Sys.Exception);
        end;
        end program;
        """, "null", "answer", "text", "failure", "other", "1", "yes", "upper", "True", "False");
}
