using Fpas.IntegrationTests.Support;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Execution;

public sealed class ContractExecutionTests
{
    [Fact]
    public Task Successful_returns_keep_the_value_once_and_check_after_finally()
        => ProgramExecution.Output("""
            program Contracts;
            uses System.Console as Console;
            function Once(State: int[]): int;
            begin State[1] := State[1] + 1; return 7; end function;
            function Choose(State: int[]; Early: bool): int;
            requires State.Length = 2;
            ensures (Returned: int): Returned = State[0];
            ensures State[1] = 1;
            begin
              try
                if Early then return Once(State); end if;
                return Once(State);
              finally State[0] := 7; end try;
            end function;
            procedure Finish(State: int[]);
            ensures State[0] = 9;
            begin try return; finally State[0] := 9; end try; end procedure;
            begin
              const State: int[] := [0, 0];
              Console.WriteLine(Choose(State, true));
              State[0] := 0; State[1] := 0;
              Console.WriteLine(Choose(State, false));
              Finish(State); Console.WriteLine(State[0]);
            end;
            end program;
            """, "7", "7", "9");

    [Fact]
    public async Task Preconditions_short_circuit_and_identify_the_original_clause()
    {
        var result = await ProgramExecution.Run("""
            program Contracts;
            uses System as Sys, System.Console as Console;
            function Check(Value: int; State: int[]): int;
            requires Value > 0;
            requires State[99] = 0;
            begin State[0] := 1; return Value; end function;
            begin
              const State: int[] := [0];
              try discard Check(0, State);
              catch Failure: Sys.InvalidOperationException do Console.WriteLine(Failure.Message); end try;
              Console.WriteLine(State[0]);
            end;
            end program;
            """);
        Assert.Equal(0, result.ExitCode);
        Assert.Contains("Contracts.Check: requires Value > 0", result.Output);
        Assert.Contains("<stdin>:", result.Output);
        Assert.EndsWith(Environment.NewLine + "0" + Environment.NewLine, result.Output);
        Assert.Empty(result.Error);
    }

    [Fact]
    public Task Contract_only_captures_generics_and_readonly_data_conditions_execute()
        => ProgramExecution.Output("""
            program Captures;
            uses System.Console as Console;
            function Identity<T>(Value: T): T;
            ensures (Returned: T): true;
            begin return Value; end function;
            function Length(Value: string?): int;
            requires Value <> null;
            requires Value.Length > 0;
            ensures (Returned: int): Returned > 0;
            begin return Value.Length; end function;
            function Present(Value: int?): int;
            requires Value.HasValue;
            ensures (Returned: int): Returned = Value.Value;
            begin return Value.Value; end function;
            begin
              var Expected: int := 1;
              const Callback: function(): int := function(): int
                ensures (Returned: int): Returned = Expected;
                begin return 2; end function;
              Expected := 2;
              Console.WriteLine(Callback());
              Console.WriteLine(Identity(7));
              Console.WriteLine(Length('abc'));
              Console.WriteLine(Present(5));
            end;
            end program;
            """, "2", "7", "3", "5");

    [Fact]
    public Task Body_and_condition_exceptions_skip_postconditions_and_are_not_wrapped()
        => ProgramExecution.Output("""
            program Exceptions;
            uses System as Sys, System.Console as Console;
            function Fail(State: int[]): int;
            ensures State[99] = 0;
            begin raise new Sys.ArgumentException('body'); end function;
            function Condition(State: int[]): int;
            requires State[99] = 0;
            begin return 1; end function;
            function Postcondition(State: int[]): int;
            ensures State[99] = 0;
            begin return 1; end function;
            begin
              const State: int[] := [0];
              try discard Fail(State); catch Failure: Sys.ArgumentException do Console.WriteLine(Failure.Message); end try;
              try discard Condition(State); catch Failure: Sys.IndexOutOfRangeException do Console.WriteLine('condition'); end try;
              try discard Postcondition(State); catch Failure: Sys.IndexOutOfRangeException do Console.WriteLine('postcondition'); end try;
            end;
            end program;
            """, "body", "condition", "postcondition");
}
