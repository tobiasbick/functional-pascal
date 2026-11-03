namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private InterfaceSyntax ReadInterface(bool isProgram)
    {
        var isPublic = At("public");
        if (isProgram && isPublic) throw source.Error("FP2003", "Public type declarations belong in units.", Current.Position);
        if (At("public") || At("private")) position++;
        Expect("type");
        var name = Identifier();
        var parameters = ReadTypeParameters();
        Expect("=");
        var constraints = ReadConstraints();
        Expect("interface");
        var bases = new List<TypeSyntax>();
        if (At("extends"))
        {
            position++;
            bases.Add(ReadType());
            while (At(",")) { position++; bases.Add(ReadType()); }
        }
        var properties = new List<InterfacePropertySyntax>();
        var methods = new List<RoutineSyntax>();
        while (!At("end"))
        {
            var documentation = Current.Documentation;
            var attributes = ReadAttributes();
            if (At("private") || At("static") || At("async"))
                throw source.Error("FP2003", "Interface members are public abstract instance signatures.", Current.Position);
            if (At("public")) position++;
            if (At("function") || At("procedure"))
                methods.Add(ReadRoutine(false, inRecord: true, abstractMember: true) with { Attributes = attributes, Documentation = documentation });
            else
            {
                var member = Identifier();
                Expect(":");
                var type = ReadType();
                Expect(";");
                properties.Add(new(member, type, documentation, attributes));
            }
        }
        Expect("end"); Expect("interface"); Expect(";");
        return new(name, isPublic, properties, methods, parameters, constraints, bases);
    }
}
