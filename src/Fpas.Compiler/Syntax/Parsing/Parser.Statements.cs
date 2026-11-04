namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private IReadOnlyList<StatementSyntax> ReadStatements(params string[] terminators)
    {
        if (terminators.Length == 0)
            terminators = ["end"];
        var statements = new List<StatementSyntax>();
        while (!terminators.Any(At) && Current.Kind != TokenKind.End)
        {
            var statement = ReadStatement(); statements.Add(statement);
            Expect(";");
            var name = statement switch { BindingSyntax binding => binding.Name, UsingSyntax use => use.Binding.Name, NestedRoutineSyntax nested => nested.Routine.Name, _ => null };
            if (name is not null) declarations[name.Position].End = LastEnd;
        }
        return statements;
    }

    private StatementSyntax ReadStatement()
    {
        var start = Current.Position;
        var documentation = Current.Documentation;
        if (At("subscribe") || At("unsubscribe"))
        {
            var subscribe = At("subscribe"); position++;
            var reference = ReadPostfix(allowWith: false);
            Expect("with");
            return new SubscriptionSyntax(start, reference, ReadExpression(), subscribe);
        }
        if (At("[") && ReadAttributedRoutine(documentation) is { } attributed) return attributed;
        if (At("function") || At("procedure") || At("async")) return new NestedRoutineSyntax(ReadRoutine(false, nested: true));
        if (At("const") || At("var"))
        {
            var mutable = At("var");
            position++;
            var name = Identifier();
            using var declaration = Declaration(name, start, documentation: documentation);
            var type = ReadBindingType(name);
            Expect(":=");
            return new BindingSyntax(name, type, ReadExpression(), mutable, documentation);
        }
        if (At("using") || At("await") && tokens[position + 1].Text == "using")
        {
            var isAsync = At("await");
            if (isAsync) position++;
            position++;
            Expect("const");
            var name = Identifier();
            using var declaration = Declaration(name, start, documentation: documentation);
            var type = ReadBindingType(name);
            Expect(":=");
            return new UsingSyntax(new(name, type, ReadExpression(), false, documentation), isAsync);
        }
        if (At("try")) return ReadTry();
        if (At("case")) { var parsed = ReadCase(expression: false); return new CaseStatementSyntax(parsed.Start, parsed.Selector, parsed.Arms); }
        if (At("raise"))
        {
            position++;
            return new RaiseSyntax(start, At(";") ? null : ReadExpression());
        }
        if (At("return"))
        {
            position++;
            return new ReturnSyntax(start, At(";") ? null : ReadExpression());
        }
        if (At("begin"))
        {
            position++;
            var body = ReadStatements();
            Expect("end");
            return new BlockSyntax(start, body);
        }
        if (At("if"))
            return ReadIfStatement();
        if (At("while"))
        {
            position++;
            var condition = ReadExpression();
            Expect("do");
            var body = ReadStatements();
            Expect("end");
            Expect("while");
            return new WhileSyntax(start, condition, body);
        }
        if (At("for"))
            return ReadFor();
        if (At("repeat"))
        {
            position++;
            var body = ReadStatements("until");
            var until = Current; Expect("until");
            return new RepeatSyntax(start, until, body, ReadExpression());
        }
        if (At("break") || At("continue"))
        {
            var isBreak = At("break");
            position++;
            return new LoopExitSyntax(start, isBreak);
        }
        var discard = At("discard");
        if (discard)
            position++;
        var expression = ReadExpression();
        if (!discard && At(":="))
        {
            position++;
            return new AssignmentSyntax(expression, ReadExpression());
        }
        return new ExpressionStatementSyntax(expression, discard);
    }

    private TypeSyntax ReadBindingType(Token name)
    {
        if (!At(":"))
            throw source.Error("FP2002", $"Binding '{name.Text}' requires an explicit type annotation ': T'.", Current.Position, Current.Length);
        position++;
        return SurfaceType();
    }

    private IfSyntax ReadIfStatement()
    {
        var start = Current.Position;
        var arms = new List<IfArmSyntax>();
        do
        {
            position++;
            var condition = ReadExpression();
            Expect("then");
            arms.Add(new(condition, ReadStatements("elsif", "else", "end")));
        } while (At("elsif"));
        IReadOnlyList<StatementSyntax> otherwise = [];
        if (At("else"))
        {
            position++;
            otherwise = ReadStatements();
        }
        Expect("end");
        Expect("if");
        return new(start, arms, otherwise);
    }

    private StatementSyntax ReadFor()
    {
        var start = Current.Position;
        position++;
        var name = Identifier();
        TypeSyntax type;
        using (Declaration(name, name.Position)) type = ReadBindingType(name);
        if (At("in"))
        {
            position++;
            var collection = ReadExpression();
            Expect("do");
            var items = ReadStatements();
            Expect("end"); Expect("for");
            return new ForEachSyntax(start, name, type, collection, items);
        }
        Expect(":=");
        var lower = ReadExpression();
        var descending = At("downto");
        Expect(descending ? "downto" : "to");
        var upper = ReadExpression();
        Expect("do");
        var body = ReadStatements();
        Expect("end");
        Expect("for");
        return new ForSyntax(start, name, type, lower, upper, descending, body);
    }
}
