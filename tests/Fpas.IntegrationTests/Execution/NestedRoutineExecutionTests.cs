using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class NestedRoutineExecutionTests
{
    [Fact]
    public Task A_block_lambda_helper_captures_a_local_in_its_outer_body()
        => ProgramExecution.Output("""
            program LambdaHelper;
            uses System.Console as Console;
            begin
              const Read: function(int): int := function(Input: int): int
                function Answer(): int; begin return Value; end function;
              begin
                const Value: int := Input + 1;
                return Answer();
              end function;
              Console.WriteLine(Read(41));
            end;
            end program;
            """, "42");

    [Fact]
    public Task Generic_nested_methods_and_lambdas_keep_outer_and_method_type_parameters_distinct() => ProgramExecution.Output("""
        program GenericNested;
        uses System.Console as Console;
        function Build<T>(Value: T): function(int): T;
          function Make<U>(Other: U): function(): T;
          where U: struct;
          begin return function(): T begin discard Other; return Value; end function; end function;
          function Run(Index: int): T; begin return Make<int>(Index)(); end function;
        begin
          return Run;
        end function;
        begin const Read: function(int): string := Build<string>('generic'); Console.WriteLine(Read(4)); end;
        end program;
        """, "generic");

    [Fact]
    public Task Async_generic_nested_methods_keep_their_receiver_and_captures_after_suspension() => ProgramExecution.Output("""
        async program AsyncNested;
        uses System.Console as Console, System.Threading.Tasks as Tasks;
        function Create<T>(Value: T): Tasks.Task<T>;
          async function Run<U>(Other: U): Tasks.Task<T>;
          where U: struct;
          begin await Tasks.Task.Delay(1); return Select<U>(Value, Other); end function;
          function Select<U>(Left: T; Right: U): T; begin discard Right; return Left; end function;
        begin
          return Run<int>(1);
        end function;
        begin Console.WriteLine(await Create<string>('async')); end;
        end program;
        """, "async");

    [Fact]
    public Task Iteration_scopes_and_contract_only_captures_keep_their_values() => ProgramExecution.Output("""
        program Scoped;
        uses System.Console as Console, System as Sys, System.Collections.Generic as C;
        begin
          const Actions: C.List<Sys.Func<int>> := new C.List<Sys.Func<int>>();
          for Index: int := 0 to 2 do
            var Count: int := Index;
            function Next(): int; begin Count := Count + 1; return Count; end function;
            Actions.Add(Next);
          end for;
          for Read: Sys.Func<int> in Actions do Console.WriteLine(Read()); end for;
          for Read: Sys.Func<int> in Actions do Console.WriteLine(Read()); end for;
          const Limit: int := 10;
          function Echo(Value: int): int;
          requires Value < Limit;
          ensures (Returned: int): Returned = Value;
          begin return Value; end function;
          Console.WriteLine(Echo(Value := 3));
        end;
        end program;
        """, "1", "2", "3", "2", "3", "4", "3");

    [Fact]
    public Task Forward_declarations_recursion_and_mutual_calls_use_the_lexical_scope() => ProgramExecution.Output("""
        program Nested;
        uses System.Console as Console;
        function Factorial(Value: int): int;
          function Multiply(N: int): int;
          begin if N = 0 then return 1; end if; return N * Multiply(N - 1); end function;
        begin
          return Multiply(Value);
        end function;
        function Parity(Value: int): bool;
          function Even(N: int): bool; begin if N = 0 then return true; end if; return Odd(N - 1); end function;
          function Odd(N: int): bool; begin if N = 0 then return false; end if; return Even(N - 1); end function;
        begin
          return Even(Value);
        end function;
        begin Console.WriteLine(Factorial(5)); Console.WriteLine(Parity(8)); Console.WriteLine(Parity(9)); end;
        end program;
        """, "120", "True", "False");

    [Fact]
    public Task Escaping_nested_delegates_share_cells_and_transitive_captures() => ProgramExecution.Output("""
        program Captures;
        uses System.Console as Console;
        function Counter(Start: int): function(): int;
          function Read(): int; begin return Next(); end function;
          function Next(): int; begin Count := Count + 1; return Count; end function;
        begin
          var Count: int := Start;
          return Read;
        end function;
        function Deep(Base: int): function(int): int;
          function Outer(Value: int): int;
            function Inner(Extra: int): int; begin return Base + Value + Extra; end function;
          begin
            return Inner(2);
          end function;
        begin
          return Outer;
        end function;
        begin
          const Read: function(): int := Counter(10); Console.WriteLine(Read()); Console.WriteLine(Read());
          const Add: function(int): int := Deep(40); Console.WriteLine(Add(3));
        end;
        end program;
        """, "11", "12", "45");

    [Fact]
    public Task Nested_overloads_generics_parameter_modes_and_contracts_keep_normal_call_rules() => ProgramExecution.Output("""
        program Calls;
        uses System.Console as Console;
        function Outer<T>(Value: T): T;
          function Echo<U>(Item: U): U; begin return Item; end function;
          function Echo(Item: int): int; begin return Item + 1; end function;
          procedure Set(out Target: int; in Input: int; Amount: int := 2);
          ensures Target >= Input;
          begin Target := Input + Amount; end procedure;
          procedure Increment(ref Target: int); begin Target := Target + 1; end procedure;
          function Sum(params Values: array of int): int;
          begin var Total: int := 0; for Item: int in Values do Total := Total + Item; end for; return Total; end function;
        begin
          var Answer: int := 0; const Input: int := 40;
          Set(out Answer, in Input); Increment(ref Answer); Console.WriteLine(Answer); Console.WriteLine(Echo(1));
          Console.WriteLine(Sum(1, 2, 3));
          return Echo<T>(Value);
        end function;
        begin Console.WriteLine(Outer<string>('typed')); end;
        end program;
        """, "43", "2", "6", "typed");
}
