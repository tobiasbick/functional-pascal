// [Documentation](../../../docs/architecture/workspaces.md)
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Build.Support;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;
using Fpas.IntegrationTests.Workspaces.Support;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class CheckTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Check_returns_one_JSON_document_and_no_root_outputs()
    {
        using var project = sdk.CreateProject();
        var result = await Check(project, "Game/Game.fpasproj", "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var body = json.RootElement;
        Assert.True(body.GetProperty("success").GetBoolean());
        Assert.True(body.GetProperty("coverage").GetProperty("complete").GetBoolean());
        Assert.Equal("checked", body.GetProperty("projects")[0].GetProperty("state").GetString());
        Assert.NotEmpty(body.GetProperty("snapshotId").GetString()!);
        Assert.True(body.GetProperty("preparation").GetProperty("restorePerformed").GetBoolean());
        Assert.NotEmpty(body.GetProperty("preparation").GetProperty("dependencyOutputs").EnumerateArray());
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
        Assert.False(File.Exists(project.File("Game/obj/Debug/net10.0/fpas.compile.json")));
    }

    [Fact]
    public async Task A_failed_check_preserves_existing_root_outputs_and_successful_build_fingerprint()
    {
        using var project = sdk.CreateProject();
        var projectPath = project.File("Game/Game.fpasproj"); var projectXml = XDocument.Load(projectPath);
        projectXml.Root!.Descendants("OutputType").Single().Value = "Exe"; projectXml.Save(projectPath);
        File.WriteAllText(project.File("Game/Bridge.fpas"), "program Sample; begin end; end program;");
        var built = await sdk.Run(project, "build", "Game/Game.fpasproj", "-p:GenerateDocumentationFile=true", "-m:1", "--disable-build-servers");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var outputs = Directory.GetFiles(project.File("Game/bin"), "*", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(project.File("Game/obj"), "fpas.compile.json*", SearchOption.AllDirectories))
            .ToDictionary(path => path, Hash);
        Assert.Contains(outputs.Keys, path => path.EndsWith("Game.pdb", StringComparison.Ordinal));
        Assert.Contains(outputs.Keys, path => path.EndsWith("Game.xml", StringComparison.Ordinal));
        Assert.Contains(outputs.Keys, path => path.EndsWith("Game.runtimeconfig.json", StringComparison.Ordinal));
        File.WriteAllText(project.File("Game/Bridge.fpas"), "program Sample; begin const Value: int := Missing; end; end program;");
        var result = await Check(project, "Game/Game.fpasproj", "--no-restore", "--report", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid", json.RootElement.GetProperty("projects")[0].GetProperty("state").GetString());
        Assert.Contains(json.RootElement.GetProperty("diagnostics").EnumerateArray(), diagnostic => diagnostic.GetProperty("code").GetString() == "FP3001");
        foreach (var (path, hash) in outputs) Assert.Equal(hash, Hash(path));
    }

    [Fact]
    public async Task Check_reports_independent_statement_errors_in_one_routine()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            function Read(Value: int): int;
            begin return Value; end function;
            public function Answer(): int;
            begin
              const First: int := Read('first');
              discard First.ToString();
              const Second: int := Read('second');
              return Second;
            end function;
            end unit;
            """);
        var result = await Check(project, "Game/Game.fpasproj", "--report", "json");
        Assert.Equal(1, result.ExitCode);
        Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var diagnostics = json.RootElement.GetProperty("diagnostics").EnumerateArray().ToArray();
        Assert.Equal(2, diagnostics.Length);
        Assert.All(diagnostics, item => Assert.Equal("FP3003", item.GetProperty("code").GetString()));
        Assert.Equal(new[] { 6, 8 }, diagnostics.Select(item => item.GetProperty("location").GetProperty("line").GetInt32()));
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
    }

    [Fact]
    public async Task Check_collects_independent_diagnostics_and_marks_failed_dependents_blocked()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Demo.Bridge; public function Answer(): int; begin return Missing; end function; end unit;");
        var solutionPath = project.File("Game.slnx"); var solution = XDocument.Load(solutionPath);
        foreach (var name in new[] { "Independent", "Dependent" })
        {
            Directory.CreateDirectory(project.File(name));
            File.WriteAllText(project.File(name + "/Main.fpas"), "program " + name + "; begin end; end program;");
            File.WriteAllText(project.File(name + "/" + name + ".fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup>" +
                (name == "Dependent" ? "<ItemGroup><ProjectReference Include=\"../Game/Game.fpasproj\" /></ItemGroup>" : "") + "</Project>");
            solution.Root!.Add(new XElement("Project", new XAttribute("Path", name + "/" + name + ".fpasproj")));
        }
        solution.Save(solutionPath);
        var result = await Check(project, "Game.slnx", "--report", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var projects = json.RootElement.GetProperty("projects").EnumerateArray().ToArray();
        Assert.Equal(3, projects.Length);
        Assert.Contains(projects, item => item.GetProperty("project").GetString() == project.File("Independent/Independent.fpasproj") && item.GetProperty("state").GetString() == "checked");
        Assert.Contains(projects, item => item.GetProperty("project").GetString() == project.File("Dependent/Dependent.fpasproj") && item.GetProperty("state").GetString() == "blocked");
        Assert.False(json.RootElement.GetProperty("coverage").GetProperty("complete").GetBoolean());
    }

    [Fact]
    public async Task Documentation_warning_policy_is_used_and_default_diagnostics_go_to_stderr()
    {
        using var project = sdk.CreateProject();
        var path = project.File("Game/Game.fpasproj"); var xml = XDocument.Load(path);
        xml.Root!.Add(new XElement("PropertyGroup", new XElement("GenerateDocumentationFile", "true"), new XElement("TreatWarningsAsErrors", "true")));
        xml.Save(path);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            // Read a value.
            // ## Parameters
            // - `Wrong`: Not a parameter.
            public function Read(Value: int): int;
            begin return Value; end function;
            end unit;
            """);
        var result = await Check(project, "Game/Game.fpasproj");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Output);
        Assert.Contains("error FP6001:", result.Error);
        Assert.Contains("Checked 1/1", result.Error);
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.xml")));
    }

    [Theory]
    [InlineData("--unknown", "bad")]
    [InlineData("--timeout", "0")]
    [InlineData("--timeout", "3601")]
    [InlineData("--timeout", "abc")]
    public async Task Invalid_check_arguments_still_return_structured_JSON(string option, string value)
    {
        using var project = sdk.CreateProject();
        var result = await Check(project, "Game/Game.fpasproj", option, value, "--report", "json");
        Assert.Equal(2, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Fact]
    public async Task Timeout_is_a_single_structured_failure()
    {
        using var project = sdk.CreateProject();
        using var waiting = await WaitingBuild.Prepare(project.File("Game/Game.fpasproj"));
        var result = await Check(project, "Game/Game.fpasproj", "--timeout", "1", "--report", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("timeout", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Fact]
    public async Task Timeout_joins_a_started_preparation_child_before_returning()
    {
        using var project = sdk.CreateProject();
        using var waiting = await WaitingBuild.Prepare(project.File("Game/Game.fpasproj"));
        var checking = ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check",
            "Game/Game.fpasproj", "--timeout", "30", "--report", "json"],
            workingDirectory: project.Path, timeLimit: TimeSpan.FromSeconds(60));
        try
        {
            while (!File.Exists(waiting.Ready))
            {
                if (checking.IsCompleted)
                {
                    var early = await checking;
                    Assert.Fail("Check ended before its preparation child became ready: " + early.Output + early.Error);
                }
                await Task.Delay(20);
            }
            int pid = int.Parse(await File.ReadAllTextAsync(waiting.Ready), System.Globalization.CultureInfo.InvariantCulture);
            using var child = ProcessExit.Track(pid);
            var result = await checking;
            Assert.Equal(1, result.ExitCode);
            Assert.Empty(result.Error);
            using var json = JsonDocument.Parse(result.Output);
            Assert.Equal("timeout", json.RootElement.GetProperty("error").GetProperty("code").GetString());
            // Observe exit before fixture disposal can remove the helper or clean up its owner.
            ProcessExit.AssertGone(child);
        }
        finally { await checking; }
    }

    [Fact]
    public async Task Diagnostic_pages_filter_output_without_changing_check_status_or_coverage()
    {
        using var directory = sdk.CreateProject();
        const string context = "Game/Game.fpasproj";
        const string file = "Game/Bridge.fpas";
        File.WriteAllText(directory.File(file), """
            unit Demo.Bridge;
            public function First(): int; begin return MissingOne; end function;
            public function Second(): int; begin return MissingTwo; end function;
            end unit;
            """);
        File.WriteAllText(directory.File("Game/Other.fpas"),
            "unit Demo.Other; public function Third(): int; begin return MissingThree; end function; end unit;");

        using var first = await Page("0");
        using var second = await Page("1", "--existing-inputs");
        var start = first.RootElement;
        var next = second.RootElement;
        var page = start.GetProperty("diagnosticPage");
        Assert.Equal("failed", start.GetProperty("status").GetString());
        Assert.Equal(start.GetProperty("snapshotId").GetString(), next.GetProperty("snapshotId").GetString());
        Assert.Equal(start.GetProperty("coverage").GetProperty("complete").GetBoolean(), next.GetProperty("coverage").GetProperty("complete").GetBoolean());
        Assert.True(page.GetProperty("totalCount").GetInt32() > page.GetProperty("matchedCount").GetInt32());
        Assert.Equal(1, page.GetProperty("nextOffset").GetInt32());
        var firstDiagnostic = Assert.Single(start.GetProperty("diagnostics").EnumerateArray());
        var secondDiagnostic = Assert.Single(next.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal(directory.File(file), firstDiagnostic.GetProperty("location").GetProperty("path").GetString());
        Assert.Equal(directory.File(file), secondDiagnostic.GetProperty("location").GetProperty("path").GetString());
        Assert.NotEqual(firstDiagnostic.GetProperty("location").GetProperty("offset").GetInt32(),
            secondDiagnostic.GetProperty("location").GetProperty("offset").GetInt32());
        var project = Assert.Single(start.GetProperty("projects").EnumerateArray());
        Assert.Equal(page.GetProperty("totalCount").GetInt32(), project.GetProperty("diagnosticCount").GetInt32());
        Assert.False(project.TryGetProperty("diagnostics", out _));

        async Task<JsonDocument> Page(string offset, params string[] extra)
        {
            var result = await Check(directory, [context, "--report", "json", "--diagnostic-file", file, "--diagnostic-project", context,
                "--diagnostic-limit", "1", "--diagnostic-offset", offset, .. extra]);
            Assert.Equal(1, result.ExitCode);
            return JsonDocument.Parse(result.Output);
        }
    }

    [Fact]
    public async Task Overlay_checks_draft_FPAS_dependencies_together_without_publishing_sources_or_assemblies()
    {
        using var directory = sdk.CreateProject();
        const string producer = "Game/Bridge.fpas";
        const string consumer = "Consumer/Client.fpas";
        const string context = "Consumer/Consumer.fpasproj";
        const string originalProducer = "unit Demo.Bridge; public function Value(): int; begin return 1; end function; end unit;";
        const string originalConsumer = "unit Client; uses Demo.Bridge as B; public function Read(): int; begin return B.Value(); end function; end unit;";
        const string draftProducer = "unit Demo.Bridge; public function Value(Text: string): string; begin return Text; end function; end unit;";
        const string draftConsumer = "unit Client; uses Demo.Bridge as B; public function Read(): string; begin return B.Value('draft'); end function; end unit;";
        Directory.CreateDirectory(directory.File("Consumer"));
        File.WriteAllText(directory.File(producer), originalProducer);
        File.WriteAllText(directory.File(consumer), originalConsumer);
        File.WriteAllText(directory.File(context), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        var restore = await sdk.Run(directory, "restore", context);
        Assert.True(restore.ExitCode == 0, restore.Output + restore.Error);
        var build = await sdk.Run(directory, "build", "Shared/Shared.csproj", "--no-restore");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);

        using var valid = await Draft(0, (producer, draftProducer), (consumer, draftConsumer));
        Assert.Equal("ok", valid.RootElement.GetProperty("status").GetString());
        Assert.Equal(new[] { directory.File(consumer), directory.File(producer) }.Order(StringComparer.Ordinal),
            valid.RootElement.GetProperty("overlayFiles").EnumerateArray().Select(item => item.GetString()));
        var preparation = valid.RootElement.GetProperty("preparation");
        Assert.False(preparation.GetProperty("restorePerformed").GetBoolean());
        Assert.False(preparation.GetProperty("dependencyBuildsAllowed").GetBoolean());

        var snapshot = valid.RootElement.GetProperty("snapshotId").GetString()!;
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var config = System.Text.Json.Nodes.JsonNode.Parse(File.ReadAllText(Path.ChangeExtension(cli, ".runtimeconfig.json")))!;
        var runtime = config["runtimeOptions"]!;
        runtime["configProperties"] ??= new System.Text.Json.Nodes.JsonObject();
        runtime["configProperties"]!["System.Text.Json.JsonSerializer.IsReflectionEnabledByDefault"] = false;
        var configPath = directory.File("no-reflection.runtimeconfig.json");
        File.WriteAllText(configPath, config.ToJsonString());

        using var analyzed = await QueryDraft(0);
        Assert.Equal(snapshot, analyzed.RootElement.GetProperty("snapshotId").GetString());
        Assert.Equal(2, analyzed.RootElement.GetProperty("overlayFiles").GetArrayLength());
        var answers = analyzed.RootElement.GetProperty("results");
        var references = answers[1].GetProperty("result").GetProperty("refs").GetProperty("references");
        Assert.Equal(directory.File(consumer), Assert.Single(references.EnumerateArray()).GetProperty("span").GetProperty("path").GetString());
        var declaration = answers[2].GetProperty("result").GetProperty("inspect").GetProperty("source").GetProperty("declaration").GetProperty("text").GetString();
        Assert.Contains("Value(Text: string): string", declaration);

        var planned = await RunRefactor("draft", context, "--overlay", "draft.json", "--expected-snapshot", snapshot,
            "--plan", "draft-plan.json", "--report", "json");
        Assert.True(planned.ExitCode == 0, planned.Output + planned.Error);
        using var plan = JsonDocument.Parse(planned.Output);
        Assert.Equal(snapshot, plan.RootElement.GetProperty("plan").GetProperty("draftSnapshotId").GetString());
        Assert.Equal(2, plan.RootElement.GetProperty("plan").GetProperty("files").GetArrayLength());

        using var incompatible = await Draft(1, (producer, draftProducer));
        Assert.Contains(incompatible.RootElement.GetProperty("diagnostics").EnumerateArray(), diagnostic =>
            diagnostic.GetProperty("location").GetProperty("path").GetString() == directory.File(consumer));
        Assert.NotEqual(valid.RootElement.GetProperty("snapshotId").GetString(), incompatible.RootElement.GetProperty("snapshotId").GetString());

        using var stale = await QueryDraft(1);
        var staleError = stale.RootElement.GetProperty("error");
        Assert.Equal("inputs_changed", staleError.GetProperty("code").GetString());
        Assert.Equal(snapshot, staleError.GetProperty("expectedSnapshotId").GetString());
        Assert.Equal(incompatible.RootElement.GetProperty("snapshotId").GetString(), staleError.GetProperty("actualSnapshotId").GetString());
        Assert.Empty(stale.RootElement.GetProperty("results").EnumerateArray());

        using var unowned = await Draft(2, ("Other.fpas", draftConsumer));
        Assert.Equal("invalid_argument", unowned.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal(originalProducer, File.ReadAllText(directory.File(producer)));
        Assert.Equal(originalConsumer, File.ReadAllText(directory.File(consumer)));
        Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
        Assert.False(File.Exists(directory.File("Consumer/bin/Debug/net10.0/Consumer.dll")));

        File.Delete(directory.File("draft.json"));
        File.WriteAllText(directory.File(consumer), originalConsumer + "\n");
        var outdated = await RunRefactor("apply", context, "--plan", "draft-plan.json", "--report", "json");
        Assert.Equal(1, outdated.ExitCode);
        using (var json = JsonDocument.Parse(outdated.Output))
            Assert.Equal("stale_plan", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal(originalProducer, File.ReadAllText(directory.File(producer)));
        Assert.Equal(originalConsumer + "\n", File.ReadAllText(directory.File(consumer)));
        File.WriteAllText(directory.File(consumer), originalConsumer);
        var applied = await RunRefactor("apply", context, "--plan", "draft-plan.json", "--report", "json");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        using (var json = JsonDocument.Parse(applied.Output))
        {
            Assert.True(json.RootElement.GetProperty("rechecked").GetBoolean());
            Assert.All(json.RootElement.GetProperty("files").EnumerateArray(), item => Assert.True(item.GetProperty("written").GetBoolean()));
        }
        Assert.Equal(draftProducer, File.ReadAllText(directory.File(producer)));
        Assert.Equal(draftConsumer, File.ReadAllText(directory.File(consumer)));
        Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));

        Task<ProcessResult> RunRefactor(params string[] arguments)
            => ProcessRunner.RunAsync([cli, "refactor", .. arguments], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));

        async Task<JsonDocument> QueryDraft(int expected)
        {
            const string requests = """
                [{"operation":"find","name":"Value","match":"exact","kinds":["routine"],"project":"Game/Game.fpasproj"},
                 {"operation":"refs","from":0},{"operation":"inspect","from":0}]
                """;
            var result = await ProcessRunner.RunAsync(["exec", "--runtimeconfig", configPath, cli, "query", "batch", context,
                "--overlay", "draft.json", "--expected-snapshot", snapshot], input: requests,
                workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
            Assert.True(result.ExitCode == expected, result.Output + result.Error);
            return JsonDocument.Parse(result.Output);
        }

        async Task<JsonDocument> Draft(int expected, params (string File, string Text)[] replacements)
        {
            File.WriteAllText(directory.File("draft.json"), JsonSerializer.Serialize(replacements.Select(item => new { file = item.File, text = item.Text })));
            var result = await Check(directory, context, "--overlay", "draft.json", "--report", "json");
            Assert.True(result.ExitCode == expected, result.Output + result.Error);
            return JsonDocument.Parse(result.Output);
        }
    }

    [Fact]
    public async Task Draft_creates_a_new_unit_and_replaces_its_caller_after_checking_actual_ownership()
    {
        using var directory = sdk.CreateProject();
        Directory.CreateDirectory(directory.File("Draft"));
        const string context = "Draft/Draft.fpasproj";
        const string source = "Draft/Core.fpas";
        const string added = "Draft/Extra unit.fpas";
        const string original = "unit Core; public function Value(): int; begin return 0; end function; end unit;";
        const string helper = "unit Extra; public function Delta(): int; begin return 3; end function; end unit;";
        const string replacement = "unit Core; uses Extra as E; public function Value(): int; begin return E.Delta(); end function; end unit;";
        File.WriteAllText(directory.File(context), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup Condition="'$(Configuration)' == 'Debug'"><Compile Remove="Excluded.fpas" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File(source), original);
        var restored = await sdk.Run(directory, "restore", context);
        Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);

        WriteOverlay("Draft/Excluded.fpas");
        var excluded = await Check(directory, context, "--overlay", "additions.json", "--report", "json");
        Assert.True(excluded.ExitCode == 2, excluded.Output + excluded.Error);
        Assert.False(File.Exists(directory.File("Draft/Excluded.fpas")));

        WriteOverlay(added);
        var checkedDraft = await Check(directory, context, "--overlay", "additions.json", "--report", "json");
        Assert.True(checkedDraft.ExitCode == 0, checkedDraft.Output + checkedDraft.Error);
        using var report = JsonDocument.Parse(checkedDraft.Output);
        var snapshot = report.RootElement.GetProperty("snapshotId").GetString()!;
        var query = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", "batch", context,
            "--overlay", "additions.json", "--expected-snapshot", snapshot],
            input: """[{"operation":"find","name":"Delta","match":"exact"},{"operation":"refs","from":0}]""",
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(query.ExitCode == 0, query.Output + query.Error);
        using (var found = JsonDocument.Parse(query.Output))
            Assert.Equal(1, found.RootElement.GetProperty("results")[1].GetProperty("result").GetProperty("refs").GetProperty("totalCount").GetInt32());
        var planned = await Refactor("draft", "--overlay", "additions.json", "--expected-snapshot", snapshot);
        Assert.True(planned.ExitCode == 0, planned.Output + planned.Error);
        Assert.False(File.Exists(directory.File(added)));
        Assert.Equal(original, File.ReadAllText(directory.File(source)));

        File.Delete(directory.File("additions.json"));
        File.WriteAllText(directory.File(added), "other writer");
        var refused = await Refactor("apply");
        Assert.Equal(1, refused.ExitCode);
        using (var error = JsonDocument.Parse(refused.Output))
            Assert.Equal("inputs_changed", error.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal("other writer", File.ReadAllText(directory.File(added)));
        Assert.Equal(original, File.ReadAllText(directory.File(source)));
        File.Delete(directory.File(added));

        var applied = await Refactor("apply");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        using (var appliedReport = JsonDocument.Parse(applied.Output))
            Assert.True(appliedReport.RootElement.GetProperty("rechecked").GetBoolean());
        Assert.Equal(helper, File.ReadAllText(directory.File(added)));
        Assert.Equal(replacement, File.ReadAllText(directory.File(source)));
        Assert.False(File.Exists(directory.File("Draft/bin/Debug/net10.0/Draft.dll")));

        void WriteOverlay(string addedPath) => File.WriteAllText(directory.File("additions.json"), JsonSerializer.Serialize(new object[]
        {
            new { file = source, text = replacement }, new { file = addedPath, text = helper, project = context }
        }));
        Task<ProcessResult> Refactor(string operation, params string[] arguments)
            => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "refactor", operation, context, "--plan", "additions.plan.json",
                "--report", "json", .. arguments], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
    }

    private Task<ProcessResult> Check(TemporaryDirectory project, params string[] arguments)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check", .. arguments], workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(3));
    private static string Hash(string path) => FileHash.Sha256(path);
}
