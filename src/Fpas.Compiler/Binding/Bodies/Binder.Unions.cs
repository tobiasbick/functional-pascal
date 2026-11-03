using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private BoundExpression BindUnionFactory(UnionCaseInfo member, IReadOnlyList<ExpressionSyntax> arguments, bool check, int position, IReadOnlyList<BoundExpression>? bound = null)
    {
        if (member.Fields.Count == 0) throw source.Error("FP3023", "A fieldless union case is a value property and has no parentheses.", position);
        var (bindings, values) = BindDataArguments(arguments, member.Fields.Select(field => (field.Name, field.Type, (ImportedMethod?)null)).ToArray(), check, position, bound);
        ForgetCapturedFacts();
        return new BoundSequence(bindings, new BoundCall(member.Factory, values));
    }

    private void BindUnion(UnionSymbol union)
    {
        var self = new BoundParameter(new("self", union.Type, -1));
        RoutineSymbol Method(GeneratedMethodKind kind) => union.Method(kind);
        static BoundParameter Parameter(RoutineSymbol method, int index) => new(method.Parameters[index]);
        void Body(RoutineSymbol method, params BoundStatement[] statements) => union.Bodies.Add(method, new(method, new(statements), []));
        static BoundIf When(BoundExpression condition, bool value) => new([new(condition, new([new BoundReturn(new BoundBoolean(value))]))], new([]));
        var typed = Method(GeneratedMethodKind.EqualsTyped);
        var objectEquals = references.FindMethod(TypeSymbol.Object, "Equals", [TypeSymbol.Object with { IsNullable = true }], 0, source, false);
        Body(typed, new BoundReturn(new BoundCall(objectEquals, [new BoundConversion(Parameter(typed, 0), TypeSymbol.Object with { IsNullable = true }, false)], self)));
        var equality = Method(GeneratedMethodKind.Equality);
        var left = Parameter(equality, 0); var right = Parameter(equality, 1);
        Body(equality, When(new BoundBinary(left, "=", right, TypeSymbol.Boolean, false), true), When(new BoundNullTest(left, true), false),
            new BoundReturn(new BoundCall(union.Import(typed), [right], left)));
        var inequality = Method(GeneratedMethodKind.Inequality);
        Body(inequality, new BoundReturn(new BoundUnary("not", new BoundCall(union.Import(equality), [Parameter(inequality, 0), Parameter(inequality, 1)]), TypeSymbol.Boolean, false)));
        foreach (var member in union.Cases)
            Body(member.Factory, new BoundReturn(new BoundConversion(new BoundNew(member.Representation.Type, member.Representation.Method(member.Representation.Constructor),
                member.Factory.Parameters.Select(parameter => (BoundExpression)new BoundParameter(parameter)).ToArray()), union.Type, false)));
    }

    private (IReadOnlyList<BoundBinding> Bindings, BoundExpression[] Values) BindDataArguments(IReadOnlyList<ExpressionSyntax> arguments,
        IReadOnlyList<(string Name, TypeSymbol Type, ImportedMethod? Default)> fields, bool check, int position, IReadOnlyList<BoundExpression>? bound = null)
    {
        var bindings = new List<BoundBinding>();
        var values = new BoundExpression?[fields.Count];
        var supplied = DataArguments(arguments, fields.Select(field => field.Name).ToArray(), position);
        for (var index = 0; index < supplied.Count; index++)
        {
            var (slot, expression) = supplied[index];
            var field = fields[slot];
            values[slot] = Save(Convert(bound?[index] ?? BindExpression(expression, check, field.Type), field.Type, expression.Position), bindings);
        }
        for (var index = 0; index < values.Length; index++)
        {
            if (values[index] is not null) continue;
            var provider = fields[index].Default;
            if (provider is null) throw source.Error("FP3022", $"Missing required field '{fields[index].Name}'.", position);
            values[index] = Save(new BoundCall(provider, []), bindings);
        }
        return (bindings, values.Select(value => value!).ToArray());
    }

    private TypeSymbol ImportedValueType(ImportTarget? target, int position, bool allowOpen = false)
    {
        var type = target switch
        {
            RecordImport record => record.Record.Type, UnionImport union => union.Union.Type,
            InterfaceImport contract => contract.Interface.Type,
            ClrTypeImport closed => closed.Type, ClrImport clr => references.Symbol(clr.Type),
            _ => TypeSymbol.Unsupported
        };
        if (!allowOpen && (target is RecordImport or UnionImport or InterfaceImport && ReferenceCatalog.SourceParameters(type).Count != 0 ||
            type.Definition is { } definition && definition.Definition.GetGenericParameters().Count != type.TypeArguments.Count))
            throw source.Error("FP3026", "A generic type receiver requires explicit type arguments.", position);
        return type;
    }
}
