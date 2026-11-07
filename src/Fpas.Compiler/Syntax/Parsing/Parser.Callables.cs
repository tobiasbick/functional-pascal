namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private TypeSyntax ReadCallableType()
    {
        var token = tokens[position++];
        var parameters = ReadList("(", ")", ",", () => ReadType(), wrap: false);
        TypeSyntax? result = null;
        if (token.Text == "function") { Expect(":"); result = ReadType(); }
        return new([new(token)], CallableParameters: parameters, CallableResult: result) { SpanEnd = tokens[position - 1].Position + tokens[position - 1].Length };
    }

    private LambdaSyntax ReadLambda()
    {
        var isAsync = At("async");
        if (isAsync) position++;
        var token = tokens[position++];
        if (isAsync && token.Text != "function") throw source.Error("FP2003", "There are no async procedures.", token.Position);
        var parameters = ReadList("(", ")", ";", ReadParameter, wrap: false);
        TypeSyntax? result = null;
        if (token.Text == "function") { Expect(":"); result = ReadType(); }
        if (token.Text == "function" && At("=>"))
        {
            position++;
            return new(token.Position, parameters, result, ReadExpression(), null, IsAsync: isAsync);
        }
        var contracts = ReadContracts();
        var helpers = ReadRoutineHelpers();
        if (contracts.Count != 0 || helpers.Count != 0) layout?.Line(position);
        Expect("begin");
        var body = ReadStatements(allowRoutineDeclarations: false);
        if (body.Count == 0 && layout is not null) layout.Breaks[position] = 0;
        Expect("end"); Expect(token.Text);
        return new(token.Position, parameters, result, null, helpers.Count == 0 ? body : [..helpers, ..body], contracts, isAsync);
    }

    private List<StatementSyntax> ReadRoutineHelpers()
    {
        var start = position;
        var helpers = new List<StatementSyntax>();
        while (At("function") || At("procedure") || At("async") || At("["))
        {
            layout?.Line(position);
            var helper = At("[")
                ? ReadAttributedRoutine(Current.Documentation) ?? throw Expected(["function", "procedure"], "Expected a local routine after attributes.")
                : new NestedRoutineSyntax(ReadRoutine(false, nested: true));
            Expect(";");
            declarations[helper.Routine.Name.Position].End = LastEnd;
            helpers.Add(helper);
        }
        if (helpers.Count != 0) layout?.Body(start, position);
        return helpers;
    }
}
