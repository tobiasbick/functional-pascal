using ModelContextProtocol.Protocol;
using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

public sealed class McpContractTests
{
    [Fact]
    public async Task Discovery_describes_parameter_limits_side_effects_and_the_session_workflow()
    {
        await using var probe = await McpProbe.ObservedServer(Host());
        Assert.Equal("fpas-debug", probe.Client.ServerInfo!.Name);
        Assert.Contains("debug_status", probe.Client.ServerInstructions);
        var tools = await probe.Client.ListToolsAsync(cancellationToken: probe.Token);
        foreach (var tool in tools)
        {
            foreach (var parameter in tool.ProtocolTool.InputSchema.GetProperty("properties").EnumerateObject())
                Assert.False(string.IsNullOrWhiteSpace(parameter.Value.GetProperty("description").GetString()));
            ResolveReferences(tool.ProtocolTool.OutputSchema!.Value, tool.ProtocolTool.OutputSchema.Value);
        }
        var status = tools.Single(tool => tool.Name == "debug_status").ProtocolTool;
        var arguments = tools.Single(tool => tool.Name == "debug_launch").ProtocolTool.InputSchema.GetProperty("properties").GetProperty("args");
        Assert.Equal("string", arguments.GetProperty("items").GetProperty("type").GetString());
        Assert.True(status.Annotations!.ReadOnlyHint);
        Assert.False(status.Annotations.OpenWorldHint);
        Assert.Equal(30000, status.InputSchema.GetProperty("properties").GetProperty("waitMilliseconds").GetProperty("maximum").GetInt32());
        var values = tools.Single(tool => tool.Name == "debug_variables").ProtocolTool;
        Assert.False(values.Annotations!.ReadOnlyHint);
        Assert.True(values.Annotations.DestructiveHint);
        Assert.Equal(128, values.InputSchema.GetProperty("properties").GetProperty("count").GetProperty("maximum").GetInt32());
        var step = tools.Single(tool => tool.Name == "debug_step").ProtocolTool;
        Assert.Equal(new[] { "over", "into", "out" }, step.InputSchema.GetProperty("properties").GetProperty("kind").GetProperty("enum").EnumerateArray().Select(item => item.GetString()));
        Assert.Contains(status.OutputSchema!.Value.GetProperty("anyOf").EnumerateArray(), branch => branch.GetProperty("properties").TryGetProperty("error", out _));
    }

    [Fact]
    public async Task Tool_errors_keep_machine_readable_codes_in_text_only_clients()
    {
        await using var probe = await McpProbe.ObservedServer(Host());
        var result = await probe.Client.CallToolAsync("debug_status", new Dictionary<string, object?> { ["sessionId"] = "missing" }, cancellationToken: probe.Token);
        Assert.True(result.IsError);
        using var text = JsonDocument.Parse(Assert.Single(result.Content.OfType<TextContentBlock>()).Text);
        Assert.Equal("invalid_session", text.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.True(JsonElement.DeepEquals(text.RootElement, result.StructuredContent!.Value));
    }

    private static string Host() => Environment.GetEnvironmentVariable("DOTNET_HOST_PATH")
        ?? Path.Combine(Environment.GetEnvironmentVariable("DOTNET_ROOT")!, OperatingSystem.IsWindows() ? "dotnet.exe" : "dotnet");

    private static void ResolveReferences(JsonElement root, JsonElement value)
    {
        if (value.ValueKind == JsonValueKind.Array)
            foreach (var item in value.EnumerateArray()) ResolveReferences(root, item);
        if (value.ValueKind != JsonValueKind.Object) return;
        foreach (var property in value.EnumerateObject())
        {
            if (property.Name == "$ref" && property.Value.GetString() is { } reference && reference.StartsWith("#/", StringComparison.Ordinal))
            {
                var selected = root;
                foreach (var encoded in Uri.UnescapeDataString(reference[2..]).Split('/'))
                {
                    var token = encoded.Replace("~1", "/", StringComparison.Ordinal).Replace("~0", "~", StringComparison.Ordinal);
                    if (selected.ValueKind == JsonValueKind.Array) selected = selected[int.Parse(token, System.Globalization.CultureInfo.InvariantCulture)];
                    else
                    {
                        Assert.True(selected.TryGetProperty(token, out var next), "Unresolved schema reference: " + reference);
                        selected = next;
                    }
                }
            }
            ResolveReferences(root, property.Value);
        }
    }
}
