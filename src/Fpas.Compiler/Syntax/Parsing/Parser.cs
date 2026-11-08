namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser(SourceText source, IReadOnlyList<Token> tokens, SyntaxLayout? layout = null, bool recover = false)
{
    private int position;
    internal string? ContainerName { get; private set; }
    private Token Current { get { source.CheckCancellation(); return tokens[position]; } }
    private bool At(string text) => Current.Text == text && Current.Kind is TokenKind.Keyword or TokenKind.Symbol;

    internal CompilationUnitSyntax ParseCompilationUnit()
    {
        var start = Current.Position;
        var documentation = Current.Documentation;
        var isAsync = At("async");
        if (isAsync) position++;
        var isProgram = At("program");
        if (!isProgram && !At("unit"))
            throw Expected(["program", "unit"], "Expected a program or unit.");
        if (isAsync && !isProgram) throw source.Error("FP2003", "Only program entry bodies and functions may be async.", Current.Position);
        position++;
        var name = ReadName();
        ContainerName = name.Text;
        using var container = Declaration(name.Parts[^1], start, documentation: documentation);
        container.Info.SourceName = name.Text;
        container.Info.Kind = "container";
        Expect(";");
        layout?.Section(position);
        var imports = new List<ImportSyntax>();
        if (At("uses"))
        {
            var list = layout?.List(position, wrap: false, closeLine: false);
            position++;
            do
            {
                list?.Item(position);
                var importStart = Current.Position;
                var target = ReadName();
                Expect("as");
                var alias = Identifier();
                using (var declaration = Declaration(alias, importStart)) declaration.Info.Kind = "alias";
                imports.Add(new(target, alias));
                if (!At(","))
                    break;
                position++;
            } while (true);
            Expect(";");
            list?.End(position - 1);
            layout?.Section(position);
        }
        var routines = new List<RoutineSyntax>();
        var records = new List<RecordSyntax>();
        var interfaces = new List<InterfaceSyntax>();
        var unions = new List<UnionSyntax>();
        var aliases = new List<AliasSyntax>(); var scalars = new List<ScalarSyntax>();
        var globals = new List<GlobalSyntax>();
        string? previousGroup = null;
        while (At("[") || At("public") || At("private") || At("function") || At("procedure") || At("async") || At("type") || At("const") || At("var"))
        {
            var first = position;
            var declarationStart = Current.Position;
            var declarationDocumentation = Current.Documentation;
            var attributes = ReadAttributes();
            var declaration = position;
            if (At("public") || At("private")) declaration++;
            if (layout is not null)
            {
                var group = tokens[declaration].Text is "const" or "var" ?
                    string.Join(' ', tokens.Skip(first).Take(declaration - first + 1).Select(token => token.Text)) : null;
                if (group is not null && group == previousGroup) layout.Line(first); else layout.Section(first);
                previousGroup = group;
            }
            if (tokens[declaration].Text is "const" or "var")
            {
                if (attributes.Count != 0) throw source.Error("FP3035", "Source attributes target types, routines, or record properties.", Current.Position);
                var doc = Current.Documentation; var visible = At("public");
                if (isProgram && visible) throw source.Error("FP2003", "Public free declarations belong in units.", Current.Position);
                position = declaration;
                globals.Add(new((BindingSyntax)ReadStatement() with { Documentation = doc }, visible));
                Expect(";");
                var global = globals[^1].Binding.Name;
                Decorate(global, declarationStart, declarationDocumentation);
                declarations[global.Position].Accessibility = visible ? "public" : "private";
                declarations[global.Position].Kind = tokens[declaration].Text == "const" ? "constant" : "variable";
                declarations[global.Position].End = LastEnd;
            }
            else if (tokens[declaration].Text == "type")
            {
                var probe = new Parser(source, tokens) { position = declaration + 2 };
                probe.ReadTypeParameters(); probe.Expect("="); probe.ReadConstraints();
                var union = probe.At("union"); var record = probe.At("record"); var protocol = probe.At("protocol");
                if (union) unions.Add(ReadUnion(isProgram) with { Attributes = attributes, Documentation = declarationDocumentation });
                else if (record) records.Add(ReadRecord(isProgram) with { Attributes = attributes, Documentation = declarationDocumentation });
                else if (protocol) interfaces.Add(ReadInterface(isProgram) with { Attributes = attributes, Documentation = declarationDocumentation });
                else ReadScalarDeclaration(isProgram, aliases, scalars, attributes, declarationDocumentation);
                Decorate(tokens[declaration + 1], declarationStart, declarationDocumentation);
            }
            else
            {
                var routine = ReadRoutine(isProgram) with { Attributes = attributes, Documentation = declarationDocumentation };
                routines.Add(routine); Decorate(routine.Name, declarationStart, declarationDocumentation);
            }
        }
        IReadOnlyList<StatementSyntax> statements = [];
        if (isProgram)
        {
            layout?.Section(position);
            var bodyStart = Current.Position;
            statements = ReadBody(bodyStart, () => { Expect("begin"); var body = ReadStatements(); Expect("end"); Expect(";"); return body; });
            container.Body(bodyStart, Math.Max(bodyStart, LastEnd));
        }
        layout?.Line(position);
        if (recover && Current.Kind == TokenKind.End)
        { source.Diagnostics.Add(Expected(["end"], "Expected the source terminator.").Diagnostic); InvalidRegions.Add((Current.Position, Current.Position)); }
        else { Expect("end"); Expect(isProgram ? "program" : "unit"); Expect(";"); }
        if (Current.Kind != TokenKind.End)
            throw source.Error("FP2001", "Unexpected content after the source terminator.", Current.Position, Current.Length);
        container.Info.End = LastEnd;
        return new(name, isProgram, imports, routines, statements, records, unions, isAsync, documentation, aliases, scalars, globals, interfaces)
            { Declarations = declarations };
    }

    private RoutineSyntax ReadRoutine(bool isProgram, bool inRecord = false, bool nested = false, bool abstractMember = false)
    {
        var start = Current.Position;
        var documentation = Current.Documentation;
        var isPublic = At("public") || abstractMember;
        if (nested && (At("public") || At("private")))
            throw source.Error("FP2003", "Local routines have lexical visibility.", Current.Position);
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
        using var declaration = Declaration(name, start, nested ? null : isPublic ? "public" : "private", documentation);
        declaration.Info.Kind = "routine";
        var typeParameters = ReadTypeParameters();
        var parameters = ReadList("(", ")", ";", ReadParameter, wrap: false);
        TypeSyntax? result = null;
        if (function)
        {
            Expect(":");
            result = SurfaceType();
        }
        Expect(";");
        var constraints = ReadConstraints();
        if (abstractMember)
            return new(name, true, parameters, result, [], false, typeParameters, constraints, Documentation: documentation);
        var contracts = ReadContracts();
        var bodyStart = Current.Position;
        var body = ReadBody(bodyStart, () =>
        {
            var helpers = ReadRoutineHelpers();
            bodyStart = Current.Position;
            layout?.Line(position);
            Expect("begin");
            var statements = ReadStatements(allowRoutineDeclarations: false);
            Expect("end");
            Expect(function ? "function" : "procedure");
            return helpers.Count == 0 ? statements : [..helpers, ..statements];
        });
        declaration.Body(bodyStart, Math.Max(bodyStart, LastEnd));
        if (!nested)
        {
            if (recover && !At(";") && (Current.Kind == TokenKind.End || At("end") || At("public") || At("private")))
            { source.Diagnostics.Add(Expected([";"], "Expected a routine terminator semicolon.").Diagnostic); InvalidRegions.Add((Current.Position, Current.Position)); }
            else Expect(";");
        }
        return new(name, isPublic, parameters, result, body, isStatic, typeParameters, constraints, contracts, isAsync, documentation);
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
            throw Expected(["identifier"], $"Expected an identifier, found '{Current.Text}'.", kind: "identifier");
        return tokens[position++];
    }

    private void Expect(string text)
    {
        if (!At(text))
            throw Expected([text], $"Expected '{text}', found '{Current.Text}'.");
        position++;
    }
}
