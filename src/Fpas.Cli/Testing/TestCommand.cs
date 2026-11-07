using Fpas.Cli.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Projects;
using Fpas.Workspaces.Testing;
using System.CommandLine;
using System.Text.Json;

namespace Fpas.Cli.Testing;

internal sealed class TestCommand
{
    private readonly Argument<string> context = new("context") { Description = "Explicit test project (.fpasproj/.csproj/.fsproj) or .slnx solution." };
    private readonly Option<string> configuration = new("--configuration") { DefaultValueFactory = _ => "Debug", Description = "Build/solution configuration." };
    private readonly Option<string> project = new("--project") { Description = "Included test project to select from the context." };
    private readonly Option<bool> noBuild = new("--no-build") { Description = "Use existing test outputs; implies --no-restore." };
    private readonly Option<bool> noRestore = new("--no-restore") { Description = "Build without restoring packages." };
    private readonly Option<bool> list = new("--list") { Description = "Discover fully qualified method identities without executing tests." };
    private readonly Option<string> filter = new("--filter") { Description = "Native VSTest or registered MTP framework filter expression." };
    private readonly Option<string> settings = new("--settings") { Description = "Existing native .runsettings file." };
    private readonly Option<string> directory = new("--results-directory") { Description = "Parent for a new, unique result directory; defaults to context/TestResults." };
    private readonly Option<string> report = new("--report") { Description = "Write one structured JSON document to stdout." };
    private readonly Option<int> timeout = new("--timeout") { DefaultValueFactory = _ => 120, Description = "Overall deadline in seconds (1..3600)." };
    private readonly Option<string[]> adapters = new("--adapter-path") { Description = "VSTest extension directory; repeat for multiple directories.", AllowMultipleArgumentsPerToken = false };
    private readonly Option<string[]> collectors = new("--collect") { Description = "VSTest data collector name, repeatable.", AllowMultipleArgumentsPerToken = false };
    private readonly Argument<string[]> arguments = new("arguments") { Arity = ArgumentArity.ZeroOrMore, Description = "MTP extension options or VSTest runsettings after --." };

    internal Command Command { get; } = new("test", """
        Build and run native .NET tests using evaluated VSTest or self-hosted Microsoft.Testing.Platform projects.
        FPAS tests use ordinary framework attributes and assertions. Solutions skip non-test projects.
        JSON uses native TRX/discovery artifacts. Frameworks own test cases, skips and lifecycle behavior.
        Exit codes: 0 success/help, 1 test/build/runner/deadline failure, 2 arguments, 130 cancellation.

        Examples:
          fpas test Tests/Tests.fpasproj

          fpas test Example.slnx --filter FullyQualifiedName~Addition --report json

          fpas test Tests/Tests.fpasproj --no-build --list

          fpas test MtpTests/MtpTests.fpasproj -- --minimum-expected-tests 1
        """);

    internal TestCommand()
    {
        report.AcceptOnlyFromAmong("json"); ContextCommandOptions.Range(timeout, 1, 3600);
        Command.Arguments.Add(context);
        Command.Arguments.Add(arguments);
        foreach (var option in new Option[] { configuration, project, noBuild, noRestore, list, filter, settings, directory, report, timeout, adapters, collectors }) Command.Options.Add(option);
        Command.SetAction(Execute);
    }

    internal void ReportArgumentError(ParseResult result, string message)
        => new TestReport(false, "failed", result.GetValue(list) ? "list" : "run", result.GetValue(context))
            { Error = new("invalid_argument", message + " See fpas test --help.") }.Write(Json(result));
    private bool Json(ParseResult result) => result.GetResult(report)?.Tokens.Any(token => token.Value == "json") == true;
    internal void Validate(ParseResult result, string[] raw)
    { if ((result.GetValue(arguments)?.Length ?? 0) != 0 && !raw.Contains("--")) throw new ArgumentException("Pass native runner options after --."); }

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        var json = Json(result); var mode = result.GetValue(list) ? "list" : "run";
        string? input = null; string? results = null; Workspaces.Processes.BuildResult? build = null;
        var completed = new List<ProjectTestResult>();
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        var token = combined.Token;
        TestReport response; int exit;
        TestReport Failure(string code, string message) => new(false, code is "cancelled" or "timeout" ? code : "failed", mode, input)
            { ResultsDirectory = results, Build = build, Projects = completed.ToArray(), Error = new(code, message) };
        try
        {
            input = ProjectDriver.ValidateContext(result.GetValue(context)!);
            var config = result.GetValue(configuration)!;
            if (string.IsNullOrWhiteSpace(config)) throw new ContextFailure("invalid_argument", "Configuration must not be empty.");
            var runsettings = result.GetValue(settings) is { } file ? Path.GetFullPath(file) : null;
            if (runsettings is not null && !File.Exists(runsettings)) throw new ContextFailure("invalid_argument", "--settings must select an existing runsettings file.");
            var paths = (result.GetValue(adapters) ?? []).Select(Path.GetFullPath).ToArray();
            if (paths.Any(path => !Directory.Exists(path) || path.Contains(';'))) throw new ContextFailure("invalid_argument", "Select existing adapter directories without semicolons.");
            var extra = result.GetValue(arguments) ?? [];
            if (extra.Length > 256 || extra.Sum(item => (long)item.Length) > 65536 || extra.Any(item => item.Contains('\0'))) throw new ContextFailure("invalid_argument", "Native options require at most 256 arguments and 65536 code units without NUL.");
            var options = new TestRunOptions(paths, result.GetValue(collectors) ?? [], extra);
            if (!result.GetValue(noBuild))
            {
                build = await ProjectDriver.BuildAsync(input, config, result.GetValue(noRestore), token);
                if (build.ExitCode != 0 || build.Truncated) { Failure("build_failed", "Build failed or exceeded its output limit.").Write(json); return 1; }
            }
            var projects = await TestDriver.SelectAsync(input, config, result.GetValue(project), token);
            results = Path.Combine(Path.GetFullPath(result.GetValue(directory) ?? Path.Combine(Path.GetDirectoryName(input)!, "TestResults")), "fpas-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(results);
            foreach (var selected in projects)
            {
                token.ThrowIfCancellationRequested(); TestDriver.RequireOutput(selected);
                var folder = Path.Combine(results, completed.Count.ToString("D4", System.Globalization.CultureInfo.InvariantCulture)); Directory.CreateDirectory(folder);
                if (mode == "list")
                {
                    var discovered = await TestDriver.DiscoverAsync(selected, folder, result.GetValue(filter), runsettings, token, options);
                    completed.Add(new(selected, discovered.Process, null, null, discovered.Tests));
                }
                else
                {
                    var process = await TestDriver.RunAsync(selected, folder, result.GetValue(filter), runsettings, token, options);
                    var trx = Path.Combine(folder, "results.trx");
                    var native = File.Exists(trx) ? TrxReport.Read(trx, token) : null;
                    completed.Add(new(selected, process, File.Exists(trx) ? trx : null, native, null));
                    if (native is null && process.ExitCode == 0 && !process.Truncated)
                        throw new ContextFailure("results_missing", "The native test runner returned success without a TRX artifact.");
                }
            }
            token.ThrowIfCancellationRequested();
            var success = completed.All(item => item.Runner.ExitCode == 0 && !item.Runner.Truncated && item.Result?.Success != false);
            response = completed.Any(item => item.Runner.Truncated) ? Failure("output_limit", "A native runner exceeded its captured output limit.") :
                !success && !completed.Any(item => (item.Result?.Counts.Failed ?? 0) != 0) ? Failure(mode == "list" ? "discovery_failed" : "runner_failed", "The native runner failed; inspect the captured runner diagnostics.") :
                new(success, success ? "ok" : "test_failed", mode, input) { ResultsDirectory = results, Build = build, Projects = completed.ToArray() };
            exit = success ? 0 : 1;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        { var cancelled = cancellationToken.IsCancellationRequested; response = Failure(cancelled ? "cancelled" : "timeout", cancelled ? "Testing cancelled." : "Testing exceeded --timeout."); exit = cancelled ? 130 : 1; }
        catch (ContextFailure failure) { response = Failure(failure.Code, failure.Message); exit = failure.Code is "invalid_argument" or "not_test_project" or "unsupported_runner" ? 2 : 1; }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or ArgumentException or JsonException or System.Xml.XmlException or System.ComponentModel.Win32Exception or PlatformNotSupportedException)
        { response = Failure("test_failed", exception.Message); exit = 1; }
        return response.Write(json) ? exit : 1;
    }
}
