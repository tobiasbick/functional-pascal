namespace Fpas.Compiler.Syntax;

internal sealed record AttributeSyntax(TypeSyntax Type, IReadOnlyList<ExpressionSyntax> Arguments);

internal sealed partial class Parser
{
    private NestedRoutineSyntax? ReadAttributedRoutine(DocumentationComment? documentation)
    {
        var saved = position;
        IReadOnlyList<AttributeSyntax> attributes;
        try { attributes = ReadAttributes(); }
        catch (CompilationFailure) { position = saved; return null; }
        if (!At("function") && !At("procedure") && !At("async") && !At("public") && !At("private"))
        { position = saved; return null; }
        return new(ReadRoutine(false, nested: true) with { Attributes = attributes, Documentation = documentation });
    }

    private IReadOnlyList<AttributeSyntax> ReadAttributes()
    {
        var attributes = new List<AttributeSyntax>();
        while (At("["))
        {
            position++;
            var type = ReadType(arrays: false);
            var arguments = At("(") ? ReadArguments() : [];
            Expect("]");
            attributes.Add(new(type, arguments));
        }
        return attributes;
    }
}
