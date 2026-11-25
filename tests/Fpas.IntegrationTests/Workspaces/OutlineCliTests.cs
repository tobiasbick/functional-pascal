// [Documentation](../../../docs/pascal/program-structure/cli.md#outline-one-file)
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Workspaces;

/// <summary>Checks declarations and diagnostics returned by the syntax outline CLI.</summary>
public sealed class OutlineCliTests
{
    /// <summary>Checks nested ownership, stable paging, and recovery after a parser error.</summary>
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

    /// <summary>Checks that a lexical error preserves declarations from the valid prefix.</summary>
    /// <param name="suffix">The invalid token appended to the unit.</param>
    /// <param name="code">The lexer diagnostic expected in the outline.</param>
    [Theory]
    [InlineData("@", "FP1001")]
    [InlineData("'unterminated", "FP1002")]
    public async Task Outline_retains_declarations_before_a_lexical_error(string suffix, string code)
    {
        using var directory = new TemporaryDirectory();
        var file = directory.File("Outline.fpas");
        File.WriteAllText(file, "unit Outline; public const Before: int := 1; end unit;\n" + suffix);
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", "outline", file],
            workingDirectory: directory.Path);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var report = JsonDocument.Parse(result.Output);
        var outline = report.RootElement;
        Assert.Equal("partial", outline.GetProperty("status").GetString());
        Assert.Equal(["Outline", "Before"], outline.GetProperty("declarations").EnumerateArray()
            .Select(item => item.GetProperty("name").GetString()));
        var diagnostic = Assert.Single(outline.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal(code, diagnostic.GetProperty("code").GetString());
    }

    /// <summary>Checks that a later parser exception preserves earlier recovered diagnostics.</summary>
    [Fact]
    public async Task Outline_retains_recovered_diagnostics_before_a_later_parser_exception()
    {
        using var directory = new TemporaryDirectory();
        var file = directory.File("Outline.fpas");
        File.WriteAllText(file, """
            unit Outline;
            public procedure First();
            begin
            end procedure
            public procedure Second();
            begin
            end procedure;
            end unit;
            ?
            """);
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", "outline", file],
            workingDirectory: directory.Path);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var report = JsonDocument.Parse(result.Output);
        var outline = report.RootElement;
        Assert.Equal("partial", outline.GetProperty("status").GetString());
        Assert.Equal([5, 9], outline.GetProperty("diagnostics").EnumerateArray()
            .Select(item => item.GetProperty("location").GetProperty("line").GetInt32()));
    }
}
