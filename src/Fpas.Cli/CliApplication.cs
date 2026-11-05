using Fpas.Cli.Compilation;
using Fpas.Cli.Debugging;
using Fpas.Cli.Analysis;
using Fpas.Cli.Projects;
using Fpas.Cli.Formatting;
using Fpas.Cli.Testing;
using System.CommandLine;
using System.CommandLine.Help;
using System.Globalization;
using System.Text;

namespace Fpas.Cli;

internal static class CliApplication
{
    internal static async Task<int> RunAsync(string[] args)
    {
        CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
        CultureInfo.CurrentUICulture = CultureInfo.InvariantCulture;
        Console.OutputEncoding = new UTF8Encoding(false);
        var configuration = new ParserConfiguration { ResponseFileTokenReplacer = null };
        var compile = CompilerCommand.Create();
        var debug = DebugCommand.Create();
        var check = new CheckCommand();
        var query = new QueryCommand();
        var init = new InitCommand(); var build = new ExecutionCommand(run: false); var run = new ExecutionCommand(run: true);
        var fmt = new FormatCommand();
        var test = new TestCommand();
        var root = new Command("fpas", """
            FPAS compiler, project analysis and agent debugger for .NET

            Examples:
              fpas compile --help

              fpas debug --help

              fpas check examples/mandelbrot/Mandelbrot.fpasproj

              fpas query --help

              fpas init project Hello --name Hello

              fpas build examples/hello/Hello.fpasproj

              fpas run examples/hello/Hello.fpasproj

              fpas fmt --check examples

              fpas test Tests/Tests.fpasproj
            """);
        root.Options.Add(new HelpOption()); root.Options.Add(new VersionOption());
        root.Subcommands.Add(compile.Command); root.Subcommands.Add(debug); root.Subcommands.Add(check.Command);
        root.Subcommands.Add(query.Command);
        root.Subcommands.Add(init.Command); root.Subcommands.Add(build.Command); root.Subcommands.Add(run.Command);
        root.Subcommands.Add(fmt.Command);
        root.Subcommands.Add(test.Command);
        var jsonRequested = false; var help = "fpas --help";
        ParseResult? checkResult = null;
        ParseResult? queryResult = null;
        ParseResult? projectResult = null;
        ParseResult? formatResult = null;
        ParseResult? testResult = null;
        ExecutionCommand? execution = null;
        try
        {
            var parsed = root.Parse(args.Length == 0 ? ["--help"] : args, configuration);
            if (parsed.CommandResult.Command == compile.Command)
            {
                help = "fpas compile --help";
                jsonRequested = compile.JsonRequested(parsed);
                var expanded = CompileResponseFile.Expand(parsed, compile, configuration);
                jsonRequested |= compile.JsonRequested(expanded);
                if (expanded != parsed && expanded.GetResult(compile.ResponseFile) is not null)
                    throw new ArgumentException("Response file nesting is unsupported.");
                parsed = expanded;
                compile.Validate(parsed);
            }
            else if (parsed.CommandResult.Command == debug) help = "fpas debug --help";
            else if (parsed.CommandResult.Command == test.Command) { help = "fpas test --help"; testResult = parsed; }
            else if (parsed.CommandResult.Command == fmt.Command) { help = "fpas fmt --help"; formatResult = parsed; fmt.Validate(parsed); }
            else if (parsed.CommandResult.Command == check.Command) { help = "fpas check --help"; checkResult = parsed; }
            else if (query.Owns(parsed.CommandResult.Command)) { help = "fpas query --help"; queryResult = parsed; }
            else if (init.Owns(parsed.CommandResult.Command)) { help = "fpas init --help"; projectResult = parsed; }
            else if (build.Owns(parsed.CommandResult.Command) || run.Owns(parsed.CommandResult.Command))
            {
                execution = run.Owns(parsed.CommandResult.Command) ? run : build;
                projectResult = parsed; help = "fpas " + execution.Command.Name + " --help";
                execution.Validate(parsed, args);
            }
            if (parsed.Errors.Count > 0) throw new ArgumentException(string.Join(" ", parsed.Errors.Select(error => error.Message)));
            if (parsed.Action is HelpAction helpAction) return await CliHelp.WriteAsync(parsed, helpAction);
            return await parsed.InvokeAsync(new InvocationConfiguration { EnableDefaultExceptionHandler = false });
        }
        catch (ArgumentException exception) when (exception is not DecoderFallbackException)
        {
            if (testResult is not null) { test.ReportArgumentError(testResult, exception.Message); return 2; }
            if (formatResult is not null) { fmt.ReportArgumentError(formatResult, exception.Message); return 2; }
            if (checkResult is not null) { check.ReportArgumentError(checkResult, exception.Message); return 2; }
            if (queryResult is not null) return await query.ReportArgumentError(queryResult, exception.Message);
            if (projectResult is not null)
            {
                if (execution is not null) execution.ReportArgumentError(projectResult, exception.Message);
                else init.ReportArgumentError(projectResult, exception.Message);
                return 2;
            }
            CompilerCommand.Report(jsonRequested, false, null, [new("FP0001", exception.Message, null) { Help = "See " + help + "." }]);
            return 2;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException)
        {
            CompilerCommand.Report(jsonRequested, false, null, [new("FP0002", exception.Message, null)]);
            return 1;
        }
    }
}
