namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private RecordSyntax ReadRecord(bool isProgram)
    {
        var start = Current.Position;
        var documentation = Current.Documentation;
        var isPublic = At("public");
        if (isProgram && isPublic) throw source.Error("FP2003", "Public type declarations belong in units.", Current.Position);
        if (At("public") || At("private")) position++;
        Expect("type");
        var name = Identifier();
        using var declaration = Declaration(name, start, isPublic ? "public" : "private", documentation);
        declaration.Info.Kind = "record";
        var parameters = ReadTypeParameters();
        Expect("=");
        var constraints = ReadConstraints();
        if (!At("record")) throw source.Error("FP2003", "This compiler currently supports record and union type declarations.", Current.Position);
        position++;
        var interfaces = new List<TypeSyntax>();
        if (At("implements"))
        {
            position++;
            interfaces.Add(SurfaceType());
            while (At(",")) { position++; interfaces.Add(SurfaceType()); }
        }
        var fields = new List<RecordFieldSyntax>();
        var methods = new List<RoutineSyntax>();
        var bodyStart = Current.Position;
        var bodyIndex = position;
        var previousMethod = false;
        while (!At("end"))
        {
            var memberIndex = position;
            layout?.Line(position);
            var memberStart = Current.Position;
            var memberDocumentation = Current.Documentation;
            var attributes = ReadAttributes();
            var member = position;
            if (At("public") || At("private")) member++;
            if (tokens[member].Text is "static" or "async" or "function" or "procedure")
            {
                if (fields.Count != 0 || methods.Count != 0) layout?.Section(memberIndex);
                previousMethod = true;
                var method = ReadRoutine(isProgram, inRecord: true) with { Attributes = attributes, Documentation = memberDocumentation };
                methods.Add(method); Decorate(method.Name, memberStart, memberDocumentation);
            }
            else
            {
                if (previousMethod) layout?.Section(memberIndex);
                previousMethod = false;
                var visible = At("public");
                if (At("public") || At("private")) position++;
                var field = Identifier();
                using var fieldDeclaration = Declaration(field, memberStart, visible ? "public" : "private", memberDocumentation);
                fieldDeclaration.Info.Kind = "property";
                var type = ReadBindingType(field);
                ExpressionSyntax? value = null;
                if (At(":=")) { position++; value = ReadExpression(); }
                Expect(";");
                fields.Add(new(field, type, value, visible, memberDocumentation, attributes));
            }
        }
        layout?.Body(bodyIndex, position);
        declaration.Body(bodyStart, Current.Position);
        Expect("end"); Expect("record"); Expect(";");
        return new(name, isPublic, fields, methods, parameters, constraints, interfaces, documentation);
    }

    private WithSyntax ReadWith(ExpressionSyntax receiver)
    {
        Expect("with");
        var bodyStart = position;
        var updates = new List<RecordUpdateSyntax>();
        do
        {
            layout?.Line(position);
            var name = Identifier();
            Expect(":=");
            updates.Add(new(name, ReadExpression()));
            Expect(";");
        } while (!At("end"));
        layout?.Body(bodyStart, position);
        Expect("end"); Expect("with");
        return new(receiver, updates);
    }
}
