namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private TrySyntax ReadTry()
    {
        var start = Current.Position;
        position++;
        var body = ReadStatements("catch", "finally", "end");
        var catches = new List<CatchSyntax>();
        while (At("catch"))
        {
            var catchStart = Current.Position;
            position++;
            Token? name = null;
            TypeSyntax? type = null;
            if (Current.Kind == TokenKind.Identifier)
            {
                name = Identifier();
                Expect(":");
                type = ReadType();
            }
            ExpressionSyntax? filter = null;
            if (At("when"))
            {
                position++;
                filter = ReadExpression();
            }
            Expect("do");
            catches.Add(new(catchStart, name, type, filter, ReadStatements("catch", "finally", "end")));
        }
        IReadOnlyList<StatementSyntax>? cleanup = null;
        if (At("finally"))
        {
            position++;
            cleanup = ReadStatements();
        }
        Expect("end");
        Expect("try");
        if (catches.Count == 0 && cleanup is null)
            throw source.Error("FP2003", "A try statement requires catch or finally.", start);
        return new(start, body, catches, cleanup);
    }
}
