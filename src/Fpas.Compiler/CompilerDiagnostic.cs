// [Documentation](../../docs/pascal/tools/diagnostics.md)
using System.Text.Json.Serialization;

namespace Fpas.Compiler;

/// <summary>One-based UTF-16 coordinates with an exclusive end and zero-based offset.</summary>
/// <param name="Path">The logical source path.</param>
/// <param name="Line">The one-based start line.</param>
/// <param name="Column">The one-based start column in UTF-16 code units.</param>
/// <param name="Length">The length in UTF-16 code units.</param>
/// <param name="EndLine">The one-based line of the exclusive end, when known.</param>
/// <param name="EndColumn">The one-based column of the exclusive end, when known.</param>
/// <param name="Offset">The zero-based UTF-16 offset of the start, when known.</param>
public sealed record SourceLocation(string Path, int Line, int Column, int Length,
    int? EndLine = null, int? EndColumn = null, int? Offset = null);

/// <summary>What the compiler expected at a diagnostic location and what it found.</summary>
/// <param name="Kind">The category of the expectation, such as a token, type, or value.</param>
/// <param name="Expected">The accepted alternatives.</param>
/// <param name="Found">The text or description of what was found.</param>
public sealed record DiagnosticExpectation(string Kind, IReadOnlyList<string> Expected, string Found)
{
    /// <summary>Compares all members, including the expected alternatives by sequence.</summary>
    /// <param name="other">The expectation to compare with.</param>
    /// <returns>Whether both expectations are equal.</returns>
    public bool Equals(DiagnosticExpectation? other) => other is not null && Kind == other.Kind && Found == other.Found && Expected.SequenceEqual(other.Expected);

    /// <summary>Returns a hash code consistent with value equality.</summary>
    /// <returns>The hash code.</returns>
    public override int GetHashCode() => HashCode.Combine(Kind, Found, string.Join('\0', Expected));
}

/// <summary>A parameter offered by one of the closest signatures.</summary>
/// <param name="Name">The parameter name, when available.</param>
/// <param name="Type">The required argument type.</param>
/// <param name="Mode">The passing mode: value, ref, out, or in.</param>
/// <param name="Requirement">An additional rule for the argument, when applicable.</param>
public sealed record DiagnosticParameter(string? Name, string Type, string Mode, string? Requirement = null);

/// <summary>The argument rejected by the closest callable signatures.</summary>
/// <param name="Index">The one-based position in written argument order.</param>
/// <param name="Name">The explicit argument label, if supplied.</param>
/// <param name="Type">The actual type, when known.</param>
/// <param name="Mode">The supplied passing mode: value, ref, out, or in.</param>
/// <param name="Parameters">The parameters those signatures map to this argument.</param>
public sealed record DiagnosticArgument(int Index, string? Name, string? Type, string Mode, IReadOnlyList<DiagnosticParameter> Parameters)
{
    /// <summary>Compares all members, including parameter alternatives by sequence.</summary>
    /// <param name="other">The argument diagnostic to compare with.</param>
    /// <returns>Whether both argument diagnostics are equal.</returns>
    public bool Equals(DiagnosticArgument? other) => other is not null && Index == other.Index && Name == other.Name &&
        Type == other.Type && Mode == other.Mode && Parameters.SequenceEqual(other.Parameters);

    /// <summary>Returns a hash code consistent with value equality.</summary>
    /// <returns>The hash code.</returns>
    public override int GetHashCode()
    {
        var hash = new HashCode();
        hash.Add(Index); hash.Add(Name); hash.Add(Type); hash.Add(Mode);
        foreach (var parameter in Parameters) hash.Add(parameter);
        return hash.ToHashCode();
    }
}

/// <summary>A related message attached to a diagnostic.</summary>
/// <param name="Message">The explanatory text.</param>
/// <param name="Location">The related source location, when one exists.</param>
public sealed record DiagnosticNote(string Message, SourceLocation? Location = null);

/// <summary>One checked text replacement of a diagnostic fix.</summary>
/// <param name="Location">The replaced source range.</param>
/// <param name="ExpectedText">The text the range must still contain for the edit to apply.</param>
/// <param name="NewText">The replacement text.</param>
public sealed record DiagnosticEdit(SourceLocation Location, string ExpectedText, string NewText);

/// <summary>Whether a fix may be applied without review.</summary>
public enum DiagnosticApplicability
{
    /// <summary>The fix preserves the intended meaning and may be applied automatically.</summary>
    Automatic,
    /// <summary>The fix is a suggestion that a person should review.</summary>
    Review
}

/// <summary>A named set of edits that resolves a diagnostic.</summary>
/// <param name="Title">The short description of the fix.</param>
/// <param name="Applicability">Whether the fix may be applied without review.</param>
/// <param name="Edits">The edits, applied together.</param>
public sealed record DiagnosticFix(string Title, DiagnosticApplicability Applicability, IReadOnlyList<DiagnosticEdit> Edits)
{
    /// <summary>Compares all members, including the edits by sequence.</summary>
    /// <param name="other">The fix to compare with.</param>
    /// <returns>Whether both fixes are equal.</returns>
    public bool Equals(DiagnosticFix? other) => other is not null && Title == other.Title && Applicability == other.Applicability && Edits.SequenceEqual(other.Edits);

    /// <summary>Returns a hash code consistent with value equality.</summary>
    /// <returns>The hash code.</returns>
    public override int GetHashCode()
    {
        var hash = new HashCode(); hash.Add(Title); hash.Add(Applicability);
        foreach (var edit in Edits) hash.Add(edit);
        return hash.ToHashCode();
    }
}

/// <summary>Compiler facts shared by renderers and checked edit consumers.</summary>
/// <param name="Code">The stable diagnostic code, such as FP3002.</param>
/// <param name="Message">The primary message.</param>
/// <param name="Location">The source range, or <see langword="null"/> for an input-level diagnostic.</param>
/// <param name="Severity">Whether the diagnostic is an error or a warning.</param>
public record CompilerDiagnostic(string Code, string Message, SourceLocation? Location, DiagnosticSeverity Severity = DiagnosticSeverity.Error)
{
    /// <summary>The input file the diagnostic concerns, such as an unreadable reference path.</summary>
    public string? InputPath { get; init; }
    /// <summary>An optional longer explanation of the rule.</summary>
    public string? Explanation { get; init; }
    /// <summary>An optional suggestion for resolving the diagnostic.</summary>
    public string? Help { get; init; }
    /// <summary>An optional link to further documentation.</summary>
    public string? HelpLink { get; init; }
    /// <summary>What was expected and found, when the diagnostic describes a mismatch.</summary>
    public DiagnosticExpectation? Expectation { get; init; }
    /// <summary>Structured details for an argument mismatch, when the compiler can identify the argument.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public DiagnosticArgument? Argument { get; init; }
    /// <summary>Related notes, such as the location of an earlier conflicting declaration.</summary>
    public IReadOnlyList<DiagnosticNote> Related { get; init; } = [];
    /// <summary>Checked fixes that resolve the diagnostic.</summary>
    public IReadOnlyList<DiagnosticFix> Fixes { get; init; } = [];

    /// <summary>Compares all members, including related notes and fixes by sequence.</summary>
    /// <param name="other">The diagnostic to compare with.</param>
    /// <returns>Whether both diagnostics are equal.</returns>
    public virtual bool Equals(CompilerDiagnostic? other) => other is not null && EqualityContract == other.EqualityContract &&
        Code == other.Code && Message == other.Message && Location == other.Location && Severity == other.Severity && InputPath == other.InputPath &&
        Explanation == other.Explanation && Help == other.Help && HelpLink == other.HelpLink && Expectation == other.Expectation && Argument == other.Argument &&
        Related.SequenceEqual(other.Related) && Fixes.SequenceEqual(other.Fixes);

    /// <summary>Returns a hash code consistent with value equality.</summary>
    /// <returns>The hash code.</returns>
    public override int GetHashCode()
    {
        var hash = new HashCode(); hash.Add(EqualityContract); hash.Add(Code); hash.Add(Message); hash.Add(Location); hash.Add(Severity);
        hash.Add(InputPath); hash.Add(Explanation); hash.Add(Help); hash.Add(HelpLink); hash.Add(Expectation); hash.Add(Argument);
        foreach (var note in Related) hash.Add(note);
        foreach (var fix in Fixes) hash.Add(fix);
        return hash.ToHashCode();
    }
}
