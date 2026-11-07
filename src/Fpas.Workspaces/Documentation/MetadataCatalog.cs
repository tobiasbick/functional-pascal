using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Documentation;

internal sealed class MetadataCatalog : IDisposable
{
    private readonly Dictionary<(string Path, string? Hash), PEReader> images = [];
    internal IEnumerable<DocumentationCandidate> Search(string project, string path, DocumentationSearch search, CancellationToken token,
        MetadataImage? captured = null)
    {
        var key = (path, captured?.ContentHash);
        if (!images.TryGetValue(key, out var image))
        {
            image = new PEReader(captured?.OpenRead() ?? File.OpenRead(path), PEStreamOptions.PrefetchMetadata);
            images.Add(key, image);
        }
        var reader = image.GetMetadataReader();
        var assembly = reader.GetString(reader.GetAssemblyDefinition().Name);
        if (search.Assembly is not null && search.Assembly != assembly) yield break;
        foreach (var owner in reader.TypeDefinitions)
        {
            token.ThrowIfCancellationRequested();
            if (!Public(reader, owner)) continue;
            var definition = reader.GetTypeDefinition(owner); var type = new MetadataSignatures(reader, owner);
            var rawTypeName = reader.GetString(definition.Name); var name = MetadataSignatureTypes.WithoutArity(rawTypeName);
            var qualified = MetadataSignatureTypes.WithoutArity(type.OwnerName);
            var typeMatch = Matches(search, rawTypeName, type.OwnerName) || Matches(search, name, qualified);
            var parent = definition.GetDeclaringType();
            var directNested = false;
            if (search.Match == "exact" && !parent.IsNil)
            {
                var parentName = MetadataSignatureTypes.DefinitionName(reader, parent); var parentSimple = reader.GetString(reader.GetTypeDefinition(parent).Name);
                directNested = Matches(search, parentSimple, parentName) || Matches(search, MetadataSignatureTypes.WithoutArity(parentSimple), MetadataSignatureTypes.WithoutArity(parentName));
            }
            if (typeMatch || directNested)
                yield return Candidate("type", name, qualified, owner, () => (type.Type(), "T:" + type.OwnerName));
            var includeMembers = typeMatch && search.Match == "exact";
            foreach (var handle in definition.GetMethods())
            {
                token.ThrowIfCancellationRequested();
                var method = reader.GetMethodDefinition(handle); var member = reader.GetString(method.Name);
                if (!type.Public(handle) || member.StartsWith("get_", StringComparison.Ordinal) && method.Attributes.HasFlag(MethodAttributes.SpecialName) ||
                    method.Attributes.HasFlag(MethodAttributes.SpecialName) && (member.StartsWith("set_", StringComparison.Ordinal) ||
                    member.StartsWith("add_", StringComparison.Ordinal) || member.StartsWith("remove_", StringComparison.Ordinal))) continue;
                var fullName = qualified + (member.StartsWith('.') ? "" : ".") + member;
                if (includeMembers || Matches(search, member, fullName) || Matches(search, member, type.OwnerName + "." + member))
                    yield return Candidate(member == ".ctor" ? "constructor" : "method", member, fullName, handle, () => type.Method(handle));
            }
            foreach (var handle in definition.GetProperties())
            {
                var property = reader.GetPropertyDefinition(handle); var accessors = property.GetAccessors(); var member = reader.GetString(property.Name);
                if (!(type.Public(accessors.Getter) || type.Public(accessors.Setter))) continue;
                if (includeMembers || Matches(search, member, qualified + "." + member) || Matches(search, member, type.OwnerName + "." + member))
                    yield return Candidate("property", member, qualified + "." + member, handle, () => type.Property(handle));
            }
            foreach (var handle in definition.GetFields())
            {
                var field = reader.GetFieldDefinition(handle); var member = reader.GetString(field.Name);
                if ((field.Attributes & FieldAttributes.FieldAccessMask) != FieldAttributes.Public || field.Attributes.HasFlag(FieldAttributes.SpecialName)) continue;
                if (includeMembers || Matches(search, member, qualified + "." + member) || Matches(search, member, type.OwnerName + "." + member))
                    yield return Candidate("field", member, qualified + "." + member, handle, () => type.Field(handle));
            }
            foreach (var handle in definition.GetEvents())
            {
                var item = reader.GetEventDefinition(handle); var accessors = item.GetAccessors(); var member = reader.GetString(item.Name);
                if (!(type.Public(accessors.Adder) || type.Public(accessors.Remover))) continue;
                if (includeMembers || Matches(search, member, qualified + "." + member) || Matches(search, member, type.OwnerName + "." + member))
                    yield return Candidate("event", member, qualified + "." + member, handle, () => type.Event(handle));
            }

            DocumentationCandidate Candidate(string kind, string member, string fullName, EntityHandle handle, Func<(string Signature, string Id)> signature)
                => new(project, "metadata", fullName, path + ":" + MetadataTokens.GetToken(handle).ToString("X8", System.Globalization.CultureInfo.InvariantCulture), () =>
                {
                    token.ThrowIfCancellationRequested(); var data = signature();
                    return new DocumentationEntry(project, "metadata", kind, member, fullName, data.Signature, "clr", assembly, path, data.Id, null,
                        new("file_missing", "xml", Path.ChangeExtension(path, ".xml"), null))
                    {
                        Identity = captured is null ? null : captured.ContentHash + ":" + MetadataTokens.GetToken(handle).ToString("X8", System.Globalization.CultureInfo.InvariantCulture)
                    };
                });
        }
    }

    internal static bool Matches(DocumentationSearch search, string name, string qualifiedName)
    {
        return Match(name) || Match(qualifiedName);
        bool Match(string value) => search.Match switch
        {
            "exact" => value.Equals(search.Name, StringComparison.Ordinal),
            "prefix" => value.StartsWith(search.Name, StringComparison.Ordinal),
            "contains" => value.Contains(search.Name, StringComparison.Ordinal),
            _ => false
        };
    }
    private static bool Public(MetadataReader reader, TypeDefinitionHandle handle)
    {
        var definition = reader.GetTypeDefinition(handle); var access = definition.Attributes & TypeAttributes.VisibilityMask;
        return access == TypeAttributes.Public || access == TypeAttributes.NestedPublic && Public(reader, definition.GetDeclaringType());
    }
    public void Dispose() { foreach (var image in images.Values) image.Dispose(); }
}
