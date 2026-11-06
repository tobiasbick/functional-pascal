using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Workspaces.Live;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class LiveCancellationTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Caller_cancellation_joins_the_owned_evaluation_process_before_returning()
    {
        using var directory = sdk.CreateProject();
        using var blocked = new BlockedEvaluation(directory);
        await using var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        using var cancellation = new CancellationTokenSource(); using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        var work = workspace.AnalyzeAsync(cancellation.Token);
        try
        {
            using var child = await blocked.ConnectedProcess(Environment.ProcessId, deadline.Token);
            cancellation.Cancel(); await Assert.ThrowsAnyAsync<OperationCanceledException>(() => work);
            Assert.True(child.HasExited); Assert.Null(workspace.Current);
        }
        finally
        {
            cancellation.Cancel(); try { await work; } catch (OperationCanceledException) { }
        }
    }
}
