namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private ExpressionSyntax Extent(ExpressionSyntax expression, int start)
        => expression with { SpanStart = start, SpanEnd = tokens[position - 1].Position + tokens[position - 1].Length };

    private static int Precedence(Token token) => token.Kind is not (TokenKind.Symbol or TokenKind.Keyword) ? 0 : token.Text switch
    {
        "??" => 1, "or" => 2, "xor" => 3, "and" => 4,
        "=" or "<>" => 5, "<" or "<=" or ">" or ">=" or "is" => 6,
        "shl" or "shr" => 7, "+" or "-" => 8, "*" or "/" or "div" or "mod" => 9,
        _ => 0
    };

    private ExpressionSyntax ReadExpression(int minimum = 1)
    {
        var left = ReadUnary();
        while (Precedence(Current) >= minimum)
        {
            var op = tokens[position++];
            var precedence = Precedence(op);
            if (op.Text == "is") { left = Extent(new TypeTestSyntax(left, ReadType()), left.SpanStart); continue; }
            var right = ReadExpression(op.Text == "??" ? precedence : precedence + 1);
            if (precedence is 5 or 6 && (IsComparison(left) || IsComparison(right)))
                throw source.Error("FP2003", "Comparisons cannot be chained; use parentheses or Boolean operators.", op.Position);
            left = Extent(new BinarySyntax(left, op, right), left.SpanStart);
        }
        return left;
    }

    private ExpressionSyntax ReadUnary()
    {
        if (At("await")) { var token = tokens[position++]; return Extent(new AwaitSyntax(token.Position, ReadUnary()), token.Position); }
        if (At("+") || At("-") || At("not"))
        {
            var token = tokens[position++]; return Extent(new UnarySyntax(token, ReadUnary()), token.Position);
        }
        var start = Current.Position;
        if (At("unchecked"))
        {
            position++;
            Expect("(");
            var operand = ReadExpression();
            Expect(")");
            return ReadSuffix(Extent(new UncheckedSyntax(start, operand), start));
        }
        if (At("cast"))
        {
            position++;
            Expect("<");
            var type = ReadType();
            Expect(">");
            Expect("(");
            var operand = ReadExpression();
            Expect(")");
            return ReadSuffix(Extent(new CastSyntax(start, type, operand), start));
        }
        return ReadPostfix();
    }

    private ExpressionSyntax ReadPostfix(bool allowWith = true)
    {
        ExpressionSyntax expression;
        var start = Current.Position;
        if (At("async") || At("function") || At("procedure")) expression = ReadLambda();
        else if (At("typeof"))
        {
            position++; Expect("(");
            var type = ReadType(); Expect(")");
            expression = new TypeOfSyntax(start, type);
        }
        else if (At("new"))
        {
            position++;
            var type = ReadType(arrays: false);
            expression = At("[") ? new NewArraySyntax(start, type, ReadIndices()) : new NewSyntax(start, type, ReadArguments());
        }
        else if (At("[")) expression = new ArrayLiteralSyntax(start, ReadIndices());
        else if (At("case")) { var parsed = ReadCase(expression: true); expression = new CaseExpressionSyntax(parsed.Start, parsed.Selector, parsed.Arms); }
        else if (At("if")) expression = ReadConditional();
        else if (Current.Kind is TokenKind.String or TokenKind.Integer or TokenKind.Real or TokenKind.Character || At("true") || At("false") || At("null"))
            expression = new LiteralSyntax(tokens[position++]);
        else if (At("("))
        {
            position++;
            expression = new ParenthesizedSyntax(start, ReadExpression());
            Expect(")");
        }
        else if (Current.Kind == TokenKind.Identifier) expression = new NameExpressionSyntax(tokens[position++]);
        else throw source.Error("FP2003", $"Expected an expression; '{Current.Text}' is not supported here.", Current.Position, Current.Length,
            expectation: new("expression", ["expression"], Current.Kind == TokenKind.End ? "end of file" : Current.Text));
        return ReadSuffix(Extent(expression, start), allowWith);
    }

    private ExpressionSyntax ReadSuffix(ExpressionSyntax expression, bool allowWith = true)
    {
        while (At(".") || At("(") || At("[") || At("<") || allowWith && At("with"))
        {
            var start = expression.SpanStart;
            if (At("with")) expression = ReadWith(expression);
            else if (At("."))
            {
                position++;
                expression = new MemberSyntax(expression, Identifier());
            }
            else if (At("[")) expression = new IndexSyntax(expression, ReadIndices());
            else if (At("<"))
            {
                var saved = position;
                IReadOnlyList<TypeSyntax>? arguments = null;
                try { arguments = ReadTypeArguments(); }
                catch (CompilationFailure) { position = saved; }
                if (arguments is null || !At(".") && !At("(") && !At(";") && !At(",") && !At(")") && !At("]") && !At("else") && !At("end")) { position = saved; break; }
                expression = new GenericNameSyntax(expression, arguments);
            }
            else expression = new CallSyntax(expression, ReadArguments());
            expression = Extent(expression, start);
        }
        return expression;
    }

    private IReadOnlyList<ExpressionSyntax> ReadIndices()
    {
        Expect("[");
        var values = new List<ExpressionSyntax>();
        if (!At("]"))
        {
            values.Add(ReadExpression());
            while (At(",")) { position++; values.Add(ReadExpression()); }
        }
        Expect("]");
        return values;
    }

    private IReadOnlyList<ExpressionSyntax> ReadArguments()
    {
        Expect("(");
        var arguments = new List<ExpressionSyntax>();
        if (!At(")"))
        {
            do
            {
                if (Current.Kind == TokenKind.Identifier && position + 1 < tokens.Count && tokens[position + 1].Text == ":=")
                {
                    var name = tokens[position++]; position++;
                    arguments.Add(Extent(new NamedArgumentSyntax(name, ReadArgumentValue()), name.Position));
                }
                else arguments.Add(ReadArgumentValue());
                if (!At(",")) break;
                position++;
            } while (true);
        }
        Expect(")");
        return arguments;
    }

    private static bool IsComparison(ExpressionSyntax expression)
        => expression is BinarySyntax binary && Precedence(binary.Operator) is 5 or 6;

    private ConditionalSyntax ReadConditional()
    {
        var start = Current.Position;
        var arms = new List<ConditionalArmSyntax>();
        do
        {
            position++;
            var condition = ReadExpression();
            Expect("then");
            arms.Add(new(condition, ReadExpression()));
        } while (At("elsif"));
        Expect("else");
        var otherwise = ReadExpression();
        Expect("end");
        Expect("if");
        return new(start, arms, otherwise);
    }
}
