using Fpas.Compiler.Diagnostics;

namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private const int MaximumNesting = 256;
    private int syntaxDepth;

    private Nesting EnterNesting()
    {
        CheckNesting(syntaxDepth + 1, Current.Position);
        syntaxDepth++;
        return new(this);
    }

    private void CheckNesting(int depth, int offset)
    {
        if (depth > MaximumNesting)
            throw source.Error("FP2003", $"Syntax exceeds {MaximumNesting} nesting levels.", offset);
    }

    private readonly struct Nesting(Parser parser) : IDisposable
    {
        void IDisposable.Dispose() => parser.syntaxDepth--;
    }

    private CompilationFailure Expected(string[] expected, string message, string kind = "token", bool programEnd = false)
        => ExpectedAt(Current, expected, message, kind, programEnd);

    private CompilationFailure ExpectedAt(Token token, string[] expected, string message, string kind = "token", bool programEnd = false)
    {
        var found = token.Kind == TokenKind.End ? "end of file" : token.Text;
        var replacement = expected.FirstOrDefault(value => value.Equals(token.Text, StringComparison.OrdinalIgnoreCase) && value != token.Text);
        if (expected.Contains(":=") && token.Text == "=") replacement = ":=";
        if (expected.Contains("elsif") && token.Text == "elseif") replacement = "elsif";
        var fixes = new List<DiagnosticFix>();
        string? help = null;
        if (replacement is not null)
        {
            help = $"Use '{replacement}'. FPAS keywords are case-sensitive.";
            var fix = DiagnosticHints.Replace(source, token.Position, token.Text, replacement, $"Use {replacement}");
            fixes.Add(token.Text == "elseif" ? fix with { Applicability = DiagnosticApplicability.Review } : fix);
        }
        else if (expected is [";"] && (token.Kind == TokenKind.End || IsStatementStart() ||
            token.Kind == TokenKind.Keyword && token.Text is "end" or "else" or "elsif" or "catch" or "finally" or "until"))
        {
            help = "Terminate the preceding statement with ';', including before else or a block closer.";
            fixes.Add(DiagnosticHints.Replace(source, token.Position, "", ";", "Insert the missing semicolon"));
        }
        else if (programEnd && expected is [";"] && token.Text == ".")
            help = "FPAS closes blocks with ';', not '.'. End a program entry body with 'end;', followed by 'end program;'.";
        else if (expected is ["program", "unit"] && token.Text.Equals("procedure", StringComparison.OrdinalIgnoreCase))
            help = "An Ada-style main procedure is not a FPAS source container. Start with 'program Name;', declare routines before the entry body, then write 'begin ... end; end program;'.";
        return source.Error("FP2001", message, token.Position, token.Length, help,
            new(kind, expected, found), fixes: fixes);
    }

    private bool IsStatementStart() => Current.Kind == TokenKind.Keyword && Current.Text is
        "const" or "var" or "return" or "discard" or "if" or "while" or "for" or "repeat" or "try" or "raise" or "using" or "begin" or
        "function" or "procedure" or "async" or "case" or "subscribe" or "unsubscribe" or "break" or "continue" or "await";

    private void SynchronizeStatement(int start, IReadOnlyList<string> terminators)
    {
        // Recover only at a statement boundary; never bind a tree containing syntax errors.
        var depth = 0;
        var cursor = start;
        while (cursor < tokens.Count - 1)
        {
            source.CheckCancellation();
            var token = tokens[cursor];
            if (cursor >= position && depth == 0)
            {
                if (terminators.Contains(token.Text)) { position = cursor; return; }
                if (token.Text == ";") { position = cursor + 1; return; }
                if (cursor == position && position > start && IsStatementStart()) return;
            }
            if (token.Kind == TokenKind.Keyword)
            {
                if (token.Text is "begin" or "if" or "while" or "for" or "try" or "repeat" or "case") depth++;
                else if (token.Text == "end")
                {
                    if (depth == 0) { position = Math.Max(cursor, start + 1); return; }
                    depth--;
                    if (tokens[cursor + 1].Text is "if" or "while" or "for" or "case" or "try" or "function" or "procedure") cursor++;
                }
                else if (token.Text == "until" && depth > 0) depth--;
            }
            cursor++;
        }
        position = tokens.Count - 1;
    }
}
