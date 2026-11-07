using Fpas.Cli.Compilation;
using Fpas.Cli.Debugging;
using Fpas.Cli.Analysis;
using Fpas.Cli.Projects;
using Fpas.Cli.Formatting;
using Fpas.Cli.Testing;
using Fpas.Cli.Documentation;
using Fpas.Cli.Refactoring;
using Fpas.Cli.Toolchain;
using Fpas.Cli.LanguageServer;
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
        var debug = new DebugCommand();
        var check = new CheckCommand();
        var query = new QueryCommand();
        var init = new InitCommand(); var build = new ExecutionCommand(run: false); var run = new ExecutionCommand(run: true);
        var fmt = new FormatCommand();
        var test = new TestCommand();
        var doc = new DocCommand();
        var refactor = new RefactorCommand();
        var environment = new EnvironmentCommand();
        var lsp = new LspCommand();
        var sdk = new Fpas.Cli.Distribution.SdkCommand();
        var publish = new PublishCommand();
        var clean = new MaintenanceCommand("clean"); var restore = new MaintenanceCommand("restore");
        var profile = new Profiling.ProfileCommand(); var terminal = new Terminal.TerminalCommand();
        var schema = new Reporting.SchemaCommand();
        var diagnose = new DiagnoseCommand();
        var root = new Command("fpas", """
            FPAS compiler, project analysis, language server and agent debugger for .NET

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

              fpas doc examples/hello/Hello.fpasproj --name System.Console.WriteLine

              fpas refactor --help

              fpas env --json

              fpas sdk export ./feed

              fpas publish App/App.fpasproj --output artifacts/app

              fpas lsp

              fpas clean App/App.fpasproj

              fpas restore App.slnx

              fpas profile --help

              fpas terminal --help

              fpas schema operation

              fpas diagnose --help
            """);
        root.Options.Add(new HelpOption()); root.Options.Add(new VersionOption());
        root.Subcommands.Add(compile.Command); root.Subcommands.Add(debug.Command); root.Subcommands.Add(check.Command);
        root.Subcommands.Add(query.Command);
        root.Subcommands.Add(init.Command); root.Subcommands.Add(build.Command); root.Subcommands.Add(run.Command);
        root.Subcommands.Add(fmt.Command);
        root.Subcommands.Add(test.Command);
        root.Subcommands.Add(doc.Command);
        root.Subcommands.Add(refactor.Command);
        root.Subcommands.Add(environment.Command);
        root.Subcommands.Add(lsp.Command);
        root.Subcommands.Add(sdk.Command);
        root.Subcommands.Add(sdk.Licenses);
        root.Subcommands.Add(publish.Command);
        root.Subcommands.Add(clean.Command); root.Subcommands.Add(restore.Command);
        root.Subcommands.Add(profile.Command); root.Subcommands.Add(terminal.Command); root.Subcommands.Add(schema.Command);
        root.Subcommands.Add(diagnose.Command);
        var jsonRequested = false; var help = "fpas --help";
        ParseResult? checkResult = null;
        ParseResult? queryResult = null;
        ParseResult? projectResult = null;
        ParseResult? formatResult = null;
        ParseResult? testResult = null;
        ParseResult? docResult = null;
        ParseResult? refactorResult = null;
        ParseResult? environmentResult = null;
        ParseResult? debugResult = null;
        ParseResult? publishResult = null;
        var lspSelected = false;
        MaintenanceCommand? maintenance = null;
        ParseResult? maintenanceResult = null;
        var profileSelected = false; var terminalSelected = false; var schemaSelected = false;
        ParseResult? diagnoseResult = null;
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
            else if (parsed.CommandResult.Command == clean.Command || parsed.CommandResult.Command == restore.Command)
            { maintenance = parsed.CommandResult.Command == clean.Command ? clean : restore; maintenanceResult = parsed; }
            else if (profile.Owns(parsed.CommandResult.Command)) { profileSelected = true; profile.Validate(parsed, args); }
            else if (diagnose.Owns(parsed.CommandResult.Command)) { diagnoseResult = parsed; diagnose.Validate(parsed, args); }
            else if (terminal.Owns(parsed.CommandResult.Command)) { terminalSelected = true; }
            else if (parsed.CommandResult.Command == schema.Command) { schemaSelected = true; }
            else if (parsed.CommandResult.Command == publish.Command) { help = "fpas publish --help"; publishResult = parsed; }
            else if (parsed.CommandResult.Command == debug.Command) { help = "fpas debug --help"; debugResult = parsed; debug.Validate(parsed, args); }
            else if (parsed.CommandResult.Command == lsp.Command) { help = "fpas lsp --help"; lspSelected = true; }
            else if (parsed.CommandResult.Command == environment.Command) { help = "fpas env --help"; environmentResult = parsed; }
            else if (parsed.CommandResult.Command == test.Command) { help = "fpas test --help"; testResult = parsed; test.Validate(parsed, args); }
            else if (parsed.CommandResult.Command == doc.Command) { help = "fpas doc --help"; docResult = parsed; }
            else if (refactor.Owns(parsed.CommandResult.Command)) { help = "fpas refactor --help"; refactorResult = parsed; }
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
            if (maintenanceResult is not null) { maintenance!.ArgumentError(maintenanceResult, exception.Message); return 2; }
            if (profileSelected) return profile.ArgumentError(exception.Message);
            if (diagnoseResult is not null) return await diagnose.ArgumentError(diagnoseResult, exception.Message);
            if (terminalSelected) return terminal.ArgumentError(exception.Message);
            if (schemaSelected) return Reporting.OperationReport.Fail("schema", "invalid_argument", exception.Message + " See fpas schema --help.", 2);
            if (publishResult is not null) { publish.ArgumentError(publishResult, exception.Message); return 2; }
            if (lspSelected) { LspCommand.ArgumentError(exception.Message); return 2; }
            if (debugResult is not null) return await debug.ReportArgumentError(exception.Message);
            if (environmentResult is not null) { environment.ReportArgumentError(environmentResult, exception.Message); return 2; }
            if (testResult is not null) { test.ReportArgumentError(testResult, exception.Message); return 2; }
            if (docResult is not null) { doc.ReportArgumentError(docResult, exception.Message); return 2; }
            if (refactorResult is not null) { refactor.ReportArgumentError(refactorResult, exception.Message); return 2; }
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
