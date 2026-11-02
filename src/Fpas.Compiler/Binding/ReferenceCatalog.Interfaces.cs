using System.Reflection;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    internal IReadOnlyList<(ImportedMethod Method, bool Required)> InterfaceRequirements(IReadOnlyList<TypeSymbol> contracts, SourceText source, int position)
    {
        var members = new List<(ImportedMethod, bool)>();
        var hierarchy = contracts.SelectMany(Hierarchy).Distinct().ToArray();
        foreach (var type in hierarchy)
        {
            var definition = Definition(type);
            foreach (var handle in definition.Definition.GetMethods())
            {
                var method = definition.Assembly.Reader.GetMethodDefinition(handle);
                var name = definition.Assembly.Reader.GetString(method.Name);
                var required = method.Attributes.HasFlag(MethodAttributes.Abstract);
                if (method.Attributes.HasFlag(MethodAttributes.Static))
                {
                    if (required) throw source.Error("FP3029", "Static abstract interface members cannot be implemented by records.", position);
                    continue;
                }
                if ((method.Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Public) continue;
                var imported = ReadMethod(type, handle, allowOpen: true);
                if (imported is null)
                {
                    if (required) throw source.Error("FP3029", $"Unsupported required interface signature '{type}.{name}'.", position);
                    continue;
                }
                if (method.Attributes.HasFlag(MethodAttributes.SpecialName) && (!name.StartsWith("get_", StringComparison.Ordinal) || imported.Parameters.Count != 0))
                {
                    if (required) throw source.Error("FP3029", "Record interface implementations support methods and getter-only non-indexed properties.", position);
                    continue;
                }
                var providers = new List<(TypeSymbol Owner, bool Abstract)> { (type, required) };
                foreach (var candidate in hierarchy)
                {
                    var candidateDefinition = Definition(candidate);
                    foreach (var implementationHandle in candidateDefinition.Definition.GetMethodImplementations())
                    {
                        var implementation = candidateDefinition.Assembly.Reader.GetMethodImplementation(implementationHandle);
                        if (!OverridesInterfaceMember(candidate, implementation.MethodDeclaration, imported)) continue;
                        var body = candidateDefinition.Assembly.Reader.GetMethodDefinition((System.Reflection.Metadata.MethodDefinitionHandle)implementation.MethodBody);
                        providers.Add((candidate, body.Attributes.HasFlag(MethodAttributes.Abstract)));
                    }
                }
                var mostSpecific = providers.Where(provider => !providers.Any(other => other.Owner != provider.Owner && Derives(other.Owner, provider.Owner))).ToArray();
                members.Add((imported, mostSpecific.Length != 1 || mostSpecific[0].Abstract));
            }
        }
        return members;
    }

    internal bool InterfaceSignature(RoutineSymbol body, TypeSymbol owner, ImportedMethod contract)
    {
        if (!body.IsPublic || !body.IsInstance || body.GenericParameters.Count != (contract.MethodArguments?.Count ?? 0)) return false;
        var arguments = body.GenericParameters.Count == 0 ? null : new TypeList(body.GenericParameters.Select(parameter => parameter.Type));
        var method = InstantiateSource(body, owner, arguments);
        contract = ReadMethod(contract.Owner, contract.DefinitionHandle, arguments)!;
        if (method.Result.Erased != contract.Result.Erased || method.Parameters.Count != contract.Parameters.Count) return false;
        if (!InterfaceNullability(method.Result, contract.Result)) return false;
        return method.Parameters.Select((type, index) => type.Erased == contract.Parameters[index].Erased &&
            (method.Parameter(index).ByRef ? method.Parameter(index).Mode : "") == (contract.Parameter(index).ByRef ? contract.Parameter(index).Mode : "") &&
            (method.Parameter(index).Mode == "out" ? InterfaceNullability(type, contract.Parameters[index]) :
                InterfaceNullability(contract.Parameters[index], type)) &&
            (method.Parameter(index).Mode != "ref" || InterfaceNullability(type, contract.Parameters[index]))).All(value => value);
    }

    private bool InterfaceNullability(TypeSymbol from, TypeSymbol to)
    {
        if (from.Kind == TypeKind.Void) return true;
        return CanAssign(from, to);
    }

    internal bool InterfaceConstraints(RoutineSymbol body, ImportedMethod contract)
    {
        var owner = Definition(contract.Owner);
        var definition = owner.Assembly.Reader.GetMethodDefinition(contract.DefinitionHandle);
        foreach (var handle in definition.GetGenericParameters())
        {
            var parameter = owner.Assembly.Reader.GetGenericParameter(handle);
            var target = body.GenericParameters[parameter.Index];
            var attributes = parameter.Attributes;
            var primary = attributes.HasFlag(GenericParameterAttributes.NotNullableValueTypeConstraint) ? "struct" :
                attributes.HasFlag(GenericParameterAttributes.ReferenceTypeConstraint) ? NullabilityMetadata.GenericNotNull(owner, parameter, definition) ? "class" : "class?" : null;
            if (target.Primary != primary || target.HasConstructor != (attributes.HasFlag(GenericParameterAttributes.DefaultConstructorConstraint) && primary != "struct")) return false;
            var expected = parameter.GetConstraints().Select(constraint => Decode(owner, owner.Assembly.Reader.GetGenericParameterConstraint(constraint).Type, contract.Owner, contract.MethodArguments))
                .Where(type => type.Definition?.FullName != "System.ValueType").Select(type => type.Erased).ToHashSet();
            var actual = target.Constraints.Select(type => SubstituteSource(type, body.GenericParameters, contract.MethodArguments!)).Select(type => type.Erased);
            if (!expected.SetEquals(actual)) return false;
        }
        return true;
    }
}
