using Fpas.Compiler;
using Fpas.Workspaces.Snapshots;
using Newtonsoft.Json.Linq;
using OmniSharp.Extensions.LanguageServer.Protocol.Models;
using LspRange = OmniSharp.Extensions.LanguageServer.Protocol.Models.Range;

namespace Fpas.LanguageServer.Protocol;

internal static class DiagnosticProjection
{
    internal static Diagnostic Convert(CompilerDiagnostic diagnostic, WorkspaceTextSnapshot texts)
    {
        var location = diagnostic.Location!;
        var related = diagnostic.Related.Where(note => note.Location is not null).Select(note => new DiagnosticRelatedInformation
        { Location = new Location { Uri = FileUris.Uri(note.Location!.Path), Range = Range(note.Location, texts) }, Message = note.Message }).ToArray();
        return new Diagnostic
        {
            Range = Range(location, texts), Code = diagnostic.Code, Source = "fpas", Message = diagnostic.Message,
            Severity = diagnostic.Severity switch
            { Fpas.Compiler.DiagnosticSeverity.Error => OmniSharp.Extensions.LanguageServer.Protocol.Models.DiagnosticSeverity.Error,
                Fpas.Compiler.DiagnosticSeverity.Warning => OmniSharp.Extensions.LanguageServer.Protocol.Models.DiagnosticSeverity.Warning,
                _ => OmniSharp.Extensions.LanguageServer.Protocol.Models.DiagnosticSeverity.Information },
            RelatedInformation = related.Length == 0 ? null : new Container<DiagnosticRelatedInformation>(related),
            Data = diagnostic.Explanation is null && diagnostic.Help is null && diagnostic.Expectation is null ? null : JObject.FromObject(new
            {
                explanation = diagnostic.Explanation, help = diagnostic.Help,
                expectation = diagnostic.Expectation is not { } expectation ? null : new { kind = expectation.Kind, expected = expectation.Expected, found = expectation.Found }
            })
        };
    }

    private static LspRange Range(SourceLocation location, WorkspaceTextSnapshot texts)
    {
        var start = new Position(Math.Max(0, location.Line - 1), Math.Max(0, location.Column - 1));
        if (location.EndLine is { } line && location.EndColumn is { } column)
            return new() { Start = start, End = new(Math.Max(0, line - 1), Math.Max(0, column - 1)) };
        var document = texts.Documents.FirstOrDefault(document => SamePath(document.Path, location.Path));
        if (location.Offset is { } offset && document is not null)
        {
            var end = Math.Min(document.Text.Length, (long)offset + Math.Max(0, location.Length)); var row = 0; var character = 0;
            for (var index = 0; index < end; index++)
            {
                if (document.Text[index] == '\r') { if (index + 1 < end && document.Text[index + 1] == '\n') index++; row++; character = 0; }
                else if (document.Text[index] == '\n') { row++; character = 0; } else character++;
            }
            return new() { Start = start, End = new(row, character) };
        }
        return new() { Start = start, End = new(start.Line, start.Character + Math.Max(0, location.Length)) };
    }

    private static bool SamePath(string a, string b) => string.Equals(a, b, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal);
}
