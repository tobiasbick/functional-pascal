using System.Diagnostics;
using Fpas.Debugging.Native;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class NativeCoreTests
{
    private static WorkerLauncher Launcher => WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
    [Fact]
    public async Task Own_CoreCLR_connection_binds_source_reads_local_steps_and_exits()
    {
        using var program = new Debuggee("own-native");
        await program.Compile("""
            program NativeProbe;
            uses System.Console as Console;
            function Calculate(Input: int): int;
            begin
              var Value: int := Input + 1;
              Value := Value * 2;
              return Value + 2;
            end function;
            begin
              Console.WriteLine(Calculate(19));
            end;
            end program;
            """);
        Assert.Equal("42\n", await program.Run());
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var session = await CoreClrSession.Launch(BuildProcess.DotnetHost(), program.Assembly,
            program.DirectoryPath, program.Source, 6, deadline.Token, Launcher);
        var pid = session.ProcessId;
        using var process = Process.GetProcessById(pid);
        _ = process.SafeHandle;
        Assert.Equal("breakpoint", await session.WaitForStop(deadline.Token));
        Assert.Equal(20, session.IntegerLocals()["Value"]);
        var position = session.Position();
        session.StepOver();
        Assert.Equal("step", await session.WaitForStop(deadline.Token));
        Assert.Equal(40, session.IntegerLocals()["Value"]);
        Assert.Equal(position.Method, session.Position().Method);
        Assert.True(session.Position().Offset > position.Offset);
        session.Continue();
        await session.WaitForExit(deadline.Token);
        Assert.Equal(0, await session.ExitCode(deadline.Token));
        AssertExited(process);
    }

    [Fact]
    public async Task A_cancelled_stop_wait_and_repeated_disposal_reap_a_stopped_target()
    {
        using var program = new Debuggee("own-native-stop");
        await program.Compile("""
            program Waiting;
            uses System.Console as Console;
            begin
              var Value: int := 19;
              Console.WriteLine(Value);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var session = await CoreClrSession.Launch(BuildProcess.DotnetHost(), program.Assembly,
            program.DirectoryPath, program.Source, 5, deadline.Token, Launcher);
        await using (session)
        {
            using var target = Process.GetProcessById(session.ProcessId);
            _ = target.SafeHandle;
            Assert.Equal("breakpoint", await session.WaitForStop(deadline.Token));
            await Assert.ThrowsAnyAsync<OperationCanceledException>(async () => await session.WaitForStop(new CancellationToken(true)));
            await session.DisposeAsync();
            AssertExited(target);
            await session.DisposeAsync();
        }
    }

    private static void AssertExited(Process process)
    {
        if (OperatingSystem.IsLinux()) Assert.False(Directory.Exists($"/proc/{process.Id}"));
        else Assert.True(process.HasExited);
    }

    [Fact]
    public async Task Invalid_launch_directory_is_rejected_and_inputs_remain_unlocked()
    {
        using var program = new Debuggee("own-native-failed-start");
        await program.Compile("program Startup; begin const Value: int := 42; discard Value; end; end program;");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        CoreClrSession? unexpected = null;
        try
        {
            await Assert.ThrowsAsync<IOException>(async () => unexpected = await CoreClrSession.Launch(BuildProcess.DotnetHost(), program.Assembly,
                program.FilePath("missing-directory"), program.Source, 1, deadline.Token, Launcher));
        }
        finally { if (unexpected is not null) await unexpected.DisposeAsync(); }
        using var image = new FileStream(program.Assembly, FileMode.Open, FileAccess.ReadWrite, FileShare.None);
        using var symbols = new FileStream(Path.ChangeExtension(program.Assembly, ".pdb"), FileMode.Open, FileAccess.ReadWrite, FileShare.None);
    }

    [Fact]
    public async Task A_host_that_exits_before_CoreCLR_start_reports_startup_failure_without_waiting_for_cancellation()
    {
        using var program = new Debuggee("own-native-missing-runtime");
        await program.Compile("program NoRuntime; begin const Value: int := 42; discard Value; end; end program;");
        File.WriteAllText(Path.ChangeExtension(program.Assembly, ".runtimeconfig.json"), """
            {"runtimeOptions":{"tfm":"net10.0","rollForward":"Disable","framework":{"name":"Microsoft.NETCore.App","version":"99.0.0"}}}
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        await Assert.ThrowsAsync<IOException>(() => CoreClrSession.Launch(BuildProcess.DotnetHost(), program.Assembly,
            program.DirectoryPath, program.Source, 1, deadline.Token, Launcher));
        using var image = new FileStream(program.Assembly, FileMode.Open, FileAccess.ReadWrite, FileShare.None);
        using var symbols = new FileStream(Path.ChangeExtension(program.Assembly, ".pdb"), FileMode.Open, FileAccess.ReadWrite, FileShare.None);
    }

    [Fact]
    public async Task Cancelling_a_registered_suspended_launch_reaps_the_target_and_releases_inputs()
    {
        using var program = new Debuggee("own-native-start-cancel");
        await program.Compile("program CancelStartup; begin const Value: int := 42; discard Value; end; end program;");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
        await using var session = await CoreClrSession.Prepare(BuildProcess.DotnetHost(), program.Assembly,
            program.DirectoryPath, program.Source, 1, deadline.Token, Launcher);
        using var target = Process.GetProcessById(session.ProcessId);
        _ = target.SafeHandle;
        Assert.False(target.HasExited);
        deadline.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => session.Initialize(deadline.Token));
        if (OperatingSystem.IsLinux()) Assert.False(Directory.Exists($"/proc/{target.Id}"));
        else Assert.True(target.HasExited);
        using var image = new FileStream(program.Assembly, FileMode.Open, FileAccess.ReadWrite, FileShare.None);
        using var symbols = new FileStream(Path.ChangeExtension(program.Assembly, ".pdb"), FileMode.Open, FileAccess.ReadWrite, FileShare.None);
    }
}
