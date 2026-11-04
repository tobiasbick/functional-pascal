namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private readonly Dictionary<int, DeclarationSyntax> declarations = [];
    private int? declarationOwner;
    private int LastEnd => tokens[position - 1].Position + tokens[position - 1].Length;

    private DeclarationScope Declaration(Token name, int start, string? accessibility = null, DocumentationComment? documentation = null)
    {
        var declaration = new DeclarationSyntax(name, start, declarationOwner, accessibility, documentation);
        declarations.Add(name.Position, declaration);
        declarationOwner = name.Position;
        return new(this, declaration);
    }

    private sealed class DeclarationScope(Parser parser, DeclarationSyntax declaration) : IDisposable
    {
        internal DeclarationSyntax Info => declaration;
        internal void Type(TypeSyntax? type) { if (type is not null) declaration.Types.Add(type); }
        internal void Body(int start, int end) { declaration.BodyStart = start; declaration.BodyEnd = end; }
        public void Dispose() { if (declaration.End == 0) declaration.End = parser.LastEnd; parser.declarationOwner = declaration.Owner; }
    }

    private TypeSyntax SurfaceType()
    {
        var type = ReadType();
        if (declarationOwner is { } owner) declarations[owner].Types.Add(type);
        return type;
    }

    private void Decorate(Token name, int start, DocumentationComment? documentation)
    {
        var declaration = declarations[name.Position];
        declaration.Start = start; declaration.Documentation = documentation;
    }
}
