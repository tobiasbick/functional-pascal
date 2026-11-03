using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class AsyncExecutionTests
{
    [Fact]
    public Task Tasks_value_tasks_generics_records_and_async_lambdas_execute()
        => ProgramExecution.Output("""
            async program Asynchronous;
            uses System.Console as Console, System.Threading.Tasks as Tasks;
            async function Echo<T>(Value: T): Tasks.Task<T>;
            begin await Tasks.Task.Delay(1); return Value; end function;
            type Box<T> = record
              Value: T;
              async function Read(): Tasks.Task<T>;
              begin await Tasks.Task.Delay(1).ConfigureAwait(false); return self.Value; end function;
            end record;
            begin
              Console.WriteLine(await Echo<int>(42));
              Console.WriteLine(await Echo<string>('text'));
              const Item: Box<string> := new Box<string>('record');
              Console.WriteLine(await Item.Read());
              var Captured: int := 1;
              const Callback: function(int): Tasks.Task<int> := async function(Value: int): Tasks.Task<int>
                requires Value > 0;
                ensures (Returned: int): Returned = Captured;
                begin await Tasks.Task.Delay(1); Captured := Captured + Value; return Captured; end function;
              Console.WriteLine(await Callback(2));
              Console.WriteLine(Captured);
              const Expression: function(int): Tasks.Task<int> := async function(Value: int): Tasks.Task<int> => await Echo<int>(Value);
              Console.WriteLine(await Expression(5));
              const Value: Tasks.ValueTask<int> := new Tasks.ValueTask<int>(7);
              Console.WriteLine(await Value.ConfigureAwait(false));
              const Empty: Tasks.ValueTask := new Tasks.ValueTask();
              await Empty;
            end;
            end program;
            """, "42", "text", "record", "3", "3", "5", "7");

    [Fact]
    public Task Await_preserves_expression_order_branches_and_loops()
        => ProgramExecution.Output("""
            async program Expressions;
            uses System.Console as Console, System.Threading.Tasks as Tasks;
            async function Number(Value: int): Tasks.Task<int>;
            begin await Tasks.Task.Delay(1); return Value; end function;
            function Mark(Value: int): int;
            begin Console.WriteLine(Value); return Value; end function;
            begin
              Console.WriteLine(Mark(1) + await Number(2) + Mark(3));
              const Values: int[] := [await Number(4), await Number(5)];
              Values[Mark(0)] := await Number(9);
              for Index: int := 0 to await Number(1) do
                Console.WriteLine(await Number(Values[Index]));
              end for;
              var Count: int := 0;
              while Count < await Number(4) do
                Count := Count + 1;
                if Count = 2 then continue; end if;
                if Count = 4 then break; end if;
                Console.WriteLine(Count);
              end while;
              if false and await Number(99) = Mark(99) then Console.WriteLine('wrong'); end if;
              if true or await Number(98) = Mark(98) then Console.WriteLine('short'); end if;
              Console.WriteLine(if await Number(1) = 1 then await Number(8) else await Number(99) end if);
            end;
            end program;
            """, "1", "3", "6", "0", "9", "5", "1", "3", "short", "8");

    [Fact]
    public Task Await_in_catch_finally_and_contracts_preserves_cleanup_and_results()
        => ProgramExecution.Output("""
            async program Cleanup;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks;
            async function Work(State: int[]; Mode: int): Tasks.Task<int>;
            requires Mode >= 0;
            ensures (Returned: int): Returned = State[0];
            ensures State[1] = 1;
            begin
              try
                await Tasks.Task.Delay(1);
                if Mode = 1 then raise new Sys.ArgumentException('body'); end if;
                return 7;
              catch Error: Sys.ArgumentException when Error.Message = 'body' do
                await Tasks.Task.Delay(1); return 8;
              finally
                await Tasks.Task.Delay(1); State[0] := 7 + Mode; State[1] := State[1] + 1;
              end try;
            end function;
            async function Fail(): Tasks.Task;
            ensures false;
            begin await Tasks.Task.Delay(1); raise new Sys.ArgumentException('original'); end function;
            begin
              const State: int[] := [0, 0];
              Console.WriteLine(await Work(State, 0));
              State[1] := 0; Console.WriteLine(await Work(State, 1));
              try discard await Work(State, -1); catch Error: Sys.InvalidOperationException do Console.WriteLine('requires'); end try;
              State[1] := 0;
              try discard await Work(State, 2); catch Error: Sys.InvalidOperationException do Console.WriteLine('ensures'); end try;
              try await Fail(); catch Error: Sys.ArgumentException do Console.WriteLine(Error.Message); end try;
            end;
            end program;
            """, "7", "8", "requires", "ensures", "original");
}
