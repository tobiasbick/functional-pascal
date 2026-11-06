using System.Text;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class CoreSessionTests
{
    [Fact]
    public async Task Output_retention_is_bounded_and_preserves_complete_utf8_text()
    {
        using var program = new Debuggee("own-core-output-limit");
        await program.Compile("""
            program OutputBound;
            uses System as Sys, System.Console as Console, System.Text.UTF8Encoding as Utf8Encoding;
            begin
              Console.OutputEncoding := new Utf8Encoding(false);
              const Text: string := new Sys.String(#955, 600000);
              Console.Write(Text);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 6)), deadline.Token);
        await host.Initialize(deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        await host.Resume(stop.StopId!.Value, step: false, deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        var output = host.Output();
        Assert.True(output.Truncated);
        Assert.InRange(output.Items.Sum(item => Encoding.UTF8.GetByteCount(item.Text)), 1, OutputCapture.MaximumBytes);
        Assert.All(output.Items, item =>
        {
            Assert.Equal("stdout", item.Stream);
            Assert.True(item.Text.All(character => character == 'λ'), "Captured text contains a split or corrupted UTF-8 character.");
        });
        Assert.False(host.Output(output.NextSequence).Truncated);
        Assert.Empty(host.Output(output.NextSequence).Items);
    }
}
