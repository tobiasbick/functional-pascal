using Fpas.Debugging.Core;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class SourceControlTests
{
    [Fact]
    public async Task Collectible_module_unload_returns_its_breakpoints_to_pending()
    {
        using var program = new Debuggee("own-unloaded-module");
        var source = program.FilePath("Collectible.fpas"); var library = program.FilePath("Collectible.dll");
        File.WriteAllText(source, """
            unit CollectiblePlugin;
            public function Compute(Value: int): int;
            begin
              const Result: int := Value + 1;
              return Result;
            end function;
            end unit;
            """);
        await program.CompileFile(source, library, "--target", "library");
        await program.Compile("""
            program UnloadModule;
            uses System as Sys, System.Console as Console, System.Type as Type,
              System.Reflection.Assembly as Assembly, System.Reflection.MethodInfo as MethodInfo,
              System.Runtime.Loader.AssemblyLoadContext as LoadContext, System.IO.Path as Path,
              System.Threading.Thread as Thread;
            function LoadAndRelease(): Sys.WeakReference;
            begin
              const Context: LoadContext := new LoadContext('collectible-test', true);
              const Weak: Sys.WeakReference := new Sys.WeakReference(Context);
              const Loaded: Assembly := Context.LoadFromAssemblyPath(Path.GetFullPath('Collectible.dll'));
              for Kind: Type in Loaded.GetTypes() do
                const Method: MethodInfo? := Kind.GetMethod('Compute');
                if Method is MethodInfo then
                  const Arguments: array of object? := [41];
                  Console.WriteLine(Method.Invoke(null, Arguments));
                end if;
              end for;
              Context.Unload();
              return Weak;
            end function;
            begin
              const Weak: Sys.WeakReference := LoadAndRelease();
              while Weak.IsAlive do
                Sys.GC.Collect();
                Sys.GC.WaitForPendingFinalizers();
              end while;
              Console.WriteLine('unloaded');
              Thread.Sleep(60000);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        var point = Assert.Single(await host.SetBreakpoints(new(source, [new(5)], host.Snapshot.StopId), deadline.Token));
        Assert.Equal("pending", point.State);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var hit = await host.WaitForStop(deadline.Token);
        Assert.Equal("breakpoint", hit.Reason);
        Assert.Equal(new[] { point.Id }, hit.HitBreakpoints);
        Assert.Equal("bound", Assert.Single(hit.Breakpoints!).State);
        await host.Resume(hit.StopId!.Value, "continue", deadline.Token);
        var state = host.Snapshot;
        while (state.Breakpoints is not [{ State: "pending", Hits: 1 }])
        {
            Assert.DoesNotContain(state.State, new[] { "exited", "failed" });
            state = await host.WaitForChange(state.Revision, deadline.Token);
        }
        Assert.Equal("running", state.State);
        Assert.Null(Assert.Single(state.Breakpoints!).Actual);
        await host.Pause(deadline.Token);
        Assert.Equal("pause", (await host.WaitForStop(deadline.Token)).Reason);
        await host.Stop(deadline.Token);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));
    }
}
