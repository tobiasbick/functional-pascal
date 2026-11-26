// [Documentation](../../../docs/pascal/tools/api-documentation.md)
using Fpas.Compiler.Analysis;
using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Documentation;
using System.Text.Json;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class MemberAccessUsageTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Cli_checks_a_discovered_generic_indexer_assignment_without_running_the_setter()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var project = directory.File("Game/Game.fpasproj");
        var found = await DocumentationFixture.Cli(directory, project, "--name", "Lookup.Api.Item", "--report", "json");
        Assert.True(found.ExitCode == 0, found.Error + found.Output);
        using var declarations = JsonDocument.Parse(found.Output);
        var id = Assert.Single(declarations.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray()).GetProperty("documentationId").GetString()!;
        var response = await DocumentationFixture.Cli(directory, project, "--usage", id, "--access", "write", "--type-arg", "System.Text.StringBuilder",
            "--no-restore", "--report", "json");
        Assert.True(response.ExitCode == 0, response.Error + response.Output);
        using var report = JsonDocument.Parse(response.Output);
        var selected = Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray());
        Assert.Equal(id, selected.GetProperty("documentationId").GetString());
        var usage = selected.GetProperty("usage");
        Assert.Equal("template", usage.GetProperty("status").GetString());
        Assert.Equal(["Receiver", "Index", "Value"], usage.GetProperty("inputs").EnumerateArray().Select(value => value.GetString()));
        Assert.Contains("Receiver[Index] := Value;", usage.GetProperty("source").GetString());
        Assert.False(File.Exists(directory.File("initializer-executed")));
    }

    [Fact]
    public async Task A_property_exported_by_an_Fpas_record_is_verified_as_storage()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var build = await sdk.Run(directory, "build", directory.File("Game/Game.fpasproj"), "-v:q");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        Directory.CreateDirectory(directory.File("Consumer"));
        var project = directory.File("Consumer/Consumer.fpasproj");
        File.WriteAllText(project, $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><Reference Include="Game"><HintPath>../Game/bin/Debug/net10.0/Game.dll</HintPath></Reference></ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File("Consumer/Main.fpas"), "unit Consumer; end unit;");
        var response = await DocumentationFixture.Cli(directory, project, "--usage", "P:Demo.Bridge.Point.X", "--assembly", "Game", "--report", "json");
        Assert.True(response.ExitCode == 0, response.Error + response.Output);
        using var report = JsonDocument.Parse(response.Output);
        var usage = Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray()).GetProperty("usage");
        Assert.Equal("template", usage.GetProperty("status").GetString());
        Assert.Contains("return Receiver.X;", usage.GetProperty("source").GetString());
    }

    [Fact]
    public async Task Reads_and_writes_require_the_exact_public_field_or_accessor()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Shared/Storage.cs"), """
            #nullable enable
            namespace Storage;
            public struct Meter
            {
                public int Count;
                public readonly string? Name;
                public static int Total;
                public const int Limit = 10;
                public int Level { get; set; }
            }
            public class Shelf
            {
                public static System.IO.Stream? Current { get; set; }
                public string? PrivateSetter { get; private set; }
                public int WriteOnly { set => throw new System.Exception(); }
                public int Initial { get; init; }
                public string this[int row, int column] { get => throw new System.Exception(); set => throw new System.Exception(); }
                public int this[string row, string column] { get => throw new System.Exception(); }
                public int this[in System.DateTime key] => throw new System.Exception();
            }
            public class Base { public int Data { get; set; } }
            public class Derived : Base { public new int Data; }
            """);
        var query = new DocumentationQuery(await DocumentationFixture.Analyze(directory));
        ApiUsage Select(string id, string? access = null)
        {
            var entry = Assert.Single(query.Search(new(id, Assembly: "Shared", Usage: true, Access: access)).Entries);
            Assert.Equal(id, entry.DocumentationId);
            return Assert.IsType<ApiUsage>(entry.Usage);
        }
        void Unavailable(string id, string access, string reason)
        {
            var usage = Select(id, access);
            Assert.Equal("unavailable", usage.Status);
            Assert.Null(usage.Source);
            Assert.Contains(reason, usage.Reason);
        }

        Assert.Contains("return Receiver.Count;", Select("F:Storage.Meter.Count").Source);
        var fieldWrite = Select("F:Storage.Meter.Count", "write");
        Assert.Contains("Use(ref Receiver: Storage.Meter; Value: int);", fieldWrite.Source);
        Assert.Contains("Receiver.Count := Value;", fieldWrite.Source);
        Assert.Contains("return Receiver.Name;", Select("F:Storage.Meter.Name").Source);
        Assert.Contains("Storage.Meter.Total := Value;", Select("F:Storage.Meter.Total", "write").Source);
        Assert.Equal("example", Select("F:Storage.Meter.Limit").Status);
        Unavailable("F:Storage.Meter.Name", "write", "read-only");
        Unavailable("F:Storage.Meter.Limit", "write", "constant");
        Assert.Contains("Receiver.Level := Value;", Select("P:Storage.Meter.Level", "write").Source);
        var borrowed = Select("P:Storage.Shelf.Current", "read");
        Assert.Contains("function Use(): IO.Stream?;", borrowed.Source);
        Assert.Contains("return Storage.Shelf.Current;", borrowed.Source);
        Assert.Contains("Storage.Shelf.Current := Value;", Select("P:Storage.Shelf.Current", "write").Source);
        Assert.Contains("Receiver.WriteOnly := Value;", Select("P:Storage.Shelf.WriteOnly", "write").Source);
        Unavailable("P:Storage.Shelf.WriteOnly", "read", "getter");
        Unavailable("P:Storage.Shelf.PrivateSetter", "write", "setter");
        Unavailable("P:Storage.Shelf.Initial", "write", "init-only");
        const string byNumber = "P:Storage.Shelf.Item(System.Int32,System.Int32)";
        const string byName = "P:Storage.Shelf.Item(System.String,System.String)";
        Assert.Contains("Row: int; Column: int): string;", Select(byNumber).Source);
        Assert.Contains("Row: string; Column: string): int;", Select(byName).Source);
        Assert.Contains("Receiver[Row, Column] := Value;", Select(byNumber, "write").Source);
        Unavailable(byName, "write", "setter");
        Unavailable("P:Storage.Shelf.Item(System.DateTime@)", "read", "by-value");
        // The FPAS binder resolves this spelling to the inherited property.
        // A successfully typed expression alone would misidentify the hidden field.
        Unavailable("F:Storage.Derived.Data", "read", "did not bind");
        Unavailable("F:Storage.Derived.Data", "write", "did not bind");

        File.WriteAllText(directory.File("Game/Index.fpas"), """
            unit Probe;
            uses Storage as S; uses System as Sys;
            public function Read(Shelf: S.Shelf; Key: Sys.DateTime): int;
            begin return Shelf[Key]; end function;
            end unit;
            """);
        var build = await sdk.Run(directory, "build", directory.File("Game/Game.fpasproj"), "--no-restore", "-v:q");
        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains("FP3015", build.Output + build.Error);
    }
}
