// [Documentation](../../../docs/pascal/tools/api-documentation.md#generate-checked-fpas-usage)
using Fpas.Compiler.Analysis;
using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Documentation;
using System.Text.Json;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class SourceUsageTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Cli_generates_a_checked_generic_call_with_the_source_unit_import()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Game/Buttons.fpas"), """
            unit Demo.Buttons;
            public type Click of TMsg = record public Message: TMsg; end record;
            public function Button of TMsg(Id: string; Message: TMsg): Click of TMsg;
            begin return new Click of TMsg(Message); end function;
            end unit;
            """);
        var result = await DocumentationFixture.Cli(directory, directory.File("Game/Game.fpasproj"),
            "--usage", "Demo.Buttons.Button", "--type-arg", "int", "--report", "json");
        Assert.True(result.ExitCode == 0, result.Error + result.Output);
        using var report = JsonDocument.Parse(result.Output);
        var entry = Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray());
        Assert.Equal("source", entry.GetProperty("origin").GetString());
        Assert.Equal("Demo.Buttons.Button", entry.GetProperty("qualifiedName").GetString());
        var usage = entry.GetProperty("usage");
        Assert.Equal("template", usage.GetProperty("status").GetString());
        Assert.Equal(new[] { "Id", "Message" }, usage.GetProperty("inputs").EnumerateArray().Select(input => input.GetString()!));
        var source = usage.GetProperty("source").GetString();
        Assert.Contains("uses Demo.Buttons as Buttons;", source);
        Assert.Contains("Use(Id: string; Message: int): Buttons.Click of int;", source);
        Assert.Contains("Buttons.Button of int(", source);
        Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
        Assert.False(File.Exists(directory.File("Game/Game.fpasproj.usage.dll")));
    }

    [Fact]
    public async Task Source_usage_checks_overloads_accessors_and_unavailable_declarations()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Game/Api.fpas"), """
            unit Demo.Tools;
            public function Choose(Value: int): int; begin return Value; end function;
            public function Choose(Value: string): string; begin return Value; end function;
            public function Identity of T(Value: T): T; begin return Value; end function;
            function Hidden(): int; begin return 1; end function;
            end unit;
            """);
        var query = new DocumentationQuery(await DocumentationFixture.Analyze(directory));

        var overloads = query.Search(new("Demo.Tools.Choose", Usage: true)).Entries;
        Assert.Equal(2, overloads.Count);
        Assert.All(overloads, entry => Assert.Equal("template", entry.Usage?.Status));
        Assert.Contains(overloads, entry => entry.Usage?.Source?.Contains("Use(Value: int): int;", StringComparison.Ordinal) == true);
        Assert.Contains(overloads, entry => entry.Usage?.Source?.Contains("Use(Value: string): string;", StringComparison.Ordinal) == true);

        var property = Usage("Demo.Bridge.Point.X");
        Assert.Equal("template", property.Status);
        Assert.Contains("return Receiver.X;", property.Source);
        Assert.Contains("type arguments", Usage("Demo.Tools.Identity").Reason);
        Assert.Equal("unavailable", Usage("Demo.Tools.Hidden").Status);
        Assert.Empty(query.Search(new("Demo.Tools.Missing", Usage: true)).Entries);

        ApiUsage Usage(string name) => Assert.IsType<ApiUsage>(Assert.Single(query.Search(new(name, Usage: true)).Entries).Usage);
    }
}
