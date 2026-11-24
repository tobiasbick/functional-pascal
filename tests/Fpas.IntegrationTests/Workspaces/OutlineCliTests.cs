// [Documentation](../../../docs/pascal/program-structure/cli.md#outline-one-file)
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class OutlineCliTests
{
    [Fact]
    public async Task Syntax_outline_pages_nested_declarations_and_retains_parser_diagnostics()
    {
        using var directory = new TemporaryDirectory();
        var file = directory.File("Outline.fpas");
        File.WriteAllText(file, """
            unit Outline;
            public type Box = record
              Value: int;
            end record;
            public function Read(Item: Box): int;
            begin
              return Item.Value;
            end function;
            end unit;
            """);
        using var first = await Read("0");
        using var second = await Read("1");
        var firstPage = first.RootElement;
        var secondPage = second.RootElement;
        Assert.Equal("ok", firstPage.GetProperty("status").GetString());
        Assert.Equal(firstPage.GetProperty("snapshotId").GetString(), secondPage.GetProperty("snapshotId").GetString());
        Assert.Equal(1, firstPage.GetProperty("nextOffset").GetInt32());
        var unit = Assert.Single(firstPage.GetProperty("declarations").EnumerateArray());
        var record = Assert.Single(secondPage.GetProperty("declarations").EnumerateArray());
        Assert.Equal("Outline", unit.GetProperty("name").GetString());
        Assert.Equal("Box", record.GetProperty("name").GetString());
        Assert.Equal(unit.GetProperty("nameSpan").GetProperty("offset").GetInt32(), record.GetProperty("owner").GetInt32());

        File.AppendAllText(file, "\n?");
        using var broken = await Read("0");
        Assert.Equal("partial", broken.RootElement.GetProperty("status").GetString());
        Assert.False(broken.RootElement.GetProperty("complete").GetBoolean());
        Assert.NotEmpty(broken.RootElement.GetProperty("diagnostics").EnumerateArray());

        async Task<JsonDocument> Read(string offset)
        {
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", "outline", file, "--limit", "1", "--offset", offset],
                workingDirectory: directory.Path);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            return JsonDocument.Parse(result.Output);
        }
    }
}
