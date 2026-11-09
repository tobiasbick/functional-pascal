using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Diagnostics;

internal static class DiagnosticHints
{
    internal static CompilerDiagnostic Enrich(CompilerDiagnostic diagnostic, SourceText source, int position)
    {
        var end = Math.Clamp(position, 0, source.Text.Length);
        while (end < source.Text.Length && (char.IsLetterOrDigit(source.Text[end]) || source.Text[end] == '_')) end++;
        var word = source.Text[Math.Clamp(position, 0, source.Text.Length)..end];
        var replacement = diagnostic.Code == "FP3005" ? word.ToLowerInvariant() switch
        {
            "integer" => "int", "boolean" => "bool", "real" => "double",
            "ansistring" or "widestring" => "string", _ => null
        } : null;
        if (replacement is not null)
            return diagnostic with
            {
                Location = source.Location(position, word.Length),
                Explanation = "Type aliases from other Pascal dialects are not predefined CLR aliases.",
                Help = $"Use the CLR type alias '{replacement}'. FPAS identifiers are case-sensitive.",
                Expectation = new("type", [replacement], word),
                Fixes = [Replace(source, position, word, replacement, $"Use {replacement}")]
            };
        if (diagnostic.Code == "FP1001" && position >= 0 &&
            (source.Text.AsSpan(position).StartsWith("!=", StringComparison.Ordinal) || source.Text.AsSpan(position).StartsWith("/=", StringComparison.Ordinal)))
            return diagnostic with
            {
                Location = source.Location(position, 2), Help = "Use '<>' for inequality.",
                Expectation = new("operator", ["<>"], source.Text.Substring(position, 2)),
                Fixes = [Replace(source, position, source.Text.Substring(position, 2), "<>", "Use <>")]
            };
        if (diagnostic.Code == "FP1001" && position >= 0 &&
            (source.Text.AsSpan(position).StartsWith("{", StringComparison.Ordinal) || source.Text.AsSpan(position).StartsWith("(*", StringComparison.Ordinal)))
            return diagnostic with
            {
                Help = "FPAS uses '//' line comments. Replace a Pascal block comment with one '//' comment per line.",
                Expectation = new("comment", ["//"], source.Text[position] == '{' ? "{" : "(*")
            };
        if (diagnostic.Code == "FP3001" && word.ToLowerInvariant() is "writeln" or "readln")
            return diagnostic with { Help = "Import System.Console explicitly, then call Console.WriteLine or Console.ReadLine; consume returned values." };
        if (diagnostic.Code == "FP3001" && word.Equals("exit", StringComparison.OrdinalIgnoreCase))
            return diagnostic with { Help = "Use return; in a procedure, or return Value; in a function." };
        return diagnostic;
    }

    internal static DiagnosticFix Replace(SourceText source, int position, string oldText, string newText, string title)
        => new(title, DiagnosticApplicability.Automatic, [new(source.Location(position, oldText.Length), oldText, newText)]);
}
