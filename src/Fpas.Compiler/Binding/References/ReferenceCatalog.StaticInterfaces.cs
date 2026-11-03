using Fpas.Compiler.Binding.Types;
using System.Reflection;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    private static ImportedMethod? ConstrainStatic(ImportedMethod? method, TypeSymbol owner)
    {
        if (method is not { IsStatic: true } || method.Owner.Definition?.Definition.Attributes.HasFlag(TypeAttributes.Interface) != true) return method;
        if (method.IsVirtual) return owner.IsParameter ? method with { ConstrainedType = owner.NonNullable } : null;
        return owner.IsParameter ? null : method;
    }

    private IReadOnlyList<ImportedMethod> MostSpecificStatic(IReadOnlyList<ImportedMethod> methods)
        => methods.Where(candidate => candidate.ConstrainedType is null || !methods.Any(other => other.ConstrainedType == candidate.ConstrainedType &&
            other.Owner != candidate.Owner && other.Name == candidate.Name && other.MethodArguments?.Count == candidate.MethodArguments?.Count &&
            other.DelegateParameters.Select(type => type.Erased).SequenceEqual(candidate.DelegateParameters.Select(type => type.Erased)) &&
            Derives(other.Owner, candidate.Owner))).ToArray();
}
