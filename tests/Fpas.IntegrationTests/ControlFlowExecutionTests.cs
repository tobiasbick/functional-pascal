namespace Fpas.IntegrationTests;

public sealed class ControlFlowExecutionTests
{
    [Fact]
    public Task Branches_recursion_and_all_return_paths_execute()
        => ProgramExecution.Output("""
            program Branches;
            uses System.Console as Console;
            function Fib(N: int): int;
            begin
              if N < 2 then return N; else return Fib(N - 1) + Fib(N - 2); end if;
            end function;
            function Sign(N: int): string;
            begin
              if N < 0 then return 'negative'; elsif N = 0 then return 'zero'; else return 'positive'; end if;
            end function;
            begin
              Console.WriteLine(Fib(10));
              Console.WriteLine(Sign(-1)); Console.WriteLine(Sign(0)); Console.WriteLine(Sign(1));
              var Value: int := 1;
              begin var Value: int := 9; Value := Value + 1; Console.WriteLine(Value); end;
              Value := Value + 1;
              Console.WriteLine(Value);
            end;
            end program;
            """, "55", "negative", "zero", "positive", "10", "2");

    [Fact]
    public Task Boolean_short_circuit_and_conditional_branches_preserve_side_effects()
        => ProgramExecution.Output("""
            program Effects;
            uses System.Console as Console;
            function Touch(Name: string): bool;
            begin Console.WriteLine(Name); return true; end function;
            begin
              Console.WriteLine(false and Touch('wrong'));
              Console.WriteLine(true or Touch('wrong'));
              Console.WriteLine(true and Touch('and'));
              Console.WriteLine(false or Touch('or'));
              Console.WriteLine(true xor Touch('xor'));
              const Value: bool := if true then Touch('chosen') else Touch('wrong') end if;
              discard Value;
            end;
            end program;
            """, "False", "True", "and", "True", "or", "True", "xor", "False", "chosen");

    [Fact]
    public Task Nested_loops_break_continue_and_while_conditions_execute()
        => ProgramExecution.Output("""
            program Loops;
            uses System.Console as Console;
            function Find(): int;
            begin while true do return 42; end while; end function;
            begin
              var Sum: int := 0;
              for I: int := 1 to 4 do
                if I = 2 then continue; end if;
                for J: int := 1 to 3 do
                  if J = 3 then break; end if;
                  Sum := Sum + I * J;
                end for;
              end for;
              Console.WriteLine(Sum);
              var N: int := 0;
              while N < 5 do
                N := N + 1;
                if N = 2 then continue; end if;
                if N = 4 then break; end if;
                Console.WriteLine(N);
              end while;
              while false do Console.WriteLine('wrong'); end while;
              Console.WriteLine(Find());
            end;
            end program;
            """, "24", "1", "3", "42");

    [Fact]
    public Task Counting_loop_bounds_are_inclusive_evaluated_once_and_never_overflow()
        => ProgramExecution.Output("""
            program Bounds;
            uses System.Console as Console;
            function Bound(Value: int): int;
            begin Console.WriteLine(Value); return Value; end function;
            begin
              for I: int := Bound(1) to Bound(2) do Console.WriteLine(I + 10); end for;
              for I: int := 2147483646 to 2147483647 do Console.WriteLine(I); continue; end for;
              for I: int := -2147483647 downto -2147483648 do Console.WriteLine(I); end for;
              for I: int := 2 to 1 do Console.WriteLine('wrong'); end for;
              for I: int := 1 downto 2 do Console.WriteLine('wrong'); end for;
            end;
            end program;
            """, "1", "2", "11", "12", "2147483646", "2147483647", "-2147483647", "-2147483648");
}
