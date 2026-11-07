using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Queries;
using OmniSharp.Extensions.LanguageServer.Protocol.Models;
using LspRange = OmniSharp.Extensions.LanguageServer.Protocol.Models.Range;

namespace Fpas.LanguageServer.Protocol;

internal static class FeatureProjection
{
    // Navigation and rename spans identify single identifier tokens, including @.
    internal static LspRange Token(SourceSpan span) => new(new(span.Line - 1, span.Column - 1), new(span.Line - 1, span.Column - 1 + span.Length));
    internal static Location Location(SourceSpan span) => new() { Uri = FileUris.Uri(span.Path), Range = Token(span) };
    internal static LspRange Range(SourceDocument document, SourceSpan span)
    {
        var end = document.Span(span.Offset + span.Length, 0);
        return new(new(span.Line - 1, span.Column - 1), new(end.Line - 1, end.Column - 1));
    }
    internal static Position End(string text)
    {
        var line = 0; var character = 0;
        for (var index = 0; index < text.Length; index++)
        {
            if (text[index] == '\r') { if (index + 1 < text.Length && text[index + 1] == '\n') index++; line++; character = 0; }
            else if (text[index] == '\n') { line++; character = 0; } else character++;
        }
        return new(line, character);
    }

    internal static WorkspaceEdit WorkspaceEdits(TextEditSet edits) => new()
    {
        DocumentChanges = new Container<WorkspaceEditDocumentChange>(edits.Documents.Select(document => new WorkspaceEditDocumentChange(new TextDocumentEdit
        {
            TextDocument = new OptionalVersionedTextDocumentIdentifier { Uri = FileUris.Uri(document.Original.Path), Version = document.Original.Version },
            Edits = Ranges(document)
        })))
    };

    private static TextEditContainer Ranges(DocumentEdits edits)
    {
        var document = new SourceDocument(edits.Original.Path, edits.Original.Text);
        return new(edits.Edits.Select(edit => new TextEdit
            { Range = Range(document, document.Span(edit.Location.Offset!.Value, edit.Location.Length)), NewText = edit.NewText }));
    }

    internal static SymbolInformation Symbol(DeclarationMatch match) => new()
    {
        Name = match.Name, Location = Location(match.Span),
        ContainerName = match.QualifiedName.Length > match.Name.Length ? match.QualifiedName[..^(match.Name.Length + 1)] : null,
        Kind = match.Kind switch
        {
            "container" => SymbolKind.Namespace, "record" => SymbolKind.Class, "interface" => SymbolKind.Interface,
            "union" or "domain" => SymbolKind.Enum, "routine" => SymbolKind.Function, "property" => SymbolKind.Property,
            "union_case" or "enum_member" => SymbolKind.EnumMember, "type_parameter" => SymbolKind.TypeParameter, _ => SymbolKind.Variable
        }
    };
}
