namespace Fpas.Compiler;

/// <summary>One-based UTF-16 coordinates with an exclusive end and zero-based offset.</summary>
public sealed record SourceLocation(string Path, int Line, int Column, int Length,
    int? EndLine = null, int? EndColumn = null, int? Offset = null);

public sealed record DiagnosticExpectation(string Kind, IReadOnlyList<string> Expected, string Found)
{
    public bool Equals(DiagnosticExpectation? other) => other is not null && Kind == other.Kind && Found == other.Found && Expected.SequenceEqual(other.Expected);
    public override int GetHashCode() => HashCode.Combine(Kind, Found, string.Join('\0', Expected));
}
public sealed record DiagnosticNote(string Message, SourceLocation? Location = null);
public sealed record DiagnosticEdit(SourceLocation Location, string ExpectedText, string NewText);
public enum DiagnosticApplicability { Automatic, Review }
public sealed record DiagnosticFix(string Title, DiagnosticApplicability Applicability, IReadOnlyList<DiagnosticEdit> Edits)
{
    public bool Equals(DiagnosticFix? other) => other is not null && Title == other.Title && Applicability == other.Applicability && Edits.SequenceEqual(other.Edits);
    public override int GetHashCode()
    {
        var hash = new HashCode(); hash.Add(Title); hash.Add(Applicability);
        foreach (var edit in Edits) hash.Add(edit);
        return hash.ToHashCode();
    }
}

/// <summary>Compiler facts shared by renderers and checked edit consumers.</summary>
public record CompilerDiagnostic(string Code, string Message, SourceLocation? Location, DiagnosticSeverity Severity = DiagnosticSeverity.Error)
{
    public string? InputPath { get; init; }
    public string? Explanation { get; init; }
    public string? Help { get; init; }
    public string? HelpLink { get; init; }
    public DiagnosticExpectation? Expectation { get; init; }
    public IReadOnlyList<DiagnosticNote> Related { get; init; } = [];
    public IReadOnlyList<DiagnosticFix> Fixes { get; init; } = [];

    public virtual bool Equals(CompilerDiagnostic? other) => other is not null && EqualityContract == other.EqualityContract &&
        Code == other.Code && Message == other.Message && Location == other.Location && Severity == other.Severity && InputPath == other.InputPath &&
        Explanation == other.Explanation && Help == other.Help && HelpLink == other.HelpLink && Expectation == other.Expectation &&
        Related.SequenceEqual(other.Related) && Fixes.SequenceEqual(other.Fixes);
    public override int GetHashCode()
    {
        var hash = new HashCode(); hash.Add(EqualityContract); hash.Add(Code); hash.Add(Message); hash.Add(Location); hash.Add(Severity);
        hash.Add(InputPath); hash.Add(Explanation); hash.Add(Help); hash.Add(HelpLink); hash.Add(Expectation);
        foreach (var note in Related) hash.Add(note);
        foreach (var fix in Fixes) hash.Add(fix);
        return hash.ToHashCode();
    }
}
