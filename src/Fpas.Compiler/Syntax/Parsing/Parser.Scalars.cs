namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private void ReadScalarDeclaration(bool isProgram, List<AliasSyntax> aliases, List<ScalarSyntax> scalars, IReadOnlyList<AttributeSyntax> attributes, DocumentationComment? documentation)
    {
        var start = Current.Position;
        var visible = At("public");
        if (isProgram && visible) throw source.Error("FP2003", "Public types belong in units.", Current.Position);
        if (At("public") || At("private")) position++;
        Expect("type"); var name = Identifier();
        using var declaration = Declaration(name, start, visible ? "public" : "private", documentation);
        if (At("<")) throw source.Error("FP3031", "Aliases, enums, and scalar domains do not declare type parameters.", Current.Position);
        Expect("=");
        if (At("enum"))
        {
            position++;
            var type = new TypeSyntax([new(new(TokenKind.Identifier, "int", name.Position, name.Length))]);
            if (At(":")) { position++; type = SurfaceType(); }
            var members = new List<EnumMemberSyntax>();
            var bodyStart = Current.Position;
            while (!At("end"))
            {
                var doc = Current.Documentation; var label = Identifier(); ExpressionSyntax? value = null;
                using var memberDeclaration = Declaration(label, label.Position, "public", doc);
                if (At("=")) { position++; value = ReadExpression(); }
                Expect(";"); members.Add(new(label, value, doc));
            }
            declaration.Body(bodyStart, Current.Position);
            Expect("end"); Expect("enum"); Expect(";");
            scalars.Add(new(name, visible, ScalarCategory.Enum, type, null, null, members, documentation, attributes)); return;
        }
        var distinct = At("distinct"); if (distinct) position++;
        var baseType = SurfaceType();
        if (distinct) { Expect(";"); scalars.Add(new(name, visible, ScalarCategory.Distinct, baseType, null, null, [], documentation, attributes)); return; }
        if (At("range"))
        {
            position++; var lower = ReadExpression(); Expect(".."); var upper = ReadExpression(); Expect(";");
            scalars.Add(new(name, visible, ScalarCategory.Range, baseType, lower, upper, [], documentation, attributes)); return;
        }
        if (attributes.Count != 0) throw source.Error("FP3035", "A transparent alias has no CLR attribute target.", name.Position);
        Expect(";"); aliases.Add(new(name, visible, baseType, documentation));
    }
}
