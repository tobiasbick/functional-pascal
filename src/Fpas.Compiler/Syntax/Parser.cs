namespace Fpas.Compiler.Syntax;

internal sealed partial class Parser(SourceText source, IReadOnlyList<Token> tokens)
{
    private int position;
    private Token Current => tokens[position];
    private bool At(string text) => Current.Text == text && Current.Kind is TokenKind.Keyword or TokenKind.Symbol;

    internal CompilationUnitSyntax ParseCompilationUnit()
    {
        var documentation = Current.Documentation;
        var isAsync = At("async");
        if (isAsync) position++;
        var isProgram = At("program");
        if (!isProgram && !At("unit"))
            throw source.Error("FP2001", "Expected a program or unit.", Current.Position, Current.Length);
        if (isAsync && !isProgram) throw source.Error("FP2003", "Only program entry bodies and functions may be async.", Current.Position);
        position++;
        var name = ReadName();
        Expect(";");
        var imports = new List<ImportSyntax>();
        if (At("uses"))
        {
            position++;
            do
            {
                var target = ReadName();
                Expect("as");
                imports.Add(new(target, Identifier()));
                if (!At(","))
                    break;
                position++;
            } while (true);
            Expect(";");
        }
        var routines = new List<RoutineSyntax>();
        var records = new List<RecordSyntax>();
        var unions = new List<UnionSyntax>();
        var aliases = new List<AliasSyntax>(); var scalars = new List<ScalarSyntax>();
        var globals = new List<GlobalSyntax>();
        while (At("public") || At("private") || At("function") || At("procedure") || At("async") || At("type") || At("const") || At("var"))
        {
            var declaration = position;
            if (At("public") || At("private")) declaration++;
            if (tokens[declaration].Text is "const" or "var")
            {
                var doc = Current.Documentation; var visible = At("public");
                if (isProgram && visible) throw source.Error("FP2003", "Public free declarations belong in units.", Current.Position);
                position = declaration;
                globals.Add(new((BindingSyntax)ReadStatement() with { Documentation = doc }, visible));
                Expect(";");
            }
            else if (tokens[declaration].Text == "type")
            {
                var saved = position;
                position = declaration + 2;
                ReadTypeParameters(); Expect("="); ReadConstraints();
                var union = At("union"); var record = At("record"); position = saved;
                if (union) unions.Add(ReadUnion(isProgram));
                else if (record) records.Add(ReadRecord(isProgram));
                else ReadScalarDeclaration(isProgram, aliases, scalars);
            }
            else routines.Add(ReadRoutine(isProgram));
        }
        IReadOnlyList<StatementSyntax> statements = [];
        if (isProgram)
        {
            Expect("begin");
            statements = ReadStatements();
            Expect("end");
            Expect(";");
        }
        Expect("end");
        Expect(isProgram ? "program" : "unit");
        Expect(";");
        if (Current.Kind != TokenKind.End)
            throw source.Error("FP2001", "Unexpected content after the source terminator.", Current.Position, Current.Length);
        return new(name, isProgram, imports, routines, statements, records, unions, isAsync, documentation, aliases, scalars, globals);
    }

    private RoutineSyntax ReadRoutine(bool isProgram, bool inRecord = false, bool nested = false)
    {
        var documentation = Current.Documentation;
        var isPublic = At("public") || inRecord && !At("private");
        if (isProgram && isPublic && !inRecord)
            throw source.Error("FP2003", "Public free declarations belong in units.", Current.Position);
        if (At("public") || At("private"))
            position++;
        var isStatic = !inRecord || At("static");
        if (inRecord && At("static")) position++;
        var isAsync = At("async");
        if (isAsync) position++;
        var function = At("function");
        if (isAsync && !function) throw source.Error("FP2003", "Async routines must be functions returning Task or Task<T>.", Current.Position);
        Expect(function ? "function" : "procedure");
        var name = Identifier();
        var typeParameters = ReadTypeParameters();
        Expect("(");
        var parameters = new List<ParameterSyntax>();
        if (!At(")"))
        {
            do
            {
                parameters.Add(ReadParameter());
                if (!At(";"))
                    break;
                position++;
            } while (true);
        }
        Expect(")");
        TypeSyntax? result = null;
        if (function)
        {
            Expect(":");
            result = ReadType();
        }
        Expect(";");
        var constraints = ReadConstraints();
        var contracts = ReadContracts();
        Expect("begin");
        var body = ReadStatements();
        Expect("end");
        Expect(function ? "function" : "procedure");
        if (!nested) Expect(";");
        return new(name, isPublic, parameters, result, body, isStatic, typeParameters, constraints, contracts, isAsync, documentation);
    }

    private TypeSyntax ReadType(bool arrays = true)
    {
        if (At("function") || At("procedure")) return ReadCallableType();
        var name = ReadName();
        var arguments = At("<") ? ReadTypeArguments() : null;
        var type = new TypeSyntax(name, Arguments: arguments);
        while (At("?") || arrays && At("[") && position + 1 < tokens.Count && tokens[position + 1].Text is "]" or ",")
        {
            if (At("?"))
            {
                if (type.Nullable) throw source.Error("FP2003", "Repeated nullable suffixes are invalid.", Current.Position);
                position++; type = type with { Nullable = true };
            }
            else
            {
                position++;
                var rank = 1;
                while (At(",")) { rank++; position++; }
                Expect("]");
                type = new(name, Element: type, Rank: rank);
            }
        }
        return type;
    }

    private IReadOnlyList<TypeSyntax> ReadTypeArguments()
    {
        Expect("<");
        var arguments = new List<TypeSyntax> { ReadType() };
        while (At(",")) { position++; arguments.Add(ReadType()); }
        Expect(">");
        return arguments;
    }

    private NameSyntax ReadName()
    {
        var parts = new List<Token> { Identifier() };
        while (At("."))
        {
            position++;
            parts.Add(Identifier());
        }
        return new(parts);
    }

    private Token Identifier()
    {
        if (Current.Kind != TokenKind.Identifier)
            throw source.Error("FP2001", $"Expected an identifier, found '{Current.Text}'.", Current.Position, Current.Length);
        return tokens[position++];
    }

    private void Expect(string text)
    {
        if (!At(text))
            throw source.Error("FP2001", $"Expected '{text}', found '{Current.Text}'.", Current.Position, Current.Length);
        position++;
    }
}
