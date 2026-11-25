// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler;
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Lint;

internal static class LonelyProtocols
{
    internal static IEnumerable<LintFinding> Find(string project, SemanticSnapshot snapshot, CancellationToken token)
    {
        var implementations = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        foreach (var relation in snapshot.Relations)
        {
            token.ThrowIfCancellationRequested();
            var contract = snapshot.Symbol(relation.Contract);
            var member = snapshot.Symbol(relation.Implementation);
            if (contract.Context?.Owner is not { } protocol || member.Context?.Owner is not { } record ||
                snapshot.Symbol(protocol).Kind != "interface" || snapshot.Symbol(record).Kind != "record") continue;
            if (!implementations.TryGetValue(protocol, out var records))
                implementations.Add(protocol, records = new(StringComparer.Ordinal));
            records.Add(record);
        }

        foreach (var (protocolId, records) in implementations)
        {
            token.ThrowIfCancellationRequested();
            var protocol = snapshot.Symbol(protocolId);
            if (records.Count != 1 || protocol is not
                { Declaration: { } span, Context: { Accessibility: "private", HasProtocolBases: false } context }) continue;
            var record = snapshot.Symbol(records.Single());
            if (record.Declaration is not { } implementation || span.Path != implementation.Path) continue;

            yield return new("lonely-protocol", $"Internal protocol '{protocol.Name}' has one known implementation, '{record.Name}'.",
                project, span, [LintFinding.Note("The implementation is declared here.", implementation)], context.QualifiedName,
                DiagnosticSeverity.Warning);
        }
    }
}
