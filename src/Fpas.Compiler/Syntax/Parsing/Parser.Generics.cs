namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private IReadOnlyList<Token> ReadTypeParameters()
    {
        if (!At("<")) return [];
        position++;
        var names = new List<Token>();
        do
        {
            var name = Identifier(); names.Add(name);
            using (Declaration(name, name.Position)) { }
            if (!At(",")) break;
            position++;
        } while (true);
        Expect(">"); return names;
    }

    private IReadOnlyList<WhereSyntax> ReadConstraints()
    {
        var clauses = new List<WhereSyntax>();
        while (At("where"))
        {
            position++; var name = Identifier(); Expect(":");
            var constraints = new List<ConstraintSyntax>();
            do
            {
                var start = Current.Position;
                if (At("class"))
                {
                    position++; var nullable = At("?"); if (nullable) position++;
                    constraints.Add(new(start, nullable ? "class?" : "class", null));
                }
                else if (At("struct")) { position++; constraints.Add(new(start, "struct", null)); }
                else if (At("new")) { position++; Expect("("); Expect(")"); constraints.Add(new(start, "new", null)); }
                else constraints.Add(new(start, null, SurfaceType()));
                if (!At(",")) break;
                position++;
            } while (true);
            Expect(";"); clauses.Add(new(name, constraints));
        }
        return clauses;
    }
}
