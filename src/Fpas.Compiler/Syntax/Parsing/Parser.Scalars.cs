namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private void ReadScalarDeclaration(bool isProgram, List<AliasSyntax> aliases, List<ScalarSyntax> scalars, IReadOnlyList<AttributeSyntax> attributes, DocumentationComment? documentation)
    {
        var visible = At("public");
        if (isProgram && visible) throw source.Error("FP2003", "Public types belong in units.", Current.Position);
        if (At("public") || At("private")) position++;
        Expect("type"); var name = Identifier();
        if (At("<")) throw source.Error("FP3031", "Aliases, enums, and scalar domains do not declare type parameters.", Current.Position);
        Expect("=");
        if (At("enum"))
        {
            position++;
            var type = new TypeSyntax([new(new(TokenKind.Identifier, "int", name.Position, name.Length))]);
            if (At(":")) { position++; type = ReadType(); }
            var members = new List<EnumMemberSyntax>();
            while (!At("end"))
            {
                var doc = Current.Documentation; var label = Identifier(); ExpressionSyntax? value = null;
                if (At("=")) { position++; value = ReadExpression(); }
                Expect(";"); members.Add(new(label, value, doc));
            }
            Expect("end"); Expect("enum"); Expect(";");
            scalars.Add(new(name, visible, ScalarCategory.Enum, type, null, null, members, documentation, attributes)); return;
        }
        var distinct = At("distinct"); if (distinct) position++;
        var baseType = ReadType();
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
