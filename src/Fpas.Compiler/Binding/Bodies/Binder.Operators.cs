using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private readonly Dictionary<ExpressionSyntax, bool> operatorCalls = new(ReferenceEqualityComparer.Instance);

    private BoundCall? BindClrOperator(string op, IReadOnlyList<BoundExpression> arguments, int position, bool check, bool lifted)
    {
        if (arguments.Any(argument => argument.Type.NullableValue)) return null;
        var names = OperatorNames(op, arguments.Count);
        if (names.Length == 0) return null;
        var types = arguments.Select(argument => argument.Type).ToArray();
        var owners = types.Select(type => type.NonNullable).Distinct().ToArray();
        var comparison = op is "=" or "<>" or "<" or ">" or "<=" or ">=";
        var constrained = Bind(owners.Where(type => type.IsParameter).ToArray(), constrained: true);
        if (constrained is not null) return constrained;

        // Boolean short-circuit expressions bind the right operand with facts from the left.
        if (types[0] == TypeSymbol.Boolean && op is "and" or "or") return null;
        return Bind(owners.Where(type => type.Kind == TypeKind.Named ||
            comparison && type.Kind is TypeKind.Record or TypeKind.Union).ToArray(), constrained: false);

        BoundCall? Bind(TypeSymbol[] candidates, bool constrained)
        {
            if (candidates.Length == 0) return null;
            foreach (var name in names)
            {
                var checkedName = check && name is "op_Addition" or "op_Subtraction" or "op_Multiply" or "op_Division" or "op_UnaryNegation"
                    ? name.Replace("op_", "op_Checked", StringComparison.Ordinal) : null;
                foreach (var candidateName in checkedName is null ? new[] { name } : new[] { checkedName, name })
                {
                    var methods = candidates.SelectMany(owner => references.Methods(owner, candidateName, isStatic: true, operatorsOnly: !constrained))
                        .Where(method => (method.ConstrainedType is not null) == constrained && method.MethodArguments is null &&
                            method.Parameters.Count == arguments.Count && method.Result != TypeSymbol.Void &&
                            method.ParameterInfo?.Any(parameter => parameter.ByRef) != true &&
                            (!lifted || LiftableSignature(method)) && (!comparison || method.Result == TypeSymbol.Boolean))
                        .DistinctBy(method => (method.Owner, method.DefinitionHandle, method.SourceMethod, method.ConstrainedType)).ToArray();
                    if (methods.Length == 0) continue;
                    var selected = references.FindMethod(candidates[0], candidateName, types, position, source,
                        optional: true, candidates: methods, accepts: (index, type) => Accepts(arguments[index], type));
                    if (selected is null) continue;
                    ForgetCapturedFacts();
                    return new(selected, ConvertArguments(arguments, selected.Parameters, position));
                }
            }
            return null;
        }
    }

    private static string[] OperatorNames(string op, int arity) => arity == 1 ? op switch
        {
            "+" => ["op_UnaryPlus"], "-" => ["op_UnaryNegation"], "not" => ["op_OnesComplement", "op_LogicalNot"], _ => []
        } : op switch
        {
            "+" => ["op_Addition"], "-" => ["op_Subtraction"], "*" => ["op_Multiply"], "/" or "div" => ["op_Division"],
            "mod" => ["op_Modulus"], "and" => ["op_BitwiseAnd"], "or" => ["op_BitwiseOr"], "xor" => ["op_ExclusiveOr"],
            "shl" => ["op_LeftShift"], "shr" => ["op_RightShift"], "=" => ["op_Equality"], "<>" => ["op_Inequality"],
            "<" => ["op_LessThan"], ">" => ["op_GreaterThan"], "<=" => ["op_LessThanOrEqual"], ">=" => ["op_GreaterThanOrEqual"], _ => []
        };
    private static bool LiftableSignature(ImportedMethod method)
        => method.Result.IsValue && !method.Result.IsNullable && method.Parameters.All(type => type.IsValue && !type.IsNullable);
}
