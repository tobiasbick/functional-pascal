// [Documentation](../../../docs/pascal/program-structure/cli.md#lint-a-project-or-solution)
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

/// <summary>Exercises lint through the CLI and evaluated FPAS projects.</summary>
/// <param name="sdk">Packaged SDK for the test projects.</param>
public sealed partial class LintTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Checks resolved edge locations and unused CLR versus source imports.</summary>
    [Fact]
    public async Task Reports_resolved_cycle_edges_and_only_unused_CLR_aliases()
    {
        using var directory = new TemporaryDirectory();
        sdk.WriteFeedConfig(directory.Path);
        AddLibrary(directory, "Rules");
        Write(directory, "Rules/A.fpas", """
            unit Demo.A;
            uses Demo.B.Payload as Payload; uses System.Console as Console; uses System as Sys; uses System.Collections.Generic as Collections;
            public function Size(): int;
            begin
              const Values: Collections.List of Payload := new Collections.List of Payload();
              const Value: Sys.Int32 := Values.Count;
              return Value;
            end function;
            end unit;
            """);
        Write(directory, "Rules/B.fpas", """
            unit Demo.B;
            uses Demo as Units; uses System.Text as Text;
            public type Payload = record public Count: int; end record;
            public function Count(): int;
            begin
              return Units.A.Size();
            end function;
            end unit;
            """);
        Write(directory, "Rules/C.fpas", "unit Demo.C; uses Demo.D as Other; end unit;");
        Write(directory, "Rules/D.fpas", "unit Demo.D; uses Demo.C as Other; uses Demo as Namespace; end unit;");

        var result = await Lint(directory, "Rules/Rules.fpasproj", "--report", "json");
        Assert.True(result.ExitCode == 1, result.Output + result.Error);
        Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var report = json.RootElement;
        Assert.Equal(1, report.GetProperty("schemaVersion").GetInt32());
        Assert.Equal("findings", report.GetProperty("status").GetString());
        Assert.False(report.GetProperty("success").GetBoolean());
        var findings = report.GetProperty("findings").EnumerateArray().ToArray();
        var cycle = Assert.Single(findings, finding => Rule(finding) == "module-cycle");
        Assert.Contains("Demo.A, Demo.B", cycle.GetProperty("message").GetString());
        var edges = cycle.GetProperty("related").EnumerateArray().ToArray();
        Assert.Equal(["Demo.A depends on Demo.B.", "Demo.B depends on Demo.A."], edges.Select(edge => edge.GetProperty("message").GetString()));
        Assert.Equal([directory.File("Rules/A.fpas"), directory.File("Rules/B.fpas")], edges.Select(PathAt));
        Assert.Equal([5, 6], edges.Select(edge => edge.GetProperty("location").GetProperty("line").GetInt32()));
        var aliases = findings.Where(finding => Rule(finding) == "unused-clr-alias").ToArray();
        Assert.Equal(2, aliases.Length);
        Assert.Contains("alias 'Console'", aliases[0].GetProperty("message").GetString());
        Assert.Contains("alias 'Text'", aliases[1].GetProperty("message").GetString());
        Assert.Equal(3, findings.Length);
        Assert.False(File.Exists(directory.File("Rules/bin/Debug/net10.0/Rules.dll")));

        var text = await Lint(directory, "Rules/Rules.fpasproj", "--existing-inputs");
        Assert.Equal(1, text.ExitCode);
        Assert.Empty(text.Output);
        Assert.Contains("error module-cycle", text.Error);
        Assert.Contains("Demo.B depends on Demo.A.", text.Error);
    }

    /// <summary>Checks findings and input preservation for linked solution sources.</summary>
    [Fact]
    public async Task Solution_reports_linked_CLR_alias_once_and_preserves_source()
    {
        using var directory = sdk.CreateProject();
        AddLibrary(directory, "First", linked: true);
        AddLibrary(directory, "Second", linked: true);
        var solution = XDocument.Load(directory.File("Game.slnx"));
        solution.Root!.Elements("Project").Remove();
        solution.Root.Add(new XElement("Project", new XAttribute("Path", "First/First.fpasproj")),
            new XElement("Project", new XAttribute("Path", "Second/Second.fpasproj")));
        solution.Save(directory.File("Both.slnx"));
        const string source = "unit Shared; uses System.Text as Text; end unit;";
        Write(directory, "Shared.fpas", source);

        var result = await Lint(directory, "Both.slnx", "--report", "json");
        Assert.True(result.ExitCode == 1, result.Output + result.Error);
        using (var json = JsonDocument.Parse(result.Output))
        {
            var report = json.RootElement;
            Assert.True(report.GetProperty("status").GetString() == "findings", result.Output + result.Error);
            var finding = Assert.Single(report.GetProperty("findings").EnumerateArray());
            Assert.Equal("unused-clr-alias", Rule(finding));
            Assert.Equal(directory.File("Shared.fpas"), PathAt(finding));
        }
        Assert.Equal(source, File.ReadAllText(directory.File("Shared.fpas")));

        Write(directory, "Shared.fpas", "unit Shared; end unit;");
        var clean = await Lint(directory, "Both.slnx", "--existing-inputs", "--report", "json");
        Assert.True(clean.ExitCode == 0, clean.Output + clean.Error);
        using var cleanReport = JsonDocument.Parse(clean.Output);
        Assert.Equal("ok", cleanReport.RootElement.GetProperty("status").GetString());
        Assert.Empty(cleanReport.RootElement.GetProperty("findings").EnumerateArray());
    }

    /// <summary>Checks report shape for syntax/binding and argument failures.</summary>
    [Fact]
    public async Task Invalid_source_and_arguments_return_structured_failures()
    {
        using var directory = new TemporaryDirectory();
        sdk.WriteFeedConfig(directory.Path);
        AddLibrary(directory, "Invalid");
        Write(directory, "Invalid/Rules.fpas", """
            unit Rules;
            uses System as Sys;
            public function MissingValue(): int;
            begin return Missing; end function;
            end unit;
            """);

        var result = await Lint(directory, "Invalid/Invalid.fpasproj", "--report", "json");
        Assert.Equal(1, result.ExitCode);
        Assert.Empty(result.Error);
        using (var json = JsonDocument.Parse(result.Output))
        {
            var report = json.RootElement;
            Assert.Equal("failed", report.GetProperty("status").GetString());
            Assert.Empty(report.GetProperty("findings").EnumerateArray());
            Assert.Contains(report.GetProperty("diagnostics").EnumerateArray(), diagnostic => diagnostic.GetProperty("code").GetString() == "FP3001");
        }

        foreach (var option in new[] { "--timeout", "--held-in-mind", "--max-directory-entries", "--max-lines" })
        {
            var invalid = await Lint(directory, "Invalid/Invalid.fpasproj", "--report", "json", option, "0");
            Assert.Equal(2, invalid.ExitCode);
            using var arguments = JsonDocument.Parse(invalid.Output);
            Assert.Equal("invalid_argument", arguments.RootElement.GetProperty("error").GetProperty("code").GetString());
        }
    }

    /// <summary>Checks declaration cycles and the local group size boundary.</summary>
    [Fact]
    public async Task Tangles_need_multiple_units_or_more_than_seven_local_declarations()
    {
        using var directory = new TemporaryDirectory();
        sdk.WriteFeedConfig(directory.Path);
        AddLibrary(directory, "Rules");
        Write(directory, "Rules/Left.fpas", """
            unit Graph.Left;
            uses Graph.Right as Right;
            public function Start(): int;
            begin return Right.Finish(); end function;
            end unit;
            """);
        Write(directory, "Rules/Right.fpas", """
            unit Graph.Right;
            uses Graph.Left as Left;
            public function Finish(): int;
            begin return Left.Start(); end function;
            end unit;
            """);
        AddCycle("Large", 8);
        AddCycle("Small", 7);

        var findings = await Findings(directory);
        Assert.Single(findings, finding => Rule(finding) == "module-cycle");
        var tangles = findings.Where(finding => Rule(finding) == "tangle").ToArray();
        Assert.Equal(2, tangles.Length);
        var local = Assert.Single(tangles, finding => PathAt(finding) == directory.File("Rules/Large.fpas"));
        Assert.Equal(8, local.GetProperty("related").GetArrayLength());
        var crossUnit = Assert.Single(tangles, finding => PathAt(finding) == directory.File("Rules/Left.fpas"));
        Assert.Equal([directory.File("Rules/Left.fpas"), directory.File("Rules/Right.fpas")],
            crossUnit.GetProperty("related").EnumerateArray().Select(PathAt));
        var relaxed = await Findings(directory, "--existing-inputs", "--held-in-mind", "8");
        var remaining = Assert.Single(relaxed, finding => Rule(finding) == "tangle");
        Assert.Equal(directory.File("Rules/Left.fpas"), PathAt(remaining));

        void AddCycle(string unit, int size)
        {
            var routines = Enumerable.Range(0, size).Select(index =>
                $"function F{index}(): int; begin return F{(index + 1) % size}(); end function;");
            Write(directory, $"Rules/{unit}.fpas", $"unit Graph.{unit};\n{string.Join('\n', routines)}\nend unit;");
        }
    }

    /// <summary>Checks direct use through encapsulated fields while allowing plain reads and data records.</summary>
    [Fact]
    public async Task Field_leaks_report_calls_and_indexing_through_record_state()
    {
        using var directory = new TemporaryDirectory();
        sdk.WriteFeedConfig(directory.Path);
        AddLibrary(directory, "Rules");
        Write(directory, "Rules/Records.fpas", """
            unit Arch.Records;
            uses System.Collections.Generic as Collections;
            public type Store = record
              Seed: int := 1;
              public Items: Collections.List of int := new Collections.List of int();
              public Values: array of int := [0];
              public Caption: string := '';
              public function Count(): int;
              begin return self.Items.Count + self.Seed; end function;
            end record;
            public type Data = record
              public Items: Collections.List of int := new Collections.List of int();
            end record;
            end unit;
            """);
        Write(directory, "Rules/Consumer.fpas", """
            unit Arch.Consumer;
            uses Arch.Records.Store as Store; uses Arch.Records.Data as Data;
            public procedure Inspect(Value: Store; Plain: Data);
            begin
              Value.Items.Add(1);
              Value.Values[0] := 2;
              discard Value.Items.Count;
              Plain.Items.Add(1);
              discard Value.Caption.Trim();
            end procedure;
            end unit;
            """);

        var findings = await Findings(directory);
        Assert.Equal(2, findings.Length);
        Assert.All(findings, finding => Assert.Equal("field-leak", Rule(finding)));
        Assert.Equal([5, 6], findings.Select(finding => finding.GetProperty("location").GetProperty("line").GetInt32()));
        Assert.All(findings, finding =>
        {
            Assert.Equal(directory.File("Rules/Consumer.fpas"), PathAt(finding));
            Assert.Equal(directory.File("Rules/Records.fpas"), PathAt(finding.GetProperty("related")[0]));
        });
    }

    /// <summary>Distinguishes wide unit and record interfaces from a small or deep interface.</summary>
    [Fact]
    public async Task Shallow_modules_need_a_wide_interface_and_little_hidden_implementation()
    {
        using var directory = new TemporaryDirectory();
        sdk.WriteFeedConfig(directory.Path);
        AddLibrary(directory, "Rules");
        Write(directory, "Rules/Wide.fpas", $"unit Wide;\n{Routines("public", 8, "F")}\nend unit;");
        Write(directory, "Rules/Small.fpas", $"unit Small;\n{Routines("public", 7, "F")}\nend unit;");
        Write(directory, "Rules/Deep.fpas", $"unit Deep;\n{Routines("public", 8, "F")}\n{Routines("private", 9, "H")}\nend unit;");
        Write(directory, "Rules/RecordApi.fpas",
            $"unit RecordApi;\npublic type Api = record\n{Routines("public", 8, "F")}\nend record;\nend unit;");
        var grouped = Enumerable.Range(0, 4).Select(index =>
            $"public type Api{index} = record\n{Routines("public", 2, "F")}\nend record;");
        Write(directory, "Rules/Grouped.fpas", $"unit Grouped;\n{string.Join('\n', grouped)}\nend unit;");

        var findings = await Findings(directory);
        Assert.Equal(2, findings.Length);
        Assert.All(findings, finding => Assert.Equal("shallow-module", Rule(finding)));
        Assert.Equal([directory.File("Rules/RecordApi.fpas"), directory.File("Rules/Wide.fpas")], findings.Select(PathAt));

        static string Routines(string access, int count, string prefix) => string.Join('\n', Enumerable.Range(0, count)
            .Select(index => $"{access} function {prefix}{index}(): int; begin return {index}; end function;"));
    }

    /// <summary>Checks upstream layout thresholds on evaluated files and nonempty FPAS directories.</summary>
    [Fact]
    public async Task Source_layout_counts_immediate_entries_and_physical_lines()
    {
        using var directory = new TemporaryDirectory();
        sdk.WriteFeedConfig(directory.Path);
        AddLibrary(directory, "Rules");
        for (var index = 0; index < 15; index++)
        {
            Write(directory, $"Rules/Wide/Item{index}.fpas", $"unit Wide.Item{index}; end unit;");
            Write(directory, $"Rules/Narrow/Item{index}.fpas", $"unit Narrow.Item{index}; end unit;");
        }
        Write(directory, "Rules/Narrow/Notes/readme.txt", "A directory with no FPAS source.");
        Write(directory, "Rules/Wide/Nested/Item.fpas", "unit Wide.Nested.Item; end unit;");
        Write(directory, "Rules/AtLimit.fpas", "unit AtLimit; end unit;\r\n" + string.Concat(Enumerable.Repeat("// line\r\n", 1_999)));
        Write(directory, "Rules/OverLimit.fpas", "unit OverLimit; end unit;\r\n" + string.Concat(Enumerable.Repeat("// line\r\n", 2_000)));

        var findings = await Findings(directory);
        var wide = Assert.Single(findings, finding => Rule(finding) == "wide-directory");
        Assert.Contains("'Wide' has 16", wide.GetProperty("message").GetString());
        Assert.Equal(16, wide.GetProperty("related").GetArrayLength());
        var file = Assert.Single(findings, finding => Rule(finding) == "long-file");
        Assert.Equal(directory.File("Rules/OverLimit.fpas"), PathAt(file));
        Assert.Contains("2001 lines", file.GetProperty("message").GetString());
        Assert.Equal(2, findings.Length);

        var relaxed = await Lint(directory, "Rules/Rules.fpasproj", "--existing-inputs", "--max-directory-entries", "16", "--max-lines", "2001");
        Assert.True(relaxed.ExitCode == 0, relaxed.Output + relaxed.Error);
    }

    /// <summary>Checks distinct exports in a real FPAS project reference and points to their consumer.</summary>
    [Fact]
    public async Task A_consumer_can_learn_too_many_exports_from_a_deep_dependency()
    {
        using var directory = new TemporaryDirectory();
        sdk.WriteFeedConfig(directory.Path);
        AddLibrary(directory, "Provider");
        AddLibrary(directory, "Rules", dependency: "Provider");
        Write(directory, "Provider/Api.fpas", ApiSource(8));
        Write(directory, "Rules/Broad.fpas", ConsumerSource("Broad", 8));
        Write(directory, "Rules/Narrow.fpas", ConsumerSource("Narrow", 7));

        var finding = Assert.Single(await Findings(directory));
        Assert.Equal("shallow-module", Rule(finding));
        Assert.Contains("8 distinct exports", finding.GetProperty("message").GetString());
        Assert.Equal(directory.File("Rules/Broad.fpas"), PathAt(finding));
        var related = finding.GetProperty("related");
        Assert.Equal(9, related.GetArrayLength());
        Assert.Equal(directory.File("Provider/Api.fpas"), PathAt(related[0]));
        Assert.Contains("Uses 'Api.F0'", related[1].GetProperty("message").GetString());

        var relaxed = await Lint(directory, "Rules/Rules.fpasproj", "--existing-inputs", "--held-in-mind", "8");
        Assert.True(relaxed.ExitCode == 0, relaxed.Output + relaxed.Error);
    }

    /// <summary>Checks evaluated test metadata and production ownership of a linked file.</summary>
    [Fact]
    public async Task Test_only_sources_are_exempt_from_file_length_and_consumer_width()
    {
        using var directory = sdk.CreateProject();
        AddLibrary(directory, "Provider");
        AddLibrary(directory, "Production", linked: true);
        AddLibrary(directory, "ASandbox", linked: true, dependency: "Provider");
        Write(directory, "Directory.Build.props", """
            <Project>
              <PropertyGroup Condition="'$(MSBuildProjectName)' == 'ASandbox'">
                <IsTestProject>true</IsTestProject>
              </PropertyGroup>
            </Project>
            """);
        Write(directory, "Provider/Api.fpas", ApiSource(3));
        Write(directory, "ASandbox/Probe.fpas", ConsumerSource("Probe", 3));
        Write(directory, "Shared.fpas", "unit Shared; end unit;\n" + string.Concat(Enumerable.Repeat("// line\n", 4)));
        var solution = XDocument.Load(directory.File("Game.slnx"));
        solution.Root!.Elements("Project").Remove();
        solution.Root.Add(new XElement("Project", new XAttribute("Path", "ASandbox/ASandbox.fpasproj")),
            new XElement("Project", new XAttribute("Path", "Production/Production.fpasproj")));
        solution.Save(directory.File("Both.slnx"));

        var result = await Lint(directory, "Both.slnx", "--held-in-mind", "2", "--max-lines", "4", "--report", "json");
        Assert.True(result.ExitCode == 1, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.True(json.RootElement.GetProperty("status").GetString() == "findings", result.Output + result.Error);
        var finding = Assert.Single(json.RootElement.GetProperty("findings").EnumerateArray());
        Assert.Equal("long-file", Rule(finding));
        Assert.Equal(directory.File("Shared.fpas"), PathAt(finding));
        Assert.Equal(directory.File("Production/Production.fpasproj"), finding.GetProperty("project").GetString());
    }

    private static string ApiSource(int exposed) => "unit Api; " +
        string.Join(' ', Enumerable.Range(0, exposed).Select(index => $"public function F{index}(): int; begin return {index}; end function;")) + " " +
        string.Join(' ', Enumerable.Range(0, exposed + 1).Select(index => $"function H{index}(): int; begin return {index}; end function;")) +
        " end unit;";

    private static string ConsumerSource(string unit, int exports) => $"""
        unit {unit};
        uses Api as Provider;
        public function Read(): int;
        begin
          return {string.Join(" + ", Enumerable.Range(0, exports).Select(index => $"Provider.F{index}()"))} + Provider.F0();
        end function;
        end unit;
        """;

    private static async Task<JsonElement[]> Findings(TemporaryDirectory directory, params string[] args)
    {
        var result = await Lint(directory, ["Rules/Rules.fpasproj", "--report", "json", .. args]);
        using var json = JsonDocument.Parse(result.Output);
        Assert.True(result.ExitCode == 1 && json.RootElement.GetProperty("status").GetString() == "findings",
            result.Output + result.Error);
        return json.RootElement.GetProperty("findings").EnumerateArray().Select(finding => finding.Clone()).ToArray();
    }

    private void AddLibrary(TemporaryDirectory directory, string name, bool linked = false, string? dependency = null)
    {
        var include = linked ? """<ItemGroup><Compile Include="../Shared.fpas" /></ItemGroup>""" : "";
        var reference = dependency is null ? "" : $"""<ItemGroup><ProjectReference Include="../{dependency}/{dependency}.fpasproj" /></ItemGroup>""";
        Write(directory, $"{name}/{name}.fpasproj", $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Library</OutputType></PropertyGroup>
              {include}
              {reference}
            </Project>
            """);
    }

    private TemporaryDirectory CreateRulesProject(string? dependency = null)
    {
        var directory = new TemporaryDirectory();
        sdk.WriteFeedConfig(directory.Path);
        AddLibrary(directory, "Rules", dependency: dependency);
        if (dependency is not null) AddLibrary(directory, dependency);
        return directory;
    }

    private static void Write(TemporaryDirectory directory, string name, string source)
    {
        var path = directory.File(name);
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        File.WriteAllText(path, source);
    }

    private static string? Rule(JsonElement finding) => finding.GetProperty("rule").GetString();
    private static string? PathAt(JsonElement entry) => entry.GetProperty("location").GetProperty("path").GetString();

    private static Task<ProcessResult> Lint(TemporaryDirectory directory, params string[] args)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "lint", .. args],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(3));
}
