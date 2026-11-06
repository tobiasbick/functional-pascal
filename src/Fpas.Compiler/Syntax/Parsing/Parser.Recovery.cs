namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    internal List<(int Start, int End)> InvalidRegions { get; } = [];

    private IReadOnlyList<StatementSyntax> ReadBody(int start, Func<IReadOnlyList<StatementSyntax>> read)
    {
        var before = source.Diagnostics.Count;
        try
        {
            var body = read();
            if (source.Diagnostics.Count != before) InvalidRegions.Add((start, LastEnd));
            return body;
        }
        catch (CompilationFailure failure) when (recover && failure.Diagnostics.Count == 1)
        {
            source.Diagnostics.Add(failure.Diagnostic);
            // Preserve a source closer consumed while looking for a routine closer.
            if ((At("unit") || At("program")) && position > 0 && tokens[position - 1].Text == "end") position--;
            if (Current.Kind != TokenKind.End && !At("end") && !At("public") && !At("private")) throw;
            InvalidRegions.Add((start, Math.Max(start, Current.Position)));
            return [];
        }
    }
}
