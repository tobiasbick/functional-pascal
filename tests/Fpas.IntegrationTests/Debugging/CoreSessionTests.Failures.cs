using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class CoreSessionTests
{
    [Fact]
    public async Task A_native_unbindable_point_preserves_failure_in_its_breakpoint_result()
    {
        using var program = new Debuggee("own-core-binding-failure");
        await program.Compile("program InvalidBreakpoint; begin const Value: int := 42; discard Value; end; end program;");
        InvalidBreakpointSymbols.Write(program);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 1)), deadline.Token);
        await host.Initialize(deadline.Token);
        var ended = await host.WaitForStop(deadline.Token);
        Assert.Equal("exited", ended.State); Assert.Null(ended.Error); Assert.Equal(0, ended.ExitCode);
        var rejected = Assert.Single(ended.Breakpoints!);
        Assert.Equal("rejected", rejected.State); Assert.Equal("unbound_breakpoint", rejected.Error!.Code); Assert.Equal(0, rejected.Hits);
        await host.DisposeAsync();
    }

    [Fact]
    public async Task A_primary_module_changed_after_preflight_fails_its_real_load_callback()
    {
        using var program = new Debuggee("own-core-load-failure");
        await program.Compile("program Original; begin const Value: int := 42; discard Value; end; end program;");
        var replacementSource = program.FilePath("Replacement.fpas"); var replacement = program.FilePath("Replacement.dll");
        File.WriteAllText(replacementSource, "program Replacement; uses System.IO.File as File; begin File.WriteAllText('executed.txt', 'bad'); end; end program;");
        await program.CompileFile(replacementSource, replacement);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), deadline.Token);
        File.Move(program.Assembly, program.Assembly + ".preflight");
        File.Move(replacement, program.Assembly);
        try { await host.Initialize(deadline.Token); }
        catch (DebugException) when (host.Snapshot.State == "failed") { }
        var ended = await host.WaitForStop(deadline.Token);
        Assert.Equal("failed", ended.State);
        Assert.Equal("module_mismatch", ended.Error!.Code);
        Assert.Null(ended.ExitCode);
        Assert.False(File.Exists(program.FilePath("executed.txt")));
        await host.DisposeAsync();
    }
}
