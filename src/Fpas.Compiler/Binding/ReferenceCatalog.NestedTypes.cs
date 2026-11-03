using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    internal ImportTarget NestedType(TypeSymbol owner, Token name, IReadOnlyList<TypeSymbol>? arguments, SourceText source,
        bool validate, int inferredArity = 0, bool allowOpenGeneric = false)
    {
        var definition = owner.Definition!;
        var arity = arguments?.Count ?? inferredArity;
        var fullName = definition.FullName + "+" + name.Text + (arity == 0 ? "" : "`" + arity);
        var nested = allowOpenGeneric && arity == 0 && !HasType(fullName)
            ? FindOpenType(fullName, name.Position, source) : FindType(fullName, name.Position, source);
        if (nested.Assembly != definition.Assembly || nested.DeclaringType != definition)
            throw source.Error("FP4002", $"Nested type '{name.Text}' does not belong to '{definition.FullName}'.", name.Position);
        if (arguments is null && arity != 0 || arguments is null && allowOpenGeneric && nested.Definition.GetGenericParameters().Count != owner.TypeArguments.Count)
            return new ClrImport(nested);
        var allArguments = owner.TypeArguments.Concat(arguments ?? []).ToArray();
        return allArguments.Length == 0 ? new ClrImport(nested) : new ClrTypeImport(Construct(nested, allArguments, source, name.Position, validate));
    }
}
