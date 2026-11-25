// [Documentation](../../../../docs/architecture/compiler.md#semantic-snapshots)
using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private readonly HashSet<PropertyReach> propertyReaches = [];

    private void RecordPropertyReaches(SourceText source, ExpressionSyntax syntax, BoundExpression value, string role)
    {
        var receiver = syntax switch
        {
            CallSyntax { Target: MemberSyntax target } when value is BoundCall { Receiver: not null } => target.Receiver,
            CallSyntax { Target: GenericNameSyntax { Name: MemberSyntax target } } when value is BoundCall { Receiver: not null } => target.Receiver,
            IndexSyntax index => index.Receiver,
            MemberSyntax member when role is "write" or "read_write" => member.Receiver,
            _ => null
        };
        for (var current = receiver; current is not null;)
        {
            if (current is MemberSyntax member &&
                occurrences.TryGetValue((source.Path, member.Name.Position, "read"), out var occurrence) &&
                occurrence.Symbol.Kind == "property")
                propertyReaches.Add(new(occurrence.Symbol.Id, occurrence.Span));

            current = current switch
            {
                MemberSyntax access => access.Receiver,
                IndexSyntax index => index.Receiver,
                ParenthesizedSyntax group => group.Expression,
                UncheckedSyntax uncheckedExpression => uncheckedExpression.Operand,
                _ => null
            };
        }
    }
}
