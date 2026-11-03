using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Syntax;
using System.Reflection;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal bool InterfaceGetter(ImportedMethod method) => method.SourceMethod is { } source
        ? source.Generated == GeneratedMethodKind.Getter
        : method.Name.StartsWith("get_", StringComparison.Ordinal) && Definition(method.Owner).Assembly.Reader
            .GetMethodDefinition(method.DefinitionHandle).Attributes.HasFlag(MethodAttributes.SpecialName);

    private ImportedProperty? SourceInterfaceProperty(TypeSymbol owner, string name, SourceText source, int position)
    {
        var declarations = new List<ImportedProperty>();
        foreach (var type in Hierarchy(owner).Where(type => type.IsInterface))
        {
            if (type.Interface is { } contract)
            {
                if (contract.Properties.SingleOrDefault(property => property.Syntax.Name.Text == name) is { } property)
                {
                    var getter = InstantiateSource(property.Getter, type, null);
                    declarations.Add(new(getter.Result, getter, null));
                }
            }
            else if (Property(type, name, false) is { } imported) declarations.Add(imported);
        }
        var unique = declarations.DistinctBy(property => (property.Getter?.Owner, property.Getter?.SourceMethod, property.Getter?.DefinitionHandle)).ToArray();
        var visible = unique.Where(property => !unique.Any(other => other.Getter?.Owner != property.Getter?.Owner &&
            other.Getter is not null && property.Getter is not null && Derives(other.Getter.Owner, property.Getter.Owner))).ToArray();
        if (visible.Length > 1) throw source.Error("FP3003", $"Ambiguous inherited interface property '{owner}.{name}'; select a declaring interface explicitly.", position);
        return visible.SingleOrDefault();
    }

    internal bool CompatibleInterfaceConstraints(ImportedMethod left, ImportedMethod right)
    {
        var first = InterfaceConstraintShapes(left);
        var second = InterfaceConstraintShapes(right);
        return first.Count == second.Count && first.Select((constraint, index) => constraint.Primary == second[index].Primary &&
            constraint.Constructor == second[index].Constructor && constraint.Bounds.SetEquals(second[index].Bounds)).All(equal => equal);
    }

    private sealed record InterfaceConstraintShape(string? Primary, bool Constructor, HashSet<TypeSymbol> Bounds);

    private IReadOnlyList<InterfaceConstraintShape> InterfaceConstraintShapes(ImportedMethod method)
    {
        InterfaceConstraintShape Shape(string? primary, bool constructor, IEnumerable<TypeSymbol> constraints)
        {
            var bounds = constraints.ToArray();
            var referenceBounds = bounds.Where(type => type.IsReference && !type.IsInterface).ToArray();
            if (primary != "struct" && referenceBounds.Length != 0)
                primary = primary == "class" || referenceBounds.Any(type => !type.NeedsNullCheck) ? "class" : "class?";
            return new(primary, constructor, bounds.Select(Normalize).ToHashSet());
        }
        TypeSymbol Normalize(TypeSymbol type)
        {
            if (type.IsParameter)
            {
                var slot = method.SourceMethod?.GenericParameters.ToList().IndexOf(type.Parameter!) ?? -1;
                var template = new TypeSymbol(slot >= 0 ? TypeKind.MethodParameter : TypeKind.TypeParameter, ParameterIndex: slot >= 0 ? slot : type.Parameter!.Index);
                return type.NullableValue ? Symbol(Required("System.Nullable`1")) with { Arguments = new([template]) } : template;
            }
            return type.Erased with { Element = type.Element is null ? null : Normalize(type.Element),
                Arguments = type.Arguments is null ? null : new(type.Arguments.Select(Normalize)) };
        }
        if (method.SourceMethod is { } source)
            return source.GenericParameters.Select(parameter => Shape(parameter.Primary, parameter.HasConstructor,
                parameter.Constraints.Select(type => SubstituteSource(type, SourceParameters(method.Owner), method.Owner.TypeArguments)))).ToArray();
        var owner = Definition(method.Owner);
        var definition = owner.Assembly.Reader.GetMethodDefinition(method.DefinitionHandle);
        return definition.GetGenericParameters().Select(handle =>
        {
            var parameter = owner.Assembly.Reader.GetGenericParameter(handle);
            var flags = parameter.Attributes;
            var primary = flags.HasFlag(GenericParameterAttributes.NotNullableValueTypeConstraint) ? "struct" :
                flags.HasFlag(GenericParameterAttributes.ReferenceTypeConstraint) ? NullabilityMetadata.GenericNotNull(owner, parameter, definition) ? "class" : "class?" : null;
            var bounds = parameter.GetConstraints().Select(constraint => Decode(owner, owner.Assembly.Reader.GetGenericParameterConstraint(constraint).Type,
                method.Owner, method.MethodArguments)).Where(type => type.Definition?.FullName != "System.ValueType");
            return Shape(primary, flags.HasFlag(GenericParameterAttributes.DefaultConstructorConstraint) && primary != "struct", bounds);
        }).ToArray();
    }
}
