using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private readonly Dictionary<ExpressionSyntax, bool> operatorCalls = new(ReferenceEqualityComparer.Instance);

    private BoundCall? BindConstrainedOperator(string op, IReadOnlyList<BoundExpression> arguments, int position, bool check)
    {
        if (arguments.Any(argument => argument.Type.NullableValue)) return null;
        var owners = arguments.Select(argument => argument.Type.NonNullable).Where(type => type.IsParameter).Distinct().ToArray();
        if (owners.Length == 0) return null;
        var names = arguments.Count == 1 ? op switch
        {
            "+" => new[] { "op_UnaryPlus" }, "-" => ["op_UnaryNegation"], "not" => ["op_OnesComplement", "op_LogicalNot"], _ => []
        } : op switch
        {
            "+" => new[] { "op_Addition" }, "-" => ["op_Subtraction"], "*" => ["op_Multiply"], "/" or "div" => ["op_Division"],
            "mod" => ["op_Modulus"], "and" => ["op_BitwiseAnd"], "or" => ["op_BitwiseOr"], "xor" => ["op_ExclusiveOr"],
            "shl" => ["op_LeftShift"], "shr" => ["op_RightShift"], "=" => ["op_Equality"], "<>" => ["op_Inequality"],
            "<" => ["op_LessThan"], ">" => ["op_GreaterThan"], "<=" => ["op_LessThanOrEqual"], ">=" => ["op_GreaterThanOrEqual"], _ => []
        };
        foreach (var name in names)
        {
            var checkedName = check && name is "op_Addition" or "op_Subtraction" or "op_Multiply" or "op_Division" or "op_UnaryNegation"
                ? name.Replace("op_", "op_Checked", StringComparison.Ordinal) : null;
            foreach (var candidateName in checkedName is null ? new[] { name } : new[] { checkedName, name })
            {
                var methods = owners.SelectMany(owner => references.Methods(owner, candidateName, true))
                    .Where(method => method.ConstrainedType is not null && method.MethodArguments is null && method.Parameters.Count == arguments.Count &&
                        method.Result != TypeSymbol.Void && !method.ParameterInfo!.Any(parameter => parameter.ByRef) &&
                        (op is not ("=" or "<>" or "<" or ">" or "<=" or ">=") || method.Result == TypeSymbol.Boolean))
                    .DistinctBy(method => (method.Owner, method.DefinitionHandle, method.ConstrainedType)).ToArray();
                var selected = references.FindMethod(owners[0], candidateName, arguments.Select(argument => argument.Type).ToArray(), position, source,
                    optional: true, candidates: methods, accepts: (index, type) => Accepts(arguments[index], type));
                if (selected is null) continue;
                ForgetCapturedFacts();
                return new(selected, ConvertArguments(arguments, selected.Parameters, position));
            }
        }
        return null;
    }

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
