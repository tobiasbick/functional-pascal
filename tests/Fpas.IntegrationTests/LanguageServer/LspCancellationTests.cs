using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Processes.Support;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspCancellationTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("shutdown")]
    [InlineData("eof")]
    [InlineData("change")]
    public async Task EOF_shutdown_or_superseding_edit_joins_blocked_owned_evaluation(string operation)
    {
        using var directory = sdk.CreateProject();
        using var blocked = new BlockedEvaluation(directory);
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        using var child = await blocked.ConnectedProcess(client.Id, deadline.Token);
        if (operation == "change")
        {
            await client.Open(directory.File("Game/Bridge.fpas"), "unit Changed; end unit;");
            // The response is a reader-order barrier after the document mutation.
            await client.Request("unknown");
            while (!child.HasExited) await Task.Delay(10, deadline.Token);
        }
        if (operation == "eof") { client.Eof(); Assert.True(await client.Exit() == 0, await client.Error); } else await client.Stop();
        Assert.True(child.HasExited);
    }
}
