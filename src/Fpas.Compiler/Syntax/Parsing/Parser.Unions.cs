namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private UnionSyntax ReadUnion(bool isProgram)
    {
        var start = Current.Position;
        var documentation = Current.Documentation;
        var visible = At("public");
        if (isProgram && visible) throw source.Error("FP2003", "Public types belong in units.", Current.Position);
        if (At("public") || At("private")) position++;
        Expect("type"); var name = Identifier();
        using var declaration = Declaration(name, start, visible ? "public" : "private", documentation);
        declaration.Info.Kind = "union";
        var parameters = ReadTypeParameters();
        Expect("="); var constraints = ReadConstraints(); Expect("union");
        var cases = new List<UnionCaseSyntax>();
        var bodyStart = Current.Position;
        var bodyIndex = position;
        while (!At("end"))
        {
            layout?.Line(position);
            var caseDocumentation = Current.Documentation;
            var label = Identifier(); IReadOnlyList<ParameterSyntax> fields = [];
            using var caseDeclaration = Declaration(label, label.Position, "public", caseDocumentation);
            caseDeclaration.Info.Kind = "union_case";
            if (At("("))
            {
                fields = ReadList("(", ")", ";", () =>
                {
                    var field = Identifier();
                    using var fieldDeclaration = Declaration(field, field.Position, "public");
                    return new ParameterSyntax(field, ReadBindingType(field));
                }, empty: false, wrap: false);
            }
            Expect(";"); cases.Add(new(label, fields, caseDocumentation));
        }
        layout?.Body(bodyIndex, position);
        declaration.Body(bodyStart, Current.Position);
        Expect("end"); Expect("union"); Expect(";");
        return new(name, visible, cases, parameters, constraints, documentation);
    }

    private (int Start, ExpressionSyntax Selector, IReadOnlyList<CaseArmSyntax> Arms) ReadCase(bool expression)
    {
        var start = Current.Position; Expect("case");
        var selector = ReadExpression(); Expect("of");
        var bodyIndex = position;
        var arms = new List<CaseArmSyntax>();
        while (At("when") || At("else"))
        {
            layout?.Line(position);
            var armPosition = Current.Position;
            var fallback = At("else"); position++;
            var patterns = new List<PatternSyntax>();
            if (fallback) patterns.Add(new AnyPatternSyntax(armPosition));
            else
            {
                do
                {
                    patterns.Add(ReadPattern());
                    if (!At(",")) break;
                    position++;
                } while (true);
            }
            ExpressionSyntax? guard = null;
            if (!fallback)
            {
                if (At("if")) { position++; guard = ReadExpression(); }
                Expect(":");
            }
            if (expression)
            {
                var value = ReadExpression(); Expect(";");
                arms.Add(new(armPosition, patterns, guard, null, value));
            }
            else arms.Add(new(armPosition, patterns, guard, ReadStatements("when", "else", "end"), null));
            if (fallback) break;
        }
        layout?.Body(bodyIndex, position);
        Expect("end"); Expect("case");
        return (start, selector, arms);
    }

    private PatternSyntax ReadPattern()
    {
        using var nesting = EnterNesting();
        var start = Current.Position;
        if (At("_")) { position++; return new AnyPatternSyntax(start); }
        if (At("const"))
        {
            position++; var name = Identifier();
            using var declaration = Declaration(name, start);
            return new BindingPatternSyntax(name, ReadBindingType(name));
        }
        if (At("is"))
        {
            position++; var type = ReadType(); Expect("as"); var name = Identifier();
            using var declaration = Declaration(name, start); declaration.Type(type);
            return new TypePatternSyntax(start, type, name);
        }
        if (Current.Kind == TokenKind.Identifier)
        {
            var name = ReadName();
            IReadOnlyList<PayloadPatternSyntax>? fields = null;
            if (At("("))
            {
                fields = ReadList("(", ")", ",", () =>
                {
                    Token? label = null;
                    if (Current.Kind == TokenKind.Identifier && tokens[position + 1].Text == ":=") { label = tokens[position++]; position++; }
                    return new PayloadPatternSyntax(label, ReadPattern());
                });
            }
            if (fields is null && At(".."))
            {
                CheckNesting(name.Parts.Count, name.Position);
                position++;
                ExpressionSyntax lowerName = new NameExpressionSyntax(name.Parts[0]) { SpanEnd = name.Parts[0].Position + name.Parts[0].Length };
                foreach (var part in name.Parts.Skip(1)) lowerName = new MemberSyntax(lowerName, part) { SpanStart = lowerName.SpanStart, SpanEnd = part.Position + part.Length };
                return new RangePatternSyntax(lowerName, ReadUnary());
            }
            return new NamedPatternSyntax(name, fields);
        }
        var lower = ReadUnary();
        if (!At("..")) return new ConstantPatternSyntax(lower);
        position++; return new RangePatternSyntax(lower, ReadUnary());
    }
}
