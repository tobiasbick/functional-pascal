// [Documentation](../../../docs/pascal/program-structure/cli.md#query-checked-sources)
using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.CommandLine;
using System.Text;

namespace Fpas.Cli.Analysis;

internal sealed class QueryCommand
{
    private sealed class Operation(string name)
    {
        internal bool AcceptsStandalone => name == "complete";
        internal ContextCommandOptions Options { get; } = new();
        internal DiagnosticPageOptions Diagnostics { get; } = new();
        internal Option<string> ExpectedSnapshot { get; } = SnapshotGuard.CreateOption();
        internal Option<string> Symbol { get; } = SourceSymbolOptions.Create();
        internal Option<string> File { get; } = new("--file")
        {
            Required = name != "complete",
            Description = name == "complete"
                ? "Required source path for a project or solution. Defaults to the standalone context file."
                : "Source path in the evaluated context."
        };
        internal Option<int> Line { get; } = new("--line") { Required = true, Description = "One-based source line." };
        internal Option<int> Column { get; } = new("--column") { Required = true, Description = "One-based UTF-16 column; tabs count as one." };
        internal Option<string> Project { get; } = new("--project") { Description = "Owning .fpasproj when the selected source belongs to several compilations." };
        internal Option<int> Limit { get; } = new("--limit") { Description = "Maximum returned references (1..1000).", DefaultValueFactory = _ => 100 };
        internal Option<string> Name { get; } = new("--name") { Required = true, Description = "Literal semantic simple or source-qualified name, case-sensitive; escaped names omit @." };
        internal Option<string> Match { get; } = new("--match") { Description = "Name comparison: exact, prefix or contains.", DefaultValueFactory = _ => "contains" };
        internal Option<string[]> Kinds { get; } = new("--kind") { HelpName = "kind", Description = "Declaration kinds (repeatable); record fields use property.\nValues: " + string.Join(", ", SourceQuery.DeclarationKinds), Arity = ArgumentArity.OneOrMore, AllowMultipleArgumentsPerToken = false };
        internal Option<int> Offset { get; } = new("--offset") { Description = "Skip this many sorted matches; combine pages only with the same snapshotId.", DefaultValueFactory = _ => 0 };
        internal Option<bool> Body { get; } = new("--body") { Description = "Select the innermost executable source region instead of a callable target." };
        internal Command Command { get; } = new(name, $"""
            {Summary(name)}

            Return one JSON document. Preparation may restore packages and build dependencies.

            Run these examples from examples/numerics/mandelbrot.

            Examples:
              {(name == "find" ? "fpas query find Mandelbrot.fpasproj --name EscapeIterations --kind routine --limit 20" : $"fpas query {name} Mandelbrot.fpasproj --file main.fpas --line 7 --column 10")}{(name == "complete" ? "\n\n  fpas query complete main.fpas --line 7 --column 10" : "")}
            """);

        private static string Summary(string operation) => operation switch
        {
            "definition" => "Locate a symbol's checked declaration.",
            "type" => "Describe the checked type at a source position.",
            "refs" => "Find checked references to a symbol.",
            "find" => "Search checked FPAS declarations by name.",
            "inspect" => "Show a declaration, its documentation and directly relevant types.",
            "callees" => "List outgoing source operations owned by a callable or executable body.",
            "complete" => "Get compiler completion candidates at a source cursor.",
            "signature" => "Get checked overloads and the active argument at a call cursor.",
            "callers" => "Find incoming checked calls and callable references in the context.",
            _ => throw new ArgumentException("Unknown source query.", nameof(operation))
        };

        internal void Bind()
        {
            if (AcceptsStandalone) Options.Context.Description = "Explicit .fpas, .fpasproj or .slnx path.";
            Options.AddTo(Command);
            Diagnostics.AddTo(Command);
            Command.Options.Add(ExpectedSnapshot);
            if (name == "find")
            {
                Match.AcceptValues("exact", "prefix", "contains");
                Kinds.AcceptValues(SourceQuery.DeclarationKinds.ToArray());
                ContextCommandOptions.Range(Limit, 1, 1000); ContextCommandOptions.Range(Offset, 0, int.MaxValue);
                Limit.Description = "Maximum returned declarations (1..1000).";
                Project.Description = "Restrict search to this analyzed FPAS project; analysis still covers the context.";
                foreach (var option in new Option[] { Name, Match, Kinds, Project, Limit, Offset }) Command.Options.Add(option);
                Command.SetAction(async (result, token) => await Execute(this, result, token));
                return;
            }
            ContextCommandOptions.Range(Line, 1, int.MaxValue); ContextCommandOptions.Range(Column, 1, int.MaxValue);
            foreach (var option in new Option[] { File, Line, Column, Project }) Command.Options.Add(option);
            if (QueryRequest.AcceptsSourceResult(name)) SourceSymbolOptions.Bind(Command, Symbol, Project, ExpectedSnapshot, File, Line, Column);
            if (name is "callees" or "callers")
            {
                Limit.Description = "Maximum returned source sites (1..1000).";
                ContextCommandOptions.Range(Limit, 1, 1000); ContextCommandOptions.Range(Offset, 0, int.MaxValue);
                foreach (var option in new Option[] { Body, Limit, Offset }) Command.Options.Add(option);
            }
            if (name is "refs" or "inspect")
            {
                if (name == "inspect")
                {
                    Limit.DefaultValueFactory = _ => 20;
                    Limit.Description = "Maximum related source declarations (1..100); copied source and documentation share a 64 KiB UTF-8 budget.";
                }
                ContextCommandOptions.Range(Limit, 1, name == "inspect" ? 100 : 1000);
                Command.Options.Add(Limit);
                if (name == "refs")
                {
                    ContextCommandOptions.Range(Offset, 0, int.MaxValue);
                    Command.Options.Add(Offset);
                }
            }
            Command.SetAction(async (result, token) => await Execute(this, result, token));
        }
    }

    internal Command Command { get; } = new("query", """
        Inspect FPAS source structure, checked declarations, references and compiler candidates.
        Return one JSON document. Complete also accepts a standalone FPAS file.
        Run these examples from examples/numerics/mandelbrot.

        Examples:
          fpas query definition Mandelbrot.fpasproj --file main.fpas --line 7 --column 10

          fpas query type Mandelbrot.fpasproj --file main.fpas --line 7 --column 10

          fpas query refs Mandelbrot.fpasproj --file main.fpas --line 7 --column 10 --limit 100 --offset 0

          fpas query find Mandelbrot.fpasproj --name EscapeIterations --kind routine

          fpas query inspect Mandelbrot.fpasproj --file main.fpas --line 7 --column 10 --limit 20

          fpas query callees Mandelbrot.fpasproj --file main.fpas --line 7 --column 10

          fpas query callers Mandelbrot.fpasproj --file main.fpas --line 7 --column 10

          fpas query outline main.fpas

          fpas query batch Mandelbrot.fpasproj < queries.json
        """);
    private readonly Operation[] operations = [new("definition"), new("type"), new("refs"), new("find"), new("inspect"), new("callees"), new("callers"), new("complete"), new("signature")];

    private readonly QueryBatchCommand batch = new();
    private readonly OutlineCommand outline = new();

    internal QueryCommand()
    {
        foreach (var operation in operations) { operation.Bind(); Command.Subcommands.Add(operation.Command); }
        Command.Subcommands.Add(batch.Command);
        Command.Subcommands.Add(outline.Command);
    }

    internal bool Owns(Command command) => command == Command || command == batch.Command || command == outline.Command || operations.Any(operation => operation.Command == command);

    internal Task<int> ReportArgumentError(ParseResult result, string message)
    {
        if (result.CommandResult.Command == outline.Command) return outline.ArgumentError(result, message);
        if (result.CommandResult.Command == batch.Command) return batch.ReportArgumentError(result, message);
        var operation = operations.FirstOrDefault(operation => operation.Command == result.CommandResult.Command);
        return QueryResult.Failure(operation is null ? null : CommandArguments.Supplied(result, operation.Options.Context),
            operation is null ? "Debug" : CommandArguments.Supplied(result, operation.Options.Configuration) ?? "Debug", "invalid_argument", message + " See fpas query --help.").Report(2);
    }

    private static async Task<int> Execute(Operation operation, ParseResult result, CancellationToken cancellationToken)
    {
        var input = operation.Options.Read(result);
        var diagnostics = operation.Diagnostics.Read(result);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(operation.Options.Timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        CheckResult? check = null; QueryResult report; int exit;
        try
        {
            var name = operation.Command.Name;
            var standalone = operation.AcceptsStandalone && Path.GetExtension(input.Context).Equals(".fpas", StringComparison.OrdinalIgnoreCase);
            var request = name == "find"
                ? new QueryRequest(name, Project: result.GetValue(operation.Project), Name: result.GetValue(operation.Name),
                    Match: result.GetValue(operation.Match), Kinds: result.GetValue(operation.Kinds), Limit: result.GetValue(operation.Limit),
                    Offset: result.GetValue(operation.Offset))
                : new QueryRequest(name, result.GetValue(operation.File) ?? (standalone ? input.Context : null), Position(operation.Line), Position(operation.Column),
                    result.GetValue(operation.Project), Limit: name is "refs" or "inspect" or "callers" or "callees" ? result.GetValue(operation.Limit) : null,
                    Offset: name is "refs" or "callers" or "callees" ? result.GetValue(operation.Offset) : null,
                    Body: name is "callers" or "callees" ? result.GetValue(operation.Body) : null,
                    Symbol: QueryRequest.AcceptsSourceResult(name) ? result.GetValue(operation.Symbol) : null);
            request.Validate(snapshotPinned: result.GetValue(operation.ExpectedSnapshot) is not null);
            if (standalone)
            {
                if (request.Project is not null || diagnostics.Project is not null ||
                    result.GetResult(operation.Options.Configuration) is { Implicit: false })
                    throw new ContextFailure("invalid_argument", "Use a .fpasproj or .slnx with --project, --diagnostic-project or --configuration.");
                if (!Path.GetFullPath(request.File!).Equals(input.Context, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal))
                    throw new ContextFailure("invalid_argument", "--file must identify the standalone context file.");
                var captured = await StandaloneSource.OpenAsync(input.Context, combined.Token);
                var compilation = captured.Request(SyntaxAnalysis.StandaloneTarget(captured.Source, combined.Token));
                var analysis = FpasCompiler.Analyze(compilation, combined.Token);
                SnapshotGuard.Verify(result, operation.ExpectedSnapshot, analysis.Snapshot?.Id);
                check = CheckResult.FromSource(captured.Source, input.Configuration, analysis).PageDiagnostics(diagnostics);
                var document = new SourceDocument(captured.Source.Path, captured.Source.Text, combined.Token);
                var offset = document.Offset(request.Line!.Value, request.Column!.Value);
                var completion = EditorAnalysis.Complete(compilation, document.Path, offset, combined.Token);
                await captured.VerifyAsync(combined.Token);
                report = QueryResult.From(check, new("ok", null, new(null, document.Span(offset, 0)), null, null, null, false, [], Complete: completion));
            }
            else
            {
                var workspace = await FileWorkspace.OpenAsync(input, combined.Token);
                var analysis = workspace.Analysis!;
                SnapshotGuard.Verify(result, operation.ExpectedSnapshot, analysis.Context.SnapshotId);
                check = CheckResult.From(analysis).PageDiagnostics(diagnostics);
                var query = request.Execute(analysis, new(() => new SourceQuery(analysis)), combined.Token);
                await workspace.VerifyAsync(combined.Token);
                report = QueryResult.From(check, query);
            }
            exit = 0;

            int? Position(Option<int> coordinate) => result.GetResult(coordinate) is { IdentifierTokenCount: > 0 } ? result.GetValue(coordinate) : null;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            report = QueryResult.Failure(input.Context, input.Configuration, cancelled ? "cancelled" : "timeout", cancelled ? "Query cancelled." : "Query exceeded --timeout.", check);
            exit = cancelled ? 130 : 1;
        }
        catch (ContextFailure failure)
        {
            report = QueryResult.Failure(input.Context, input.Configuration, failure.Code, failure.Message, check) with { Error = AnalysisError.From(failure) };
            exit = failure.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException or ArgumentException or PlatformNotSupportedException)
        {
            var argument = exception is ArgumentException && exception is not DecoderFallbackException;
            report = QueryResult.Failure(input.Context, input.Configuration, argument ? "invalid_argument" : "context_failed", exception.Message, check); exit = argument ? 2 : 1;
        }
        return await report.Report(exit);
    }
}
