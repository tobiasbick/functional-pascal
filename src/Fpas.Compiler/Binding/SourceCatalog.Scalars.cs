using System.Numerics;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class SourceCatalog
{
    private void DeclareScalarNames()
    {
        foreach (var unit in Units)
        {
            var names = unit.Syntax.Records.Select(type => type.Name.Text).Concat(unit.Syntax.Unions.Select(type => type.Name.Text))
                .Concat(unit.Syntax.Imports.Select(import => import.Alias.Text)).ToHashSet(StringComparer.Ordinal);
            void Name(Token name)
            {
                if (TypeFacts.PredefinedName(name.Text) || !names.Add(name.Text)) throw unit.Source.Error("FP3031", "A type name is duplicated or reserved.", name.Position);
            }
            foreach (var syntax in unit.Syntax.Aliases ?? []) { Name(syntax.Name); unit.Aliases.Add(syntax.Name.Text, new(unit, syntax)); }
            foreach (var syntax in unit.Syntax.Scalars ?? [])
            {
                Name(syntax.Name); var scalar = new ScalarSymbol(unit, syntax); unit.Scalars.Add(scalar.Name, scalar);
                if (syntax.Category == ScalarCategory.Enum && (syntax.Members.Count == 0 || syntax.Members.Select(member => member.Name.Text).Distinct(StringComparer.Ordinal).Count() != syntax.Members.Count || syntax.Members.Any(member => member.Name.Text == "value__")))
                    throw unit.Source.Error("FP3031", "Enum members must be nonempty, unique, and cannot use value__.", syntax.Name.Position);
            }
        }
    }

    private ImportTarget? ResolveScalarPath(SourceUnit owner, string name, SourceUnit? access, ReferenceCatalog references, SourceText source, int position)
    {
        if (owner.Scalars.TryGetValue(name, out var scalar))
        {
            if (!scalar.IsPublic && access != owner) throw source.Error("FP3011", "The scalar type is private to its unit.", position);
            return new ClrTypeImport(scalar.Type);
        }
        if (!owner.Aliases.TryGetValue(name, out var alias)) return null;
        if (!alias.Syntax.IsPublic && access != owner) throw source.Error("FP3011", "The alias is private to its unit.", position);
        return new ClrTypeImport(ResolveAlias(alias, references));
    }

    private TypeSymbol ResolveAlias(AliasSymbol alias, ReferenceCatalog references)
    {
        if (alias.Target is not null) return alias.Target;
        if (alias.Resolving) throw alias.Owner.Source.Error("FP3031", "Transparent aliases cannot form a cycle.", alias.Syntax.Name.Position);
        alias.Resolving = true;
        try { return alias.Target = ResolveType(alias.Syntax.Target, alias.Owner, references); }
        finally { alias.Resolving = false; }
    }

    private void DeclareScalarBases(ReferenceCatalog references)
    {
        foreach (var scalar in Units.SelectMany(unit => unit.Scalars.Values))
        {
            var type = ResolveType(scalar.Syntax.Base, scalar.Owner, references);
            var valid = scalar.Syntax.Category == ScalarCategory.Distinct
                ? TypeFacts.Numeric(type) || type == TypeSymbol.Boolean || type == TypeSymbol.String
                : NumericFacts.Integral(type) && type.Kind is not (TypeKind.NativeInt or TypeKind.NativeUInt or TypeKind.Char);
            if (type.IsNullable || !valid)
                throw scalar.Owner.Source.Error("FP3031", "Invalid primitive base for a distinct, subrange, or enum declaration.", scalar.Syntax.Base.Position);
            scalar.Base = type;
        }
        if (Units.SelectMany(unit => unit.Scalars.Values).Any(scalar => scalar.Syntax.Category != ScalarCategory.Enum)) references.Required("Fpas.Core.DomainAttribute");
    }

    private void BindScalarValues(ReferenceCatalog references)
    {
        foreach (var scalar in Units.SelectMany(unit => unit.Scalars.Values))
        {
            object Constant(ExpressionSyntax expression) => new Binder(scalar.Owner, this, references).ScalarConstant(expression, scalar.Base, scalar.Syntax.Category == ScalarCategory.Enum ? scalar : null);
            if (scalar.Syntax.Category == ScalarCategory.Enum)
                for (var index = 0; index < scalar.Syntax.Members.Count; index++)
                {
                    var syntax = scalar.Syntax.Members[index]; var previous = index == 0 ? null : scalar.Members[index - 1];
                    scalar.Members.Add(new(syntax.Name, () => syntax.Value is not null ? Constant(syntax.Value) : NumericValue.ConvertTo(previous is null ? BigInteger.Zero : NumericValue.Integer(previous.Value) + 1, scalar.Base, true)));
                }
            else if (scalar.Syntax.Category == ScalarCategory.Range)
                scalar.InitializeBounds = () =>
                {
                    scalar.Lower = NumericValue.Integer(Constant(scalar.Syntax.Lower!)); scalar.Upper = NumericValue.Integer(Constant(scalar.Syntax.Upper!));
                    if (scalar.Lower > scalar.Upper) throw scalar.Owner.Source.Error("FP3031", "Subrange lower bound exceeds its upper bound.", scalar.Syntax.Name.Position);
                    scalar.InitializeBounds = null;
                };
        }
        foreach (var scalar in Units.SelectMany(unit => unit.Scalars.Values))
        {
            try { scalar.EnsureBounds(); foreach (var member in scalar.Members) _ = member.Value; }
            catch (Exception error) when (error is OverflowException or DivideByZeroException or InvalidOperationException)
            { throw scalar.Owner.Source.Error("FP3031", "Scalar bounds and enum values require valid representable compile-time constants without cycles.", scalar.Syntax.Name.Position); }
        }
    }

    private void ValidateAliases(ReferenceCatalog references)
    {
        if (Units.SelectMany(unit => unit.Aliases.Values).Any(alias => alias.Syntax.IsPublic)) references.Required("Fpas.Core.AliasAttribute");
        static bool Visible(TypeSymbol type) => type.Scalar?.IsPublic != false && type.Record?.IsPublic != false && type.Union?.IsPublic != false &&
            (type.Element is null || Visible(type.Element)) && type.TypeArguments.All(Visible);
        foreach (var alias in Units.SelectMany(unit => unit.Aliases.Values))
            if (alias.Syntax.IsPublic && !Visible(ResolveAlias(alias, references)))
                throw alias.Owner.Source.Error("FP3031", "A public alias cannot expose private types.", alias.Syntax.Name.Position);
            else _ = ResolveAlias(alias, references);
    }
}
