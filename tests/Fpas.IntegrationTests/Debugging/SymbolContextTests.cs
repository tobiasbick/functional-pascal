using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class SymbolContextTests
{
    private static WorkerLauncher Launcher => WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());

    [Fact]
    public async Task Recursive_frames_have_distinct_arguments_locals_source_positions_and_stop_scoped_handles()
    {
        using var program = new Debuggee("source space ü");
        await program.Compile("""
            program RecursiveContext;
            uses System.Console as Console;
            function Total(Depth: int): int;
            begin
              const Marker: int := Depth + 10;
              if Depth = 0 then
                begin
                  const Marker: int := 100;
                  return Marker;
                end;
              end if;
              return Total(Depth - 1) + Marker;
            end function;
            begin
              Console.WriteLine(Total(2));
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 9)), deadline.Token);
        await host.Initialize(deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        Assert.Equal("stopped", stop.State);
        var threads = await host.Threads(stop.StopId!.Value, deadline.Token);
        var selected = Assert.Single(threads, thread => thread.IsStopThread);
        Assert.Equal(selected, threads[0]);
        Assert.Equal(threads.Length, threads.Select(thread => thread.Id).Distinct().Count());
        var first = await host.Stack(new(stop.StopId.Value, selected.Id, 0, 1), deadline.Token);
        Assert.False(first.Truncated);
        var leaf = Assert.Single(first.Items);
        Assert.EndsWith(".Total", leaf.Name);
        Assert.Equal(program.Source, leaf.Source!.Path);
        Assert.Equal(9, leaf.Source.Line); Assert.Equal("available", leaf.Source.Status);
        Assert.False(leaf.Generated);
        var parents = await host.Stack(new(stop.StopId.Value, selected.Id, 1, 2), deadline.Token);
        Assert.Equal(first.Total, parents.Total);
        Assert.Equal(2, parents.Items.Length);
        var frames = new[] { leaf }.Concat(parents.Items).ToArray();
        Assert.Equal(3, frames.Select(frame => frame.Id).Distinct().Count());
        for (var depth = 0; depth < 3; depth++)
        {
            Assert.EndsWith(".Total", frames[depth].Name);
            var bindings = (await host.FrameBindings(new(stop.StopId.Value, frames[depth].Id), deadline.Token)).Items;
            Assert.Contains(bindings, value => value.Name == "Depth" && value.Scalar == depth.ToString(System.Globalization.CultureInfo.InvariantCulture));
            Assert.Contains(bindings, value => value.Name == "Marker" && value.Scalar == (depth == 0 ? 100 : 10 + depth).ToString(System.Globalization.CultureInfo.InvariantCulture));
        }
        Assert.Equal(leaf.Id, Assert.Single((await host.Stack(new(stop.StopId.Value, selected.Id, 0, 1), deadline.Token)).Items).Id);
        await host.Resume(stop.StopId.Value, step: true, deadline.Token);
        var next = await host.WaitForStop(deadline.Token);
        Assert.Equal("stopped", next.State);
        var stale = await Assert.ThrowsAsync<DebugException>(() => host.FrameBindings(new(next.StopId!.Value, leaf.Id), deadline.Token));
        Assert.Equal("stale_reference", stale.Code);
        await host.Resume(next.StopId!.Value, step: false, deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("123", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Theory]
    [InlineData("source_mismatch")]
    [InlineData("missing_symbols")]
    [InlineData("symbols_mismatch")]
    public async Task Invalid_symbols_or_source_fail_before_any_target_is_created(string problem)
    {
        using var program = new Debuggee("source-validation");
        await program.Compile("program CheckedSource; begin const Value: int := 42; discard Value; end; end program;");
        var pdb = Path.ChangeExtension(program.Assembly, ".pdb");
        if (problem == "source_mismatch") File.AppendAllText(program.Source, "\n// Changed after compilation.\n");
        else if (problem == "missing_symbols") File.Delete(pdb);
        else
        {
            var otherSource = program.FilePath("Other.fpas"); var otherAssembly = program.FilePath("Other.dll");
            File.WriteAllText(otherSource, "program Other; begin const Value: int := 73; discard Value; end; end program;");
            await program.CompileFile(otherSource, otherAssembly);
            File.Copy(Path.ChangeExtension(otherAssembly, ".pdb"), pdb, overwrite: true);
        }
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token);
        var error = await Assert.ThrowsAsync<DebugException>(() => host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly,
            program.DirectoryPath, new(program.Source, 1)), deadline.Token));
        Assert.Equal(problem, error.Code);
        var state = host.Snapshot;
        while (state.State != "failed") state = await host.WaitForChange(state.Revision, deadline.Token);
        Assert.Null(host.Snapshot.ProcessId);
        Assert.Equal("failed", host.Snapshot.State);
        await host.DisposeAsync();
        using var unlocked = new FileStream(program.Assembly, FileMode.Open, FileAccess.ReadWrite, FileShare.None);
    }

    [Fact]
    public async Task Source_path_case_follows_the_target_operating_system()
    {
        using var program = new Debuggee("case-sensitive-source");
        await program.Compile("program SourceCase; begin const Value: int := 42; discard Value; end; end program;");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token);
        var target = new LaunchTarget(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath,
            new(program.FilePath("PROGRAM.FPAS"), 1));
        if (OperatingSystem.IsWindows())
        {
            await host.Prepare(target, deadline.Token);
            await host.Initialize(deadline.Token);
            Assert.Equal("stopped", (await host.WaitForStop(deadline.Token)).State);
            await host.Stop(deadline.Token);
        }
        else
        {
            var error = await Assert.ThrowsAsync<DebugException>(() => host.Prepare(target, deadline.Token));
            Assert.Equal("invalid_argument", error.Code);
            Assert.Null(host.Snapshot.ProcessId);
        }
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Library_and_program_frames_with_equal_source_filenames_keep_their_own_metadata(bool replaceLoadedModule)
    {
        using var program = new Debuggee("module-source-context");
        var libraryDirectory = program.FilePath("library"); Directory.CreateDirectory(libraryDirectory);
        var librarySource = Path.Combine(libraryDirectory, "Program.fpas");
        var library = program.FilePath("Library.dll");
        File.WriteAllText(librarySource, """
            unit ExternalFrames;
            public function Invoke(Call: function(int): int): int;
            begin
              const Value: int := 40;
              return Call(Value);
            end function;
            end unit;
            """);
        await program.CompileFile(librarySource, library, "--target", "library");
        var replacementSource = Path.Combine(libraryDirectory, "Replacement.fpas");
        var replacement = program.FilePath("Replacement.dll");
        if (replaceLoadedModule)
        {
            File.WriteAllText(replacementSource, "unit ExternalFrames; public function Invoke(Call: function(int): int): int; begin const Other: int := 99; return Call(Other); end function; end unit;");
            await program.CompileFile(replacementSource, replacement, "--target", "library");
        }
        await program.Compile("""
            program ModuleFrames;
            uses ExternalFrames as L, System.Console as Console;
            function Probe(Input: int): int;
            begin
              const Result: int := Input + 2;
              return Result;
            end function;
            begin
              Console.WriteLine(L.Invoke(Probe));
            end;
            end program;
            """, "--reference", library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 6)), deadline.Token);
        await host.Initialize(deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        Assert.Equal("stopped", stop.State);
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, deadline.Token), item => item.IsStopThread);
        if (replaceLoadedModule)
        {
            File.Move(library, library + ".loaded");
            File.Copy(replacement, library);
            File.Move(Path.ChangeExtension(library, ".pdb"), library + ".loaded.pdb");
            File.Copy(Path.ChangeExtension(replacement, ".pdb"), Path.ChangeExtension(library, ".pdb"));
        }
        StackPage stack;
        try { stack = await host.Stack(new(stop.StopId.Value, thread.Id), deadline.Token); }
        catch (DebugException exception) when (replaceLoadedModule && exception.Code == "module_mismatch")
        { await host.Stop(deadline.Token); return; }
        Assert.Equal(program.Source, stack.Items[0].Source!.Path);
        var libraryFrame = Assert.Single(stack.Items, frame => frame.Module == library);
        if (replaceLoadedModule && libraryFrame.Status == "module_mismatch")
        {
            Assert.Null(libraryFrame.Source);
            var unavailable = await Assert.ThrowsAsync<DebugException>(() => host.FrameBindings(new(stop.StopId.Value, libraryFrame.Id), deadline.Token));
            Assert.Equal("unavailable", unavailable.Code);
            await host.Stop(deadline.Token); return;
        }
        Assert.EndsWith(".Invoke", libraryFrame.Name);
        Assert.Equal(librarySource, libraryFrame.Source!.Path);
        Assert.Equal(5, libraryFrame.Source.Line);
        var values = (await host.FrameBindings(new(stop.StopId.Value, libraryFrame.Id), deadline.Token)).Items;
        Assert.Contains(values, item => item.Name == "Value" && item.Scalar == "40");
        await host.Stop(deadline.Token);
    }
}
