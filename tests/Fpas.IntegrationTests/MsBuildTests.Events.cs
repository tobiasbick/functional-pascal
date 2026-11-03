namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Collection_events_example_builds_and_runs()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Events"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "events")))
            File.WriteAllText(project.File("Events/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Events/Events.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Events/bin/Debug/net10.0/Events.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("ItemAdded" + Environment.NewLine + "1" + Environment.NewLine, run.Output);
    }

    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Events_preserve_evaluation_order_dispatch_delegate_identity_and_struct_storage(string configuration)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public delegate void Handler<T>(T value);
            public interface IEvents<T> { event Handler<T> Changed; }
            public interface IDerived<T> : IEvents<T> { }
            public class Publisher<T> : IDerived<T>
            {
                private Handler<T>? handlers;
                public virtual event Handler<T> Changed
                {
                    add { Log.Trace += "+"; handlers += value; }
                    remove { Log.Trace += "-"; handlers -= value; }
                }
                public void Fire(T value) => handlers?.Invoke(value);
            }
            public sealed class Derived : Publisher<int>
            {
                public override event Handler<int> Changed
                {
                    add { Log.Trace += "v"; base.Changed += value; }
                    remove { Log.Trace += "v"; base.Changed -= value; }
                }
            }
            public static class Log
            {
                public static string Trace = "";
                public static Publisher<int> Source = new Derived();
                public static Handler<int> Handler = value => Trace += value.ToString();
                public static Publisher<int> Receiver() { Trace += "R"; return Source; }
                public static Handler<int> ReadHandler() { Trace += "H"; return Handler; }
                public static event Handler<int>? Changed;
                public static void Fire(int value) => Changed?.Invoke(value);
                public static Publisher<int> FailReceiver() { Trace += "R"; throw new InvalidOperationException("receiver"); }
                public static Handler<int> FailHandler() { Trace += "H"; throw new InvalidOperationException("handler"); }
            }
            public sealed class Inherited : Publisher<int> { }
            public interface ICounter { event Action Changed; }
            public struct Counter : ICounter
            {
                public int Adds;
                public int Removes;
                public event Action Changed { add { Adds++; value(); } remove { Removes++; } }
            }
            public sealed class Throwing
            {
                public event Handler<int> Changed
                {
                    add { Log.Trace += "+"; throw new InvalidOperationException("add"); }
                    remove { Log.Trace += "-"; throw new InvalidOperationException("remove"); }
                }
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Events;
            uses Shared as S, System as Sys;
            public procedure Wire();
            begin
              subscribe S.Log.Receiver().Changed with S.Log.ReadHandler();
              subscribe S.Log.Receiver().Changed with S.Log.ReadHandler();
              S.Log.Source.Fire(1);
              unsubscribe S.Log.Receiver().Changed with S.Log.ReadHandler();
              S.Log.Source.Fire(2);
              unsubscribe S.Log.Receiver().Changed with S.Log.ReadHandler();
              S.Log.Source.Fire(3);
            end procedure;
            public procedure InterfaceWire(Value: S.IDerived<int>; Handler: S.Handler<int>);
            begin subscribe Value.Changed with Handler; unsubscribe Value.Changed with Handler; end procedure;
            public procedure InheritedWire(Value: S.Inherited; Handler: S.Handler<int>);
            begin subscribe Value.Changed with Handler; Value.Fire(4); unsubscribe Value.Changed with Handler; end procedure;
            public procedure StaticWire();
            begin
              const Handler: S.Handler<int> := procedure(Value: int) begin S.Log.Trace := S.Log.Trace + Value.ToString(); end procedure;
              subscribe S.Log.Changed with Handler; S.Log.Fire(5);
              unsubscribe S.Log.Changed with Handler; S.Log.Fire(6);
            end procedure;
            public procedure StructWire(ref Value: S.Counter; Handler: Sys.Action);
            begin subscribe Value.Changed with Handler; unsubscribe Value.Changed with Handler; end procedure;
            public procedure GenericWire<T>(ref Value: T; Handler: Sys.Action);
            where T: S.ICounter;
            begin subscribe Value.Changed with Handler; unsubscribe Value.Changed with Handler; end procedure;
            public function ReadonlyStruct(Value: S.Counter; Handler: Sys.Action): int;
            begin subscribe Value.Changed with Handler; return Value.Adds; end function;
            public function StructLocal(Handler: Sys.Action): int;
            begin
              var Value: S.Counter := new S.Counter();
              subscribe Value.Changed with Handler; unsubscribe Value.Changed with Handler;
              const Items: S.Counter[] := new S.Counter[1];
              subscribe Items[0].Changed with Handler; unsubscribe Items[0].Changed with Handler;
              return Value.Adds + Value.Removes + Items[0].Adds + Items[0].Removes;
            end function;
            public procedure FailedReceiver(); begin subscribe S.Log.FailReceiver().Changed with S.Log.ReadHandler(); end procedure;
            public procedure FailedHandler(); begin subscribe S.Log.Receiver().Changed with S.Log.FailHandler(); end procedure;
            public procedure FailedAccessor(Value: S.Throwing; Remove: bool);
            begin
              if Remove then unsubscribe Value.Changed with S.Log.ReadHandler();
              else subscribe Value.Changed with S.Log.ReadHandler(); end if;
            end procedure;
            public procedure Cleanup(Value: S.Publisher<int>; Handler: S.Handler<int>);
            begin
              subscribe Value.Changed with Handler;
              try Value.Fire(7); finally unsubscribe Value.Changed with Handler; end try;
            end procedure;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using Shared;
            Events.Wire();
            if (Log.Trace != "RHv+RHv+11RHv-2RHv-") throw new Exception("Evaluation/identity/virtual dispatch: " + Log.Trace);
            Log.Trace = ""; Events.InterfaceWire(Log.Source, Log.Handler);
            if (Log.Trace != "v+v-") throw new Exception("Inherited interface dispatch: " + Log.Trace);
            Log.Trace = ""; Events.InheritedWire(new Inherited(), Log.Handler);
            if (Log.Trace != "+4-") throw new Exception("Inherited generic class event: " + Log.Trace);
            Log.Trace = ""; Events.StaticWire(); if (Log.Trace != "5") throw new Exception("Static removal: " + Log.Trace);
            var callbacks = 0; Action handler = () => callbacks++;
            var counter = new Shared.Counter(); Events.StructWire(ref counter, handler);
            if (counter.Adds != 1 || counter.Removes != 1 || callbacks != 1 || Events.StructLocal(handler) != 4 || callbacks != 3)
                throw new Exception("Struct event mutated a copy or lost capture storage");
            Events.GenericWire(ref counter, handler);
            if (counter.Adds != 2 || counter.Removes != 2 || callbacks != 4 || Events.ReadonlyStruct(counter, handler) != 2 || callbacks != 5)
                throw new Exception("Constrained dispatch or readonly defensive copy failed");
            void Fails(Action action, string message, string trace)
            {
                Log.Trace = "";
                try { action(); throw new Exception("Expected accessor/operand failure"); }
                catch (InvalidOperationException ex) when (ex.Message == message) { }
                if (Log.Trace != trace) throw new Exception("Exception ordering: " + Log.Trace);
            }
            Fails(Events.FailedReceiver, "receiver", "R");
            Fails(Events.FailedHandler, "handler", "RH");
            Fails(() => Events.FailedAccessor(new Throwing(), false), "add", "H+");
            Fails(() => Events.FailedAccessor(new Throwing(), true), "remove", "H-");
            Log.Trace = "";
            Fails(() => Events.Cleanup(Log.Source, _ => throw new InvalidOperationException("callback")), "callback", "v+v-");
            Log.Source.Fire(8);
            if (Log.Trace != "v+v-") throw new Exception("Finally did not remove failed callback");
            Console.WriteLine("Events checked");
            """);
        await BuildAndRunContractProject(project, configuration, "Events checked");
    }

    [Fact]
    public async Task Event_operands_suspend_in_written_order_and_cleanup_removes_the_same_delegate()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public sealed class Publisher
            {
                private Action? handlers;
                public int Adds;
                public int Removes;
                public event Action Changed { add { Adds++; handlers += value; } remove { Removes++; handlers -= value; } }
                public void Fire() => handlers?.Invoke();
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Events;
            uses Shared as S, System as Sys, System.Threading.Tasks as Tasks;
            public async function Wire(Source: Tasks.Task<S.Publisher>; Handler: Tasks.Task<Sys.Action>): Tasks.Task;
            begin subscribe (await Source).Changed with await Handler; end function;
            public async function Cleanup(Source: S.Publisher; Handler: Sys.Action; Gate: Tasks.Task): Tasks.Task;
            begin
              subscribe Source.Changed with Handler;
              try await Gate;
              finally
                await Tasks.Task.Delay(1);
                unsubscribe Source.Changed with Handler;
              end try;
            end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using Shared;
            var source = new TaskCompletionSource<Publisher>(TaskCreationOptions.RunContinuationsAsynchronously);
            var handler = new TaskCompletionSource<Action>(TaskCreationOptions.RunContinuationsAsynchronously);
            var publisher = new Publisher(); var calls = 0; Action callback = () => calls++;
            var pending = Events.Wire(source.Task, handler.Task);
            if (pending.IsCompleted || publisher.Adds != 0) throw new Exception("Event did not suspend");
            source.SetResult(publisher);
            handler.SetResult(callback);
            await pending.WaitAsync(TimeSpan.FromSeconds(10));
            publisher.Fire();
            if (publisher.Adds != 1 || calls != 1) throw new Exception("Lost receiver or handler across await");
            var gate = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var cleanup = Events.Cleanup(publisher, callback, gate.Task);
            if (cleanup.IsCompleted || publisher.Adds != 2) throw new Exception("Subscription did not precede suspension");
            gate.SetException(new InvalidOperationException("gate"));
            try { await cleanup.WaitAsync(TimeSpan.FromSeconds(10)); throw new Exception("Missing fault"); }
            catch (InvalidOperationException ex) when (ex.Message == "gate") { }
            publisher.Fire();
            if (publisher.Removes != 1 || calls != 2) throw new Exception("Async finally changed handler identity");
            Console.WriteLine("Async events checked");
            """);
        await BuildAndRunContractProject(project, "Debug", "Async events checked");
    }
}
