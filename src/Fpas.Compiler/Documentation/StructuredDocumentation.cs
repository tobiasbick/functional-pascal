using System.Text.RegularExpressions;
using System.Xml.Linq;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Documentation;

internal static class StructuredDocumentation
{
    private sealed record Section(string Name, DocumentationLine Header, List<DocumentationLine> Lines, bool Valid);
    private static readonly HashSet<string> Reserved = new(["## Parameters", "## Type parameters", "## Returns", "## Remarks"], StringComparer.Ordinal);
    private static readonly Regex ParameterItem = new("^- `([^`]+)`: *(.*)$", RegexOptions.CultureInvariant);

    internal static IReadOnlyList<XElement> Parse(DocumentationComment comment, IReadOnlyDictionary<string, string> parameters,
        IReadOnlyDictionary<string, string> typeParameters, bool parametersAllowed, bool typeParametersAllowed, bool returnsAllowed, Action<string, int> warn,
        CancellationToken cancellationToken = default)
    {
        var sections = new List<Section>();
        var summary = new Section("summary", comment.Lines[0], [], true); sections.Add(summary);
        var current = summary; var seen = new HashSet<string>(StringComparer.Ordinal);
        char fence = '\0'; var fenceLength = 0;
        foreach (var line in comment.Lines)
        {
            cancellationToken.ThrowIfCancellationRequested();
            var fenced = fence != '\0';
            MarkdownXml.Fence(line.Text, ref fence, ref fenceLength);
            var heading = line.Text.TrimEnd();
            if (!fenced && fence == '\0' && Reserved.Contains(heading))
            {
                var valid = seen.Add(heading);
                if (!valid) warn("Duplicate documentation section '" + heading + "'.", line.Position);
                var allowed = heading switch { "## Parameters" => parametersAllowed, "## Type parameters" => typeParametersAllowed, "## Returns" => returnsAllowed, _ => true };
                if (!allowed) { warn("Documentation section '" + heading + "' does not apply to this declaration.", line.Position); valid = false; }
                current = new(heading, line, [], valid); sections.Add(current);
            }
            else current.Lines.Add(line);
        }
        var result = new List<XElement>(); var remarks = new List<DocumentationLine>();
        foreach (var section in sections)
        {
            cancellationToken.ThrowIfCancellationRequested();
            if (!section.Valid)
            {
                Remarks(remarks, [section.Header, .. section.Lines]); continue;
            }
            switch (section.Name)
            {
                case "summary": Add("summary", section.Lines); break;
                case "## Remarks": Remarks(remarks, section.Lines); break;
                case "## Returns": Add("returns", section.Lines); break;
                case "## Parameters": Items(section, parameters, "param"); break;
                case "## Type parameters": Items(section, typeParameters, "typeparam"); break;
            }
        }
        Add("remarks", remarks);
        return result;

        void Add(string element, IReadOnlyList<DocumentationLine> lines)
        {
            if (lines.Any(line => !string.IsNullOrWhiteSpace(line.Text))) result.Add(new(element, MarkdownXml.Render(lines.Select(line => line.Text).ToArray(), cancellationToken)));
        }
        void Items(Section section, IReadOnlyDictionary<string, string> names, string element)
        {
            var documented = new HashSet<string>(StringComparer.Ordinal);
            for (var index = 0; index < section.Lines.Count;)
            {
                cancellationToken.ThrowIfCancellationRequested();
                if (string.IsNullOrWhiteSpace(section.Lines[index].Text)) { index++; continue; }
                var first = section.Lines[index++]; var match = ParameterItem.Match(first.Text);
                var original = new List<DocumentationLine> { first };
                var description = new List<string> { match.Success ? match.Groups[2].Value : first.Text };
                while (index < section.Lines.Count && (string.IsNullOrWhiteSpace(section.Lines[index].Text) || char.IsWhiteSpace(section.Lines[index].Text[0])))
                {
                    var continuation = section.Lines[index++]; original.Add(continuation);
                    var text = continuation.Text;
                    var indentation = first.Text.IndexOf(':') + 2;
                    var remove = 0; while (remove < Math.Min(indentation, text.Length) && text[remove] is ' ' or '\t') remove++;
                    description.Add(text[remove..]);
                }
                var name = match.Groups[1].Value;
                var valid = match.Success && names.ContainsKey(name) && description.Any(line => !string.IsNullOrWhiteSpace(line)) && !documented.Contains(name);
                if (!valid)
                {
                    warn(!match.Success ? "Malformed documentation parameter item; use - `Name`: Description." :
                        !names.ContainsKey(name) ? "Unknown documentation parameter '" + name + "'." :
                        documented.Contains(name) ? "Duplicate documentation parameter '" + name + "'." : "A documentation parameter needs a description.", first.Position);
                    Remarks(remarks, original); continue;
                }
                documented.Add(name);
                result.Add(new(element, new XAttribute("name", names[name]), MarkdownXml.Render(description, cancellationToken)));
            }
        }
    }

    private static void Remarks(List<DocumentationLine> target, IEnumerable<DocumentationLine> lines)
    {
        if (target.Count != 0) target.Add(new("", 0));
        target.AddRange(lines);
    }
}
