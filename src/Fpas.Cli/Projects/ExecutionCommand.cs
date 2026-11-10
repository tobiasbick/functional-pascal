using Fpas.Cli.Analysis;
using Fpas.Cli.Reporting;
using Fpas.Debugging;
using Fpas.Processes;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Projects;
using System.CommandLine;
using System.Text;
using System.Text.Json;

namespace Fpas.Cli.Projects;

internal sealed class ExecutionCommand
{
    private readonly bool run;
    private readonly Argument<string> input = new("input") { Description = "Explicit source, project, solution or (run only) managed DLL." };
    private readonly Argument<string[]> arguments = new("arguments") { Arity = ArgumentArity.ZeroOrMore, Description = "Program arguments after --." };
    private readonly Option<string> configuration = new("--configuration") { DefaultValueFactory = _ => "Debug", Description = "MSBuild configuration." };
    private readonly Option<bool> noRestore = new("--no-restore") { Description = "Build with existing restored assets." };
    private readonly Option<bool> noBuild = new("--no-build") { Description = "Run the project's existing output; implies --no-restore." };
    private readonly Option<string> project = new("--project") { Description = "Executable project in a solution; required when ambiguous." };
    private readonly Option<string> workingDirectory = new("--working-directory") { Description = "Program working directory; defaults to the caller's directory." };
    private readonly Option<string> version = new("--sdk-version") { Description = "SDK version for standalone source input." };
    private readonly Option<string> feed = new("--sdk-feed") { Description = "SDK feed for standalone source input." };
    private readonly Option<int> timeout;
    private readonly Option<string> report = new("--report") { Description = "Write one JSON result, capturing program output." };
    private readonly Option<string> failureReport = new("--failure-report") { Description = "Save the first unhandled exception as debug JSON to a new file (requires matching PDB and sources)." };
    internal Command Command { get; }

    private readonly bool editorControlled;

    internal ExecutionCommand(bool run, bool editorControlled = false)
    {
        this.run = run;
        this.editorControlled = editorControlled;
        timeout = new("--timeout") { DefaultValueFactory = _ => run ? 0 : 120, Description = run ? "Overall deadline in seconds (0..3600; 0 means no deadline)." : "Overall deadline in seconds (1..3600)." };
        Command = new(run ? "run" : "build", run ? """
            Build and run an explicit source, project or solution, or run an existing managed DLL.
            A solution needs one executable or --project. Program arguments follow --.
            Exit codes preserve the program result; host failures use 1, arguments 2, cancellation 130.

            Examples:
              fpas run examples/basics/hello/Hello.fpasproj -- first --second

              fpas run App.slnx --project App/App.fpasproj --configuration Release
            """ : """
            Build an explicit FPAS source, project or solution through MSBuild.
            Restore is enabled unless --no-restore is selected. Output uses the project settings.

            Examples:
              fpas build examples/basics/hello/Hello.fpasproj

              fpas build App.slnx --configuration Release --report json
            """);
        report.AcceptOnlyFromAmong("json"); ContextCommandOptions.Range(timeout, run ? 0 : 1, 3600);
        Command.Arguments.Add(input);
        foreach (var option in new Option[] { configuration, noRestore, version, feed, timeout, report }) Command.Options.Add(option);
        if (run)
        {
            Command.Arguments.Add(arguments);
            foreach (var option in new Option[] { noBuild, project, workingDirectory, failureReport }) Command.Options.Add(option);
        }
        Command.SetAction(Execute);
    }

    internal bool Owns(Command command) => command == Command;
    internal bool Json(ParseResult result) => result.GetResult(report)?.Tokens.Any(token => token.Value == "json") == true;
    internal void Validate(ParseResult result, string[] raw)
    {
        if (run && (result.GetValue(arguments)?.Length ?? 0) != 0 && !raw.Contains("--", StringComparer.Ordinal))
            throw new ArgumentException("Pass program arguments after --.");
        if (editorControlled && !Json(result)) throw new ArgumentException("Captured runs require --report json.");
    }
    internal void ReportArgumentError(ParseResult result, string message)
        => Failure(null, "invalid_argument", message + $" See fpas {Command.Name} --help.").Write(Json(result));
    private ProjectReport Failure(string? context, string code, string message, IReadOnlyList<string>? candidates = null, BuildResult? built = null)
        => new(Command.Name, false, code == "cancelled" ? "cancelled" : "failed", context) { Build = built, Error = new(code, message, candidates) };

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        var json = Json(result);
        string? context = null; BuildResult? built = null;
        using var deadline = new CancellationTokenSource();
        if (result.GetValue(timeout) > 0) deadline.CancelAfter(TimeSpan.FromSeconds(result.GetValue(timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        try
        {
            context = Path.GetFullPath(result.GetValue(input)!);
            var selectedConfiguration = result.GetValue(configuration)!;
            if (string.IsNullOrWhiteSpace(selectedConfiguration)) throw new ArgumentException("Configuration must not be empty.");
            var extension = Path.GetExtension(context).ToLowerInvariant();
            var selectedProject = run ? result.GetValue(project) : null;
            var skipBuild = run && result.GetValue(noBuild);
            var cwd = run && result.GetValue(workingDirectory) is { } selectedDirectory ? Path.GetFullPath(selectedDirectory) : Environment.CurrentDirectory;
            if (!Directory.Exists(cwd)) throw new ArgumentException("The working directory does not exist.");
            if (extension != ".fpas" && (result.GetValue(version) is not null || result.GetValue(feed) is not null))
                throw new ArgumentException("--sdk-version and --sdk-feed apply only to standalone .fpas sources.");
            var buildContext = context;
            if (extension == ".fpas")
            {
                if (selectedProject is not null) throw new ArgumentException("Standalone sources cannot use --project.");
                buildContext = new SourceProject(context, ToolSdk.ValidateVersion(result.GetValue(version)), result.GetValue(feed)).Project;
            }
            ProjectOutput? target = null;
            if (extension == ".dll")
            {
                if (!run || !File.Exists(context) || selectedProject is not null || skipBuild || result.GetValue(noRestore) ||
                    result.GetResult(configuration)?.Tokens.Count > 0)
                    throw new ArgumentException("Run an existing DLL without project/build options.");
                target = new(context, selectedConfiguration, "AnyCPU", "", "Exe", context);
            }
            else
            {
                ProjectDriver.ValidateContext(buildContext);
                if (run)
                {
                    if (!skipBuild) ProjectProgress.Phase(json, "Resolving", context);
                    target = await ProjectDriver.SelectExecutableAsync(buildContext, selectedConfiguration, selectedProject, combined.Token);
                }
                if (!skipBuild)
                {
                    var liveOutput = ProjectProgress.BuildOutput(json, "Building", context);
                    built = await ProjectDriver.BuildAsync(buildContext, selectedConfiguration, result.GetValue(noRestore), combined.Token,
                        captureDiagnostics: json, liveOutput: liveOutput);
                    if (built.ExitCode != 0 || built.Truncated)
                    {
                        Failure(context, built.Truncated ? "output_limit" : "build_failed", "Build failed or exceeded its output limit.", built: built).Write(json); return 1;
                    }
                }
                if (run && !skipBuild) target = await ProjectDriver.DescribeAsync(buildContext, selectedConfiguration, target!.Project, combined.Token);
            }
            if (!run)
            {
                new ProjectReport("build", true, "ok", context) { Build = built }.Write(json); return 0;
            }
            if (!target!.Executable || !File.Exists(target.Assembly) || !File.Exists(Path.ChangeExtension(target.Assembly, ".runtimeconfig.json")))
                throw new ContextFailure("not_executable", "The selected managed output and its runtimeconfig.json must exist. Build the executable first.");
            ProgramResult executed;
            string? savedFailure = null;
            var programArguments = new[] { target.Assembly }.Concat(result.GetValue(arguments) ?? []).ToArray();
            if (result.GetValue(failureReport) is { } destination)
            {
                if (string.IsNullOrWhiteSpace(destination)) throw new ArgumentException("The failure report path must not be empty.");
                if (built is not null) ProjectProgress.Phase(json, "Running", target.Assembly);
                var observed = await DebuggerApplication.RunWithFailureReportAsync(new(target.Assembly, cwd,
                    result.GetValue(arguments) ?? [], Path.GetFullPath(destination), json, !editorControlled, result.GetValue(timeout)),
                    BuildProcess.DotnetHost(), combined.Token);
                executed = new(observed.ExitCode, observed.Output, observed.Error, observed.Truncated);
                savedFailure = observed.ReportPath;
            }
            else if (json)
            {
                var output = await BuildProcess.Run(cwd, programArguments, combined.Token, inheritInput: !editorControlled, buildEnvironment: false);
                executed = new(output.ExitCode, output.Output, output.Error, output.Truncated);
            }
            else
            {
                if (built is not null) ProjectProgress.Phase(json, "Running", target.Assembly);
                await using var process = OwnedProcess.Start(BuildProcess.DotnetHost(), programArguments, cwd, stdio: ProcessStdio.Inherit);
                await process.WaitForExit(combined.Token);
                executed = new(process.ExitCode, "", "", false);
            }
            new ProjectReport("run", executed.ExitCode == 0 && !executed.Truncated, executed.Truncated ? "output_limit" : executed.ExitCode == 0 ? "ok" : "program_failed", context)
                { Target = target, Build = built, Program = executed, FailureReport = savedFailure }.Write(json);
            return executed.Truncated ? 1 : executed.ExitCode;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            Failure(context, cancelled ? "cancelled" : "timeout", cancelled ? "Operation cancelled." : "Operation exceeded --timeout.", built: built).Write(json);
            return cancelled ? 130 : 1;
        }
        catch (ContextFailure failure)
        {
            Failure(context, failure.Code, failure.Message, failure.Candidates, built).Write(json);
            return failure.Code is "invalid_argument" or "ambiguous_project" or "not_executable" ? 2 : 1;
        }
        catch (Exception exception) when (exception is ArgumentException or IOException or UnauthorizedAccessException or JsonException or System.Xml.XmlException or System.ComponentModel.Win32Exception or PlatformNotSupportedException)
        {
            var argument = exception is ArgumentException && exception is not DecoderFallbackException;
            Failure(context, argument ? "invalid_argument" : "operation_failed", exception.Message, built: built).Write(json); return argument ? 2 : 1;
        }
    }
}
