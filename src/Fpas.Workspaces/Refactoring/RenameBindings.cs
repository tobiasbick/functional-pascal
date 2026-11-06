using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Text.RegularExpressions;

namespace Fpas.Workspaces.Refactoring;

internal sealed class RenameBindings
{
    private readonly SourceQuery query;
    private readonly Dictionary<string, string> types = new(StringComparer.Ordinal);
    private readonly Dictionary<(string Project, string Symbol), string> keys = [];
    private readonly Func<string, int, int> position;
    internal WorkspaceAnalysis Analysis { get; }
    internal SourceQuery Query => query;
    internal RenameBindings(WorkspaceAnalysis analysis, Func<string, int, int>? position = null)
    {
        if (!analysis.Success || !analysis.IsComplete) throw new ContextFailure("analysis_failed", "Rename requires successful, complete analysis of the affected context.");
        Analysis = analysis; query = new(analysis); this.position = position ?? ((_, offset) => offset);
        foreach (var project in analysis.Projects)
            foreach (var export in project.Result!.Snapshot!.Exports.Where(export => export.Identity.Kind == "type"))
            {
                var declaration = query.Resolve(project.Project, export.Symbol);
                if (declaration.Origin == "source") types[export.Identity.Assembly + "\0" + export.Identity.Type] = SourceKey(declaration);
            }
    }
    internal static string Address(string project, string path, int offset, string kind) => project + "\0" + path + "\0" + offset + "\0" + kind;
    private string SourceKey(DefinitionResult definition)
        => Address(definition.Project!, definition.Symbol.Declaration!.Path, position(definition.Symbol.Declaration.Path, definition.Symbol.Declaration.Offset), definition.Symbol.Kind);
    internal DefinitionResult Resolve(string project, string symbol) => query.Resolve(project, symbol);
    internal string Key(string project, string symbol)
    {
        if (keys.TryGetValue((project, symbol), out var key)) return key;
        var definition = Resolve(project, symbol);
        key = definition.Origin == "source" ? SourceKey(definition) : definition.Symbol.Metadata is { } identity ? Metadata(identity) :
            definition.Symbol.Kind == "namespace" ? "namespace:" + Normalize(definition.Symbol.Name) : "implicit:" + project + ":" + definition.Symbol.Kind + ":" + definition.Symbol.Name;
        keys.Add((project, symbol), key); return key;
    }
    private string Metadata(MetadataIdentity identity)
    {
        var owned = types.Keys.Any(key => key.StartsWith(identity.Assembly + "\0", StringComparison.Ordinal));
        return "metadata:" + identity.Assembly + ":" + identity.Version + ":" + identity.Culture + ":" + identity.PublicKeyToken + ":" +
            Normalize(identity.Type, identity.Assembly) + ":" + identity.Kind + ":" + Normalize(identity.Name, identity.Assembly) + ":" + identity.Arity + ":" +
            string.Join(';', identity.Parameters.Select(value => Normalize(value))) + ":" + Normalize(identity.Result ?? "") + (owned ? "" : ":" + identity.AssemblyHash);
    }
    private string Normalize(string text, string? assembly = null)
    {
        foreach (var (key, value) in types.OrderByDescending(pair => pair.Key.Length))
        {
            var split = key.IndexOf('\0'); if (assembly is not null && key[..split] != assembly) continue;
            var name = key[(split + 1)..];
            if (text.Contains(name, StringComparison.Ordinal))
                text = Regex.Replace(text, @"(?<![A-Za-z0-9_.])" + Regex.Escape(name) + @"(?=$|[+`<>,\[\]&])", _ => "{" + value + "}", RegexOptions.CultureInvariant);
        }
        return text;
    }

    internal Dictionary<(string Project, string Path, int Offset, string Role), string> Occurrences(CancellationToken token)
    {
        var result = new Dictionary<(string, string, int, string), string>();
        foreach (var project in Analysis.Projects)
            foreach (var occurrence in project.Result!.Snapshot!.Occurrences)
            {
                token.ThrowIfCancellationRequested();
                if (result.Count >= 100_000) throw new ContextFailure("rename_limit", "Rename validation exceeds 100,000 source occurrences.");
                result.Add((project.Project, occurrence.Span.Path, position(occurrence.Span.Path, occurrence.Span.Offset), occurrence.Role), Key(project.Project, occurrence.Symbol));
            }
        return result;
    }
    internal HashSet<(string Project, string Implementation, string Contract)> Relations(CancellationToken token)
    {
        var result = new HashSet<(string, string, string)>();
        foreach (var project in Analysis.Projects)
            foreach (var relation in project.Result!.Snapshot!.Relations)
            { token.ThrowIfCancellationRequested(); result.Add((project.Project, Key(project.Project, relation.Implementation), Key(project.Project, relation.Contract))); }
        return result;
    }
}
