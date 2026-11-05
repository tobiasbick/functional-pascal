using Fpas.Compiler.Syntax;

namespace Fpas.Compiler;

/// <summary>Canonical source spelling for generated qualified names.</summary>
public static class FpasIdentifiers
{
    public static string SpellQualified(string name)
    {
        ArgumentNullException.ThrowIfNull(name);
        var parts = name.Split('.');
        if (parts.Any(part => part.Length == 0 || part == "_" || !(char.IsAsciiLetter(part[0]) || part[0] == '_') || part.Any(character => !char.IsAsciiLetterOrDigit(character) && character != '_')))
            throw new ArgumentException("A qualified identifier is required.", nameof(name));
        return string.Join('.', parts.Select(part => Lexer.IsKeyword(part) ? "@" + part : part));
    }
}
