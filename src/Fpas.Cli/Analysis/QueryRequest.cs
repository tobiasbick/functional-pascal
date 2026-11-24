// [Documentation](../../../docs/pascal/program-structure/cli.md#query-checked-sources)
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

[JsonUnmappedMemberHandling(JsonUnmappedMemberHandling.Disallow)]
internal sealed record QueryRequest(string Operation, string? File = null, int? Line = null, int? Column = null, string? Project = null,
    string? Name = null, string? Match = null, string[]? Kinds = null, int? Limit = null, int? Offset = null, bool? Body = null, int? From = null,
    string? Symbol = null)
{
    private NameSearch Search => new(Name!, Match ?? "contains", Kinds, Project, Limit ?? 100, Offset ?? 0);

    internal static bool AcceptsSourceResult(string operation) => operation is "inspect" or "refs";

    internal void Validate(int? requestIndex = null, bool snapshotPinned = false)
    {
        if (From is { } from && (!AcceptsSourceResult(Operation) || requestIndex is null || from < 0 || from >= requestIndex ||
            File is not null || Line is not null || Column is not null || Project is not null))
            throw new ContextFailure("invalid_argument", "from must select an earlier batch entry for inspect or refs and replaces file, line, column and project.");
        if (Symbol is not null && (!AcceptsSourceResult(Operation) || string.IsNullOrWhiteSpace(Symbol) || string.IsNullOrWhiteSpace(Project) ||
            !snapshotPinned || From is not null || File is not null || Line is not null || Column is not null))
            throw new ContextFailure("invalid_argument", "A symbol for inspect or refs requires its project and --expected-snapshot and replaces source coordinates.");
        switch (Operation)
        {
            case "find":
                if (string.IsNullOrEmpty(Name) || File is not null || Line is not null || Column is not null || Body is not null)
                    throw new ContextFailure("invalid_argument", "find requires name and accepts match, kinds, project, limit and offset.");
                SourceQuery.Validate(Search);
                break;
            case "definition" or "type" or "refs" or "inspect" or "callees" or "callers" or "complete" or "signature":
                if (From is null && Symbol is null && (string.IsNullOrWhiteSpace(File) || Line is not >= 1 || Column is not >= 1))
                    throw new ContextFailure("invalid_argument", "Supply a source file and positive one-based line and column.");
                if (Name is not null || Match is not null || Kinds is not null ||
                    Body is not null && Operation is not ("callees" or "callers") ||
                    Limit is not null && Operation is not ("refs" or "inspect" or "callers" or "callees") ||
                    Offset is not null && Operation is not ("refs" or "callees" or "callers"))
                    throw new ContextFailure("invalid_argument", $"Unexpected option for {Operation}.");
                if (Limit < 1 || Limit > (Operation == "inspect" ? 100 : 1000) || Offset < 0)
                    throw new ContextFailure("invalid_argument", "The query limit or offset is out of range.");
                break;
            default:
                throw new ContextFailure("invalid_argument", "Unknown source query operation.");
        }
    }

    internal SourceQueryResult Execute(WorkspaceAnalysis analysis, Lazy<SourceQuery> queries, CancellationToken token, DeclarationMatch? from = null)
    {
        if (Operation == "find") return queries.Value.Find(Search, token);
        if ((from?.Symbol ?? Symbol) is { } symbol)
            return queries.Value.Follow(Operation, from?.Project ?? Project!, symbol, Limit ?? (Operation == "inspect" ? 20 : 100), Offset ?? 0, token);
        var selection = new SourceSelection(File!, Line!.Value, Column!.Value, Project);
        if (Operation is "complete" or "signature") return SourceQuery.Assist(analysis, Operation, selection, token);
        var query = queries.Value;
        return Operation switch
        {
            "inspect" => query.Inspect(selection, Limit ?? 20, token),
            "callees" or "callers" => query.Calls(Operation, selection, Body ?? false, Limit ?? 100, Offset ?? 0, token),
            _ => query.Execute(Operation, selection, Limit ?? 100, token, Offset ?? 0)
        };
    }
}
