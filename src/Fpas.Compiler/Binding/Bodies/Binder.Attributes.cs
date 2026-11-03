using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;
using System.Reflection;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private IReadOnlyList<BoundAttribute> BindAttributes(IReadOnlyList<AttributeSyntax>? syntax, AttributeTargets target)
    {
        var result = new List<BoundAttribute>();
        foreach (var attribute in syntax ?? [])
        {
            var position = attribute.Type.Position;
            var type = BindType(attribute.Type);
            if (ContainsAttributeParameter(type)) throw source.Error("FP3035", "Attribute types must be closed CLR types.", position);
            var usage = references.AttributeUsage(type, source, position);
            if ((usage.Targets & target) == 0) throw source.Error("FP3035", $"Attribute '{type}' is not valid on the emitted {target} target.", position);
            if (!usage.Multiple && result.Any(other => other.Constructor.Owner.Erased == type.Erased))
                throw source.Error("FP3035", "This attribute does not allow multiple applications to the same target.", position);
            var positional = new List<ExpressionSyntax>(); var named = new List<NamedArgumentSyntax>();
            foreach (var argument in attribute.Arguments)
            {
                if (argument is NamedArgumentSyntax assignment) named.Add(assignment);
                else if (named.Count == 0) positional.Add(argument);
                else throw source.Error("FP3035", "Attribute constructor arguments precede named field/property assignments.", argument.Position);
            }
            var constructors = references.Methods(type, ".ctor", false).Where(method => method.Parameters.All(parameter => references.AttributeType(parameter)) &&
                method.ParameterInfo?.Any(parameter => parameter.ByRef) != true).ToArray();
            var (selected, values) = SelectInvocation(constructors, positional, true, position, inferGenerics: false);
            var supplied = new List<AttributeValue>();
            for (var slot = 0; slot < selected.Method.Parameters.Count; slot++)
            {
                var indices = selected.Slots.Select((targetSlot, index) => (targetSlot, index)).Where(item => item.targetSlot == slot).Select(item => item.index).ToArray();
                var parameterType = selected.Method.Parameters[slot];
                supplied.Add(selected.Expanded && slot == selected.Method.Parameters.Count - 1
                    ? new(parameterType, indices.Select(index => AttributeConstant(values[index], parameterType.Element!, positional[index].Position)).ToArray())
                    : indices.Length == 1 ? AttributeConstant(values[indices[0]], parameterType, positional[indices[0]].Position)
                    : AttributeConstant(selected.Method.Parameter(slot).Default!, parameterType, position));
            }
            var assignments = new List<NamedAttributeValue>(); var names = new HashSet<string>(StringComparer.Ordinal);
            foreach (var assignment in named)
            {
                if (!names.Add(assignment.Name.Text)) throw source.Error("FP3035", "Duplicate named attribute member.", assignment.Name.Position);
                var member = references.AttributeMember(type, assignment.Name.Text, source, assignment.Name.Position);
                var value = BindExpression(assignment.Value, expected: member.Type);
                assignments.Add(new(assignment.Name.Text, member.Type, AttributeConstant(value, member.Type, assignment.Value.Position), member.Field));
            }
            if (assignments.Count > ushort.MaxValue) throw source.Error("FP3035", "Too many named attribute arguments.", position);
            var bound = new BoundAttribute(selected.Method, supplied, assignments);
            if (type.Definition?.FullName == "System.Runtime.CompilerServices.MethodImplAttribute")
            {
                var flags = supplied.Count == 0 ? 0 : System.Convert.ToInt32(supplied[0].Value);
                const MethodImplAttributes supported = MethodImplAttributes.NoInlining | MethodImplAttributes.NoOptimization |
                    MethodImplAttributes.Synchronized | MethodImplAttributes.AggressiveInlining | MethodImplAttributes.AggressiveOptimization;
                if ((flags & ~(int)supported) != 0 || assignments.Any(member => member.Name == "MethodCodeType" && System.Convert.ToInt32(member.Value.Value) != 0))
                    throw source.Error("FP3035", "MethodImpl supports managed IL with inlining, optimization, and synchronization flags.", position);
            }
            result.Add(bound);
        }
        return result;
    }

    private void BindTypeAttributes()
    {
        Binder Context(IReadOnlyList<GenericParameterSymbol>? parameters = null, RecordSymbol? record = null)
        {
            var binder = new Binder(unit, catalog, references, recordContext: record)
                { currentRoutine = new("<attributes>", TypeSymbol.Void, [], false, false, -1, TypeParameters: parameters) };
            foreach (var import in imports) binder.imports.Add(import.Key, import.Value);
            return binder;
        }
        foreach (var record in unit.Records.Values)
        {
            var binder = Context(record: record);
            record.Attributes = binder.BindAttributes(record.Syntax.Attributes, AttributeTargets.Class);
            foreach (var field in record.Fields)
                field.Attributes = binder.BindPropertyAttributes(field.Syntax.Attributes, field.Type, field.Syntax.Name.Position);
        }
        foreach (var union in unit.Unions.Values)
            union.Attributes = Context(union.GenericParameters).BindAttributes(union.Syntax.Attributes, AttributeTargets.Class);
        foreach (var contract in unit.Interfaces.Values)
        {
            var binder = Context(contract.GenericParameters);
            contract.Attributes = binder.BindAttributes(contract.Syntax.Attributes, AttributeTargets.Interface);
            foreach (var property in contract.Properties)
                property.Attributes = binder.BindPropertyAttributes(property.Syntax.Attributes, property.Getter.Result, property.Syntax.Name.Position);
            foreach (var method in contract.Methods.Where(method => method.Declaration is not null))
            {
                binder.currentRoutine = method;
                contract.MethodAttributes.Add(method, binder.BindAttributes(method.Declaration!.Attributes, AttributeTargets.Method));
            }
        }
        foreach (var scalar in unit.Scalars.Values)
            scalar.Attributes = Context().BindAttributes(scalar.Syntax.Attributes, scalar.Syntax.Category == ScalarCategory.Enum ? AttributeTargets.Enum :
                scalar.Type.IsValue ? AttributeTargets.Struct : AttributeTargets.Class);
    }

    private IReadOnlyList<BoundAttribute> BindPropertyAttributes(IReadOnlyList<AttributeSyntax>? syntax, TypeSymbol type, int position)
    {
        var attributes = BindAttributes(syntax, AttributeTargets.Property);
        if (type.CanHaveNull && attributes.Any(attribute => attribute.Constructor.Owner.Definition?.FullName ==
            (type.NeedsNullCheck ? "System.Diagnostics.CodeAnalysis.NotNullAttribute" : "System.Diagnostics.CodeAnalysis.MaybeNullAttribute")))
            throw source.Error("FP3035", "A property attribute cannot contradict its declared nullable type; use the matching source type annotation.", position);
        return attributes;
    }

    private static bool ContainsAttributeParameter(TypeSymbol type) => type.IsParameter ||
        type.Element is not null && ContainsAttributeParameter(type.Element) || type.TypeArguments.Any(ContainsAttributeParameter);
}
