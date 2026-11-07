namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    internal ExpressionSyntax ParseExpressionFragment() => ReadExpression();

    private ExpressionSyntax Extent(ExpressionSyntax expression, int start)
        => expression with { SpanStart = start, SpanEnd = tokens[position - 1].Position + tokens[position - 1].Length };

    private const int LogicalPrecedence = 4;

    private static int Precedence(Token token) => token.Kind is not (TokenKind.Symbol or TokenKind.Keyword) ? 0 : token.Text switch
    {
        "??" => 1, "or" or "xor" or "and" => LogicalPrecedence,
        "=" or "<>" => 5, "<" or "<=" or ">" or ">=" or "is" => 6,
        "shl" or "shr" => 7, "+" or "-" => 8, "*" or "/" or "div" or "mod" => 9,
        _ => 0
    };

    private ExpressionSyntax ReadExpression(int minimum = 1)
    {
        var start = position;
        var left = ReadUnary();
        string? logicalOperator = null;
        while (Precedence(Current) >= minimum)
        {
            var operatorIndex = position;
            var op = ReadBinaryOperator();
            var rightIndex = position;
            var precedence = Precedence(op);
            if (precedence == LogicalPrecedence)
            {
                logicalOperator ??= op.Text;
                if (logicalOperator != op.Text)
                    throw source.Error("FP2003", "Group mixed 'and', 'or', and 'xor' operators with parentheses.", op.Position, op.Length);
            }
            if (op.Text == "is")
            {
                left = Extent(new TypeTestSyntax(left, ReadType()), left.SpanStart);
                layout?.Binary.Add(new(start, position, operatorIndex, rightIndex));
                continue;
            }
            var right = ReadExpression(op.Text == "??" ? precedence : precedence + 1);
            if (precedence is 5 or 6 && (IsComparison(left) || IsComparison(right)))
                throw source.Error("FP2003", "Comparisons cannot be chained; use parentheses or Boolean operators.", op.Position);
            left = Extent(new BinarySyntax(left, op, right), left.SpanStart);
            layout?.Binary.Add(new(start, position, operatorIndex, rightIndex));
        }
        return left;
    }

    private Token ReadBinaryOperator()
    {
        var op = tokens[position++];
        if (op.Text == ">" && At("=") && op.Position + op.Length == Current.Position)
        {
            // Comparisons require adjacent characters; '> =' remains invalid.
            op = op with { Text = ">=", Length = op.Length + Current.Length };
            layout?.Tight.Add(position);
            position++;
        }
        return op;
    }

    private ExpressionSyntax ReadUnary()
    {
        if (At("await")) { var token = tokens[position++]; return Extent(new AwaitSyntax(token.Position, ReadUnary()), token.Position); }
        if (At("+") || At("-") || At("not"))
        {
            layout?.Unary.Add(position);
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
            var angle = position;
            Expect("<");
            var type = ReadType();
            Expect(">");
            if (layout is not null) { layout.Tight.Add(angle); layout.Tight.Add(position - 1); }
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
                var probe = layout is null ? this : new Parser(source, tokens) { position = position };
                IReadOnlyList<TypeSyntax>? arguments = null;
                try { arguments = probe.ReadTypeArguments(); }
                catch (CompilationFailure) { probe.position = saved; }
                if (arguments is null || !probe.At(".") && !probe.At("(") && !probe.At(";") && !probe.At(",") && !probe.At(")") && !probe.At("]") && !probe.At("else") && !probe.At("end")) { position = saved; break; }
                if (layout is not null) arguments = ReadTypeArguments();
                expression = new GenericNameSyntax(expression, arguments);
            }
            else
            {
                var argumentsStart = Current.Position;
                expression = new CallSyntax(expression, ReadArguments(), argumentsStart);
            }
            expression = Extent(expression, start);
        }
        return expression;
    }

    private IReadOnlyList<ExpressionSyntax> ReadIndices()
        => ReadList("[", "]", ",", () => ReadExpression());

    private IReadOnlyList<ExpressionSyntax> ReadArguments()
        => ReadList("(", ")", ",", ReadCallArgument);

    private ExpressionSyntax ReadCallArgument()
    {
        if (Current.Kind == TokenKind.Identifier && position + 1 < tokens.Count && tokens[position + 1].Text == ":=")
        {
            var name = tokens[position++]; position++;
            return Extent(new NamedArgumentSyntax(name, ReadArgumentValue()), name.Position);
        }
        return ReadArgumentValue();
    }

    private static bool IsComparison(ExpressionSyntax expression)
        => expression is BinarySyntax binary && Precedence(binary.Operator) is 5 or 6;

    private ConditionalSyntax ReadConditional()
    {
        var layoutStart = position;
        var branches = layout is null ? null : new List<int>();
        var start = Current.Position;
        var arms = new List<ConditionalArmSyntax>();
        do
        {
            if (arms.Count != 0) branches?.Add(position);
            position++;
            var condition = ReadExpression();
            Expect("then");
            arms.Add(new(condition, ReadExpression()));
        } while (At("elsif"));
        branches?.Add(position);
        Expect("else");
        var otherwise = ReadExpression();
        branches?.Add(position);
        Expect("end");
        Expect("if");
        if (branches is not null) layout!.Conditionals.Add(new(layoutStart, position, branches));
        return new(start, arms, otherwise);
    }
}
