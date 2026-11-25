// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Lint;

internal static class ExportedSynchronization
{
    private static readonly HashSet<string> Primitives = new(StringComparer.Ordinal)
    {
        "System.Threading.Mutex", "System.Threading.Semaphore", "System.Threading.SemaphoreSlim",
        "System.Threading.ReaderWriterLockSlim", "System.Threading.AutoResetEvent", "System.Threading.ManualResetEvent",
        "System.Threading.ManualResetEventSlim", "System.Threading.CountdownEvent", "System.Threading.Barrier",
        "System.Threading.CancellationTokenSource", "System.Threading.Tasks.TaskCompletionSource",
        "System.Threading.Tasks.TaskCompletionSource`1", "System.Threading.Channels.Channel`1",
        "System.Threading.Channels.Channel`2", "System.Threading.Channels.ChannelWriter`1"
    };

    internal static IEnumerable<LintFinding> Find(string project, SemanticSnapshot snapshot, CancellationToken token)
    {
        var types = snapshot.Types.ToDictionary(type => type.Id, StringComparer.Ordinal);
        foreach (var field in snapshot.Symbols)
        {
            token.ThrowIfCancellationRequested();
            if (field is not { Kind: "property", Type: { } id, Declaration: { } span, Context: { Accessibility: "public", Owner: { } owner } context }
                || snapshot.Symbol(owner) is not { Kind: "record", Context.Accessibility: "public" }
                || types[id] is not { IsReference: true, Metadata.Type: { } type } || !Primitives.Contains(type)) continue;

            yield return new("exported-sync", $"Public field '{context.QualifiedName}' exposes synchronization through '{types[id].Display}'.",
                project, span, [], context.QualifiedName);
        }
    }
}
