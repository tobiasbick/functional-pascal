namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private IReadOnlyList<ContractSyntax> ReadContracts()
    {
        var clauses = new List<ContractSyntax>();
        var postconditions = false;
        while (At("requires") || At("ensures"))
        {
            var keyword = tokens[position++];
            var ensures = keyword.Text == "ensures";
            if (!ensures && postconditions)
                throw source.Error("FP3027", "Requires clauses must precede ensures clauses.", keyword.Position);
            postconditions |= ensures;
            ParameterSyntax? result = null;
            if (ensures && At("(") && position + 2 < tokens.Count &&
                tokens[position + 1].Kind == TokenKind.Identifier && tokens[position + 2].Text == ":")
            {
                position++;
                var name = Identifier();
                Expect(":");
                result = new(name, ReadType());
                Expect(")");
                Expect(":");
            }
            var start = Current.Position;
            var condition = ReadExpression();
            var text = source.Text[start..Current.Position].Trim();
            Expect(";");
            clauses.Add(new(keyword, result, condition, text));
        }
        return clauses;
    }
}
