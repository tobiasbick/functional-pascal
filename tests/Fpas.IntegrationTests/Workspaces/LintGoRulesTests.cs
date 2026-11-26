// [Documentation](../../../docs/pascal/program-structure/cli.md#lint-a-project-or-solution)
using System.Text.Json;

namespace Fpas.IntegrationTests.Workspaces;

public sealed partial class LintTests
{
    /// <summary>Finds synchronization capabilities, while permitting reader and token result ports.</summary>
    [Fact]
    public async Task Exported_synchronization_reports_fields_of_public_records()
    {
        using var directory = CreateRulesProject();
        Write(directory, "Rules/Ports.fpas", """
            unit Ports;
            uses System.Threading as T; uses System.Threading.Channels as C; uses System.Threading.Tasks as Tasks;
            public type Session = record
              public Gate: T.SemaphoreSlim;
              public Inbox: C.Channel<int>;
              public Writer: C.ChannelWriter<int>;
              public Stop: T.CancellationTokenSource;
              public Complete: Tasks.TaskCompletionSource<int>;
              public Reader: C.ChannelReader<int>;
              public Token: T.CancellationToken;
              Hidden: T.Mutex;
            end record;
            type Local = record
              public Gate: T.SemaphoreSlim;
            end record;
            end unit;
            """);
        var findings = await Findings(directory);
        Assert.All(findings, finding => Assert.Equal("exported-sync", Rule(finding)));
        Assert.Equal(["Ports.Session.Gate", "Ports.Session.Inbox", "Ports.Session.Writer",
            "Ports.Session.Stop", "Ports.Session.Complete"], findings.Select(Key));
        Assert.All(findings, finding => Assert.Equal("error", finding.GetProperty("severity").GetString()));
    }

    /// <summary>Warns for a private protocol with one implementing record without failing the command.</summary>
    [Fact]
    public async Task Lonely_protocol_remains_a_nonblocking_warning()
    {
        using var directory = CreateRulesProject();
        Write(directory, "Rules/Protocols.fpas", """
            unit Contracts;
            uses System as Sys;
            type ILonely = protocol
              function Read(): int;
              function Count(): int;
            end protocol;
            type IMultiple = protocol
              function Read(): int;
            end protocol;
            type IUnimplemented = protocol
              function Read(): int;
            end protocol;
            type IInherited = protocol extends Sys.IDisposable
              function Read(): int;
            end protocol;
            public type IOpen = protocol
              function Read(): int;
            end protocol;
            type One = record implements ILonely, IMultiple, IInherited, IOpen
              public function Read(): int;
              begin return 1; end function;
              public function Count(): int;
              begin return 1; end function;
              public procedure Dispose();
              begin end procedure;
            end record;
            type Two = record implements IMultiple
              public function Read(): int;
              begin return 2; end function;
            end record;
            end unit;
            """);

        var result = await Lint(directory, "Rules/Rules.fpasproj", "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("ok", report.RootElement.GetProperty("status").GetString());
        var finding = Assert.Single(report.RootElement.GetProperty("findings").EnumerateArray());
        Assert.Equal("lonely-protocol", Rule(finding));
        Assert.Equal("Contracts.ILonely", Key(finding));
        Assert.Equal("warning", finding.GetProperty("severity").GetString());
    }

    /// <summary>Finds sibling namespace cycles through individual acyclic dependencies in different projects.</summary>
    [Fact]
    public async Task Namespace_cycle_uses_resolved_sources_across_projects()
    {
        using var directory = CreateRulesProject(dependency: "Provider");
        Write(directory, "Provider/LibraryModel.fpas", ValueUnit("App.Library.Model"));
        Write(directory, "Provider/PlayerModel.fpas", ValueUnit("App.Player.Model"));
        Write(directory, "Provider/Nested.fpas", ValueUnit("App.Library.Detail.Model"));
        Write(directory, "Rules/LibraryScan.fpas", ConsumerUnit("App.Library.Scan", "App.Player.Model"));
        Write(directory, "Rules/PlayerCommands.fpas", ConsumerUnit("App.Player.Commands", "App.Library.Model"));
        Write(directory, "Rules/LibrarySetup.fpas", ConsumerUnit("App.Library.Setup", "App.Library.Detail.Model"));
        Write(directory, "Rules/LibraryDetail.fpas", ConsumerUnit("App.Library.Detail.Read", "App.Library.Model"));

        var finding = Assert.Single(await Findings(directory));
        Assert.Equal("namespace-cycle", Rule(finding));
        Assert.Equal("App.Library, App.Player", Key(finding));
        Assert.Equal(["App.Library -> App.Player", "App.Player -> App.Library"],
            finding.GetProperty("cycleEdges").EnumerateArray().Select(edge => edge.GetString()));
        Assert.Equal([directory.File("Rules/LibraryScan.fpas"), directory.File("Rules/PlayerCommands.fpas")],
            finding.GetProperty("related").EnumerateArray().Select(PathAt));

        static string ValueUnit(string name) => $"unit {name}; public function Read(): int; begin return 1; end function; end unit;";
        static string ConsumerUnit(string name, string target) =>
            $"unit {name}; uses {target} as Source; public function Read(): int; begin return Source.Read(); end function; end unit;";
    }

    /// <summary>Exercises line-stable keys, a shrinking cycle, a new edge and refresh of resolved entries.</summary>
    [Fact]
    public async Task Baseline_accepts_smaller_cycles_but_reports_new_edges()
    {
        using var directory = CreateRulesProject();
        Write(directory, "Rules/A.fpas", Unit("A", ["B"]));
        Write(directory, "Rules/B.fpas", Unit("B", ["A", "C"]));
        Write(directory, "Rules/C.fpas", Unit("C", ["B"]));
        Write(directory, "Rules/Unused.fpas", "unit Unused; uses System.Text as Text; end unit;");
        var baseline = directory.File("lint-baseline.json");
        var written = await Lint(directory, "Rules/Rules.fpasproj", "--write-baseline", baseline, "--report", "json");
        Assert.True(written.ExitCode == 0, written.Output + written.Error);
        var original = File.ReadAllText(baseline);
        Assert.DoesNotContain(directory.Path, original);
        using (var document = JsonDocument.Parse(original))
            Assert.Equal(3, document.RootElement.GetProperty("entries").GetArrayLength());

        Write(directory, "Rules/B.fpas", "\n\n" + Unit("B", ["A"]));
        var shrunk = await Lint(directory, "Rules/Rules.fpasproj", "--existing-inputs", "--baseline", baseline, "--report", "json");
        Assert.True(shrunk.ExitCode == 0, shrunk.Output + shrunk.Error);
        using (var report = JsonDocument.Parse(shrunk.Output))
            Assert.Equal(3, report.RootElement.GetProperty("baseline").GetProperty("suppressedCount").GetInt32());

        Write(directory, "Rules/Unused.fpas", "unit Unused; end unit;");
        Write(directory, "Rules/A.fpas", Unit("A", ["B", "C"]));
        Write(directory, "Rules/B.fpas", Unit("B", ["A", "C"]));
        var regression = await Lint(directory, "Rules/Rules.fpasproj", "--existing-inputs", "--baseline", baseline, "--report", "json");
        Assert.True(regression.ExitCode == 1, regression.Output + regression.Error);
        using (var report = JsonDocument.Parse(regression.Output))
        {
            var root = report.RootElement;
            Assert.Equal(["module-cycle", "tangle"], root.GetProperty("findings").EnumerateArray().Select(Rule));
            var resolved = Assert.Single(root.GetProperty("baseline").GetProperty("resolved").EnumerateArray());
            Assert.Equal("unused-clr-alias", resolved.GetProperty("rule").GetString());
        }
        Assert.Equal(original, File.ReadAllText(baseline));

        var refreshed = await Lint(directory, "Rules/Rules.fpasproj", "--existing-inputs", "--write-baseline", baseline, "--report", "json");
        Assert.True(refreshed.ExitCode == 0, refreshed.Output + refreshed.Error);
        using (var document = JsonDocument.Parse(File.ReadAllText(baseline)))
            Assert.Equal(2, document.RootElement.GetProperty("entries").GetArrayLength());

        var mismatch = await Lint(directory, "Rules/Rules.fpasproj", "--existing-inputs", "--baseline", baseline, "--held-in-mind", "8", "--report", "json");
        Assert.Equal(1, mismatch.ExitCode);
        using (var document = JsonDocument.Parse(mismatch.Output))
            Assert.Equal("invalid_baseline", document.RootElement.GetProperty("error").GetProperty("code").GetString());

        var fresh = File.ReadAllText(baseline);
        Write(directory, "Rules/A.fpas", "unit A; public function Read(): int; begin return Missing; end function; end unit;");
        var failed = await Lint(directory, "Rules/Rules.fpasproj", "--existing-inputs", "--write-baseline", baseline, "--report", "json");
        Assert.Equal(1, failed.ExitCode);
        Assert.Equal(fresh, File.ReadAllText(baseline));

        static string Unit(string name, string[] imports) =>
            $"unit {name}; {string.Concat(imports.Select(other => $"uses {other} as Use{other}; "))}" +
            $"public function Read(): int; begin return {string.Join(" + ", imports.Select(other => $"Use{other}.Read()"))}; end function; end unit;";
    }

    private static string? Key(JsonElement value) => value.GetProperty("key").GetString();
}
