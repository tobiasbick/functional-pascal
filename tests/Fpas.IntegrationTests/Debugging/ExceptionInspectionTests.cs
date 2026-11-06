using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class ExceptionInspectionTests
{
    private static async Task<DebugSessionHost> Launch(Debuggee program, CancellationToken token)
    {
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, token);
        try
        {
            await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), token);
            await host.Initialize(token);
            Assert.Equal("entry", (await host.WaitForStop(token)).Reason);
            return host;
        }
        catch { await host.DisposeAsync(); throw; }
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Raised_policy_and_default_unhandled_policy_preserve_passive_exception_facts(bool raised)
    {
        using var program = new Debuggee("own-exception-policy");
        await program.Compile("""
            program ExceptionPolicy;
            uses System as Sys, System.Console as Console;
            procedure Work();
            begin
              try
                raise new Sys.ArgumentException('handled λ');
              catch Failure: Sys.ArgumentException do
                Console.WriteLine('handled');
              end try;
              raise new Sys.InvalidOperationException('unhandled' + #10 + '😀');
            end procedure;
            begin
              Work();
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        if (raised) await host.SetExceptions(new(Raised: true), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        if (raised)
        {
            Assert.Equal("exception", stop.Reason);
            var first = await host.ExceptionInfo(stop.StopId!.Value, deadline.Token);
            Assert.Equal("System.ArgumentException", first.Type);
            Assert.Equal("handled λ", first.Message.Text);
            Assert.Equal("raised", first.Policy);
            Assert.Contains(first.CurrentStack, frame => frame.Source?.Path == program.Source && frame.Source.Line == 6);
            await host.SetExceptions(new(), deadline.Token);
            await host.Resume(stop.StopId.Value, "continue", deadline.Token);
            stop = await host.WaitForStop(deadline.Token);
        }
        Assert.Equal("exception", stop.Reason);
        var failure = await host.ExceptionInfo(stop.StopId!.Value, deadline.Token);
        Assert.Equal("System.InvalidOperationException", failure.Type);
        Assert.Equal("unhandled\n😀", failure.Message.Text);
        Assert.False(failure.Message.Truncated); Assert.Null(failure.Message.RawUtf16);
        Assert.Equal("unhandled", failure.Policy);
        Assert.Equal("available", failure.Status);
        Assert.Equal(program.Source, failure.Source!.Path);
        Assert.Equal(10, failure.Source.Line);
        Assert.Contains(failure.ThrowStack, frame => frame.Source?.Path == program.Source);
        await host.Stop(deadline.Token);
        Assert.Contains("handled", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Fact]
    public async Task Exception_inspection_reads_stored_fields_without_calling_message_or_display_overrides()
    {
        using var program = new Debuggee("own-passive-exception");
        var project = program.FilePath("Hostile.csproj");
        File.WriteAllText(project, "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><NuGetAudit>false</NuGetAudit></PropertyGroup></Project>");
        File.WriteAllText(program.FilePath("Hostile.cs"), """
            using System;
            using System.IO;
            public sealed class HostileException : Exception
            {
                public HostileException() : base("stored λ") { }
                public override string Message { get { File.WriteAllText("message-called.txt", "bad"); return "computed"; } }
                public override string ToString() { File.WriteAllText("display-called.txt", "bad"); return "display"; }
            }
            """);
        var build = await ProcessRunner.RunAsync(["build", project, "--configuration", "Release", "--nologo", "--verbosity", "quiet"],
            workingDirectory: program.DirectoryPath);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var library = program.FilePath("Hostile.dll");
        File.Copy(program.FilePath("bin/Release/net10.0/Hostile.dll"), library);
        await program.Compile("""
            program PassiveException;
            uses HostileException as Hostile;
            begin
              raise new Hostile();
            end;
            end program;
            """, "--reference", library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        Assert.Equal("exception", stop.Reason);
        var exception = await host.ExceptionInfo(stop.StopId!.Value, deadline.Token);
        Assert.Equal("HostileException", exception.Type);
        Assert.Equal("stored λ", exception.Message.Text);
        Assert.Equal("available", exception.Message.Status);
        Assert.Equal(4, exception.Source!.Line);
        Assert.False(File.Exists(program.FilePath("message-called.txt")));
        Assert.False(File.Exists(program.FilePath("display-called.txt")));
        await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task A_deep_exception_marks_omitted_current_frames_even_when_its_throw_stack_fits()
    {
        using var program = new Debuggee("own-deep-exception");
        await program.Compile("""
            program DeepException;
            uses System as Sys;
            procedure Recurse(Depth: int);
            begin
              if Depth = 0 then raise new Sys.InvalidOperationException('deep'); end if;
              Recurse(Depth - 1);
            end procedure;
            begin
              Recurse(40);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        Assert.Equal("exception", stop.Reason);
        var failure = await host.ExceptionInfo(stop.StopId!.Value, deadline.Token);
        Assert.Equal(41, failure.ThrowStack.Count(frame => frame.Name.EndsWith(".Recurse", StringComparison.Ordinal)));
        Assert.True(failure.ThrowStack.Length < 64);
        Assert.Equal(32, failure.CurrentStack.Length);
        Assert.True(failure.Truncated);
        await host.Stop(deadline.Token);
    }
}
