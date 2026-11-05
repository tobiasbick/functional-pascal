using Fpas.Cli.Compilation;
using Fpas.Cli.Debugging;
using Fpas.Cli.Analysis;
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
        var root = new Command("fpas", """
            FPAS compiler, project analysis and agent debugger for .NET

            Examples:
              fpas compile --help

              fpas debug --help

              fpas check examples/mandelbrot/Mandelbrot.fpasproj

              fpas query --help
            """);
        root.Options.Add(new HelpOption()); root.Options.Add(new VersionOption());
        root.Subcommands.Add(compile.Command); root.Subcommands.Add(debug); root.Subcommands.Add(check.Command);
        root.Subcommands.Add(query.Command);
        var jsonRequested = false; var help = "fpas --help";
        ParseResult? checkResult = null;
        ParseResult? queryResult = null;
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
            else if (parsed.CommandResult.Command == check.Command) { help = "fpas check --help"; checkResult = parsed; }
            else if (query.Owns(parsed.CommandResult.Command)) { help = "fpas query --help"; queryResult = parsed; }
            if (parsed.Errors.Count > 0) throw new ArgumentException(string.Join(" ", parsed.Errors.Select(error => error.Message)));
            if (parsed.Action is HelpAction helpAction) return await CliHelp.WriteAsync(parsed, helpAction);
            return await parsed.InvokeAsync(new InvocationConfiguration { EnableDefaultExceptionHandler = false });
        }
        catch (ArgumentException exception) when (exception is not DecoderFallbackException)
        {
            if (checkResult is not null) { check.ReportArgumentError(checkResult, exception.Message); return 2; }
            if (queryResult is not null) return await query.ReportArgumentError(queryResult, exception.Message);
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
