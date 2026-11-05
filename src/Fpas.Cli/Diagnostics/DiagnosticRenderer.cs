using Fpas.Compiler;
using System.Globalization;
using System.Text;

namespace Fpas.Cli.Diagnostics;

internal static class DiagnosticRenderer
{
    internal static void Write(IEnumerable<CompilerDiagnostic> diagnostics, IReadOnlyList<CompilationSource>? sources = null)
    {
        var color = !Console.IsErrorRedirected && Environment.GetEnvironmentVariable("NO_COLOR") is null && Environment.GetEnvironmentVariable("TERM") != "dumb";
        foreach (var diagnostic in diagnostics)
        {
            var severity = diagnostic.Severity == DiagnosticSeverity.Warning ? "warning" : "error";
            var location = diagnostic.Location;
            var prefix = location is null ? diagnostic.InputPath is { } path ? Safe(path) + ": " : "" : $"{Safe(location.Path)}({location.Line},{location.Column}): ";
            var label = $"{severity} {diagnostic.Code}";
            if (color) label = (severity == "error" ? "\u001b[31m" : "\u001b[33m") + label + "\u001b[0m";
            Console.Error.WriteLine(prefix + label + ": " + Safe(diagnostic.Message));
            var source = location is null ? null : sources?.FirstOrDefault(source => source.Path == location.Path);
            if (source is not null) Excerpt(source.Text, location!);
            if (diagnostic.Explanation is { } explanation) Detail("note", explanation);
            foreach (var note in diagnostic.Related)
                Detail("note", (note.Location is { } at ? $"{at.Path}({at.Line},{at.Column}): " : "") + note.Message);
            if (diagnostic.Help is { } help) Detail("help", help);
            if (diagnostic.HelpLink is { } link) Detail("help", link);
            foreach (var fix in diagnostic.Fixes) Detail(fix.Applicability == DiagnosticApplicability.Automatic ? "fix" : "suggestion", fix.Title);
        }
    }

    private static void Detail(string label, string text) => Console.Error.WriteLine($"  {label}: {Safe(text)}");

    private static void Excerpt(string text, SourceLocation location)
    {
        if (location.Offset is not { } offset || offset < 0 || offset > text.Length) return;
        var start = offset;
        while (start > 0 && text[start - 1] is not ('\r' or '\n')) start--;
        var end = offset;
        while (end < text.Length && text[end] is not ('\r' or '\n')) end++;
        var line = text[start..end];
        var selection = offset - start;
        var length = Math.Min(location.Length, end - offset);
        var rendered = DisplayLine.Render(line, selection, length);
        var number = location.Line.ToString(CultureInfo.InvariantCulture);
        Console.Error.WriteLine($" {number} | {rendered.Text}");
        Console.Error.WriteLine($" {new string(' ', number.Length)} | {new string(' ', rendered.Column)}^{new string('~', Math.Max(0, rendered.Width - 1))}");
        if (location.EndLine > location.Line) Console.Error.WriteLine($"   = range ends at {location.EndLine}:{location.EndColumn}");
    }

    internal static string Safe(string text)
    {
        var result = new StringBuilder();
        foreach (var rune in text.EnumerateRunes())
            result.Append(Rune.IsControl(rune) || rune.Value is >= 0x202A and <= 0x202E or >= 0x2066 and <= 0x2069 ? $"\\u{rune.Value:X4}" : rune.ToString());
        return result.ToString();
    }
}
