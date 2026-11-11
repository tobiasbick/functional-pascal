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
              public Value: T;
              public async function Read(): Tasks.Task<T>;
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
              const Values: array of int := [await Number(4), await Number(5)];
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

    /// <summary>Checks that async assignments mutate CLR struct storage through both execution paths.</summary>
    /// <returns>The completed execution and output assertions.</returns>
    [Fact]
    public Task Async_assignments_mutate_the_original_struct_storage()
        => ProgramExecution.Output("""
            async program StructAssignments;
            uses System as Sys, System.Console as Console, System.Drawing as Drawing,
              System.Numerics as Numerics, System.Threading.Tasks as Tasks;

            async function Update(Gate: Tasks.Task<int>): Tasks.Task;
            begin
              var Point: Drawing.Point := new Drawing.Point(1, 2);
              const Points: array of Drawing.Point := [Point];
              Points[0].Y := 3;
              Console.WriteLine(Points[0].Y);

              var Vector: Numerics.Vector2 := new Numerics.Vector2(0.0F, 0.0F);
              Point.X := await Gate;
              Vector.X := cast<float>(await Gate);
              Console.WriteLine(Point.X);
              Console.WriteLine(cast<int>(Vector.X));
            end function;

            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>();
              const Pending: Tasks.Task := Update(Gate.Task);
              if Pending.IsCompleted then
                raise new Sys.InvalidOperationException('The assignment must wait for the gate');
              end if;
              Gate.SetResult(42);
              await Pending;
            end;
            end program;
            """, "3", "42", "42");

    /// <summary>Checks that an awaited assignment keeps the original generic reference receiver.</summary>
    /// <returns>The completed execution and output assertions.</returns>
    [Fact]
    public Task Await_keeps_the_original_generic_reference_receiver()
        => ProgramExecution.Output("""
            async program GenericReceiver;
            uses System.Console as Console, System.Collections.Generic as C, System.Threading.Tasks as Tasks;

            async function Update<T>(Original: T; Replacement: T; Gate: Tasks.Task<int>): Tasks.Task;
            where T: class, C.IList<int>;
              function ChangeReceiver(): Tasks.Task<int>;
              begin
                Current := Replacement;
                return Gate;
              end function;
            begin
              var Current: T := Original;
              Current[0] := await ChangeReceiver();
            end function;

            begin
              const First: C.List<int> := new C.List<int>();
              First.Add(1);
              const Second: C.List<int> := new C.List<int>();
              Second.Add(2);

              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>();
              const Pending: Tasks.Task := Update(First, Second, Gate.Task);
              Gate.SetResult(42);
              await Pending;
              Console.WriteLine(First[0]);
              Console.WriteLine(Second[0]);
            end;
            end program;
            """, "42", "2");

    [Fact]
    public Task Await_in_catch_finally_and_contracts_preserves_cleanup_and_results()
        => ProgramExecution.Output("""
            async program Cleanup;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks;
            async function Work(State: array of int; Mode: int): Tasks.Task<int>;
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
              const State: array of int := [0, 0];
              Console.WriteLine(await Work(State, 0));
              State[1] := 0; Console.WriteLine(await Work(State, 1));
              try discard await Work(State, -1); catch Error: Sys.InvalidOperationException do Console.WriteLine('requires'); end try;
              State[1] := 0;
              try discard await Work(State, 2); catch Error: Sys.InvalidOperationException do Console.WriteLine('ensures'); end try;
              try await Fail(); catch Error: Sys.ArgumentException do Console.WriteLine(Error.Message); end try;
            end;
            end program;
            """, "7", "8", "requires", "ensures", "original");
    [Fact]
    public Task Filters_search_before_async_unwind_and_cleanup_failures_restart_search()
        => ProgramExecution.Output("""
            async program FilterOrdering;
            uses System as Sys, System.Console as Console, System.IO as IO, System.Threading.Tasks as Tasks;

            function Filter(State: array of int; Throwing: bool): bool;
            begin
              State[0] := State[0] + 1;
              Console.WriteLine(State[1]);
              if Throwing then raise new Sys.Exception('filter'); end if;
              return true;
            end function;

            begin
              const State: array of int := [0, 0];
              const Stream: IO.MemoryStream := new IO.MemoryStream();
              try
                try
                  await using const Owned: IO.MemoryStream := Stream;
                  try
                    await Tasks.Task.Delay(1);
                    raise new Sys.Exception('body');
                  finally
                    await Tasks.Task.Delay(1);
                    State[1] := 1;
                    Console.WriteLine(Stream.CanRead);
                  end try;
                finally
                  State[1] := 2;
                end try;
              catch Error: Sys.Exception when Filter(State, true) do
                Console.WriteLine('wrong');
              catch Error: Sys.Exception when Filter(State, false) do
                Console.WriteLine(State[0]);
                Console.WriteLine(State[1]);
                Console.WriteLine(Stream.CanRead);
                Console.WriteLine(Error.Message);
              end try;

              State[0] := 0; State[1] := 0;
              try
                try
                  await Tasks.Task.Delay(1);
                  raise new Sys.Exception('original');
                finally
                  State[1] := 1;
                  raise new Sys.InvalidOperationException('cleanup');
                end try;
              catch Error: Sys.Exception when Filter(State, false) do
                Console.WriteLine(State[0]);
                Console.WriteLine(Error.Message);
              end try;
            end;
            end program;
            """, "0", "0", "True", "2", "2", "False", "body", "0", "1", "2", "cleanup");
}
