using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using System.Reflection;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    internal RoutineSymbol DeclareRoutine(SourceUnit unit, RoutineSyntax syntax, ReferenceCatalog references, int index, RecordSymbol? owner = null,
        IReadOnlyList<GenericParameterSymbol>? enclosing = null)
    {
        var outer = enclosing ?? owner?.GenericParameters ?? [];
        var generic = CreateTypeParameters(unit, syntax.TypeParameters, outer);
        BindTypeParameterConstraints(unit, syntax.Name, generic, syntax.Constraints, references, outer);
        var scope = outer.Concat(generic).ToDictionary(parameter => parameter.Name, StringComparer.Ordinal);
        var parameters = syntax.Parameters.Select((parameter, slot) => new ParameterSymbol(parameter.Name.Text,
            ResolveType(parameter.Type, unit, references, scope), slot, parameter.Mode)).ToArray();
        ValidateParameterModes(parameters, syntax.Parameters, unit.Source);
        if (parameters.Select(parameter => parameter.Name).Distinct(StringComparer.Ordinal).Count() != parameters.Length ||
            parameters.Any(parameter => unit.Syntax.Imports.Any(import => import.Alias.Text == parameter.Name) || owner is not null && parameter.Name == "self"))
            throw unit.Source.Error("FP3004", "Parameters must be unique and cannot shadow imports or self.", syntax.Name.Position);
        return new(syntax.Name.Text, syntax.ResultType is null ? TypeSymbol.Void : ResolveType(syntax.ResultType, unit, references, scope),
            parameters, syntax.IsPublic, false, index, owner is not null && !syntax.IsStatic, owner, TypeParameters: generic, Declaration: syntax, IsAsync: syntax.IsAsync);
    }

    internal static void ValidateParameterModes(IReadOnlyList<ParameterSymbol> parameters, IReadOnlyList<ParameterSyntax> syntax, SourceText source)
    {
        var optional = false;
        for (var index = 0; index < parameters.Count; index++)
        {
            var parameter = parameters[index];
            if (parameter.Mode == "params" && (index != parameters.Count - 1 || parameter.Type.Kind != TypeKind.Array || parameter.Type.Rank != 1 || syntax[index].Default is not null) ||
                parameter.ByRef && syntax[index].Default is not null || optional && syntax[index].Default is null && parameter.Mode != "params")
                throw source.Error("FP3028", "Byref parameters cannot have defaults; params must be a final vector parameter; required parameters precede optional parameters.", syntax[index].Name.Position);
            optional |= syntax[index].Default is not null;
        }
    }

    private static IReadOnlyList<GenericParameterSymbol> CreateTypeParameters(SourceUnit unit, IReadOnlyList<Token>? names, IReadOnlyList<GenericParameterSymbol> outer)
    {
        var scope = new Dictionary<string, GenericParameterSymbol>(StringComparer.Ordinal);
        foreach (var name in names ?? [])
        {
            if (TypeFacts.PredefinedName(name.Text) || name.Text is "self" or "args" || unit.Syntax.Imports.Any(import => import.Alias.Text == name.Text) ||
                unit.HasType(name.Text) || outer.Any(parameter => parameter.Name == name.Text) ||
                !scope.TryAdd(name.Text, new(name.Text, scope.Count)))
                throw unit.Source.Error("FP3026", "Type parameters must be unique and cannot shadow types, imports, enclosing parameters, self, or args.", name.Position);
        }
        return scope.Values.ToArray();
    }

    private void BindTypeParameterConstraints(SourceUnit unit, Token name, IReadOnlyList<GenericParameterSymbol> parameters,
        IReadOnlyList<WhereSyntax>? constraints, ReferenceCatalog references, IReadOnlyList<GenericParameterSymbol>? outer = null, bool validate = true)
    {
        var scope = (outer ?? []).Concat(parameters).ToDictionary(parameter => parameter.Name, StringComparer.Ordinal);
        var clauses = new Dictionary<string, WhereSyntax>(StringComparer.Ordinal);
        foreach (var clause in constraints ?? [])
        {
            if (!scope.TryGetValue(clause.Parameter.Text, out var parameter) || !parameters.Contains(parameter) || !clauses.TryAdd(clause.Parameter.Text, clause))
                throw unit.Source.Error("FP3026", "A where clause requires a declared type parameter and may appear only once per parameter.", clause.Parameter.Position);
            for (var index = 0; index < clause.Constraints.Count; index++)
            {
                var constraint = clause.Constraints[index];
                if (constraint.Kind is "class" or "class?" or "struct")
                {
                    if (index != 0 || parameter.Primary is not null) throw unit.Source.Error("FP3026", "A primary class/struct constraint must appear first and cannot be combined with another primary constraint.", constraint.Position);
                    parameter.Primary = constraint.Kind;
                    parameter.IsReference = constraint.Kind != "struct";
                }
                if (constraint.Kind == "new")
                {
                    if (index != clause.Constraints.Count - 1 || parameter.Primary == "struct" || parameter.HasConstructor)
                        throw unit.Source.Error("FP3026", "new() must be last and cannot be combined with struct.", constraint.Position);
                    parameter.HasConstructor = true;
                }
            }
        }
        foreach (var clause in clauses.Values)
        {
            var parameter = scope[clause.Parameter.Text]; var hasBase = false;
            foreach (var syntaxConstraint in clause.Constraints.Where(constraint => constraint.Type is not null))
            {
                var type = ResolveType(syntaxConstraint.Type!, unit, references, scope, validate: false);
                var isInterface = type.IsInterface;
                var isEnumBase = type.Definition?.FullName == "System.Enum";
                if (!isInterface)
                {
                    if (hasBase || parameter.Constraints.Count != 0 || parameter.Primary is not null && !(parameter.Primary == "struct" && isEnumBase) ||
                        type.IsValue || type.Kind is TypeKind.Object or TypeKind.String or TypeKind.Array or TypeKind.Generated || type.Record is not null || type.Scalar is not null ||
                        type.Definition?.FullName is "System.ValueType" or "System.Array" ||
                        type.Definition?.Definition.Attributes.HasFlag(TypeAttributes.Sealed) == true || type.IsParameter && type.IsNullable)
                        throw unit.Source.Error("FP3026", "Invalid or incompatible base/type-parameter constraint.", syntaxConstraint.Position);
                    hasBase = true;
                }
                if (parameter.Constraints.Any(bound => bound.Erased == type.Erased))
                    throw unit.Source.Error("FP3026", "Duplicate generic constraints are invalid.", syntaxConstraint.Position);
                parameter.Constraints.Add(type);
            }
        }
        foreach (var parameter in parameters)
        {
            var visiting = new HashSet<GenericParameterSymbol>();
            void Check(GenericParameterSymbol current)
            {
                if (!visiting.Add(current)) throw unit.Source.Error("FP3026", "Type-parameter constraints cannot form a cycle.", name.Position);
                foreach (var bound in current.Constraints.Where(bound => bound.IsParameter))
                {
                    if (bound.Parameter!.Primary == "struct") throw unit.Source.Error("FP3026", "A struct-constrained parameter cannot be used as a base constraint.", name.Position);
                    Check(bound.Parameter);
                }
                visiting.Remove(current);
            }
            Check(parameter);
        }
        for (var pass = 0; pass <= scope.Count; pass++)
            foreach (var parameter in parameters)
            {
                parameter.IsReference |= parameter.Primary != "struct" && parameter.Constraints.Any(bound => bound.IsReference && bound.Definition?.FullName != "System.Enum" && !bound.IsInterface);
                parameter.IsNotNull |= parameter.Primary is "struct" or "class" || parameter.Constraints.Any(bound =>
                    bound.IsParameter ? bound.Parameter!.IsNotNull && !bound.IsNullable : bound.IsReference && !bound.IsNullable);
            }
        if (validate) ValidateTypeParameters(unit, name, parameters, references);
    }

    private static void ValidateTypeParameters(SourceUnit unit, Token name, IReadOnlyList<GenericParameterSymbol> parameters, ReferenceCatalog references)
    {
        foreach (var parameter in parameters)
            foreach (var bound in parameter.Constraints)
            {
                references.ValidateGenericType(bound, unit.Source, name.Position);
            }
    }
}
