namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private int typeDepth;

    private TypeSyntax ReadType(bool arrays = true)
    {
        if (typeDepth == 256)
            throw source.Error("FP2003", "Type syntax exceeds 256 nesting levels.", Current.Position);
        typeDepth++;
        try
        {
            var type = ReadTypeCore(arrays);
            if (arrays && At("[") && tokens[position + 1].Text is "]" or ",")
                throw source.Error("FP2003", "Spell an array type with 'array of' or 'array[,] of'.", Current.Position);
            return type;
        }
        finally { typeDepth--; }
    }

    private TypeSyntax ReadTypeCore(bool arrays)
    {
        if (arrays && At("array")) return ReadArrayType();
        if (arrays && At("("))
        {
            var open = tokens[position++];
            var type = ReadArrayType();
            Expect(")");
            layout?.Tight.Add(position);
            Expect("?");
            return type with { Position = open.Position, SpanEnd = LastEnd, Nullable = true };
        }
        if (At("function") || At("procedure")) return ReadCallableType();

        var segments = new List<TypeSegmentSyntax>();
        do
        {
            var name = Identifier();
            segments.Add(new(name, At("<") ? ReadTypeArguments() : null));
            if (!At(".")) break;
            position++;
        } while (true);
        var nullable = At("?");
        if (nullable)
        {
            layout?.Tight.Add(position);
            position++;
            if (At("?")) throw source.Error("FP2003", "Repeated nullable suffixes are invalid.", Current.Position);
        }
        return new TypeSyntax(segments, Nullable: nullable) { SpanEnd = LastEnd };
    }

    private TypeSyntax ReadArrayType()
    {
        var array = Current;
        Expect("array");
        var rank = 1;
        if (At("["))
        {
            layout?.Tight.Add(position);
            position++;
            if (!At(",")) throw Expected([","], "Use 'array of' for a vector and commas for a rectangular array.");
            while (At(",")) { rank++; position++; }
            Expect("]");
        }
        Expect("of");
        var element = ReadType();
        return new TypeSyntax([new(array)], Element: element, Rank: rank) { SpanEnd = LastEnd };
    }

    private IReadOnlyList<TypeSyntax> ReadTypeArguments()
        => ReadList("<", ">", ",", () => ReadType(), empty: false, wrap: false, tight: true);
}
