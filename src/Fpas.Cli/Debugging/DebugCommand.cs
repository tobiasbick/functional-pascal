using Fpas.Debugging;
using Fpas.Debugging.Batch;
using Fpas.Workspaces.Processes;
using System.CommandLine;

namespace Fpas.Cli.Debugging;

internal sealed class DebugCommand
{
    private readonly Argument<string?> program = new("program") { Arity = ArgumentArity.ZeroOrOne, Description = "Already built executable DLL, with matching portable PDB and sources." };
    private readonly Argument<string[]> arguments = new("arguments") { Arity = ArgumentArity.ZeroOrMore, Description = "Individual target arguments after --." };
    private readonly Option<string> protocol = new("--protocol") { DefaultValueFactory = _ => "batch", Description = "Debugger frontend: batch or dap." };
    private readonly Option<string> cwd = new("--cwd") { Description = "Target directory; defaults to the DLL directory." };
    private readonly Option<string[]> breakpoints = new("--break") { Description = "Source path:line[:column], repeatable; relative to --cwd." };
    private readonly Option<string[]> expressions = new("--print") { Description = "Passive FPAS expression to capture at the stop, repeatable." };
    private readonly Option<string> stdin = new("--stdin") { Description = "File of at most 1 MiB to deliver to target stdin, then EOF. Default: closed." };
    private readonly Option<int> timeout = new("--timeout") { DefaultValueFactory = _ => 30, Description = "Whole invocation deadline in seconds (1..3600)." };
    private readonly Option<int> outputLimit = new("--output-limit") { DefaultValueFactory = _ => 1_048_576, Description = "Captured target output byte limit (1..1048576)." };
    internal Command Command { get; }

    internal DebugCommand()
    {
        protocol.AcceptOnlyFromAmong("batch", "dap");
        Command = new("debug", """
            Debug a built .NET executable using the FPAS debugger core.
            Batch returns one bounded JSON report and terminates the owned target after capture.
            No implicit build or restore. Inspection never calls target methods or getters.
            Exits: 0 success, 1 setup/debugger failure, 2 arguments, 3 failed observation,
            4 deadline/output limit, 130 cancellation.

            Examples:
              fpas debug artifacts/Mandelbrot.dll --break main.fpas:13 --print Iteration

              fpas debug artifacts/Mandelbrot.dll --cwd examples/mandelbrot --timeout 10 -- first "two words"

              fpas debug --protocol dap
            """);
        Command.Arguments.Add(program); Command.Arguments.Add(arguments);
        foreach (var option in new Option[] { protocol, cwd, breakpoints, expressions, stdin, timeout, outputLimit }) Command.Options.Add(option);
        Command.SetAction(Execute);
    }

    internal void Validate(ParseResult result, string[] raw)
    {
        if (result.Errors.Count != 0) return;
        if ((result.GetValue(arguments)?.Length ?? 0) != 0 && !raw.Contains("--", StringComparer.Ordinal))
            throw new ArgumentException("Pass target arguments after --.");
        if (result.GetValue(protocol) == "dap" && (result.GetValue(program) is not null || (result.GetValue(arguments)?.Length ?? 0) != 0 ||
            new Option[] { cwd, breakpoints, expressions, stdin, timeout, outputLimit }.Any(option => result.GetResult(option)?.Implicit == false)))
            throw new ArgumentException("DAP takes its target and launch options through the protocol; omit batch options and CLI target arguments.");
    }

    internal Task<int> ReportArgumentError(string message) => DebuggerApplication.ArgumentErrorAsync(message, Console.OpenStandardOutput());

    private async Task<int> Execute(ParseResult result, CancellationToken token)
    {
        string dotnet;
        try
        {
            var current = Environment.ProcessPath;
            dotnet = Environment.GetEnvironmentVariable("DOTNET_HOST_PATH") is null && current is not null &&
                Path.GetFileNameWithoutExtension(current).Equals("dotnet", StringComparison.OrdinalIgnoreCase) ? current : BuildProcess.DotnetHost();
        }
        catch (IOException failure) { return await ReportArgumentError(failure.Message); }
        if (result.GetValue(protocol) == "dap") return await DebuggerApplication.RunDapAsync(dotnet, Console.OpenStandardInput(), Console.OpenStandardOutput(), token);
        return await DebuggerApplication.RunBatchAsync(new(result.GetValue(program), result.GetValue(cwd), result.GetValue(breakpoints),
            result.GetValue(expressions), result.GetValue(stdin), result.GetValue(timeout), result.GetValue(outputLimit), result.GetValue(arguments)),
            dotnet, Console.OpenStandardOutput(), token);
    }
}
