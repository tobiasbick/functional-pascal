using System.CommandLine;
using Fpas.Cli.Analysis;
using Fpas.Cli.Reporting;
using Fpas.Processes;

namespace Fpas.Cli.Profiling;

internal sealed class ProfileCommand
{
    private readonly Option<string> tools = new("--tools-directory") { Recursive = true, DefaultValueFactory = _ => DiagnosticTools.DefaultDirectory, Description = "Private directory for pinned Microsoft diagnostic tools." };
    private readonly Option<int> timeout = new("--timeout") { Recursive = true, DefaultValueFactory = _ => 120, Description = "Whole-operation deadline, 1..3600 seconds." };
    private readonly Option<string> mode = new("--mode") { DefaultValueFactory = _ => "threads", Description = "threads, gc or counters." };
    private readonly Option<string> output = new("--output") { Description = "Fresh artifact filename; never replace an existing file.", Required = true };
    private readonly Option<string> cwd = new("--cwd") { Description = "Target working directory; defaults to invocation directory." };
    private readonly Option<int> duration = new("--duration") { DefaultValueFactory = _ => 10, Description = "Collection duration in seconds, 1..300." };
    private readonly Option<int> limit = new("--limit") { DefaultValueFactory = _ => 20, Description = "Maximum frames per thread, 1..100." };
    private readonly Argument<string> executable = new("executable") { Description = "Built DLL or absolute native executable; target arguments follow --." };
    private readonly Argument<string[]> arguments = new("arguments") { Arity = ArgumentArity.ZeroOrMore };
    private readonly Argument<string> trace = new("trace") { Description = "Existing .nettrace recording." };
    private readonly Command setup = new("setup", "Install pinned tools in an isolated directory. Requires package access on first use.\n\nExamples:\n  fpas profile setup\n");
    private readonly Command collect = new("collect", "Record an owned target with Microsoft's pinned collector. Output is one JSON report.\n\nExamples:\n  fpas profile collect App.dll --output cpu.nettrace --duration 5 -- cpu 8000\n");
    private readonly Command summary = new("summary", "Decode EventPipe stacks, per-thread wall-time samples and GC events.\n\nExamples:\n  fpas profile summary cpu.nettrace --limit 10\n");
    internal Command Command { get; } = new("profile", "Collect and summarize .NET diagnostic artifacts. Timings under a profiler are not benchmark results.\n\nExamples:\n  fpas profile setup\n\n  fpas profile collect --help\n\n  fpas profile summary --help\n");

    internal ProfileCommand()
    {
        mode.AcceptOnlyFromAmong("threads", "gc", "counters"); ContextCommandOptions.Range(timeout, 1, 3600);
        ContextCommandOptions.Range(duration, 1, 300); ContextCommandOptions.Range(limit, 1, 100);
        Command.Options.Add(tools); Command.Options.Add(timeout);
        foreach (var sub in new[] { setup, collect, summary }) { Command.Subcommands.Add(sub); sub.SetAction(Execute); }
        collect.Arguments.Add(executable); collect.Arguments.Add(arguments);
        foreach (var option in new Option[] { mode, output, cwd, duration }) collect.Options.Add(option);
        summary.Arguments.Add(trace); summary.Options.Add(limit);
    }
    internal bool Owns(Command command) => command == Command || Command.Subcommands.Contains(command);
    internal int ArgumentError(string message) => OperationReport.Fail("profile", "invalid_argument", message + " See fpas profile --help.", 2);
    internal void Validate(ParseResult result, string[] raw)
    { if (result.CommandResult.Command == collect && (result.GetValue(arguments)?.Length ?? 0) > 0 && !raw.Contains("--")) throw new ArgumentException("Pass target arguments after --."); }

    private async Task<int> Execute(ParseResult result, CancellationToken cancellation)
    {
        var name = "profile " + result.CommandResult.Command.Name;
        ArtifactCapture? capture = null;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(timeout)));
        using var linked = CancellationTokenSource.CreateLinkedTokenSource(cancellation, deadline.Token);
        try
        {
            if (result.CommandResult.Command == setup)
            {
                var installed = await DiagnosticTools.Setup(result.GetValue(tools)!, linked.Token);
                var ok = installed.ExitCode == 0 && !installed.Truncated;
                return new OperationReport(name, ok, ok ? "ok" : "failed") { Process = installed, Error = ok ? null : new("setup_failed", "Inspect the package-tool output.") }.Write(ok ? 0 : 1);
            }
            if (result.CommandResult.Command == summary)
            {
                var data = await ProfileWorker.Analyze(result.GetValue(trace)!, result.GetValue(limit), linked.Token);
                return new OperationReport(name, !data.Truncated && data.LostEvents == 0, data.Truncated || data.LostEvents != 0 ? "incomplete" : "ok") { Trace = data }.Write(data.Truncated || data.LostEvents != 0 ? 1 : 0);
            }
            var target = result.GetValue(executable)!;
            var targetArguments = result.GetValue(arguments) ?? [];
            if (target.EndsWith(".dll", StringComparison.OrdinalIgnoreCase))
            {
                var assembly = Path.GetFullPath(target);
                if (!File.Exists(assembly) || !File.Exists(Path.ChangeExtension(assembly, ".runtimeconfig.json"))) throw new ArgumentException("Select a built DLL with its runtimeconfig.json.");
                target = Workspaces.Processes.BuildProcess.DotnetHost(); targetArguments = [assembly, .. targetArguments];
            }
            OwnedProcess.Validate(target);
            if (targetArguments.Length > 1024 || targetArguments.Sum(item => (long)item.Length) > 65536) throw new ArgumentException("Target arguments require at most 1024 elements and 65536 code units.");
            var file = Path.GetFullPath(result.GetValue(output)!);
            if (File.Exists(file) || Directory.Exists(file)) throw new ArgumentException("Output already exists. Choose a fresh artifact filename.");
            var selectedMode = result.GetValue(mode)!; var tool = selectedMode == "counters" ? "dotnet-counters" : "dotnet-trace";
            _ = DiagnosticTools.Assembly(result.GetValue(tools)!, tool);
            if (Path.GetExtension(file) != (selectedMode == "counters" ? ".csv" : ".nettrace")) throw new ArgumentException("Use .csv for counters and .nettrace for threads/gc.");
            var artifact = new ArtifactCapture(file); capture = artifact;
            var args = new List<string> { "collect", "--output", artifact.StagedPath, "--duration", TimeSpan.FromSeconds(result.GetValue(duration)).ToString(@"hh\:mm\:ss", System.Globalization.CultureInfo.InvariantCulture) };
            if (selectedMode == "counters") args.AddRange(["--format", "csv", "--counters", "System.Runtime"]);
            else args.AddRange(["--profile", selectedMode == "gc" ? "gc-verbose" : "dotnet-sampled-thread-time,dotnet-common"]);
            args.AddRange(["--", target, .. targetArguments]);
            var process = await DiagnosticTools.Run(result.GetValue(tools)!, tool, Path.GetFullPath(result.GetValue(cwd) ?? Environment.CurrentDirectory), args, linked.Token);
            var success = process.ExitCode == 0 && !process.Truncated && artifact.Partial is not null && new FileInfo(artifact.StagedPath).Length > 0;
            return new OperationReport(name, success, success ? "ok" : "failed") { Artifact = success ? artifact.Publish() : artifact.Partial, Process = process,
                Error = success ? null : new("collection_failed", "Collection failed, exceeded its output limit or produced no artifact.") }.Write(success ? 0 : 1);
        }
        catch (OperationCanceledException)
        {
            var reason = cancellation.IsCancellationRequested ? "cancelled" : "timeout";
            return new OperationReport(name, false, reason) { Artifact = capture?.Partial, Error = new(reason, "Collection/analysis interrupted after owned-process cleanup.") }.Write(cancellation.IsCancellationRequested ? 130 : 1);
        }
        catch (Exception failure) when (failure is not OutOfMemoryException)
        { return new OperationReport(name, false, "failed") { Artifact = capture?.Partial, Error = new(failure is ArgumentException ? "invalid_argument" : "profile_failed", failure.Message) }.Write(failure is ArgumentException ? 2 : 1); }
    }
}
