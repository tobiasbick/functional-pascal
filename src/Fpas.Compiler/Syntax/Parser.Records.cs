namespace Fpas.Compiler.Syntax;

internal sealed partial class Parser
{
    private RecordSyntax ReadRecord(bool isProgram)
    {
        var documentation = Current.Documentation;
        var isPublic = At("public");
        if (isProgram && isPublic) throw source.Error("FP2003", "Public type declarations belong in units.", Current.Position);
        if (At("public") || At("private")) position++;
        Expect("type");
        var name = Identifier();
        var parameters = ReadTypeParameters();
        Expect("=");
        var constraints = ReadConstraints();
        if (!At("record")) throw source.Error("FP2003", "This compiler currently supports record and union type declarations.", Current.Position);
        position++;
        var interfaces = new List<TypeSyntax>();
        if (At("implements"))
        {
            position++;
            interfaces.Add(ReadType());
            while (At(",")) { position++; interfaces.Add(ReadType()); }
        }
        var fields = new List<RecordFieldSyntax>();
        var methods = new List<RoutineSyntax>();
        while (!At("end"))
        {
            var member = position;
            if (At("public") || At("private")) member++;
            if (tokens[member].Text is "static" or "async" or "function" or "procedure") methods.Add(ReadRoutine(isProgram, inRecord: true));
            else
            {
                var fieldDocumentation = Current.Documentation;
                var visible = !At("private");
                if (At("public") || At("private")) position++;
                var field = Identifier();
                var type = ReadBindingType(field);
                ExpressionSyntax? value = null;
                if (At(":=")) { position++; value = ReadExpression(); }
                Expect(";");
                fields.Add(new(field, type, value, visible, fieldDocumentation));
            }
        }
        Expect("end"); Expect("record"); Expect(";");
        return new(name, isPublic, fields, methods, parameters, constraints, interfaces, documentation);
    }

    private WithSyntax ReadWith(ExpressionSyntax receiver)
    {
        Expect("with");
        var updates = new List<RecordUpdateSyntax>();
        do
        {
            var name = Identifier();
            Expect(":=");
            updates.Add(new(name, ReadExpression()));
            Expect(";");
        } while (!At("end"));
        Expect("end"); Expect("with");
        return new(receiver, updates);
    }
}
