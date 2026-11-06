using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text;
using System.Text.Json;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspLifecycleTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Lifecycle_capabilities_string_IDs_cancellation_and_explicit_shutdown_use_only_protocol_stdout()
    {
        using var project = sdk.CreateProject(); await using var client = new LspProbe();
        var early = await client.Request("unknown"); Assert.Equal(-32002, early.GetProperty("error").GetProperty("code").GetInt32());
        var initialized = await client.InitializeResponse(new Uri(project.File("Game/Game.fpasproj")).AbsoluteUri, watch: true);
        var capabilities = initialized.GetProperty("result").GetProperty("capabilities");
        Assert.Equal("utf-16", capabilities.GetProperty("positionEncoding").GetString());
        Assert.Equal(2, capabilities.GetProperty("textDocumentSync").GetProperty("change").GetInt32());
        Assert.True(capabilities.GetProperty("textDocumentSync").GetProperty("openClose").GetBoolean());
        Assert.True(capabilities.GetProperty("definitionProvider").GetBoolean());
        Assert.True(capabilities.GetProperty("referencesProvider").GetBoolean());
        Assert.True(capabilities.GetProperty("documentFormattingProvider").GetBoolean());
        Assert.True(capabilities.GetProperty("renameProvider").GetProperty("prepareProvider").GetBoolean());
        await client.Notify("initialized", new { });
        var registration = await client.Notification("client/registerCapability"); await client.Reply(registration);
        await client.Notify("$/cancelRequest", new { id = "unknown" });
        var unknown = await client.Request("missing/method", stringId: "opaque-ID");
        Assert.Equal("opaque-ID", unknown.GetProperty("id").GetString()); Assert.Equal(-32601, unknown.GetProperty("error").GetProperty("code").GetInt32());
        await client.Stop();
    }

    [Fact]
    public async Task Invalid_initialization_is_retryable_and_does_not_prepare_the_project()
    {
        using var project = sdk.CreateProject(); await using var client = new LspProbe();
        var before = Directory.GetFiles(project.Path, "*", SearchOption.AllDirectories).Order().ToArray();
        var relative = await client.InitializeResponse("Game/Game.fpasproj"); Assert.Equal(-32602, relative.GetProperty("error").GetProperty("code").GetInt32());
        var encoding = await client.Request("initialize", new { capabilities = new { general = new { positionEncodings = new[] { "utf-8" } } }, initializationOptions = new { context = project.File("Game/Game.fpasproj") } });
        Assert.Equal(-32602, encoding.GetProperty("error").GetProperty("code").GetInt32());
        var shape = await client.Request("initialize", new { capabilities = new { general = false }, initializationOptions = new { context = project.File("Game/Game.fpasproj") } });
        Assert.Equal(-32602, shape.GetProperty("error").GetProperty("code").GetInt32());
        var kinds = await client.Request("initialize", new { capabilities = new { workspace = new { symbol = new { symbolKind = new { valueSet = new[] { int.MaxValue } } } } }, initializationOptions = new { context = project.File("Game/Game.fpasproj") } });
        Assert.Equal(-32602, kinds.GetProperty("error").GetProperty("code").GetInt32());
        var accepted = await client.InitializeResponse(project.File("Game/Game.fpasproj")); Assert.True(accepted.TryGetProperty("result", out _), accepted.GetRawText());
        var duplicate = await client.InitializeResponse(project.File("Game/Game.fpasproj")); Assert.Equal(-32600, duplicate.GetProperty("error").GetProperty("code").GetInt32());
        Assert.Equal(before, Directory.GetFiles(project.Path, "*", SearchOption.AllDirectories).Order().ToArray());
        await client.Stop();
    }

    [Fact]
    public async Task EOF_and_exit_without_shutdown_release_the_connection()
    {
        await using (var client = new LspProbe()) { client.Eof(); Assert.Equal(0, await client.Exit()); }
        await using (var client = new LspProbe()) { await client.Notify("exit"); Assert.Equal(1, await client.Exit()); }
    }

    [Theory]
    [InlineData("Content-Length: not-a-number\r\n\r\n")]
    [InlineData("Content-Length: 40\r\n\r\n{}")]
    [InlineData("Content-Length: 8388609\r\n\r\n")]
    [InlineData("Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}")]
    [InlineData("Content-Length: 1\r\n\r\n{")]
    public async Task Invalid_or_truncated_framing_terminates_without_nonprotocol_stdout(string input)
    {
        await using var client = new LspProbe(); await client.Raw(Encoding.ASCII.GetBytes(input)); client.Eof();
        Assert.True(await client.Exit() == 1, await client.Error);
    }

    [Fact]
    public async Task Oversized_declared_body_and_header_are_rejected_without_waiting_for_body_or_EOF()
    {
        await using (var client = new LspProbe())
        { await client.Raw("Content-Length: 8388609\r\n\r\n"u8.ToArray()); Assert.True(await client.Exit() == 1, await client.Error); }
        await using (var client = new LspProbe())
        { await client.Raw(Encoding.ASCII.GetBytes("X: " + new string('a', 8192))); Assert.True(await client.Exit() == 1, await client.Error); }
    }

    [Fact]
    public async Task Fragmented_and_coalesced_frames_preserve_notification_order()
    {
        using var project = sdk.CreateProject(); await using var client = new LspProbe(); await client.Initialize(project.File("Game/Game.fpasproj"));
        static byte[] Frame(string text) => Encoding.UTF8.GetBytes("Content-Length: " + Encoding.UTF8.GetByteCount(text) + "\r\n\r\n" + text);
        var initialized = Frame("{\"jsonrpc\":\"2.0\",\"method\":\"initialized\",\"params\":{}}");
        foreach (var value in initialized.Take(17)) await client.Raw([value]);
        await client.Raw(initialized[17..].Concat(Frame("{\"jsonrpc\":\"2.0\",\"method\":\"$/cancelRequest\",\"params\":{\"id\":\"absent\"}}")).ToArray());
        var response = await client.Request("unknown"); Assert.Equal(-32601, response.GetProperty("error").GetProperty("code").GetInt32()); await client.Stop();
    }

    [Fact]
    public async Task Argument_errors_keep_stdout_empty()
    {
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "lsp", "--context", "Example.fpasproj"]);
        Assert.Equal(2, result.ExitCode); Assert.Equal("", result.Output); Assert.Contains("fpas lsp --help", result.Error);
    }
}
