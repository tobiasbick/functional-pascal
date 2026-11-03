namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private ParameterSyntax ReadParameter()
    {
        var mode = At("ref") || At("out") || At("in") || At("params") ? tokens[position++].Text : "";
        var name = Identifier();
        var type = ReadBindingType(name);
        ExpressionSyntax? value = null;
        if (At(":=")) { position++; value = ReadExpression(); }
        return new(name, type, mode, value);
    }

    private ExpressionSyntax ReadArgumentValue()
    {
        if (At("ref") || At("out") || At("in"))
        {
            var mode = tokens[position++];
            return new ArgumentModeSyntax(mode.Position, mode.Text, ReadExpression());
        }
        return ReadExpression();
    }
}
