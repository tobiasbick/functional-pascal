// [Documentation](../../docs/pascal/language/basics/lexical-rules.md)
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler;

/// <summary>Canonical source spelling for generated qualified names.</summary>
public static class FpasIdentifiers
{
    /// <summary>Spells a dot-qualified name, escaping keyword parts with '@'.</summary>
    /// <param name="name">A qualified name of ASCII identifier parts separated by '.'.</param>
    /// <returns>The source spelling of <paramref name="name"/>.</returns>
    /// <exception cref="ArgumentException"><paramref name="name"/> is not a qualified identifier.</exception>
    public static string SpellQualified(string name)
    {
        ArgumentNullException.ThrowIfNull(name);
        var parts = name.Split('.');
        if (parts.Any(part => part.Length == 0 || part == "_" || !(char.IsAsciiLetter(part[0]) || part[0] == '_') || part.Any(character => !char.IsAsciiLetterOrDigit(character) && character != '_')))
            throw new ArgumentException("A qualified identifier is required.", nameof(name));
        return string.Join('.', parts.Select(part => Lexer.IsKeyword(part) ? "@" + part : part));
    }
}
