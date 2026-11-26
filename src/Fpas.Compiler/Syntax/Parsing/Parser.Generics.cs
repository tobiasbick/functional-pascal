// [Documentation](../../../../docs/pascal/language/types/generics.md)
namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private IReadOnlyList<Token> ReadTypeParameters()
    {
        if (!At("of")) return [];
        position++;
        if (!At("(")) return [ReadParameterName()];

        var start = Current.Position;
        var names = ReadList("(", ")", ",", ReadParameterName, empty: false, wrap: false);
        if (names.Count < 2)
            throw source.Error("FP2003", "Write one type parameter directly after 'of'; use parentheses for multiple parameters.", start);
        return names;

        Token ReadParameterName()
        {
            var name = Identifier();
            using (Declaration(name, name.Position)) { }
            return name;
        }
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
