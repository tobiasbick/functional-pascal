// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Lint;

internal sealed class ArchitectureGraph
{
    private readonly Dictionary<string, Dictionary<string, SourceSpan>> outgoing = new(StringComparer.Ordinal);

    internal bool Add(string from, string to, SourceSpan use)
    {
        if (from == to) return false;
        if (!outgoing.TryGetValue(from, out var edges)) outgoing.Add(from, edges = new(StringComparer.Ordinal));
        if (!outgoing.ContainsKey(to)) outgoing.Add(to, new(StringComparer.Ordinal));
        return edges.TryAdd(to, use);
    }

    internal IEnumerable<(string From, string To, SourceSpan Use)> Edges(HashSet<string> component) =>
        component.Order(StringComparer.Ordinal).SelectMany(from => outgoing[from].OrderBy(edge => edge.Key, StringComparer.Ordinal)
            .Where(edge => component.Contains(edge.Key)).Select(edge => (from, edge.Key, edge.Value)));

    // Two iterative traversals find strongly connected groups without using the process stack.
    internal IEnumerable<HashSet<string>> Cycles(CancellationToken token)
    {
        var incoming = outgoing.Keys.ToDictionary(node => node, _ => new List<string>(), StringComparer.Ordinal);
        foreach (var (from, edges) in outgoing)
            foreach (var to in edges.Keys) incoming[to].Add(from);

        var visited = new HashSet<string>(StringComparer.Ordinal);
        var finished = new Stack<string>();
        var search = new Stack<(string Node, bool Exit)>();
        foreach (var root in outgoing.Keys)
        {
            search.Push((root, false));
            while (search.TryPop(out var entry))
            {
                token.ThrowIfCancellationRequested();
                if (entry.Exit) finished.Push(entry.Node);
                else if (visited.Add(entry.Node))
                {
                    search.Push((entry.Node, true));
                    foreach (var next in outgoing[entry.Node].Keys) search.Push((next, false));
                }
            }
        }

        visited.Clear();
        var reverse = new Stack<string>();
        while (finished.TryPop(out var root))
        {
            if (!visited.Add(root)) continue;
            var component = new HashSet<string>(StringComparer.Ordinal);
            reverse.Push(root);
            while (reverse.TryPop(out var node))
            {
                token.ThrowIfCancellationRequested();
                component.Add(node);
                foreach (var next in incoming[node])
                    if (visited.Add(next)) reverse.Push(next);
            }
            if (component.Count > 1) yield return component;
        }
    }
}
