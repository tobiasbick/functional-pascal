using Fpas.Compiler.Binding.Types;
using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal ImportedProperty Indexer(TypeSymbol owner, IReadOnlyList<TypeSymbol> arguments, bool writing, SourceText source, int position)
    {
        foreach (var type in Hierarchy(owner))
        {
            if (type.Interface is not null || type.Record is not null || type.Variant is not null) continue;
            var definition = Definition(type);
            var name = DefaultMember(definition);
            if (name is null) continue;
            var candidates = new List<ImportedMethod>();
            foreach (var handle in definition.Definition.GetProperties())
            {
                var property = definition.Assembly.Reader.GetPropertyDefinition(handle);
                if (definition.Assembly.Reader.GetString(property.Name) != name) continue;
                var accessors = property.GetAccessors();
                var methodHandle = writing ? accessors.Setter : accessors.Getter;
                if (methodHandle.IsNil) continue;
                var method = definition.Assembly.Reader.GetMethodDefinition(methodHandle);
                if (method.Attributes.HasFlag(MethodAttributes.Static) || (method.Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Public) continue;
                var imported = ReadMethod(type, methodHandle);
                if (imported is null || imported.Parameters.Count != arguments.Count + (writing ? 1 : 0)) continue;
                var indices = writing ? imported.Parameters.SkipLast(1).ToArray() : imported.Parameters;
                if (indices.Where((parameter, index) => !Applicable(arguments[index], parameter)).Any()) continue;
                candidates.Add(imported);
            }
            ImportedMethod Indices(ImportedMethod method) => writing ? method with { Parameters = method.Parameters.SkipLast(1).ToArray() } : method;
            var best = candidates.Where(candidate => !candidates.Any(other => Better(Indices(other), Indices(candidate), arguments))).ToArray();
            if (best.Length == 1) return writing ? new(best[0].Parameters[^1], null, best[0]) : new(best[0].Result, best[0], null);
            if (best.Length > 1) break;
        }
        foreach (var type in Hierarchy(owner).Where(type => type.Interface is null && type.Record is null && type.Variant is null))
            if (DefaultMember(Definition(type)) is { } name)
                DiagnoseUnsupportedMember(type, name, false, writing, source, position);
        throw source.Error("FP3015", $"No unique supported public {(writing ? "setter" : "getter")} indexer for '{owner}'.", position);
    }

    private static string? DefaultMember(ImportedType definition)
    {
        var reader = definition.Assembly.Reader;
        foreach (var handle in definition.Definition.GetCustomAttributes())
        {
            var attribute = reader.GetCustomAttribute(handle);
            if (MetadataAttributes.Name(reader, attribute) != "System.Reflection.DefaultMemberAttribute") continue;
            var blob = reader.GetBlobReader(attribute.Value);
            if (blob.ReadUInt16() == 1) return blob.ReadSerializedString();
        }
        return null;
    }
}
