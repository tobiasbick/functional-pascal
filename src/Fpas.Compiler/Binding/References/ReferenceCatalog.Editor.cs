using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.Types;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal IEnumerable<ImportedType> EditorImportTypes(string name, CancellationToken token)
    {
        foreach (var (path, matches) in types)
        {
            token.ThrowIfCancellationRequested();
            if (!path.Contains('+') && !path.Contains('`') && path.AsSpan(path.LastIndexOf('.') + 1).SequenceEqual(name))
                foreach (var match in matches) yield return match;
        }
    }

    internal IEnumerable<string> EditorMemberNames(TypeSymbol owner)
    {
        foreach (var type in Hierarchy(owner))
        {
            if (type.Record is { } record)
            {
                foreach (var name in record.UserMethods.Keys.Concat(record.Fields.Select(field => field.Name))) yield return name;
            }
            else if (type.Union is { } union)
            {
                foreach (var name in union.Methods.Select(method => method.Name).Concat(union.Cases.Select(item => item.Syntax.Name.Text))) yield return name;
            }
            else if (type.Interface is { } contract)
            {
                foreach (var name in contract.Methods.Select(method => method.Name).Concat(contract.Properties.Select(property => property.Syntax.Name.Text))) yield return name;
            }
            else if (type.Kind == TypeKind.Scalar)
            {
                foreach (var method in ScalarMethods(type)) yield return method.Name;
            }
            else
            {
                var definition = Definition(type);
                var reader = definition.Assembly.Reader;
                foreach (var member in definition.Definition.GetMethods())
                {
                    var method = reader.GetMethodDefinition(member);
                    if ((method.Attributes & (System.Reflection.MethodAttributes.SpecialName | System.Reflection.MethodAttributes.MemberAccessMask)) ==
                        System.Reflection.MethodAttributes.Public)
                        yield return reader.GetString(method.Name);
                }
                foreach (var member in definition.Definition.GetFields()) yield return reader.GetString(reader.GetFieldDefinition(member).Name);
                foreach (var member in definition.Definition.GetProperties()) yield return reader.GetString(reader.GetPropertyDefinition(member).Name);
            }
        }
    }

    internal IEnumerable<EditorCandidate> EditorPaths(string qualifier, string prefix, CancellationToken token)
    {
        var seen = new HashSet<string>(StringComparer.Ordinal);
        foreach (var (path, definitions) in types)
        {
            token.ThrowIfCancellationRequested();
            if (definitions.Count != 1 || path.Contains('+') || !path.StartsWith(qualifier, StringComparison.Ordinal)) continue;
            var remaining = path[qualifier.Length..];
            var end = remaining.IndexOfAny(['.', '`']);
            var name = end < 0 ? remaining : remaining[..end];
            if (!name.StartsWith(prefix, StringComparison.OrdinalIgnoreCase) || !seen.Add(name)) continue;
            yield return new(name, end >= 0 && remaining[end] == '.' ? "namespace" : "type", qualifier + name);
        }
    }
}
