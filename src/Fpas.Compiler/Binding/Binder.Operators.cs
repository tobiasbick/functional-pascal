using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private readonly Dictionary<ExpressionSyntax, bool> comparisonCalls = new(ReferenceEqualityComparer.Instance);

    private BoundExpression? BindComparisonOperator(BoundExpression left, string op, BoundExpression right, int position)
    {
        var name = op switch
        {
            "=" => "op_Equality", "<>" => "op_Inequality", "<" => "op_LessThan", ">" => "op_GreaterThan",
            "<=" => "op_LessThanOrEqual", ">=" => "op_GreaterThanOrEqual", _ => null
        };
        if (name is null || left.Type.NullableValue || right.Type.NullableValue) return null;
        var owners = new[] { left.Type.NonNullable, right.Type.NonNullable }
            .Where(type => type.Kind is TypeKind.Named or TypeKind.Record or TypeKind.Union).Distinct().ToArray();
        var methods = owners.SelectMany(owner => references.Methods(owner, name, true))
            .Where(method => method.Result == TypeSymbol.Boolean && method.Parameters.Count == 2)
            .DistinctBy(method => (method.Owner, method.DefinitionHandle, method.SourceMethod)).ToArray();
        if (methods.Length == 0) return null;
        var selected = references.FindMethod(owners[0], name, [left.Type, right.Type], position, source,
            optional: true, candidates: methods, accepts: (index, type) => Accepts(index == 0 ? left : right, type));
        if (selected is null) return null;
        ForgetCapturedFacts();
        return new BoundCall(selected, ConvertArguments([left, right], selected.Parameters, position));
    }
}
