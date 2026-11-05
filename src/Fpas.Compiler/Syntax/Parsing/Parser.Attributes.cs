namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private NestedRoutineSyntax? ReadAttributedRoutine(DocumentationComment? documentation)
    {
        var saved = position;
        var probe = new Parser(source, tokens) { position = position };
        try { probe.ReadAttributes(); }
        catch (CompilationFailure) { return null; }
        if (!probe.At("function") && !probe.At("procedure") && !probe.At("async") && !probe.At("public") && !probe.At("private")) return null;
        var attributes = ReadAttributes();
        var routine = ReadRoutine(false, nested: true) with { Attributes = attributes, Documentation = documentation };
        Decorate(routine.Name, tokens[saved].Position, documentation);
        return new(routine);
    }

    private IReadOnlyList<AttributeSyntax> ReadAttributes()
    {
        var attributes = new List<AttributeSyntax>();
        while (At("["))
        {
            layout?.Line(position);
            position++;
            var type = ReadType(arrays: false);
            var arguments = At("(") ? ReadArguments() : [];
            Expect("]");
            layout?.Line(position);
            attributes.Add(new(type, arguments));
        }
        return attributes;
    }
}
