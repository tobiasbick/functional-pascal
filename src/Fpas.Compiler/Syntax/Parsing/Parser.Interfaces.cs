namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private InterfaceSyntax ReadInterface(bool isProgram)
    {
        var start = Current.Position;
        var documentation = Current.Documentation;
        var isPublic = At("public");
        if (isProgram && isPublic) throw source.Error("FP2003", "Public type declarations belong in units.", Current.Position);
        if (At("public") || At("private")) position++;
        Expect("type");
        var name = Identifier();
        using var declaration = Declaration(name, start, isPublic ? "public" : "private", documentation);
        var parameters = ReadTypeParameters();
        Expect("=");
        var constraints = ReadConstraints();
        Expect("interface");
        var bases = new List<TypeSyntax>();
        if (At("extends"))
        {
            position++;
            bases.Add(SurfaceType());
            while (At(",")) { position++; bases.Add(SurfaceType()); }
        }
        var properties = new List<InterfacePropertySyntax>();
        var methods = new List<RoutineSyntax>();
        var bodyStart = Current.Position;
        while (!At("end"))
        {
            var memberStart = Current.Position;
            var memberDocumentation = Current.Documentation;
            var attributes = ReadAttributes();
            if (At("private") || At("static") || At("async"))
                throw source.Error("FP2003", "Interface members are public abstract instance signatures.", Current.Position);
            if (At("public")) position++;
            if (At("function") || At("procedure"))
            {
                var method = ReadRoutine(false, inRecord: true, abstractMember: true) with { Attributes = attributes, Documentation = memberDocumentation };
                methods.Add(method); Decorate(method.Name, memberStart, memberDocumentation);
            }
            else
            {
                var member = Identifier();
                using var propertyDeclaration = Declaration(member, memberStart, "public", memberDocumentation);
                Expect(":");
                var type = SurfaceType();
                Expect(";");
                properties.Add(new(member, type, memberDocumentation, attributes));
            }
        }
        declaration.Body(bodyStart, Current.Position);
        Expect("end"); Expect("interface"); Expect(";");
        return new(name, isPublic, properties, methods, parameters, constraints, bases);
    }
}
