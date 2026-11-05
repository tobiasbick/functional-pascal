namespace Fpas.Compiler.Syntax;

/// <summary>Canonical source spelling for an unescaped semantic identifier.</summary>
public static class IdentifierFacts
{
    public static string Spell(string name)
    {
        if (string.IsNullOrEmpty(name) || name == "_" || name.Length > 1024 ||
            !(char.IsAsciiLetter(name[0]) || name[0] == '_') || name.Any(character => !(char.IsAsciiLetterOrDigit(character) || character == '_')))
            throw new ArgumentException("Supply an ASCII identifier of at most 1,024 characters, without @ or qualification.", nameof(name));
        return Lexer.IsKeyword(name) ? "@" + name : name;
    }
}
