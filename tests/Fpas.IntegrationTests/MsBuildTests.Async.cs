using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Async_functions_suspend_and_preserve_context_cleanup_faults_and_cancellation(string configuration)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/AsyncApi.cs"), """
            namespace Demo;
            public sealed class AsyncResource(int[] state, int digit, bool fail) : IAsyncDisposable, IDisposable
            {
                public async ValueTask DisposeAsync()
                {
                    await Task.Yield(); state[0] = state[0] * 10 + digit;
                    if (fail) throw new ArgumentException("cleanup");
                }
                public void Dispose() { state[0] = state[0] * 10 + digit; }
            }
            public static class AsyncApi
            {
                public static int ThreadId() => Environment.CurrentManagedThreadId;
                public static bool BrokenFilter() => throw new InvalidOperationException("filter");
                public static AsyncResource Acquire(int[] state, int digit, bool fail)
                    => fail ? throw new ArgumentException("acquire") : new AsyncResource(state, digit, false);
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses System as Sys, System.Threading.Tasks as Tasks, Demo as D;
            public async function Wait(Gate: Tasks.Task<int>; State: int[]): Tasks.Task<int>;
            requires State.Length = 3;
            ensures (Returned: int): Returned = State[2];
            begin
              State[0] := D.AsyncApi.ThreadId();
              const Value: int := await Gate;
              State[1] := D.AsyncApi.ThreadId();
              return Value + State[2];
            end function;
            public async function Configure(Gate: Tasks.Task<int>; State: int[]): Tasks.Task<int>;
            begin
              const Value: int := await Gate.ConfigureAwait(false);
              State[1] := D.AsyncApi.ThreadId(); return Value;
            end function;
            public async function Guard(Value: int): Tasks.Task<int>;
            requires Value > 0;
            begin return Value; end function;
            public async function Clean(Gate: Tasks.Task<int>; State: int[]; Mode: int): Tasks.Task<int>;
            ensures (Returned: int): Returned = 7;
            ensures State[0] = 3214;
            begin
              try
                await using const Outer: D.AsyncResource := new D.AsyncResource(State, 1, Mode = 1);
                await using const Inner: D.AsyncResource := new D.AsyncResource(State, 2, false);
                using const Sync: D.AsyncResource := new D.AsyncResource(State, 3, false);
                return await Gate;
              finally
                await Tasks.Task.Delay(1); State[0] := State[0] * 10 + 4;
              end try;
            end function;
            public async function Catch(Gate: Tasks.Task<int>; State: int[]): Tasks.Task<int>;
            begin
              try
                try discard await Gate;
                catch Error: Sys.Exception when D.AsyncApi.BrokenFilter() do return -1;
                catch Error: Sys.ArgumentException do
                  await Tasks.Task.Delay(1); State[0] := 1; raise;
                finally await Tasks.Task.Delay(1); State[0] := State[0] + 1; end try;
              catch Error: Sys.ArgumentException do
                await Tasks.Task.Delay(1); return State[0];
              end try;
              return 0;
            end function;
            public async function Identity<T>(Gate: Tasks.Task<T>): Tasks.Task<T>;
            begin return await Gate; end function;
            public async function Loops(State: int[]): Tasks.Task<int>;
            begin
              for Index: int := 1 to 3 do
                await using const Resource: D.AsyncResource := new D.AsyncResource(State, Index, false);
                await Tasks.Task.Delay(1);
                if Index = 1 then continue; end if;
                break;
              end for;
              const Values: int[] := [3, 4];
              for Value: int in Values do await Tasks.Task.Delay(1); State[0] := State[0] * 10 + Value; end for;
              await using const Empty: D.AsyncResource? := null;
              return State[0];
            end function;
            public async function Acquisition(State: int[]): Tasks.Task;
            begin
              await using const First: D.AsyncResource := D.AsyncApi.Acquire(State, 1, false);
              await using const Second: D.AsyncResource := D.AsyncApi.Acquire(State, 2, true);
            end function;
            public function Capture<T>(Value: T): function(): Tasks.Task<T>;
            begin return async function(): Tasks.Task<T> begin await Tasks.Task.Delay(1); return Value; end function; end function;
            public type Box<T> = record
              Value: T;
              async function Read<U>(Other: U): Tasks.Task<U>;
              begin await Tasks.Task.Delay(1); return Other; end function;
            end record;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using System.Collections.Concurrent;
            using System.Reflection;
            using System.Runtime.CompilerServices;
            static void Require(bool value) { if (!value) throw new Exception("Wrong async behavior."); }
            var gate = new TaskCompletionSource<int>(TaskCreationOptions.RunContinuationsAsynchronously);
            var state = new int[3];
            var caller = Environment.CurrentManagedThreadId;
            var pending = Bridge.Wait(gate.Task, state);
            Require(state[0] == caller && !pending.IsCompleted);
            gate.SetResult(0); Require(await pending == 0);
            var rejected = Bridge.Guard(0);
            Require(rejected.IsFaulted);
            try { await rejected; throw new Exception("Missing contract."); }
            catch (InvalidOperationException error) { Require(error.Message.Contains("requires Value > 0")); }
            foreach (var mode in new[] { 0, 1, 2, 3 })
            {
                gate = new(TaskCreationOptions.RunContinuationsAsynchronously); state = new int[1];
                var cleanup = Bridge.Clean(gate.Task, state, mode);
                Require(!cleanup.IsCompleted && state[0] == 0);
                if (mode == 2) gate.SetException(new ArgumentException("body"));
                else if (mode == 3) gate.SetCanceled(new CancellationToken(true));
                else gate.SetResult(7);
                try { Require(await cleanup == 7 && mode == 0); }
                catch (ArgumentException error) { Require(error.Message == (mode == 1 ? "cleanup" : "body")); }
                catch (OperationCanceledException) { Require(mode == 3 && cleanup.IsCanceled); }
                Require(state[0] == 3214);
            }
            state = new int[1];
            Require(await Bridge.Catch(Task.FromException<int>(new ArgumentException("original")), state) == 2);
            Require(await Bridge.Identity(Task.FromResult("generic")) == "generic");
            state = new int[1]; Require(await Bridge.Loops(state) == 1234);
            state = new int[1];
            try { await Bridge.Acquisition(state); throw new Exception("Missing acquisition failure."); }
            catch (ArgumentException error) { Require(error.Message == "acquire" && state[0] == 1); }
            Require(await Bridge.Capture(9)() == 9);
            Require(await new Bridge.Box<string>("box").Read(11) == 11);
            var attribute = typeof(Bridge).GetMethod(nameof(Bridge.Identity))!.GetCustomAttribute<AsyncStateMachineAttribute>()!;
            Require(typeof(IAsyncStateMachine).IsAssignableFrom(attribute.StateMachineType));
            var old = SynchronizationContext.Current;
            using var context = new PumpContext(); SynchronizationContext.SetSynchronizationContext(context);
            try
            {
                state = new int[3]; gate = new(TaskCreationOptions.RunContinuationsAsynchronously);
                pending = Bridge.Wait(gate.Task, state);
                Task.Run(() => gate.SetResult(0)).GetAwaiter().GetResult();
                context.CompleteOne(); Require(pending.GetAwaiter().GetResult() == 0 && state[0] == state[1]);
                state = new int[3]; gate = new(TaskCreationOptions.RunContinuationsAsynchronously);
                pending = Bridge.Configure(gate.Task, state);
                Task.Run(() => gate.SetResult(13)).GetAwaiter().GetResult();
                Require(pending.GetAwaiter().GetResult() == 13 && state[1] != Environment.CurrentManagedThreadId);
            }
            finally { SynchronizationContext.SetSynchronizationContext(old); }
            Console.WriteLine("Async checked");
            sealed class PumpContext : SynchronizationContext, IDisposable
            {
                readonly BlockingCollection<(SendOrPostCallback Callback, object? State)> queue = new();
                public override void Post(SendOrPostCallback callback, object? state) => queue.Add((callback, state));
                public void CompleteOne()
                {
                    if (!queue.TryTake(out var work, TimeSpan.FromSeconds(10))) throw new Exception("Continuation was not posted.");
                    work.Callback(work.State);
                }
                public void Dispose() => queue.Dispose();
            }
            """);
        await BuildAndRunContractProject(project, configuration, "Async checked");
    }
}
