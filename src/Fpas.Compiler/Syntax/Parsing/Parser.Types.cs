// [Documentation](../../../../docs/specs/syntax.md#sources-and-declarations)
namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private int typeDepth;

    internal TypeSyntax ParseStandaloneType()
    {
        var type = ReadType();
        if (Current.Kind != TokenKind.End)
            throw source.Error("FP2003", "Unexpected content after the type argument.", Current.Position, Current.Length);
        return type;
    }

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
        if (At("("))
        {
            var open = tokens[position++];
            var type = ReadType(arrays);
            Expect(")");
            return ReadGroupedTypeSuffix(type, open.Position);
        }
        if (At("function") || At("procedure")) return ReadCallableType();
        return ReadNamedType([]);
    }

    private TypeSyntax ReadNamedType(List<TypeSegmentSyntax> segments)
    {
        do
        {
            var name = Identifier();
            segments.Add(new(name, AtTypeArguments() ? ReadTypeArguments() : null));
            if (!At(".")) break;
            position++;
        } while (true);
        return ReadNullableSuffix(new(segments));
    }

    private TypeSyntax ReadGroupedTypeSuffix(TypeSyntax type, int start)
    {
        var unaryGeneric = type.Rank == 0 && type.Segments[^1].Arguments?.Count == 1;
        if (!type.Nullable && unaryGeneric && At("."))
        {
            position++;
            return ReadNamedType([.. type.Segments]) with { Position = start };
        }
        if (!type.Nullable && (type.Rank != 0 || unaryGeneric) && At("?"))
            return ReadNullableSuffix(type with { Position = start });
        throw source.Error("FP2003", "Use parentheses for multiple type arguments or to qualify a type before '.' or '?'.", start);
    }

    private TypeSyntax ReadNullableSuffix(TypeSyntax type)
    {
        if (At("?"))
        {
            layout?.Tight.Add(position);
            position++;
            if (At("?")) throw source.Error("FP2003", "Repeated nullable suffixes are invalid.", Current.Position);
            type = type with { Nullable = true };
        }
        return type with { SpanEnd = LastEnd };
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

    // The 'of' terminating a case selector is followed by 'when', 'else', or 'end'.
    private bool AtTypeArguments() => At("of") && tokens[position + 1] is
        { Kind: TokenKind.Identifier } or { Kind: TokenKind.Symbol, Text: "(" } or
        { Kind: TokenKind.Keyword, Text: "array" or "function" or "procedure" };

    private IReadOnlyList<TypeSyntax> ReadTypeArguments()
    {
        Expect("of");
        if (!At("(")) return [ReadType()];

        var start = Current.Position;
        var types = ReadList("(", ")", ",", () => ReadType(), empty: false, wrap: false);
        return types.Count == 1 ? [ReadGroupedTypeSuffix(types[0], start)] : types;
    }
}
