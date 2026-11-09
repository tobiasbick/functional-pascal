using Fpas.Compiler.Analysis;
using System.Text.RegularExpressions;

namespace Fpas.Workspaces.Documentation;

internal static partial class BoundDocumentation
{
    internal static string? Read(MetadataIdentity identity, AnalysisRequest request, string snapshot, CancellationToken token)
    {
        var path = request.ReferenceImages?.SingleOrDefault(image => image.ContentHash == identity.AssemblyHash)?.Path;
        if (path is null || Id(identity) is not { } id) return null;
        var inputs = new DocumentationInputs(snapshot, [path], token);
        var reader = new XmlDocumentation(inputs, token);
        var content = reader.Read(new("", "metadata", identity.Kind, identity.Name, identity.Type, "", "clr", identity.Assembly, path, id, null,
            new("file_missing", "xml", null, null)));
        inputs.Verify(token);
        return content.Content is null ? null : DocumentationMarkdown.PlainText(DocumentationDisplay.Text(content));
    }

    private static string? Id(MetadataIdentity identity)
    {
        var owner = identity.Type.Replace('+', '.');
        return identity.Kind switch
        {
            "type" => "T:" + owner,
            "method" => "M:" + owner + "." + identity.Name.Replace('.', '#') +
                (identity.Arity == 0 ? "" : "``" + identity.Arity) + Parameters(identity.Parameters) +
                (identity.Name is "op_Implicit" or "op_Explicit" or "op_CheckedExplicit" or "op_CheckedImplicit" && identity.Result is { } type ? "~" + XmlType(type) : ""),
            "field" => "F:" + owner + "." + identity.Name,
            "property" => "P:" + owner + "." + identity.Name + Parameters(identity.Parameters),
            "event" => "E:" + owner + "." + identity.Name,
            _ => null
        };
    }

    private static string Parameters(IReadOnlyList<string> parameters) => parameters.Count == 0 ? "" : "(" + string.Join(',', parameters.Select(XmlType)) + ")";

    private static string XmlType(string type)
    {
        var text = ConstructedArity().Replace(type, "").Replace("!!", "``", StringComparison.Ordinal).Replace('!', '`')
            .Replace('<', '{').Replace('>', '}').Replace('&', '@').Replace('+', '.');
        return MultiDimensional().Replace(text, match => "[" + string.Join(',', Enumerable.Repeat("0:", match.Length - 1)) + "]");
    }

    [GeneratedRegex("`[0-9]+(?=<)", RegexOptions.CultureInvariant)]
    private static partial Regex ConstructedArity();

    [GeneratedRegex(@"\[,+\]", RegexOptions.CultureInvariant)]
    private static partial Regex MultiDimensional();
}
