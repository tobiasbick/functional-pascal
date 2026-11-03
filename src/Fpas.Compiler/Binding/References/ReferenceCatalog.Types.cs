using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal bool HasType(string name) => types.ContainsKey(name);
    internal bool HasNamespace(string name) => types.Keys.Any(key => key.StartsWith(name + ".", StringComparison.Ordinal));

    internal ImportedType FindType(string name, int position, SourceText source)
    {
        if (!types.TryGetValue(name, out var candidates))
            candidates = types.Values.SelectMany(value => value).Where(type => type.FullName.Replace('+', '.') == name).ToList();
        if (candidates.Count == 0)
            throw source.Error("FP3001", $"Public type '{name}' was not found in the supplied references.", position);
        if (candidates.Count != 1)
            throw source.Error("FP4002", $"Type '{name}' is ambiguous across reference assemblies.", position);
        var definition = candidates[0];
        if (Symbol(definition) == TypeSymbol.Unsupported)
            throw source.Error("FP3005", $"Type '{name}' is byref-like or otherwise unsupported.", position);
        return definition;
    }

    internal TypeSymbol Symbol(ImportedType type)
    {
        if (MetadataAttributes.Has(type.Assembly.Reader, type.Definition.GetCustomAttributes(), "System.Runtime.CompilerServices.IsByRefLikeAttribute")) return TypeSymbol.Unsupported;
        var primitive = TypeFacts.Primitive(type.FullName);
        return primitive != TypeSymbol.Unsupported ? primitive : new(TypeKind.Named, type);
    }

    internal ImportedType Definition(TypeSymbol type)
    {
        if (type.NullableValue) return Required("System.Nullable`1");
        if (type.Kind == TypeKind.Array) return Required("System.Array");
        if (type.Definition is not null) return type.Definition;
        return Required(TypeFacts.ClrName(type) ?? type.Kind switch
        {
            TypeKind.String => "System.String", TypeKind.Object => "System.Object", TypeKind.Int32 => "System.Int32",
            TypeKind.Boolean => "System.Boolean", TypeKind.Double => "System.Double", TypeKind.Char => "System.Char",
            _ => throw new BadImageFormatException("Unsupported CLR type.")
        });
    }

    internal ImportedType Required(string name)
        => types.TryGetValue(name, out var candidates) && candidates.Count == 1 ? candidates[0]
            : throw new BadImageFormatException($"Missing or ambiguous reference type '{name}'.");

    internal TypeSymbol FromDefinition(ReferenceAssembly assembly, TypeDefinitionHandle handle, bool includePrivate = false)
    {
        var type = assembly.Reader.GetTypeDefinition(handle);
        var imported = new ImportedType(assembly, handle, assembly.Reader.GetString(type.Namespace), assembly.Reader.GetString(type.Name));
        return imported.IsPublic || includePrivate ? Symbol(imported) : TypeSymbol.Unsupported;
    }

    internal TypeSymbol FromReference(ReferenceAssembly assembly, TypeReferenceHandle handle)
    {
        var type = assembly.Reader.GetTypeReference(handle);
        var ns = assembly.Reader.GetString(type.Namespace);
        var shortName = assembly.Reader.GetString(type.Name);
        var name = ns.Length == 0 ? shortName : ns + "." + shortName;
        var scope = type.ResolutionScope;
        if (scope.Kind == HandleKind.TypeReference)
        {
            var parent = FromReference(assembly, (TypeReferenceHandle)scope).Definition;
            return parent is not null && types.TryGetValue(parent.FullName + "+" + shortName, out var nested) &&
                nested.FirstOrDefault(candidate => candidate.Assembly == parent.Assembly) is { } found ? Symbol(found) : TypeSymbol.Unsupported;
        }
        if (scope.Kind is not (HandleKind.AssemblyReference or HandleKind.ModuleDefinition)) return TypeSymbol.Unsupported;
        var owner = scope.Kind == HandleKind.ModuleDefinition ? assembly.Identity.Name :
            assembly.Reader.GetString(assembly.Reader.GetAssemblyReference((AssemblyReferenceHandle)scope).Name);
        var visited = new HashSet<string>(StringComparer.Ordinal);
        while (visited.Add(owner))
        {
            if (types.TryGetValue(name, out var candidates) && candidates.FirstOrDefault(candidate => candidate.Assembly.Identity.Name == owner) is { } found)
                return Symbol(found);
            var target = assemblies.FirstOrDefault(candidate => candidate.Identity.Name == owner);
            if (target is null) break;
            var forwarded = false;
            foreach (var exportedHandle in target.Reader.ExportedTypes)
            {
                var exported = target.Reader.GetExportedType(exportedHandle);
                var exportedNamespace = target.Reader.GetString(exported.Namespace);
                var exportedName = target.Reader.GetString(exported.Name);
                var fullName = exportedNamespace.Length == 0 ? exportedName : exportedNamespace + "." + exportedName;
                if (fullName != name ||
                    exported.Implementation.Kind != HandleKind.AssemblyReference) continue;
                owner = target.Reader.GetString(target.Reader.GetAssemblyReference((AssemblyReferenceHandle)exported.Implementation).Name);
                forwarded = true;
                break;
            }
            if (!forwarded) break;
        }
        return TypeSymbol.Unsupported;
    }

    private TypeSymbol Decode(ImportedType owner, EntityHandle handle, TypeSymbol instance, TypeList? methodArguments = null)
    {
        if (handle.IsNil) return TypeSymbol.Unsupported;
        var provider = new SignatureTypes(this, owner.Assembly);
        return handle.Kind switch
        {
            HandleKind.TypeDefinition => FromDefinition(owner.Assembly, (TypeDefinitionHandle)handle),
            HandleKind.TypeReference => FromReference(owner.Assembly, (TypeReferenceHandle)handle),
            HandleKind.TypeSpecification => Substitute(owner.Assembly.Reader.GetTypeSpecification((TypeSpecificationHandle)handle).DecodeSignature(provider, null), instance, methodArguments),
            _ => TypeSymbol.Unsupported
        };
    }

    private IEnumerable<TypeSymbol> Hierarchy(TypeSymbol type)
    {
        if (type.NullableValue) type = Symbol(Required("System.Nullable`1")) with { Arguments = type.TypeArguments };
        if (type.IsParameter)
        {
            var seen = new HashSet<TypeSymbol>();
            foreach (var bound in Parents(type))
                foreach (var member in Hierarchy(bound)) if (seen.Add(member)) yield return member;
            yield break;
        }
        if (type.Kind == TypeKind.Scalar) { yield return type; type = Parents(type).First(); }
        if (type.Kind == TypeKind.Record) { yield return type; type = type.Record!.UnionCase is { } member ? member.Owner.Type with { Arguments = type.Arguments } : TypeSymbol.Object; }
        if (type.Kind == TypeKind.Union) { yield return type; type = TypeSymbol.Object; }
        if (type.Kind == TypeKind.Array) type = Symbol(Required("System.Array"));
        var pending = new Queue<TypeSymbol>();
        var visited = new HashSet<TypeSymbol>();
        pending.Enqueue(type);
        while (pending.TryDequeue(out var current))
        {
            if (current == TypeSymbol.Unsupported || !visited.Add(current)) continue;
            yield return current;
            if (current.Interface is not null)
            {
                foreach (var parent in Parents(current)) pending.Enqueue(parent);
                continue;
            }
            var definition = Definition(current);
            if (definition.Definition.Attributes.HasFlag(TypeAttributes.Interface))
                foreach (var parent in Parents(current)) pending.Enqueue(parent);
            else pending.Enqueue(Decode(definition, definition.Definition.BaseType, current));
        }
    }

    internal bool Derives(TypeSymbol from, TypeSymbol to)
    {
        from = from.NonNullable; to = to.NonNullable;
        if (to == TypeSymbol.Object) return from.IsReference || from.IsValue || from.IsParameter;
        var pending = new Stack<TypeSymbol>();
        var seen = new HashSet<TypeSymbol>();
        pending.Push(from);
        while (pending.TryPop(out var current))
        {
            if (SameAssignableDefinition(current, to)) return true;
            if (current == TypeSymbol.Unsupported || !seen.Add(current)) continue;
            foreach (var parent in Parents(current)) pending.Push(parent);
        }
        return false;
    }

    internal bool CanAssign(TypeSymbol from, TypeSymbol to)
    {
        if (from.NonNullable != to.NonNullable && TypeFacts.Numeric(to.NonNullable) && ScalarInfo(from) is { Category: Syntax.ScalarCategory.Range } range)
            return CanAssign(range.Base with { IsNullable = from.IsNullable }, to);
        if (from.Kind == TypeKind.Void || to.Kind == TypeKind.Void) return false;
        if (from == TypeSymbol.Null) return to.IsNullable;
        if (from.IsParameter && to.IsParameter && from.Parameter == to.Parameter)
            return !from.IsNullable || to.IsNullable || from.NullValidated;
        if ((from.CanHaveNull && from.NeedsNullCheck || from.NullableValue) && to.IsReference && !to.IsNullable) return false;
        if (from.NullableValue && !to.IsNullable && !(to.IsReference && to.IsNullable)) return false;
        if (from.NonNullable == to.NonNullable || NumericFacts.Widen(from.NonNullable, to.NonNullable))
            return !from.IsNullable || to.IsNullable;
        if (from.Kind == TypeKind.Array && to.Kind == TypeKind.Array && from.Rank == to.Rank)
            return from.Element!.IsReference && to.Element!.IsReference && from.Element.IsNullable == to.Element.IsNullable && CanAssign(from.Element, to.Element);
        return (to.IsReference || to.IsParameter) && Derives(from, to) && (!from.NullableValue || to.IsNullable);
    }

    internal bool Applicable(TypeSymbol from, TypeSymbol to)
        => CanAssign(from.IsReference ? from.NonNullable : from, to.IsReference ? to with { IsNullable = from == TypeSymbol.Null } : to);

    internal TypeSymbol Common(TypeSymbol left, TypeSymbol right)
    {
        if (left == TypeSymbol.Null && right == TypeSymbol.Null) return TypeSymbol.Null;
        if (left == TypeSymbol.Null && (right.IsReference || right.IsValue)) return right with { IsNullable = true };
        if (right == TypeSymbol.Null && (left.IsReference || left.IsValue)) return left with { IsNullable = true };
        if (CanAssign(left, right)) return right;
        if (CanAssign(right, left)) return left;
        if (left.IsReference && right.IsReference)
            foreach (var type in Hierarchy(left.NonNullable))
                if (type.IsReference && Derives(right, type)) return type with { IsNullable = left.IsNullable || right.IsNullable };
        return TypeSymbol.Unsupported;
    }
}
