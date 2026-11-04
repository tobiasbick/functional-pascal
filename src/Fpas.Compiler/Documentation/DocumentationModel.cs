using System.Text;
using System.Xml;
using System.Xml.Linq;

namespace Fpas.Compiler.Documentation;

internal sealed record DocumentationModel(IReadOnlyList<XElement> Members, IReadOnlyList<CompilerDiagnostic> Diagnostics)
{
    internal byte[] Serialize(string assemblyName)
    {
        var document = new XDocument(new XDeclaration("1.0", "utf-8", null), new XElement("doc",
            new XElement("assembly", new XElement("name", assemblyName)),
            new XElement("members", Members.OrderBy(member => (string)member.Attribute("name")!, StringComparer.Ordinal))));
        using var stream = new MemoryStream();
        using (var xml = XmlWriter.Create(stream, new XmlWriterSettings { Encoding = new UTF8Encoding(false), Indent = true, NewLineChars = "\n", NewLineHandling = NewLineHandling.None })) document.Save(xml);
        return stream.ToArray();
    }
}
