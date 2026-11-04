using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.CommandLine;
using System.Text;

namespace Fpas.Cli.Analysis;

internal sealed class QueryCommand
{
    private sealed class Operation(string name)
    {
        internal ContextCommandOptions Options { get; } = new();
        internal Option<string> File { get; } = new("--file") { Required = true, Description = "Source path in the evaluated context." };
        internal Option<int> Line { get; } = new("--line") { Required = true, Description = "One-based source line." };
        internal Option<int> Column { get; } = new("--column") { Required = true, Description = "One-based UTF-16 column; tabs count as one." };
        internal Option<string> Project { get; } = new("--project") { Description = "Owning .fpasproj when the selected source belongs to several compilations." };
        internal Option<int> Limit { get; } = new("--limit") { Description = "Maximum returned references (1..1000).", DefaultValueFactory = _ => 100 };
        internal Option<string> Name { get; } = new("--name") { Required = true, Description = "Literal semantic simple or source-qualified name, case-sensitive; escaped names omit @." };
        internal Option<string> Match { get; } = new("--match") { Description = "Name comparison: exact, prefix or contains.", DefaultValueFactory = _ => "contains" };
        internal Option<string[]> Kinds { get; } = new("--kind") { Description = "Declaration kinds (repeatable); record fields are property. Values: " + string.Join(", ", SourceQuery.DeclarationKinds), Arity = ArgumentArity.OneOrMore, AllowMultipleArgumentsPerToken = false };
        internal Option<int> Offset { get; } = new("--offset") { Description = "Skip this many sorted matches; combine pages only with the same snapshotId.", DefaultValueFactory = _ => 0 };
        internal Command Command { get; } = new(name, $"""
            Query checked source {name}; returns one JSON document. Preparation may restore/build dependencies.

            Examples:
              {(name == "find" ? "fpas query find Game.slnx --name Message --kind routine --limit 20" : $"fpas query {name} Game.slnx --file Game/Program.fpas --line 12 --column 9")}
            """);

        internal void Bind()
        {
            Options.AddTo(Command);
            if (name == "find")
            {
                Match.AcceptOnlyFromAmong("exact", "prefix", "contains");
                Kinds.AcceptOnlyFromAmong(SourceQuery.DeclarationKinds.ToArray());
                ContextCommandOptions.Range(Limit, 1, 1000); ContextCommandOptions.Range(Offset, 0, int.MaxValue);
                Limit.Description = "Maximum returned declarations (1..1000).";
                Project.Description = "Restrict search to this analyzed FPAS project; analysis still covers the context.";
                foreach (var option in new Option[] { Name, Match, Kinds, Project, Limit, Offset }) Command.Options.Add(option);
                Command.SetAction(async (result, token) => await Execute(this, result, token));
                return;
            }
            ContextCommandOptions.Range(Line, 1, int.MaxValue); ContextCommandOptions.Range(Column, 1, int.MaxValue);
            foreach (var option in new Option[] { File, Line, Column, Project }) Command.Options.Add(option);
            if (name is "refs" or "inspect")
            {
                if (name == "inspect")
                {
                    Limit.DefaultValueFactory = _ => 20;
                    Limit.Description = "Maximum related source declarations (1..100); copied source and documentation share a 64 KiB UTF-8 budget.";
                }
                ContextCommandOptions.Range(Limit, 1, name == "inspect" ? 100 : 1000); Command.Options.Add(Limit);
            }
            Command.SetAction(async (result, token) => await Execute(this, result, token));
        }
    }

    internal Command Command { get; } = new("query", """
        Inspect fully checked project/solution sources without executing the application.
        Uses explicit on-disk context and exact source positions; queries return JSON only.

        Examples:
          fpas query definition Game.slnx --file Game/Program.fpas --line 12 --column 9
          fpas query type Game/Game.fpasproj --file Game/Program.fpas --line 12 --column 9
          fpas query refs Game.slnx --file Library/Math.fpas --line 4 --column 17 --limit 100
          fpas query find Game.slnx --name Message --match contains --kind routine
          fpas query inspect Game.slnx --file Game/Program.fpas --line 12 --column 9 --limit 20
        """);
    private readonly Operation[] operations = [new("definition"), new("type"), new("refs"), new("find"), new("inspect")];

    internal QueryCommand()
    {
        foreach (var operation in operations) { operation.Bind(); Command.Subcommands.Add(operation.Command); }
    }

    internal bool Owns(Command command) => command == Command || operations.Any(operation => operation.Command == command);

    internal Task<int> ReportArgumentError(ParseResult result, string message)
    {
        var operation = operations.FirstOrDefault(operation => operation.Command == result.CommandResult.Command);
        return QueryResult.Failure(operation is null ? null : result.GetValue(operation.Options.Context),
            operation is null ? "Debug" : result.GetValue(operation.Options.Configuration) ?? "Debug", "invalid_argument", message + " See fpas query --help.").Report(2);
    }

    private static async Task<int> Execute(Operation operation, ParseResult result, CancellationToken cancellationToken)
    {
        var input = operation.Options.Read(result);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(operation.Options.Timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        CheckResult? check = null; QueryResult report; int exit;
        try
        {
            var search = operation.Command.Name == "find" ? new NameSearch(result.GetValue(operation.Name)!, result.GetValue(operation.Match)!,
                result.GetValue(operation.Kinds), result.GetValue(operation.Project), result.GetValue(operation.Limit), result.GetValue(operation.Offset)) : null;
            if (search is not null) SourceQuery.Validate(search);
            var workspace = await WorkspaceLoader.LoadAsync(input, combined.Token);
            var analysis = WorkspaceAnalysis.Analyze(workspace, combined.Token); check = CheckResult.From(analysis);
            var module = new SourceQuery(analysis);
            SourceQueryResult query;
            if (search is not null) query = module.Find(search, combined.Token);
            else
            {
                var selection = new SourceSelection(result.GetValue(operation.File)!, result.GetValue(operation.Line), result.GetValue(operation.Column), result.GetValue(operation.Project));
                query = operation.Command.Name == "inspect" ? module.Inspect(selection, result.GetValue(operation.Limit), combined.Token)
                    : module.Execute(operation.Command.Name, selection, operation.Command.Name == "refs" ? result.GetValue(operation.Limit) : 100, combined.Token);
            }
            await workspace.VerifyInputsAsync(combined.Token); combined.Token.ThrowIfCancellationRequested();
            report = QueryResult.From(check, query); exit = 0;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            report = QueryResult.Failure(input.Context, input.Configuration, cancelled ? "cancelled" : "timeout", cancelled ? "Query cancelled." : "Query exceeded --timeout.", check);
            exit = cancelled ? 130 : 1;
        }
        catch (ContextFailure failure)
        {
            report = QueryResult.Failure(input.Context, input.Configuration, failure.Code, failure.Message, check, failure.OutputTruncated, failure.Candidates);
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
