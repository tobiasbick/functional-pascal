namespace Fpas.Compiler.Syntax;

internal sealed class DeclarationSyntax(Token name, int start, int? owner, string? accessibility, DocumentationComment? documentation)
{
    internal Token Name { get; } = name;
    internal string SourceName { get; set; } = name.Text;
    internal int Start { get; set; } = start;
    internal int End { get; set; }
    internal int? Owner { get; } = owner;
    internal string? Accessibility { get; set; } = accessibility;
    internal DocumentationComment? Documentation { get; set; } = documentation;
    internal int? BodyStart { get; set; }
    internal int? BodyEnd { get; set; }
    internal List<TypeSyntax> Types { get; } = [];
}
