using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private bool bindingContract;
    private IReadOnlyList<BoundContract> BindContracts(IReadOnlyList<ContractSyntax>? syntax, bool ensures, out LocalSymbol? result)
    {
        result = null;
        var conditions = new List<BoundContract>();
        foreach (var clause in syntax ?? [])
        {
            if ((clause.Keyword.Text == "ensures") != ensures) continue;
            using var phase = catalog.Analysis?.Phase(ensures ? "ensures" : "requires");
            scopes.Push(new(StringComparer.Ordinal));
            try
            {
                bindingContract = true;
                if (clause.Result is { } binding)
                {
                    if (currentRoutine.BodyResult == TypeSymbol.Void || BindType(binding.Type) != currentRoutine.BodyResult)
                        throw source.Error("FP3027", "An ensures result annotation must match the function result type.", binding.Name.Position);
                    if (parameters.ContainsKey(binding.Name.Text) || binding.Name.Text == "self" || imports.ContainsKey(binding.Name.Text))
                        throw source.Error("FP3004", "An ensures result cannot shadow a parameter, self, or an import alias.", binding.Name.Position);
                    result ??= Allocate("<contract-result>", currentRoutine.BodyResult);
                    catalog.Analysis?.Local(result, source, binding.Name.Position);
                    scopes.Peek().Add(binding.Name.Text, result);
                }
                var condition = Condition(clause.Condition);
                if (!ContractExpression(condition))
                    throw source.Error("FP3027", "Contract conditions must use read-only built-in expressions and data reads; calls and arbitrary CLR getters are forbidden.", clause.Condition.Position);
                var location = source.StatementLocation(clause.Keyword.Position);
                var name = unit.Syntax.Name.Text + "." + (recordContext is null ? "" : recordContext.Name + ".") + currentRoutine.Name;
                conditions.Add(new(condition, $"Contract failed in {name}: {clause.Keyword.Text} {clause.Text} ({location.Path}:{location.Line}:{location.Column}).", location));
                SetFacts(Refine(clause.Condition, true, nonNull.ToHashSet()));
            }
            finally
            {
                bindingContract = false;
                foreach (var local in scopes.Pop().Values) nonNull.Remove(local);
            }
        }
        return conditions;
    }

    private bool ContractExpression(BoundExpression value) => value switch
    {
        BoundNull or BoundBoolean or BoundInteger or BoundDouble or BoundNumber or BoundCharacter or BoundString or
            BoundLocal or BoundParameter or BoundCapture or BoundFailure => true,
        BoundField field => field.Field.IsLiteral,
        BoundRecordField field => ContractExpression(field.Receiver),
        BoundArrayLength length => length.Receiver.Type.Rank == 1 && ContractExpression(length.Receiver),
        BoundArrayIndex index => ContractExpression(index.Receiver) && index.Indices.All(ContractExpression),
        BoundUnary unary => ContractExpression(unary.Operand),
        BoundBinary binary => ContractExpression(binary.Left) && ContractExpression(binary.Right),
        BoundConversion conversion => ContractExpression(conversion.Operand),
        BoundNullTest test => ContractExpression(test.Operand),
        BoundTypeTest test => ContractExpression(test.Value),
        BoundSafeCast cast => ContractExpression(cast.Value),
        BoundCoalesce coalesce => ContractExpression(coalesce.Left) && ContractExpression(coalesce.Right),
        BoundConditional conditional => conditional.Arms.All(arm => ContractExpression(arm.Condition) && ContractExpression(arm.Value)) && ContractExpression(conditional.Else),
        BoundSequence sequence => sequence.Bindings.All(binding => ContractExpression(binding.Initializer)) && ContractExpression(sequence.Value),
        BoundProperty property => ContractGetter(property.Property.Getter!) && property.Receiver is not null && ContractExpression(property.Receiver) && (property.Arguments ?? []).All(ContractExpression),
        BoundCall call => ContractGetter(call.Method) && (call.Receiver is null || ContractExpression(call.Receiver)) && call.Arguments.All(ContractExpression),
        _ => false
    };

    private bool ContractGetter(ImportedMethod method)
    {
        if (references.ScalarInfo(method.Owner) is { Category: not ScalarCategory.Enum })
            return method.Name is "get_Value" or "op_Equality" or "op_Inequality";
        if (method.Owner == TypeSymbol.String)
            return method.Name is "get_Length" or "get_Chars" or "op_Equality" or "op_Inequality" or "CompareOrdinal";
        if (method.Owner.NullableValue || method.Owner.Definition?.FullName == "System.Nullable`1")
            return method.Name is "get_HasValue" or "get_Value";
        if (method.IsStatic || method.Parameters.Count != 0) return false;
        if (references.RecordInfo(method.Owner) is { } record)
            return record.Fields.Any(field => SameGetter(field.Getter, method));
        var owner = method.Owner;
        var union = owner.Record?.UnionCase is { } member ? member.Owner.Type with { Arguments = owner.Arguments } : null;
        if (union is null && owner.Definition?.DeclaringType is { } parent)
            union = references.Symbol(parent) with { Arguments = owner.Arguments };
        return union is not null && references.UnionInfo(union)?.Cases.SelectMany(item => item.Fields).Any(field => SameGetter(field.Getter, method)) == true;
    }

    private static bool SameGetter(ImportedMethod left, ImportedMethod right)
        => left.Owner == right.Owner && left.Name == right.Name && left.DefinitionHandle == right.DefinitionHandle;
}
