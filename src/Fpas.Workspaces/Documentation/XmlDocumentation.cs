using Fpas.Workspaces.Loading;
using System.Xml;
using System.Xml.Linq;

namespace Fpas.Workspaces.Documentation;

internal sealed class XmlDocumentation(DocumentationInputs inputs, CancellationToken token)
{
    private readonly Dictionary<string, (string? Assembly, Dictionary<string, XElement> Members)> documents = new(StringComparer.Ordinal);
    internal DocumentationContent Read(DocumentationEntry entry)
    {
        var path = Path.ChangeExtension(entry.AssemblyPath!, ".xml");
        if (!inputs.Exists(path)) return new("file_missing", "xml", path, null);
        token.ThrowIfCancellationRequested();
        if (!documents.TryGetValue(path, out var document))
        {
            try
            {
                using var bytes = new MemoryStream(inputs.Read(path, token), writable: false);
                using var reader = XmlReader.Create(bytes, new XmlReaderSettings { DtdProcessing = DtdProcessing.Prohibit, XmlResolver = null, MaxCharactersInDocument = 16_777_216 });
                var xml = XDocument.Load(reader);
                if (xml.Root?.Name != "doc" || xml.Root.Element("members") is not { } members)
                    throw new XmlException("Expected a .NET XML documentation file.");
                var found = new Dictionary<string, XElement>(StringComparer.Ordinal);
                foreach (var member in members.Elements("member"))
                {
                    token.ThrowIfCancellationRequested();
                    if (member.Attribute("name") is not { Value.Length: > 0 } id || !found.TryAdd(id.Value, member))
                        throw new XmlException("Missing or duplicate documentation member ID.");
                }
                document = (xml.Root.Element("assembly")?.Element("name")?.Value, found); documents.Add(path, document);
            }
            catch (XmlException exception) { throw new ContextFailure("documentation_invalid", "Cannot read XML documentation '" + path + "': " + exception.Message); }
        }
        if (document.Assembly is { } name && name != entry.Assembly)
            throw new ContextFailure("documentation_invalid", "XML documentation assembly identity does not match: " + path);
        return document.Members.TryGetValue(entry.DocumentationId!, out var content)
            ? new("available", "xml", path, content.ToString(SaveOptions.DisableFormatting)) : new("member_missing", "xml", path, null);
    }

}

public static class DocumentationDisplay
{
    public static string Text(DocumentationContent documentation)
    {
        if (documentation.Content is not { } content) return "(" + documentation.Availability + ")";
        if (documentation.Format != "xml") return content;
        var root = XElement.Parse(content);
        return string.Join(Environment.NewLine, root.Elements().Select(element => element.Name.LocalName +
            (element.Attribute("name") is { } name ? " " + name.Value : element.Attribute("cref") is { } cref ? " " + cref.Value : "") + ": " + Text(element))).Trim();
        static string Text(XNode node) => node switch
        {
            XText text => text.Value,
            XElement element when element.Name.LocalName is "see" or "seealso" => element.Value.Length != 0 ? element.Value : (string?)element.Attribute("cref") ?? (string?)element.Attribute("href") ?? (string?)element.Attribute("langword") ?? "",
            XElement element when element.Name.LocalName is "paramref" or "typeparamref" => (string?)element.Attribute("name") ?? "",
            XElement element => string.Concat(element.Nodes().Select(Text)) + (element.Name.LocalName is "para" or "br" or "code" ? Environment.NewLine : ""),
            _ => ""
        };
    }
}
