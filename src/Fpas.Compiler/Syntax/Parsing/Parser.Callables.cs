namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private TypeSyntax ReadCallableType()
    {
        var token = tokens[position++];
        Expect("(");
        var parameters = new List<TypeSyntax>();
        if (!At(")"))
        {
            parameters.Add(ReadType());
            while (At(",")) { position++; parameters.Add(ReadType()); }
        }
        Expect(")");
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
        Expect("(");
        var parameters = new List<ParameterSyntax>();
        if (!At(")"))
        {
            do
            {
                parameters.Add(ReadParameter());
                if (!At(";")) break;
                position++;
            } while (true);
        }
        Expect(")");
        TypeSyntax? result = null;
        if (token.Text == "function") { Expect(":"); result = ReadType(); }
        if (token.Text == "function" && At("=>"))
        {
            position++;
            return new(token.Position, parameters, result, ReadExpression(), null, IsAsync: isAsync);
        }
        var contracts = ReadContracts();
        Expect("begin");
        var body = ReadStatements();
        Expect("end"); Expect(token.Text);
        return new(token.Position, parameters, result, null, body, contracts, isAsync);
    }
}
