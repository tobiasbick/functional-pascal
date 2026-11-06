using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Processes.Support;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspRequestLifecycleTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("cancel")]
    [InlineData("change")]
    [InlineData("shutdown")]
    public async Task Accepted_requests_cancel_or_reject_superseded_content_and_shutdown_joins_evaluation(string operation)
    {
        using var directory = sdk.CreateProject();
        using var blocked = new BlockedEvaluation(directory);
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        var path = directory.File("Game/Bridge.fpas"); await client.Open(path, "unit Changed; end unit;");
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1)); using var child = await blocked.ConnectedProcess(client.Id, deadline.Token);
        var pending = client.Request("workspace/symbol", new { query = "" }, stringId: "in-flight");
        await client.Request("unknown");
        if (operation == "cancel") await client.Notify("$/cancelRequest", new { id = "in-flight" });
        else if (operation == "change") await client.FullChange(path, "unit ChangedAgain; end unit;", 2);
        if (operation == "shutdown") await client.Stop();
        Error(await pending, operation == "change" ? -32801 : -32800);
        if (operation != "shutdown") await client.Stop();
        Assert.True(child.HasExited);
    }
}
