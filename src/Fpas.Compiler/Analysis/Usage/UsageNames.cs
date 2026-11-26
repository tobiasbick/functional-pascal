// [Documentation](../../../../docs/pascal/tools/api-documentation.md)
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Types;
using System.Globalization;

namespace Fpas.Compiler.Analysis;

internal sealed class UsageNames
{
    private readonly Dictionary<string, string> imports = new(StringComparer.Ordinal);
    private readonly HashSet<string> used = new(StringComparer.Ordinal) { "ApiUsage", "Use", "Resource" };

    internal string Imports => imports.Count == 0 ? "" : string.Join("\n",
        imports.OrderBy(pair => pair.Key, StringComparer.Ordinal)
            .Select(pair => "uses " + FpasIdentifiers.SpellQualified(pair.Key) + " as " + pair.Value + ";")) + "\n\n";

    internal string Parameter(string name, int index)
    {
        var fallback = "Argument" + (index + 1).ToString(CultureInfo.InvariantCulture);
        if (string.IsNullOrEmpty(name) || !char.IsAsciiLetter(name[0]) || !name.All(c => char.IsAsciiLetterOrDigit(c) || c == '_'))
            return Unique(fallback);
        return Unique(char.ToUpperInvariant(name[0]) + name[1..]);
    }

    internal string Type(TypeSymbol type)
    {
        if (type.Kind == TypeKind.Array)
        {
            var rank = type.Rank == 1 ? "" : "[" + new string(',', type.Rank - 1) + "]";
            var array = "array" + rank + " of " + Type(type.Element!);
            return type.IsNullable ? "(" + array + ")?" : array;
        }
        if (type.Definition is not { } definition) return type.ToString();
        return Named(definition, type.TypeArguments) + (type.IsNullable ? "?" : "");
    }

    internal string Named(ImportedType definition, IReadOnlyList<TypeSymbol> arguments)
        => Named(definition, arguments, Type, Alias);

    internal static string Named(ImportedType definition, IReadOnlyList<TypeSymbol> arguments, Func<TypeSymbol, string> spellType,
        Func<string, string> qualify)
    {
        var segments = new Stack<ImportedType>();
        for (var type = definition; type is not null; type = type.DeclaringType) segments.Push(type);
        var ns = segments.Peek().Namespace;
        var names = new List<string>();
        if (ns.Length > 0) names.Add(qualify(ns));
        var index = 0;
        foreach (var type in segments)
        {
            var parts = type.Name.Split('`');
            var name = FpasIdentifiers.SpellQualified(parts[0]);
            if (ns.Length == 0 && names.Count == 0) name = qualify(parts[0]);
            if (parts.Length > 1)
            {
                if (!int.TryParse(parts[1], NumberStyles.None, CultureInfo.InvariantCulture, out var count) || count > arguments.Count - index)
                    throw new ArgumentException("Missing concrete type arguments for " + type.FullName + ".");
                name += "<" + string.Join(", ", arguments.Skip(index).Take(count).Select(spellType)) + ">";
                index += count;
            }
            names.Add(name);
        }
        return string.Join('.', names);
    }

    private string Alias(string path)
    {
        if (!imports.TryGetValue(path, out var alias))
        {
            var last = path[(path.LastIndexOf('.') + 1)..];
            alias = Unique(path == "System" ? "Sys" : last);
            imports.Add(path, alias);
        }
        return alias;
    }

    private string Unique(string stem)
    {
        var name = FpasIdentifiers.SpellQualified(stem);
        for (int suffix = 2; !used.Add(name); suffix++)
            name = stem + suffix.ToString(CultureInfo.InvariantCulture);
        return name;
    }
}
