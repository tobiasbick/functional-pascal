namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private IReadOnlyList<Token> ReadTypeParameters()
    {
        if (!At("<")) return [];
        return ReadList("<", ">", ",", () =>
        {
            var name = Identifier();
            using (Declaration(name, name.Position)) { }
            return name;
        }, empty: false, wrap: false, tight: true);
    }

    private IReadOnlyList<WhereSyntax> ReadConstraints()
    {
        var clauses = new List<WhereSyntax>();
        while (At("where"))
        {
            layout?.Line(position);
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
        if (clauses.Count != 0) layout?.Line(position);
        return clauses;
    }
}
